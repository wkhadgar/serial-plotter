import serial
import time


class TemperatureController:
    def __init__(self, port, baudrate=9600, timeout=0.1):
        self.ser = serial.Serial(port, baudrate, timeout=timeout)
        time.sleep(2)

    def get_temperatures(self):
        self.ser.write(b"GET_TEMP\n")
        return [int(x) for x in self.ser.readline().decode().strip().split(',')]

    def set_pwm(self, pwm1, pwm2):
        self.ser.write(f"SET_PWM:{pwm1},{pwm2}\n".encode())
        return [int(x) for x in self.ser.readline().decode().strip()[4:].split(',')]

    def close(self):
        self.ser.close()


# Usage example
if __name__ == "__main__":
    controller = TemperatureController('/dev/ttyACM0')
    try:
        while True:
            temp_a, temp_b = controller.get_temperatures()
            print("temps = ", temp_a, temp_b)
            a, b = controller.set_pwm(128, 128)
            print("acks = ", a, b)
            time.sleep(0.001)
    finally:
        controller.close()
