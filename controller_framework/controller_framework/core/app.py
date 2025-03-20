
import threading
import time
from typing import Optional

import pandas as pd

from .mcu_driver import MCUDriver, MCUType
from .controller import Controller
from controller_framework.gui import MainGUI

class AppManager:
    def __init__(self, mcu_type: MCUType, port :str, baud_rate: int):
        if not isinstance(mcu_type, MCUType):
            raise ValueError(f"MCU inválida: {mcu}. Escolha entre {list(MCUType)}")

        self.__mcu = MCUDriver.create_driver(mcu_type, port, baud_rate)
        self.__last_timestamp = pd.Timestamp.now()
        
        self.control_instances = {}
        
        self.running_instance: Optional[Controller] = None
        self.running_thread = None
        self.stop_event = threading.Event()
        
        self.gui = None
        self.setpoint = 0
        self.sensor_a = 0
        self.sensor_b = 0
        self.duty = 0
    
    def __read_values(self):
        self.sensor_a, self.sensor_b, self.duty = self.__mcu.read()
        
        if self.running_instance != None:
            self.running_instance.sensor_a = self.sensor_a
            self.running_instance.sensor_b = self.sensor_b
    
    def __feedback(self):
        self.__mcu.send(self.running_instance.out)
        
    def __update_setpoint(self, setpoint):
        try:
            self.running_instance.setpoint = setpoint
        except AttributeError as e:
            print('nenhuma rodando')
        
    def __control_thread(self):
        while not self.stop_event.is_set():
            self.__read_values()
            timestamp = pd.Timestamp.now()
            dt_t = timestamp - self.__last_timestamp
            self.dt = dt_t.microseconds
            self.running_instance.set_dt(self.dt)
            self.running_instance.control()
            self.__feedback()
            time.sleep(0.01)
    
    def __connect(self):
        self.__mcu.connect()
        
    def init(self):
        print("Connect")
        self.__connect()
        self.gui = MainGUI.start_gui(app_manager=self)
        
    def start_controller(self, label):
         if label in self.control_instances:
            if self.running_thread and self.running_thread.is_alive():
                print(f"Stop cotnroller: {self.running_instance.label}")
                self.stop_event.set()
                self.running_thread.join()
                self.stop_event.clear()
                self.running_instance = None
                self.running_thread = None
            
            try:
                self.running_instance = self.control_instances[label]
                self.setpoint = self.running_instance.setpoint
                self.running_thread = threading.Thread(target=self.__control_thread, daemon=True)
                print(f"Start cotnroller: {label}")
                self.running_thread.start()
            except Exception as e:
                print(f"value error {e}")
            
    def append_instance(self, instance:Controller):
        self.control_instances[instance.label] = instance
        
    def list_instances(self):
        return list(self.control_instances.keys())
    
    def get_setpoint(self):
        return self.setpoint
        
    def update_setpoint(self, setpoint):
        self.setpoint = setpoint
        
        if self.running_instance != None:
            self.running_instance.setpoint = setpoint
            
            if "setpoint" in self.running_instance.configurable_vars:
                self.running_instance.configurable_vars["setpoint"] = setpoint