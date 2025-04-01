from queue import Queue, Empty
import queue
import sys
import threading
import time
from typing import Optional

from .mcu_driver import MCUDriver, MCUType
from .controller import Controller
from .ipcmanager import IPCManager
from controller_framework.gui import MainGUI

import multiprocessing as mp

class AppManager:
    def __init__(self, mcu_type: MCUType, port: str, baud_rate: int):
        if not isinstance(mcu_type, MCUType):
            raise ValueError(f"MCU inválida: {mcu}. Escolha entre {list(MCUType)}")
        self.__mcu: MCUDriver = MCUDriver.create_driver(mcu_type, port, baud_rate)

        self.control_instances: dict[Controller] = {}
        self.running_instance: Optional[Controller] = None

        self.sample_time = 1000.0 # ms
        self.setpoint = 0
        self.sensor_a = 0
        self.sensor_b = 0
        self.duty1 = 0
        self.duty2 = 0
        self.dt = 0

        self.teste1 = 0
        self.teste2 = 0
        self.teste3 = 0

        self.__last_read_timestamp = 0
        self.__last_control_timestamp = 0

        self.reading_buffer_semaphore = threading.Semaphore()

        self.control_dts = list()
        self.read_dts = list()

        self.reading_thread = None
        self.reading_stop_event = threading.Event()

        self.command_thread = None
        self.command_stop_event = threading.Event()

        self.gui = None

        self.reading_buffer = Queue()

        self.queue_to_gui = mp.Queue()
        self.queue_from_gui = mp.Queue()
        self.ipcmanager = IPCManager(self, self.queue_to_gui, self.queue_from_gui)

    def __getstate__(self):
        state = self.__dict__.copy()

        del state['reading_buffer_semaphore']
        del state['reading_stop_event']
        del state['command_stop_event']
        del state['reading_thread']
        del state['command_thread']
        del state['reading_buffer']
        del state['ipcmanager']
        del state['_AppManager__mcu']

        state['mcu_config'] = {
            'mcu_type': self.__mcu.mcu_type.name, 
            'port': self.__mcu.port,
            'baud_rate': self.__mcu.baud_rate
        }

        return state

    def __setstate__(self, state):
        self.__dict__.update(state)
        
        self.reading_buffer_semaphore = threading.Semaphore()
        self.reading_stop_event = threading.Event()
        self.command_stop_event = threading.Event()
        self.reading_thread = None
        self.command_thread = None
        self.reading_buffer = Queue()
        self.ipcmanager = IPCManager(self, self.queue_to_gui, self.queue_from_gui)

        mcu_config = state.pop('mcu_config')
        self.__mcu = MCUDriver.create_driver(
            MCUType[mcu_config['mcu_type']],
            mcu_config['port'],
            mcu_config['baud_rate']
        )
        

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
        print('[APP:read] started')
        now = time.perf_counter()
        target_dt_s = self.sample_time / 1000.0
        next_read_time = now + target_dt_s

        while not self.reading_stop_event.is_set():
            elapsed = 0
            control_elapsed = 0
            read_elapsed = 0
            feedback_elapsed = 0
            read_dt = 0
            write_dt = 0
            control_dt = 0

            now = time.perf_counter()
            
            dt_s = now - self.__last_read_timestamp if self.__last_read_timestamp != 0 else target_dt_s
            dt_ms = dt_s * 1000.0
            self.read_dts.append(dt_s)

            try:
                read_start = time.perf_counter()

                self.sensor_a, self.sensor_b, self.duty1, self.duty2 = self.__mcu.read()

                read_elapsed = (time.perf_counter() - read_start) * 1e3
            except Exception as e:
                print(f"[APP:read] Erro ao ler dados dos sensores: {e}")

            if self.running_instance is not None:
                control_start = time.perf_counter()
                self.__control()

                control_dt = time.perf_counter() - self.teste2 if self.teste2 != 0 else target_dt_s
                control_dt = control_dt * 1e3
                self.teste2 = time.perf_counter()

                control_elapsed = (time.perf_counter() - control_start) * 1e3

                feedback_start = time.perf_counter()
                self.__feedback()

                write_dt = time.perf_counter() - self.teste3 if self.teste3 != 0 else target_dt_s
                write_dt = write_dt * 1e3
                self.teste3 = time.perf_counter()

                feedback_elapsed = (time.perf_counter() - feedback_start) * 1e3

            self.__last_read_timestamp = now

            elapsed = (time.perf_counter() - now) * 1e3
            sleep_time = next_read_time - time.perf_counter()

            print(  
                    f'[APP:read] {self.sensor_a}, {self.sensor_b}, {self.duty1}, {self.duty2} | '
                    f'read dt: {dt_ms:.3f} ms, control dt: {self.dt:.3f} ms | all elapsed: {elapsed:.3f} ms, sleep: {(sleep_time * 1e3):.3f} ms | '
                    f'read elapsed: {read_elapsed:.3f} ms, control elapsed: {control_elapsed:.3f} ms | feedback elapsed: {feedback_elapsed:.3f} ms'
                 )
            
            print(
                    f'[APP:read] teste: {read_dt:.3f}, {control_dt:.3f}, {write_dt:.3f}'
                 )

            if sleep_time > 0:
                time.sleep(sleep_time)
            else:
                print("[APP:read] Leitura atrasada.")
                next_read_time = time.perf_counter()

            next_read_time += target_dt_s

    def __feedback(self):
        self.__mcu.send(self.running_instance.out1, self.running_instance.out2)

    def __update_setpoint(self, setpoint):
        try:
            self.running_instance.setpoint = setpoint
        except AttributeError as e:
            pass

    def __control(self):
        now = time.perf_counter()
        dt = now - self.__last_control_timestamp if self.__last_control_timestamp != 0 else 0
        self.dt = dt * 1e3

        self.running_instance.set_dt(dt)
        self.running_instance.sensor_a = self.sensor_a
        self.running_instance.sensor_b = self.sensor_b

        control_done = threading.Event()
        control_result = [self.running_instance.out1]

        def run_control():
            try:
                result = self.running_instance.control()
                control_result[0] = result
            finally:
                control_done.set()

        thread = threading.Thread(target=run_control, daemon=True)
        thread.start()

        start_time = time.perf_counter()
        while thread.is_alive():
            elapsed = time.perf_counter() - start_time
            if elapsed >= (self.sample_time / 1000.0) * 0.9:
                print('[CONTROL] Controle demorou demais, usando valor anterior')
                break
            time.sleep(0.01)

        if control_done.is_set():
            self.running_instance.out1 = control_result[0]
        self.__last_control_timestamp = time.perf_counter()

    def __connect(self):
        self.__mcu.connect()

    def init(self):
        print("Connect")
        self.__connect()

        self.reading_thread = threading.Thread(target=self.__read_values, daemon=True)
        self.reading_thread.start()

        time.sleep(1)
        self.ipcmanager.init()

        # self.command_thread = threading.Thread(target=self.__read_command, daemon=True)
        # self.command_thread.start()

        self.gui_process = mp.Process(target=MainGUI.start_gui, args=(self,))
        self.gui_process.start()
        self.gui_process.join()

        self.reading_stop_event.set()
        self.reading_thread.join()

        self.ipcmanager.stop()

        # self.command_stop_event.set()
        # self.command_thread.join()

    def start_controller(self, label):
        if label in self.control_instances:
            if self.running_instance != None:
                self.stop_controller()

            try:
                self.running_instance = self.control_instances[label]
                self.setpoint = self.running_instance.setpoint
                print(f"[APP] Start controller: {label}")
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