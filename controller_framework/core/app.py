from queue import Queue
import random
import threading
import time
from typing import Optional
import colorsys

from .mcu_driver import MCUDriver, MCUType
from .controller import Controller
from .ipcmanager import IPCManager
from .logmanager import LogManager
from controller_framework.gui import MainGUI

import multiprocessing as mp


class AppManager:
    def __init__(self, mcu_type: MCUType, sample_time = 1000, **kwargs):
        if not isinstance(mcu_type, MCUType):
            raise ValueError(f"MCU inválida: {mcu_type}. Escolha entre {list(MCUType)}")
        self.__mcu: MCUDriver = MCUDriver.create_driver(mcu_type, **kwargs)

        self.control_instances: dict[Controller] = {}
        self.running_instance: Optional[Controller] = None

        self.sample_time = sample_time # ms

        self.actuator_vars = {}
        self.sensor_vars = {}

        self.num_sensors = 0
        self.num_actuators = 0

        self.__last_read_timestamp = 0
        self.__last_control_timestamp = 0

        self.reading_buffer_semaphore = threading.Semaphore()

        self.control_dts = list()
        self.read_dts = list()

        self.dt = 0

        self.setpoints = []

        self.reading_thread = None
        self.reading_stop_event = threading.Event()

        self.command_thread = None
        self.command_stop_event = threading.Event()

        self.gui = None

        self.reading_buffer = Queue()

        self.queue_to_gui = mp.Queue()
        self.queue_from_gui = mp.Queue()
        self.data_updated = False
        self.ipcmanager = IPCManager(self, self.queue_to_gui, self.queue_from_gui)

        self.color_index = 0

        self.log_manager = LogManager('APP')
        self.log = self.log_manager.get_logger(component='APP')

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
            'kwargs': self.__mcu.kwargs,
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
            **mcu_config['kwargs'],
        )

    def __read_values(self):
        self.log.info('started', extra={'method': 'read'})
        now = time.perf_counter()
        target_dt_s = self.sample_time / 1000.0
        next_read_time = now + target_dt_s

        while not self.reading_stop_event.is_set():
            elapsed = 0
            control_elapsed = 0
            read_elapsed = 0
            feedback_elapsed = 0

            now = time.perf_counter()
            
            dt_s = now - self.__last_read_timestamp if self.__last_read_timestamp != 0 else target_dt_s
            dt_ms = dt_s * 1000.0
            self.read_dts.append(dt_s)

            read_start = time.perf_counter()
            try:
                self.last_timestamp = now
                sensor_values, actuator_values = self.__mcu.read()
                self.data_updated = True

                self.update_actuator_vars(actuator_values)
                self.update_sensors_vars(sensor_values)

            except Exception as e:
                self.log.error("Erro ao ler dados dos sensores: %s", e, extra={'method':'read'})
            read_elapsed = (time.perf_counter() - read_start) * 1e3

            if self.running_instance is not None:
                control_start = time.perf_counter()
                self.__control()
                control_elapsed = (time.perf_counter() - control_start) * 1e3

                feedback_start = time.perf_counter()
                self.__feedback()
                feedback_elapsed = (time.perf_counter() - feedback_start) * 1e3

            self.__last_read_timestamp = now

            elapsed = (time.perf_counter() - now) * 1e3
            sleep_time = next_read_time - time.perf_counter()

            self.log.info(
                "\ndt:         %8.3f ms       | all elapsed:      %8.3f ms | sleep time:      %8.3f ms\n"
                "read elapsed:     %8.3f ms | control elapsed: %8.3f ms | feedback elapsed: %8.3f ms",
                dt_ms, elapsed, sleep_time * 1e3, read_elapsed, control_elapsed, feedback_elapsed,
                extra={'method': 'read'}
            )

            if sleep_time > 0:
                time.sleep(sleep_time)
            else:
                self.log.warning("Leitura atrasada.", extra={'method': 'read'})
                next_read_time = time.perf_counter()

            next_read_time += target_dt_s

    def __feedback(self):
        self.__mcu.send(*self.running_instance.actuator_values)

    def __control(self):
        now = time.perf_counter()
        dt = now - self.__last_control_timestamp if self.__last_control_timestamp != 0 else 0
        self.dt = dt * 1e3

        self.running_instance.set_dt(dt)
        self.running_instance.actuator_values = self.get_actuator_values()
        self.running_instance.sensor_values = self.get_sensor_values()

        control_done = threading.Event()
        control_result = [self.running_instance.actuator_values]

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
                self.log.warning('Controle demorou demais, usando valor anterior', extra={'method':'control'})
                break
            time.sleep(0.01)

        if control_done.is_set():
            self.running_instance.actuator_values = control_result[0]
        self.__last_control_timestamp = time.perf_counter()

    def __connect(self):
        self.__mcu.connect()

    def init(self):
        self.log.info("Try connect mcu", extra={'method':'init'})
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
                self.log.info("Start controller: %s", label, extra={'method':'start control'})
                self.running_instance = self.control_instances[label]
                self.update_setpoint(self.running_instance.setpoints)
            except Exception as e:
                self.log.error("Erro ao inicializar controle: %s", e, extra={'method':'start control'})

    def stop_controller(self):
        if self.running_instance is not None:
            self.log.info("Stop controller: %s", self.running_instance.label, extra={'method':'stop control'})
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
        if len(self.setpoints) == 0:
            return 0
        
        return self.setpoints

    def update_setpoint(self, setpoints):
        tam = min(self.num_sensors, len(setpoints))
        self.setpoints[:tam] = map(float, setpoints[:tam])

        if self.running_instance != None:
            self.running_instance.setpoints = self.setpoints

            if "setpoints" in self.running_instance.configurable_vars:
                self.running_instance.configurable_vars["setpoints"]["value"] = setpoints

    def get_var_values(self, var_dict):
        values = []

        for var_name in var_dict:
            var_value = var_dict[var_name]['value']
            values.append(var_value)

        return values
    
    def _random_color(self):
        n = 12
        hue = (self.color_index / n) % 1.0
        self.color_index += 2

        r, g, b = colorsys.hsv_to_rgb(hue, 0.9, 0.9)
        return '#{:02X}{:02X}{:02X}'.format(int(r*255), int(g*255), int(b*255))

    def __set_var(self, var_dict, *args):
        for var in args:
            var_name, var_unit, var_type = var

            var_dict[var_name] = {
                "type": var_type,
                "value": 0,
                "unit": var_unit,
                "color": self._random_color()
            }
    
    def set_actuator_vars(self, *args):
        self.num_actuators += len(args)
        self.__set_var(self.actuator_vars, *args)
    
    def set_sensor_vars(self, *args):
        self.num_sensors += len(args)
        self.__set_var(self.sensor_vars, *args)

    def get_actuator_values(self):
        return self.get_var_values(self.actuator_vars)
    
    def get_sensor_values(self):
        return self.get_var_values(self.sensor_vars)
    
    def __update_var(self, var_dict, new_values):
        for (var_name, value) in zip(var_dict, new_values):
            expected_type = var_dict[var_name]['type']

            if not isinstance(value, expected_type):
                raise TypeError(f"Variável '{var_name}' espera tipo {expected_type.__name__}, recebeu {type(value).__name__}")
            
            var_dict[var_name]['value'] = value

    def update_actuator_vars(self, new_values):
        self.__update_var(self.actuator_vars, new_values)
    
    def update_sensors_vars(self, new_values):
        self.__update_var(self.sensor_vars, new_values)