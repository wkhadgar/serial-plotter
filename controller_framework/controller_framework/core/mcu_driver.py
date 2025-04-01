from abc import ABC, abstractmethod
from enum import Enum
import random
import struct
import numpy as np
import tclab
from pyocd.core.helpers import ConnectHelper, Session

class MCUType(Enum):
    STM32 = "STM32"
    RDATA = "RDATA"
    TCLAB = "TCLAB"

class MCUDriver(ABC):    
    def __init__(self, mcu_type, port, baud_rate):
        self.baud_rate = baud_rate
        self.port = port
        self.mcu_type: MCUType = mcu_type
        
    @abstractmethod
    def send(self, out):
        pass
    
    @abstractmethod
    def read(self):
        pass
    
    @abstractmethod
    def connect(self):
        pass
    
    @staticmethod
    def create_driver(mcu_type: MCUType, port: str, baud_rate: int):
        driver_map = {
            MCUType.RDATA: RandomDataDriver,
            MCUType.STM32: STM32Driver,
            MCUType.TCLAB: TCLABDriver
        }
        
        if mcu_type not in driver_map:
            raise ValueError(f"MCU não suportada: {mcu_type}")
        
        return driver_map[mcu_type](mcu_type, port, baud_rate)
    
class STM32Driver(MCUDriver):
    def __init__(self, mcu_type, port, baud_rate):
        super().__init__(mcu_type, port, baud_rate)
        
        self.control_block_addr = 0x0
        self.ram = None
        self.ser: Session | None = None
        
    def send(self, out):
        data_bytes = struct.pack("<f", out)
        data = struct.unpack("<I", data_bytes)[0]
        self.ser.target.write32(self.control_block_addr + (2 * 4), data)
    
    def read(self):
        def __read_float(_from: int) -> float:
            data_bytes = struct.pack("<I", self.ser.target.read32(_from))
            return struct.unpack("<f", data_bytes)[0]

        control_floats = []
        for i in range(3):
            control_floats.append(__read_float(self.control_block_addr + (i * 4)))

        self.sensor_a = control_floats[0]
        self.sensor_b = control_floats[1]
        self.duty = control_floats[2]

        return self.sensor_a, self.sensor_b, self.duty
    
    def connect(self):
        self.ser = ConnectHelper.session_with_chosen_probe(target_override="stm32f103c8", connect_mode="attach")
        self.ram = self.ser.target.get_memory_map()[1]

        print("[MCU] Finding control block area...")
        key = [ord(c) for c in "!CTR"]
        for addr in range(self.ram.start, self.ram.end):
            byte = self.ser.target.read8(addr)
            if byte != key[0]:
                continue

            if self.ser.target.read_memory_block8(addr, len(key)) == key:
                print(f"[MCU] Control block area found at 0x{addr:X}!")
                self.control_block_addr = addr + len(key)
                break
        else:
            print("[MCU] Block control area not found!!!")
            raise ValueError("Error")

class RandomDataDriver(MCUDriver):
    def __init__(self, mcu_type, port, baud_rate):
        super().__init__(mcu_type, port, baud_rate)
    
    def read(self):
        self.sensor_a = round(np.random.uniform(20, 50), 2)  # Temperatura entre 20°C e 50°C
        self.sensor_b = round(np.random.uniform(20, 50), 2)  # Temperatura entre 20°C e 50°C
        self.duty1 = round(random.uniform(-100, 100), 2)      # Duty cycle entre -100% e 100%
        self.duty2 = round(random.uniform(-100, 100), 2)      # Duty cycle entre -100% e 100%
            
        return self.sensor_a, self.sensor_b, self.duty1, self.duty2
    
    def send(self, out1, out2):
        # Not necessary logic to send function
        pass
    
    def connect(self):
        # Not necessary logic to connect function
        pass

class TCLABDriver(MCUDriver):
    def __init__(self, mcu_type, port, baud_rate):
        super().__init__(mcu_type, port, baud_rate)

        self.a:tclab.TCLab = None

    def connect(self):
        self.a = tclab.TCLab(debug=False)

    def read(self):
        self.sensor_a, self.sensor_b, self.duty1, self.duty2 = self.a.T1, self.a.T2, self.a.Q1(None), self.a.Q2(None)

        return self.sensor_a, self.sensor_b, self.duty1, self.duty2

    def send(self, out1 = 0.0, out2 = 0.0):
        self.a.Q1(out1)
        # self.a.Q2(out2)