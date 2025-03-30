from abc import ABC, abstractmethod

class Controller(ABC):
    def __init__(self,  label:str, setpoint):
        self.setpoint = setpoint
        self.sensor_a = 0
        self.sensor_b = 0
        self.out = 0
        self.dt = 0
        
        self.configurable_vars = {}
        self.label = label
    
    @abstractmethod
    def control(self):
        pass
    
    def set_dt(self, dt):
        self.dt = dt
        
    def set_config_variable(self, var):
        var_name, var_type = var

        if hasattr(self, var_name):
            current_value = getattr(self, var_name)
            self.configurable_vars[var_name] = {
                "value": current_value,
                "type": var_type
            }
        else:
            print(f"[ERRO] Variável '{var_name}' não encontrada em {self.__class__.__name__}.")

    def update_variable(self, var_name, new_value):
        if var_name in self.configurable_vars:
            var_type = self.configurable_vars[var_name]["type"]

            try:
                casted_value = var_type(new_value)

                setattr(self, var_name, casted_value)
                self.configurable_vars[var_name]["value"] = casted_value
            except ValueError:
                print(f"[ERRO] Valor inválido para '{var_name}'. Esperado {var_type.__name__}, recebido '{new_value}'")
        else:
            print(f"[ERRO] Variável '{var_name}' não está registrada como configurável.")