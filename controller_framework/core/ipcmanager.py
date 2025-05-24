import logging
import multiprocessing as mp
from queue import Empty
import threading
import time

from .logmanager import LogManager

# {
#     "type": "update_variable",
#     "payload": {
#         "var_name": "setpoint",
#         "new_value": 52.5
#     }
# }


class IPCManager:
    def __init__(self, app_manager, queue_to_gui, queue_from_gui):
        from controller_framework.core import AppManager

        assert isinstance(app_manager, AppManager)
        self.core = app_manager

        self.log_manager = LogManager('IPC', logging.DEBUG)
        self.log = self.log_manager.get_logger(component='IPC')

        self.tx_queue: mp.Queue = queue_to_gui
        self.rx_queue: mp.Queue = queue_from_gui

        self.thread: threading.Thread = None
        self.stop_event = threading.Event()

        self.command_registry = {
            "update_variable": self.handler_update_variable,
            "stop_controller": lambda core, _: core.stop_controller(),
            "start_controller": self.handler_start_control,
            "update_setpoint": self.handler_update_setpoint,
        }

    def __run(self):
        self.log.info('started', extra={'method':'run'})
        while not self.stop_event.is_set():
            self.__parse_command()

            if self.core.data_updated:
                self.__send_full_state()
                self.core.data_updated = False

            time.sleep(0.1)

    def init(self):
        self.thread = threading.Thread(target=self.__run)
        self.thread.start()

    def stop(self):
        self.stop_event.set()
        self.thread.join()

    def __send(self, command, payload):
        data = {"type": command, "payload": payload}

        self.tx_queue.put(data)

    def __send_full_state(self):
        command = "full_state"
        payload = {
            "sensors": self.core.get_sensor_values(),
            "actuators": self.core.get_actuator_values(),
            "setpoints": self.core.setpoints,
            "running_instance": self.core.running_instance,
            "control_instances": self.core.control_instances,
            "last_timestamp": self.core.last_timestamp,
        }

        self.__send(
            command, payload
        )

    def __parse_command(self):
        try:
            data = self.rx_queue.get_nowait()
            command = data.get("type")
            payload = data.get("payload", {})
            self.log.debug("%s recebido com payload: %s", command, payload, extra={'method':'parse'})

            if not command:
                self.log.warning("Comando sem 'type'. Ignorado.", extra={'method':'parse'})
                return

            handler = self.command_registry.get(command)
            if not handler:
                self.log.warning("Comando '%s' não registrado.", command, extra={'method':'parse'})
                return

            handler(self.core, payload)

        except Empty:
            pass
        except ValueError as e:
            self.log.error("Erro de validação: %s", e, extra={'method':'parse'})
        except Exception as e:
            self.log.error("[IPC] Erro ao executar comando '%s': %s", command, e,    extra={'method':'parse'})
            self.log.debug("[IPC] Dados recebidos: %s", data, extra={'method':'parse'})

    def handler_update_variable(self, core, payload):
        control_name = payload.get("control_name")
        var_name = payload.get("var_name")
        new_value = payload.get("new_value")

        if not all([control_name, var_name]):
            raise ValueError("[IPC] update_variable exige 'control_name' e 'var_name'")

        instance = core.get_instance(control_name)
        instance.update_variable(var_name, new_value)

    def handler_start_control(self, core, payload):
        control_name = payload.get("control_name")

        if not control_name:
            raise ValueError("[IPC] start_control exige 'control_name'")

        if not isinstance(control_name, str):
            raise ValueError("[IPC] 'control_name' para start_control deve ser str")

        core.start_controller(control_name)

    def handler_update_setpoint(self, core, payload):
        value = payload.get("value")

        if not value:
            raise ValueError("[IPC] update_setpoint exige 'value'")

        if not isinstance(value, (list)):
            raise ValueError(
                "[IPC] 'value' para 'update_setpoint' deve ser int ou float"
            )

        core.update_setpoint(value)
