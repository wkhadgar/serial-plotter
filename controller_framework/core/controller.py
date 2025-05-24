from abc import ABC, abstractmethod
import ast

from .logmanager import LogManager

class Controller(ABC):
    def __init__(self, label:str, setpoints:list):
        self.log_manager = LogManager('Controller')
        self.log = self.log_manager.get_logger(component='CONTROLLER')

        self.dt = 0
        
        self.configurable_vars = {}

        self.setpoints = setpoints
        self.sensor_values = []
        self.actuator_values = []
        self.set_config_variable(("setpoints", list))

        self.label = label
    
    def __getstate__(self):
        state = {
            'label': self.label,
            'setpoints': self.setpoints,
            'dt': self.dt,
            'configurable_vars': self.configurable_vars,
            'sensor_values': self.sensor_values,
            'actuator_values': self.actuator_values,
        }
        return state

    def __setstate__(self, state):
        self.label             = state['label']
        self.setpoints         = state['setpoints']
        self.dt                = state['dt']
        self.configurable_vars = state['configurable_vars']
        self.sensor_values     = state.get('sensor_values', [])
        self.actuator_values   = state.get('actuator_values', [])

        self.log_manager = LogManager('Controller')
        self.log         = self.log_manager.get_logger(component='CONTROLLER')

    @abstractmethod
    def control(self):
        pass
    
    def set_dt(self, dt):
        self.dt = dt
    
    def __set_var(self, var_dict, *args):
        for var in args:
            var_name, var_type = var

            if hasattr(self, var_name):
                current_value = getattr(self, var_name)

                var_dict[var_name] = {
                    "value": current_value,
                    "type": var_type
                }
            else:
                raise Exception(f"[ERRO] Variável '{var_name}' não encontrada em {self.__class__.__name__}.")

    def set_config_variable(self, *args):
        self.__set_var(self.configurable_vars, *args)

    def update_variable(self, var_name, new_value):
        if var_name in self.configurable_vars:
            var_type = self.configurable_vars[var_name]["type"]

            try:
                if var_name == 'setpoints':
                    casted_value = ast.literal_eval(new_value)
                    casted_value = [float(x) for x in casted_value]
                else:
                    casted_value = var_type(new_value)

                setattr(self, var_name, casted_value)
                self.configurable_vars[var_name]["value"] = casted_value
            except ValueError:
                self.log.error("Valor inválido para '%s'. Esperado %s, recebido '%s'",
                                var_name, var_type.__name__, new_value, extra={'method':'update var'}
                )
        else:
            self.log.error("Variável '%s' não está registrada como configurável.", var_name, extra={'method':'update var'})
