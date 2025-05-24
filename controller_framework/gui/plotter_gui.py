from collections.abc import Callable
from functools import partial
import logging
import os
import queue
from typing import Optional

from controller_framework.core.controller import Controller
import numpy as np
import pandas as pd

from PySide6 import QtCore
from PySide6.QtWidgets import ( QGroupBox, QFormLayout, QVBoxLayout, QWidget, QLabel, QScrollArea,
                             QPushButton, QHBoxLayout, QLineEdit, QGraphicsProxyWidget, QListWidget, QCheckBox )

import re

from controller_framework.core.logmanager import LogManager

from .utils_gui import PlotWidget, Mode

class ControlGUI(QWidget):
    def __init__(self, *, parent, app_mirror, x_label: str, y_label: str):
        super().__init__(parent)

        self.parent = parent

        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.fullscreen = False
        
        self.init_timestamp = None
        self.plot_seconds = []
        self.actuator_data = [[] for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [[] for _ in range(self.app_mirror.num_sensors)]

        self.sensor_labels = [chr(ord("A")+i) for i in range(self.app_mirror.num_sensors)]

        self.plot_views = ["ALL"] + self.sensor_labels
        self.current_mode = -1

        self.layout = QVBoxLayout()
        self.setLayout(self.layout)

        self.container = QWidget()
        self.container_layout = QVBoxLayout()
        self.container.setLayout(self.container_layout)
        
        self.plot_widget = PlotWidget(self.container_layout, Mode.PLOTTER)
        # self.plot_widget.plotter_plot()
        self.toggle_plot_view()

        self.temp_input = QLineEdit()
        self.temp_input.setPlaceholderText("Defina os setpoints desejados [separados por vírgulas ou espaços]")
        self.temp_input.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.container_layout.addWidget(self.temp_input)

        self.temp_input.returnPressed.connect(self.__on_return_pressed)
        self.layout.addWidget(self.container)

        self.container.keyPressEvent = partial(self.key_press_handle, self.container.keyPressEvent)
        
        log_path = "./temp_logs/"
        if not os.path.exists(log_path):
            os.makedirs(log_path)
        self.parent.log.debug("Salvando dados em %s", log_path, extra={'method':'init'})
        
        datetime = pd.Timestamp.now()
        sensor_columns = [f"sensor_{i}" for i in range(self.app_mirror.num_sensors)]
        actuator_columns = [f"actuator_{i}" for i in range(self.app_mirror.num_actuators)]

        columns = ["timestamp", "seconds"] + sensor_columns + actuator_columns + ["target"]

        self.df = pd.DataFrame(columns=columns)
        
        self.log_file_path = log_path + f"log_{datetime.year}-{datetime.month}-{datetime.day}-{datetime.hour}-{datetime.minute}-{datetime.second}.csv"
        self.df.to_csv(self.log_file_path, index=False)
        
        self.update_delay = 100
        self.plot_timer = QtCore.QTimer()
        self.plot_timer.timeout.connect(self.update_data)
        self.plot_timer.start(self.update_delay)

        self.is_selected = True

    def __on_return_pressed(self):
        setpoint_string = self.temp_input.text()
        self.temp_input.clear()

        setpoint = re.split(r'[,\s]+', setpoint_string.strip())

        self.app_mirror.update_setpoint(setpoint)

        self.parent.command_triggered.emit("update_setpoint", {"value": setpoint})

    def update_setpoint_label(self):
        self.current_setpoint_line.setValue(self.app_mirror.get_setpoint())
        self.current_setpoint_line.label.setText(f"Temperatura desejada [{self.app_mirror.get_setpoint()}°C]")
        self.current_setpoint_line.update()

    def __retrieve_message(self, timeout: float = 0.01) -> Optional[dict]:
        try:
            return self.app_mirror.queue_to_gui.get(timeout=timeout)
        except queue.Empty:
            return None
        
    def __process_message(self, data):
        command = data.get('type')
        payload = data.get('payload')

        if command == "full_state":
            sensors = payload.get('sensors')
            actuators = payload.get('actuators')
            setpoints = payload.get('setpoints')
            running_instance = payload.get('running_instance')
            control_instances_data = payload.get('control_instances')
            last_timestamp = payload.get('last_timestamp')

            if self.init_timestamp is None:
                self.init_timestamp = last_timestamp
            self.last_timestamp = last_timestamp

            self.app_mirror.running_instance = running_instance
            self.app_mirror.control_instances = control_instances_data
            self.app_mirror.update_sensors_vars(sensors)
            self.app_mirror.update_actuator_vars(actuators)
            self.app_mirror.update_setpoint(setpoints)

            return True
        else:
            return False

    def __append_plot_data(self, sensor_values, actuator_values):
        self.plot_seconds.append(self.last_timestamp - self.init_timestamp)

        for lista, value in zip(self.sensor_data, sensor_values):
            lista.append(value)

        for lista, value in zip(self.actuator_data, actuator_values):
            lista.append(value)

    def __write_csv(self, sensor_values, actuator_values):
        target_str = '"' + " ".join(map(str, self.app_mirror.setpoints)) + '"'
        row = {
            "timestamp": self.last_timestamp,
            "seconds": f"{self.plot_seconds[-1]:.4f}",

            **{f"sensor_{i}": f"{sensor_values[i]:.4f}"
                for i in range(self.app_mirror.num_sensors)},

            **{f"actuator_{i}": f"{actuator_values[i]:.4f}"
                for i in range(self.app_mirror.num_actuators)},

            "target": target_str
        }

        with open(self.log_file_path, "a") as f:
            data = ",".join(map(str, row.values())) + "\n"
            f.write(data)

    def update_data(self):
        data = self.__retrieve_message()
        if data is None:
            return

        if not self.__process_message(data):
            return
        
        sensor_values = self.app_mirror.get_sensor_values()
        actuator_values = self.app_mirror.get_actuator_values()
        
        self.__append_plot_data(sensor_values, actuator_values)

        if self.is_selected:
            self.update_plots()

        self.__write_csv(sensor_values, actuator_values)
        
    def update_plots(self):      
        view = self.plot_views[self.current_mode]
        plot_seconds = np.array(self.plot_seconds)

        match view:
            case "ALL":
                for (i, sensor_data), (var_name, props) in zip(enumerate(self.sensor_data), self.app_mirror.sensor_vars.items()):
                    np_sensor_data = np.array(sensor_data)
                    self.plot_widget.update_curve(plot_seconds, np_sensor_data, 0, i)

                    legenda = f'{var_name}: {np_sensor_data[-1]:.4f} {props['unit']}'
                    self.plot_widget.update_legend(text=legenda, plot_n=0, idx=i)
                for (i, actuator_data), (var_name, props) in zip(enumerate(self.actuator_data), self.app_mirror.actuator_vars.items()):
                    np_actuator_data = np.array(actuator_data)
                    self.plot_widget.update_curve(plot_seconds, np_actuator_data, 1, i)

                    legenda = f'{var_name}: {np_actuator_data[-1]:.4f} {props['unit']}'
                    self.plot_widget.update_legend(text=legenda, plot_n=1, idx=i)

            case _ if view in self.sensor_labels:
                letters = self.sensor_labels
                idx = letters.index(view)

                sensor_data = np.array(self.sensor_data[idx])
                self.plot_widget.update_curve(plot_seconds, sensor_data, plot_n=0, curve_n=0)

                var_name, props = list(self.app_mirror.sensor_vars.items())[idx]
                legenda = f'{var_name}: {sensor_data[-1]:.4f} {props['unit']}'
                self.plot_widget.update_legend(text=legenda, plot_n=0, idx=0)
            case _:
                self.parent.log.warning("Visualização '%s' não reconhecida.", view, extra={'method':'update plot'})

    def toggle_plot_view(self):
        self.current_mode = (self.current_mode + 1) % len(self.plot_views)
        view = self.plot_views[self.current_mode]

        self.plot_widget.clear()

        match view:
            case "ALL":
                self.plot_widget.plotter_dual_plot('Sensors', 'Actuators')

                for i, (var_name, props) in enumerate(self.app_mirror.sensor_vars.items()):
                    self.plot_widget.add_curve([0], [0], color=props['color'], plot_n=0)
                    self.plot_widget.add_legend(text=var_name, color=props['color'], plot_n=0)
                
                for i, (var_name, props) in enumerate(self.app_mirror.actuator_vars.items()):
                    self.plot_widget.add_curve([0], [0], color=props['color'], plot_n=1)
                    self.plot_widget.add_legend(text=var_name, color=props['color'], plot_n=1)
                    
            case _ if view in self.sensor_labels:
                idx = self.sensor_labels.index(view)
                var_name, props = list(self.app_mirror.sensor_vars.items())[idx]

                self.plot_widget.plotter_single_plot(var_name)  
                self.plot_widget.add_curve([0], [0], color=props['color'])
                self.plot_widget.add_legend(text=var_name, color=props['color'], plot_n=0)
                    
            case _:
                self.parent.log.warning("Visualização '%s' não reconhecida.", view, extra={'method':'toggle plot'})

    def reset_data(self):
        self.plot_seconds = []
        self.actuator_data = [[] for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [[] for _ in range(self.app_mirror.num_sensors)]
        self.init_timestamp = None

        self.df = pd.DataFrame(columns=self.df.columns)
        self.df.to_csv(self.log_file_path, index=False)

    def key_press_handle(self, super_press_handler: Callable, ev):
        if self.temp_input.hasFocus():
            super_press_handler(ev)
        else:
            if ev.key() == QtCore.Qt.Key.Key_Space:
                self.toggle_plot_view()
            elif ev.key() == QtCore.Qt.Key.Key_E:
                self.reset_data()
            elif ev.key() == QtCore.Qt.Key.Key_Escape:
                super_press_handler(ev)
            elif ev.key() == QtCore.Qt.Key.Key_F:
                super_press_handler(ev)


class SidebarGUI(QWidget):
    def __init__(self, parent, app_mirror, control_gui):
        super().__init__(parent)
        
        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.parent = parent

        self.control_gui = control_gui
        self.current_control = None
        self.input_fields = {}

        self.layout = QVBoxLayout()
        self.setLayout(self.layout)

        self.controls_group = QGroupBox("Controles Disponíveis")
        self.controls_layout = QVBoxLayout()
        self.controls_group.setLayout(self.controls_layout)

        self.control_list = QListWidget()
        self.controls_layout.addWidget(self.control_list)
        
        self.btn_activate_control = QPushButton("Ativar Controle")
        self.btn_activate_control.clicked.connect(self.activate_control)
        
        self.btn_deactivate_control = QPushButton("Desativar Controle")
        self.btn_deactivate_control.clicked.connect(self.deactivate_control)
        self.btn_deactivate_control.setEnabled(False)

        self.layout.addWidget(self.controls_group)
        
        self.hbox = QHBoxLayout()
        self.hbox.addWidget(self.btn_activate_control)
        self.hbox.addWidget(self.btn_deactivate_control)
        
        self.layout.addLayout(self.hbox)

        self.settings_group = QGroupBox("Configurações do Controle")
        self.settings_group.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.settings_layout = QFormLayout()
        self.settings_group.setLayout(self.settings_layout)
        
        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFixedHeight(300)
        self.scroll_area.setWidget(self.settings_group)

        self.btn_update_settings = QPushButton("Atualizar Configurações")
        self.layout.addWidget(self.scroll_area)
        self.layout.addWidget(self.btn_update_settings)

        self.control_list.itemSelectionChanged.connect(self.update_config_fields)
        self.btn_update_settings.clicked.connect(self.update_control_settings)
        
        self.controls_group.setStyleSheet("QGroupBox { font-size: 16px; font-weight: bold; }")
        self.settings_group.setStyleSheet("QGroupBox { font-size: 16px; font-weight: bold; }")
        self.control_list.setStyleSheet("QListWidget { font-size: 14px; }")
        
        btn_label_style = "QPushButton { font-size: 14px; }"
        self.btn_activate_control.setStyleSheet(btn_label_style)
        self.btn_deactivate_control.setStyleSheet(btn_label_style)
        self.btn_update_settings.setStyleSheet(btn_label_style)
        
        self.scroll_area.setStyleSheet("""
                                            QScrollBar:vertical {
                                                background: white;
                                                width: 10px;
                                            }
                                            QScrollBar:horizontal {
                                                background: white;
                                                height: 10px;
                                            }
                                            QScrollBar::handle:vertical {
                                                background: #f0f0f0;
                                            }
                                            QScrollBar::handle:horizontal {
                                                background: #f0f0f0;
                                            }
                                        """)

        self.update_control_list()

    def update_control_list(self):
        self.control_list.clear()
        for control_name in self.app_mirror.list_instances():
            self.control_list.addItem(control_name)

    def update_config_fields(self):
        selected_item = self.control_list.currentItem()
        
        if selected_item:
            control_name = selected_item.text()
            self.current_control:Controller = self.app_mirror.control_instances[control_name]
                
            for i in reversed(range(self.settings_layout.count())):
                self.settings_layout.itemAt(i).widget().deleteLater()
            self.input_fields.clear()

            for var_name, var_data in self.current_control.configurable_vars.items():
                value = var_data['value']
                var_type = var_data['type']
                
                label = QLabel(f"{var_name}")
                label.setStyleSheet("QLabel { font-size: 14px; }")

                if var_type == bool:
                    input_field = QCheckBox()
                    input_field.setChecked(bool(value))
                    input_field.setStyleSheet("QCheckBox { font-size: 14px; }")
                else:
                    input_field = QLineEdit()
                    input_field.setText(str(value))
                    input_field.setStyleSheet("QLineEdit { font-size: 14px; }")

                self.settings_layout.addRow(label, input_field)

                self.input_fields[var_name] = input_field

            self.settings_group.setTitle(f"Configurações de {control_name}")

    def update_control_settings(self):
        if self.current_control:
            for var_name, widget in self.input_fields.items():
                try:
                    if isinstance(widget, QCheckBox):
                        new_value = widget.isChecked()
                    elif isinstance(widget, QLineEdit):
                        new_value = widget.text()
                    else:
                        continue

                    self.current_control.update_variable(var_name, new_value)

                    command = "update_variable"
                    payload = {
                        "control_name": self.current_control.label,
                        "var_name": var_name,
                        "new_value": new_value
                    }

                    self.parent.command_triggered.emit(command, payload)
                    
                except ValueError:
                    self.parent.log.error("Entrada inválida para '%s'", var_name, extra={'method':'update control'})
            
            if(self.app_mirror.running_instance and self.app_mirror.running_instance.label == self.current_control.label):
                self.app_mirror.update_setpoint(self.current_control.setpoints)
                self.parent.command_triggered.emit("update_setpoint", {"value": self.current_control.setpoints})
                    
    def activate_control(self):
        current_control = self.control_list.currentItem()
        
        if(current_control != None):
            current_control_label = current_control.text()
            self.app_mirror.update_setpoint(self.current_control.setpoints)

            self.app_mirror.running_instance = self.app_mirror.get_instance(current_control_label)
            self.parent.command_triggered.emit("start_controller", {"control_name": current_control_label})
            self.parent.command_triggered.emit("update_setpoint", {"value": self.current_control.setpoints})

            # self.control_gui.update_setpoint_label()
            
            self.btn_deactivate_control.setEnabled(True)
    
    def deactivate_control(self):
        self.app_mirror.stop_controller()
        self.parent.command_triggered.emit("stop_controller", {})
        
        self.btn_deactivate_control.setEnabled(False)


class PlotterGUI(QWidget):
    command_triggered = QtCore.Signal(str, object)

    def __init__(self, app_mirror):
        super().__init__()
        
        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.log_manager = LogManager('Plotter', logging.DEBUG)
        self.log = self.log_manager.get_logger(component='PLOTTER')

        self.layout = QHBoxLayout()
        self.setLayout(self.layout)

        self.plotter_gui = ControlGUI(parent=self, app_mirror=self.app_mirror, x_label="Tempo decorrido (s)", y_label="Temperatura (°C)")
        self.sidebar = SidebarGUI(parent=self, app_mirror=self.app_mirror, control_gui=self.plotter_gui)

        self.layout.addWidget(self.sidebar, 1)
        self.layout.addWidget(self.plotter_gui, 4)
        
        self.hide_mode = False
   
    def toggle_hide_mode(self):
        if self.hide_mode:
            self.sidebar.show()
            self.layout.insertWidget(0, self.sidebar, 1)
            self.layout.setStretchFactor(self.sidebar, 1)
            self.layout.setStretchFactor(self.plotter_gui, 4)
        else:
            self.sidebar.hide()
            self.layout.setStretchFactor(self, 5)

        self.hide_mode = not self.hide_mode

    def toggle_select(self, param):
        self.plotter_gui.is_selected = param

