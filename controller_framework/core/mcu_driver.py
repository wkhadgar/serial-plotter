from abc import ABC, abstractmethod
from enum import Enum
import random
import struct
import time
import numpy as np
import serial
from pyocd.core.helpers import ConnectHelper, Session


class MCUType(Enum):
    STM32 = "STM32"
    RDATA = "RDATA"
    TCLAB = "TCLAB"


class MCUDriver(ABC):
    def __init__(self, mcu_type, **kwargs):
        self.mcu_type: MCUType = mcu_type
        self.kwargs = kwargs

    @abstractmethod
    def send(self, *outs):
        pass

    @abstractmethod
    def read(self):
        pass

    @abstractmethod
    def connect(self):
        pass

    @staticmethod
    def create_driver(mcu_type: MCUType, **kwargs):
        driver_map = {
            MCUType.RDATA: RandomDataDriver,
            MCUType.STM32: STM32Driver,
            MCUType.TCLAB: TCLABDriver
        }

        if mcu_type not in driver_map:
            raise ValueError(f"MCU não suportada: {mcu_type}")

        return driver_map[mcu_type](mcu_type, **kwargs)


class STM32Driver(MCUDriver):
    def __init__(self, mcu_type, **kwargs):
        super().__init__(mcu_type, **kwargs)

        self.control_block_addr = 0x0
        self.ram = None
        self.ser: Session | None = None

    def send(self, *outs):
        for i, out in enumerate(outs):
            data_bytes = struct.pack("<f", out)
            data = struct.unpack("<I", data_bytes)[0]
            self.ser.target.write32(self.control_block_addr + ((2 + i) * 4), data)

    def read(self):
        def __read_float(_from: int) -> float:
            data_bytes = struct.pack("<I", self.ser.target.read32(_from))
            return struct.unpack("<f", data_bytes)[0]

        control_floats = []
        for i in range(len(self.kwargs.items())):
            control_floats.append(__read_float(self.control_block_addr + (i * 4)))

        for i, (kw, _) in enumerate(self.kwargs.items()):
            self.kwargs[kw] = control_floats[i]

        return self.kwargs.values()

    def connect(self):
        self.ser = ConnectHelper.session_with_chosen_probe(target_override="stm32f103c8", connect_mode="attach")
        self.ram = self.ser.target.get_memory_map()[1]
        self.ser.open()

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
    def __init__(self, mcu_type):
        super().__init__(mcu_type)
        self.sensor_a = None
        self.sensor_b = None
        self.duty1 = None
        self.duty2 = None

    def read(self):
        self.sensor_a = round(np.random.uniform(20, 50), 2)  # Temperatura entre 20°C e 50°C
        self.sensor_b = round(np.random.uniform(20, 50), 2)  # Temperatura entre 20°C e 50°C
        self.duty1 = round(random.uniform(-100, 100), 2)  # Duty cycle entre -100% e 100%
        self.duty2 = round(random.uniform(-100, 100), 2)  # Duty cycle entre -100% e 100%

        return self.sensor_a, self.sensor_b, self.duty1, self.duty2

    def send(self, out1, out2):
        # Not necessary logic to send function
        pass

    def connect(self):
        # Not necessary logic to connect function
        pass


class TCLABDriver(MCUDriver):
    def __init__(self, mcu_type, timeout=0.1, **kwargs):
        super().__init__(mcu_type, **kwargs)
        self.ser = None
        self.timeout = timeout

    def connect(self):
        self.ser = serial.Serial(port=self.kwargs["port"], baudrate=self.kwargs["baud"], timeout=self.timeout)
        time.sleep(2)

    def read(self, out1=0.0, out2=0.0):
        def convert_raw(raw_adc, aref=3.3):
            return raw_adc * (aref / 1023.0 * 100.0) - 50.0

        self.ser.write(b"GET_TEMP\n")

        raw_temps = [int(x) for x in self.ser.readline().decode().strip().split(',')]

        celsius_temps = [convert_raw(raw_temp) for raw_temp in raw_temps]
        self.ser.write("GET_PWM\n".encode())
        dutys = [float(x) for x in self.ser.readline().decode().strip().split(',')]

        return celsius_temps + dutys

    def send(self, out1=0.0, out2=0.0):
        self.ser.write(f"SET_PWM:{out1},{out2}\n".encode())
