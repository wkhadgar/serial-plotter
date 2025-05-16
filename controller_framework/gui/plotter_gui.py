from collections.abc import Callable
from functools import partial
import os
import queue

from controller_framework.core.controller import Controller
import numpy as np
import pandas as pd

from PySide6 import QtCore
from PySide6.QtWidgets import ( QGroupBox, QFormLayout, QVBoxLayout, QWidget, QLabel, QScrollArea,
                             QPushButton, QHBoxLayout, QLineEdit, QGraphicsProxyWidget, QListWidget, QCheckBox )

import re

from .utils_gui import PlotWidget

class ControlGUI(QWidget):
    def __init__(self, *, parent, app_mirror, x_label: str, y_label: str):
        super().__init__(parent)

        self.parent = parent

        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.fullscreen = False
        
        self.init_timestamp = None
        self.plot_seconds = np.array([])
        self.actuator_data = [np.array([]) for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [np.array([]) for _ in range(self.app_mirror.num_sensors)]

        sensor_labels = [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]
        self.plot_views = ["ALL"] + sensor_labels
        self.current_mode = -1

        self.layout = QVBoxLayout()
        self.setLayout(self.layout)

        self.container = QWidget()
        self.container_layout = QVBoxLayout()
        self.container.setLayout(self.container_layout)
        
        self.plot_widget = PlotWidget(self.container_layout)
        self.plot_widget.plotter_plot()
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
        print(f"Salvando dados em {log_path}")
        
        datetime = pd.Timestamp.now()
        sensor_columns = [f"sensor_{i}" for i in range(self.app_mirror.num_sensors)]
        actuator_columns = [f"actuator_{i}" for i in range(self.app_mirror.num_actuators)]

        columns = ["timestamp", "seconds"] + sensor_columns + actuator_columns + ["target"]

        self.df = pd.DataFrame(columns=columns)
        
        self.log_file_path = log_path + f"log_{datetime.year}-{datetime.month}-{datetime.day}-{datetime.hour}-{datetime.minute}-{datetime.second}.csv"
        self.df.to_csv(self.log_file_path, index=False)
        
        self.update_delay = 15
        self.plot_timer = QtCore.QTimer()
        self.plot_timer.timeout.connect(partial(self.update_data, self.log_file_path))
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

    def update_data(self, log_f_path: str):
        data = None

        try:
            data = self.app_mirror.queue_to_gui.get(timeout=0.01)  # Espera até 10ms
        except queue.Empty:
            return

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
        else:
            return
        
        sensor_values = self.app_mirror.get_sensor_values()
        actuator_values = self.app_mirror.get_actuator_values()

        self.plot_seconds = np.append(self.plot_seconds, (self.last_timestamp - self.init_timestamp))
        for i in range(self.app_mirror.num_sensors):
            self.sensor_data[i] = np.append(self.sensor_data[i], sensor_values[i])
        for i in range(self.app_mirror.num_actuators):
            self.actuator_data[i] = np.append(self.actuator_data[i], actuator_values[i])

        target_str = '"' + " ".join(map(str, self.app_mirror.setpoints)) + '"'
        row = {
            "timestamp": self.last_timestamp,
            "seconds": f"{self.plot_seconds[-1]:.4f}",
            **{f"sensor_{i}": sensor_values[i] for i in range(self.app_mirror.num_sensors)},
            **{f"actuator_{i}": actuator_values[i] for i in range(self.app_mirror.num_actuators)},
            "target": target_str
        }

        with open(log_f_path, "a") as f:
            data = ",".join(map(str, row.values())) + "\n"
            f.write(data)

        if self.is_selected:
            self.update_plots()
        
    def update_plots(self):      
        view = self.plot_views[self.current_mode]
        match view:
            case "ALL":
                for i, sensor_data in enumerate(self.sensor_data):
                    self.plot_widget.update_curve(self.plot_seconds, sensor_data, 0, i)

                for i, actuator_data in enumerate(self.actuator_data):
                    self.plot_widget.update_curve(self.plot_seconds, actuator_data, 1, i)

            case _ if view in [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]:
                letters = [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]
                idx = letters.index(view)
                self.plot_widget.update_curve(self.plot_seconds, self.sensor_data[idx], 0, 0)
            case _:
                print(f"Visualização '{view}' não reconhecida.")

    def toggle_plot_view(self):
        self.current_mode = (self.current_mode + 1) % len(self.plot_views)
        view = self.plot_views[self.current_mode]

        self.plot_widget.clear()

        match view:
            case "ALL":
                self.plot_widget.plotter_dual_plott('Sensors', 'Actuators')

                for i, _ in enumerate(self.sensor_data):
                    sensor = list(self.app_mirror.sensor_vars.items())[i]

                    self.plot_widget.add_curve([0], [0], color=sensor[-1], plot_n=0)
                    self.plot_widget.add_legend(legenda=sensor[0], color=sensor[-1], plot_n=0)
                
                for i, _ in enumerate(self.actuator_data):
                    atuador = list(self.app_mirror.actuator_vars.items())[i]

                    self.plot_widget.add_curve([0], [0], color=atuador[-1], plot_n=1)
                    self.plot_widget.add_legend(legenda=atuador[0], color=atuador[-1], plot_n=1)
                    
            case _ if view in [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]:
                letters = [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]
                idx = letters.index(view)
                sensor = list(self.app_mirror.sensor_vars.items())[idx]

                self.plot_widget.plotter_single_plot(sensor[0])  
                self.plot_widget.add_curve([0], [0], color=sensor[-1])
                self.plot_widget.add_legend(legenda=sensor[0], color=sensor[-1], plot_n=0)
                    
            case _:
                print(f"Visualização '{view}' não reconhecida.")

    def reset_data(self):
        self.plot_seconds = np.array([])
        self.actuator_data = [np.array([]) for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [np.array([]) for _ in range(self.app_mirror.num_sensors)]
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
                    print(f"Entrada inválida para '{var_name}'")
            
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