import numpy as np
from controller_framework import AppManager
from controller_framework import Controller
from controller_framework import MCUType


class PIDControl(Controller):
    def __init__(self, label, init_setpoint, l, t):
        super().__init__(label, init_setpoint)
        ti = (l / 0.3)
        td = 0

        self.open_loop = False
        self.open_duty = 0

        self.Kp = (0.9 * (t / l))
        self.Ki = (self.Kp / ti)
        self.Kd = (self.Kp * td)

        self.error = [0 for _ in range(3)]
        self.accumulated_I = [0 for _ in range(3)]

    def step(self, i, setpoint, measure):
        dt_s = self.dt / 10 ** 6

        err = setpoint - measure
        P = self.Kp * err
        i_inc = self.Ki * err * dt_s
        D = self.Kd * (err - self.error[i]) / (dt_s + 0.000001)

        self.error[i] = err

        windup_check = P + self.accumulated_I[i] + i_inc + D
        self.accumulated_I[i] += i_inc if windup_check < 100 else 0

        return max(-100.0, min(100.0, windup_check))

    def control(self):
        result = []

        if self.open_loop:
            return [self.open_duty]

        sens_sum = self.sensor_values[0] + self.sensor_values[1]
        out = self.step(0, self.setpoints[0], sens_sum / 2)
        result.append(out)

        return result


def convert_values(ntc_a_raw, ntc_b_raw, duty_raw):
    raw_n = np.array([ntc_a_raw, ntc_b_raw])
    r_n = 10_000.0 / ((4025.0 / raw_n) - 1.0)

    t_n = ((298.15 * 3435.0) / (298.15 * np.log(r_n / 10_000.0) + 3435.0)) - 273.15

    return *t_n, round(duty_raw, 2)


if __name__ == '__main__':
    plant = AppManager(mcu_type=MCUType.STM32, sample_time=20, convert_cb=convert_values, ntc_a=0, ntc_b=0, duty=0)
    plant.set_sensor_vars(("NTC Inner", "ºC", float), ("NTC Outer", "ºC", float))
    plant.set_actuator_vars(("Peltier", "%", float))

    pid_control1 = PIDControl("PID Control 1", init_setpoint=25, l=9.02, t=344.21)
    pid_control1.set_config_variable(("open_loop", bool), ("open_duty", float), ("Kp", float), ("Ki", float),
                                     ("Kd", float))

    plant.append_instance(pid_control1)
    plant.init()
