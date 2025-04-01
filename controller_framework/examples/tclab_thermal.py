from controller_framework import AppManager
from controller_framework import Controller
from controller_framework import MCUType

class ThermalControler(Controller):
    def __init__(self, label, setpoint):
        super().__init__(label, setpoint)

        self.out1 = 0.0
        self.out2 = 0.0

        self.closed_loop = True

        self.sensor_a_last = 0
        self.ierr = 0

    def pid(self, sp,pv,pv_last,ierr,dt):
        Kc   = 10.0 # K/%Heater
        tauI = 50.0 # sec
        tauD = 1.0  # sec
        # Parameters in terms of PID coefficients
        KP = Kc
        KI = Kc/tauI
        KD = Kc*tauD
        # ubias for controller (initial heater)
        op0 = 0 
        # upper and lower bounds on heater level
        ophi = 100
        oplo = 0
        # calculate the error
        error = sp-pv
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
            op = max(oplo,min(ophi,op))
        # return the controller output and PID terms
        return [op,P,I,D]

    def control(self):
        if self.closed_loop:
            if self.sensor_a_last == 0:
                self.sensor_a_last = self.sensor_a

            out, _, ierr, _ = self.pid(self.setpoint, self.sensor_a, self.sensor_a_last, self.ierr, self.dt)

            self.sensor_a_last = self.sensor_a
            self.ierr = ierr
            self.out1 = out
            return self.out1

if __name__ == '__main__':
    thermal = ThermalControler("Thermal Controller", 25)
    thermal.set_config_variable(("out1", float))
    thermal.set_config_variable(("out2", float))
    thermal.set_config_variable(("closed_loop", bool))
    thermal.set_config_variable(("setpoint", float))

    app = AppManager(MCUType.TCLAB, "tty", 12000)
    app.append_instance(thermal)
    app.init()