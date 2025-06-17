class MyDriverName(MCUDriver):
    def __init__(self, mcu_type, **kwargs):
        super().__init__(mcu_type, **kwargs)

    def connect(self):
        pass

    def disconnect(self):
        pass

    def read(self):
        # You should return a tuple with sensor values + actuator values
        #return (sensor_values + actuator_values)
        pass

    def send(self, *outs):
        # You receive a tuple of outs and should send to your mcu
        pass