from abc import ABC, abstractmethod

class Controller(ABC):
    def __init__(self,  label:str, setpoint):
        self.setpoint = setpoint
        self.sensor_a = 0
        self.sensor_b = 0
        self.out = 0
        
        self.configurable_vars = {}
        self.label = label
    
    @abstractmethod
    def control(self):
        pass
    
    def set_dt(self, dt):
        self.dt = dt
        
    def set_config_variable(self, var_name):
        if hasattr(self, var_name):
            self.configurable_vars[var_name] = getattr(self, var_name)
        else:
            print(f"[ERRO] Variável '{var_name}' não encontrada em {self.__class__.__name__}.")

    def update_variable(self, var_name, new_value):
        if var_name in self.configurable_vars:
            setattr(self, var_name, new_value)
            self.configurable_vars[var_name] = new_value
            