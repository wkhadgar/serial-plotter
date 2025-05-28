from controller_framework import AppManager
from controller_framework import Controller
from controller_framework import MCUType


class BallNPlateControler(Controller):
    def __init__(self, label, setpoint):
        super().__init__(label, setpoint)

        self.x_out = 1750
        self.y_out = 1750

        self.closed_loop = True

        self.x_last = 0
        self.x_ierr = 0
        self.y_last = 0
        self.y_ierr = 0

    def x_pid(self, sp, pv, pv_last, ierr, dt):
        Kc = 4  # K/%Heater
        tauI = 0.4  # sec
        tauD = 0.06  # sec
        # Parameters in terms of PID coefficients
        KP = Kc
        KI = Kc / tauI
        KD = Kc * tauD
        # ubias for controller (initial heater)
        op0 = 17500
        # upper and lower bounds on heater level
        ophi = 22500
        oplo = 12500
        # calculate the error
        error = (sp - pv) * 0.6
        print("x_err:", error)
        # calculate the integral error
        ierr = ierr + KI * error * dt
        # calculate the measurement derivative
        dpv = (pv - pv_last) / (dt + 0.000001)
        # calculate the PID output
        P = KP * error
        I = ierr
        D = -KD * dpv
        op = op0 + P + I + D
        # implement anti-reset 10windup
        if op < oplo or op > ophi:
            I = I - KI * error * dt
            # clip output
            op = max(oplo, min(ophi, op))
            # invert for x axis
            # op = -op + ophi + oplo
        # return the controller output and PID terms
        return [op, P, I, D]

    def y_pid(self, sp, pv, pv_last, ierr, dt):
        Kc = 3  # K/%Heater
        tauI = 0.3  # sec
        tauD = 0.08  # sec
        # Parameters in terms of PID coefficients
        KP = Kc
        KI = Kc / tauI
        KD = Kc * tauD
        # ubias for controller (initial heater)
        op0 = 17500
        # upper and lower bounds on heater level
        ophi = 22500
        oplo = 12500
        # calculate the error
        error = sp - pv
        print("y_err:", error)
        # calculate the integral error
        ierr = ierr + KI * error * dt
        # calculate the measurement derivative
        dpv = (pv - pv_last) / (dt + 0.000001)
        # calculate the PID output
        P = KP * error
        I = ierr
        D = -KD * dpv
        op = op0 + P + I + D
        # implement anti-reset windup
        if op < oplo or op > ophi:
            I = I - KI * error * dt
            # clip output
            op = max(oplo, min(ophi, op))
        # return the controller output and PID terms
        return [op, P, I, D]

    def control(self):
        if self.closed_loop:
            if self.x_last == 0:
                self.x_last = self.x_out
            if self.y_last == 0:
                self.y_last = self.y_out

            out_x, _, x_ierr, _ = self.x_pid(self.setpoints[0], self.sensor_values[0], self.x_last, self.x_ierr, self.dt)
            out_y, _, y_ierr, _ = self.y_pid(self.setpoints[1], self.sensor_values[1], self.y_last, self.y_ierr, self.dt)

            self.x_last = self.sensor_values[0]
            self.y_last = self.sensor_values[1]
            self.x_ierr = x_ierr
            self.y_ierr = y_ierr
            self.out1 = out_x / 10
            self.out2 = out_y / 10
        return self.out1, self.out2


if __name__ == '__main__':
    ball_n_plate = BallNPlateControler("Ball and Plate Controller", [2400, 2400])
    ball_n_plate.set_config_variable(("closed_loop", bool))

    app = AppManager(sample_time=10, mcu_type=MCUType.VIRUTAL, x_pos=2400.0, y_pos=2400.0, x_out=1750.0, y_out=1750.0)
    app.append_instance(ball_n_plate)
    app.set_actuator_vars(('Servo X', '', float), ('Servo Y', '', float))
    app.set_sensor_vars(('Pos X', '', float), ('Pos Y', '', float))
    app.init()
