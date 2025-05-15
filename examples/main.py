import time
from controller_framework import AppManager
from controller_framework import Controller
from controller_framework import MCUType

class PIDControl(Controller):
    def __init__(self, label, init_setpoint, l, t):
        super().__init__(label)
        ti = (l / 0.3)
        td = 0
        self.Kp = (0.9 * (t / l))
        self.Ki = (self.Kp / ti)
        self.Kd = (self.Kp * td)

        self.error = 0
        self.accumulated_I = 0

        self.ntc_1 = 0
        self.ntc_2 = 0

        self.heater_1 = 0
        self.heater_2 = 0

        for setpoint in init_setpoint:
            self.setpoints.append(setpoint)
        
    def control(self):
        dt_s = self.dt / 10 ** 6
        
        measure = (self.ntc_1 + self.ntc_2) / 2

        err = self.setpoints[0] - measure
        P = self.Kp * err
        i_inc = self.Ki * err * dt_s
        D = self.Kd * (err - self.error) / (dt_s + 0.000001)

        self.error = err

        windup_check = P + self.accumulated_I + i_inc + D

        return [max(-100, min(100, windup_check))]

class PIDControl2(Controller):
    def __init__(self, label, setpoint, l, t):
        super().__init__(label, setpoint)
        ti = (l / 0.3)
        td = 0
        self.Kp = (0.9 * (t / l))
        self.Ki = (self.Kp / ti)
        self.Kd = (self.Kp * td)

        self.error = 0
        self.accumulated_I = 0
        
    def control(self):
        time.sleep(0.48)

        dt_s = self.dt
        
        measure = (self.sensor_a + self.sensor_b) / 2

        err = self.setpoint - measure
        P = self.Kp * err
        i_inc = self.Ki * err * dt_s
        D = self.Kd * (err - self.error) / (dt_s + 0.000001)

        self.error = err

        windup_check = P + self.accumulated_I + i_inc + D

        if windup_check > 100:
            self.out = 100

        if windup_check < -100:
            self.out = -100

        self.accumulated_I += i_inc
        self.out = windup_check

if __name__ == '__main__':
    plant = AppManager(MCUType.RDATA, "COM1", 14000)
    plant.set_actuator_vars(("Heater 1", float), ("Heater 2", float),  ("Heater 3", float))
    plant.set_sensor_vars(("NTC 1", float), ("NTC 2", float ), ("NTC 3", float))

    pidcontrol1 = PIDControl("PID Control 1", init_setpoint=(25, 30), l=9.02, t=344.21)
    pidcontrol1.set_config_variable(("Kp", float), ("Ki", float), ("Kd", float))

    # pidcontrol2 = PIDControl2("PID Control 2", 250, 15.02, 18.21)
    # pidcontrol2.set_config_variable(("Kp", float))
    # pidcontrol2.set_config_variable(("Ki", float))
    # teste.append_instance(pidcontrol2)

    plant.append_instance(pidcontrol1)
    plant.init()