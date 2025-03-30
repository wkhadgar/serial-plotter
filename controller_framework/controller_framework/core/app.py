from queue import Queue, Empty
import queue
import sys
import threading
import time
from typing import Optional

from .mcu_driver import MCUDriver, MCUType
from .controller import Controller
from controller_framework.gui import MainGUI

import multiprocessing as mp

class AppManager:
    def __init__(self, mcu_type: MCUType, port: str, baud_rate: int):
        if not isinstance(mcu_type, MCUType):
            raise ValueError(f"MCU inválida: {mcu}. Escolha entre {list(MCUType)}")

        self.__mcu = MCUDriver.create_driver(mcu_type, port, baud_rate)

        self.__last_read_timestamp = 0
        self.__last_control_timestamp = 0

        self.reading_buffer_semaphore = threading.Semaphore()

        self.control_dts = list()
        self.read_dts = list()

        self.control_instances = {}

        self.running_instance: Optional[Controller] = None

        self.reading_thread = None
        self.reading_stop_event = threading.Event()

        self.command_thread = None
        self.command_stop_event = threading.Event()

        self.gui = None
        self.setpoint = 0
        self.sensor_a = 0
        self.sensor_b = 0
        self.duty = 0
        self.duty2 = 0

        self.reading_buffer = Queue()
        self.gui_data_queue = mp.Queue()
        self.command_data_queue = mp.Queue()

    def __read_command(self):
        while not self.command_stop_event.is_set():
            try:
                command, values = self.command_data_queue.get_nowait()
                print(f"[APP:command] Comando {command} recebido com valor {values}")

                method = None
                if(command == "update_variable"):
                    instance = self.get_instance(values[0])
                    method = getattr(instance, command)
                    values = values[1:]
                else:
                    method = getattr(self, command)
                
                if isinstance(values, list):
                    method(*values)
                else:
                    method(values)
            except Empty:
                pass
            except Exception as e:
                print(f'[APP:command] Comando nao existe! {e, command, values}')
            time.sleep(0.05)

    def __read_values(self):
        target_dt_ms = 500 # ms
        target_dt_s = target_dt_ms / 1000.0  # s
        next_read_time = 0

        while not self.reading_stop_event.is_set():
            next_read_time += target_dt_s
            now = time.perf_counter()

            dt_s = (
                now - self.__last_read_timestamp
                if self.__last_read_timestamp != 0
                else target_dt_s
            )
            dt_ms = dt_s * 1000.0

            self.read_dts.append(dt_s)

            try:
                # print(
                #     f"{dt_ms:.3f} ms, {now:.6f} s, {self.__last_read_timestamp:.6f} s"
                # )
                self.sensor_a, self.sensor_b, self.duty1, self.duty2 = self.__mcu.read()
                print(f'[APP:read] {self.sensor_a}, {self.sensor_b}, {self.duty1}, {self.duty2}')

                data = [self.sensor_a, self.sensor_b, self.duty1, self.duty2]
                self.gui_data_queue.put(data)

                if self.running_instance is not None:
                    self.reading_buffer.put(data)
                    # self.reading_buffer_semaphore.release()
                    self.__control_thread()
            except Exception as e:
                print(f"[APP:read] Erro ao ler dados dos sensores: {e}")

            self.__last_read_timestamp = now

            sleep_time = next_read_time - time.perf_counter()

            if sleep_time > 0:
                time.sleep(sleep_time)
            else:
                print("[APP:read] Leitura atrasada.")
                next_read_time = time.perf_counter() + target_dt_s

    def __feedback(self):
        self.__mcu.send(self.running_instance.out1, self.running_instance.out2)

    def __update_setpoint(self, setpoint):
        try:
            self.running_instance.setpoint = setpoint
        except AttributeError as e:
            pass

    def __control_thread(self):
        if not self.reading_buffer.empty():
            a, b, _, _ = self.reading_buffer.get()

            now = time.perf_counter()
            dt = now - self.__last_control_timestamp
            self.__last_control_timestamp = now
            self.dt = dt * 1e3

            self.running_instance.set_dt(dt)
            self.running_instance.sensor_a = a
            self.running_instance.sensor_b = b

            self.running_instance.control()
            self.__feedback()
            self.control_dts.append(self.dt)

    def __connect(self):
        self.__mcu.connect()

    def init(self):
        print("Connect")
        self.__connect()

        print("Start reading values!")
        self.reading_thread = threading.Thread(target=self.__read_values, daemon=True)
        self.reading_thread.start()

        self.command_thread = threading.Thread(target=self.__read_command, daemon=True)
        self.command_thread.start()

        self.gui_process = mp.Process(target=MainGUI.start_gui, args=(self,))
        self.gui_process.start()

        self.gui_process.join()
        self.reading_stop_event.set()
        self.reading_thread.join()

        self.command_stop_event.set()
        self.command_thread.join()

    def start_controller(self, label):
        if label in self.control_instances:
            if self.running_instance != None:
                self.stop_controller()

            try:
                print(f"[APP] Start controller: {label}")
                self.running_instance = self.control_instances[label]
                self.setpoint = self.running_instance.setpoint
            except Exception as e:
                print(f"value error {e}")

    def stop_controller(self):
        if self.running_instance is not None:
            print(f"[APP] Stop controller: {self.running_instance.label}")
            self.running_instance = None

    def append_instance(self, instance: Controller):
        self.control_instances[instance.label] = instance

    def list_instances(self):
        return list(self.control_instances.keys())
    
    def get_instance(self, label):
        if(label in self.control_instances):
            return self.control_instances[label]
        else:
            return None

    def get_setpoint(self):
        return self.setpoint

    def update_setpoint(self, setpoint):
        self.setpoint = setpoint

        if self.running_instance != None:
            self.running_instance.setpoint = setpoint

            if "setpoint" in self.running_instance.configurable_vars:
                self.running_instance.configurable_vars["setpoint"]["value"] = setpoint