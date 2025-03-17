from controller_framework import AppManager
from controller_framework import Controller
from controller_framework import MCUType

class PIDControl(Controller):
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
        dt_s = self.dt / 10 ** 6
        
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
        # print(self.setpoint)     

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
        dt_s = self.dt / 10 ** 6
        
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
         
teste = AppManager(MCUType.STM32, "COM1", 14000)

pidcontrol1 = PIDControl("PID Control 1", 25, 9.02, 344.21)
pidcontrol1.set_config_variable("setpoint")
pidcontrol1.set_config_variable("Kp")
pidcontrol1.set_config_variable("Ki")
pidcontrol1.set_config_variable("Kd")

pidcontrol2 = PIDControl2("PID Control 2", 250, 15.02, 18.21)
pidcontrol2.set_config_variable("Kp")
pidcontrol2.set_config_variable("Ki")

teste.append_instance(pidcontrol1)
teste.append_instance(pidcontrol2)

teste.init()