from collections.abc import Callable
from functools import partial
import logging
import os
import queue

from controller_framework.core.controller import Controller
import numpy as np
import pandas as pd

from PySide6 import QtCore
from PySide6.QtWidgets import (QGroupBox, QFormLayout, QVBoxLayout, QWidget, QLabel, QScrollArea,
                               QPushButton, QHBoxLayout, QLineEdit, QListWidget, QCheckBox)

import re

from controller_framework.core.logmanager import LogManager

from .utils_gui import PlotWidget, Mode


class DataWorker(QtCore.QObject):
    dataReady = QtCore.Signal()

    def __init__(self, fn: Callable, interval_ms: int = 30):
        super().__init__()
        self.fn = fn
        self.interval_ms = interval_ms
        self._timer: QtCore.QTimer | None = None

    @QtCore.Slot()
    def start(self):
        self._timer = QtCore.QTimer(self)
        self._timer.setTimerType(QtCore.Qt.TimerType.PreciseTimer)
        self._timer.timeout.connect(self._on_timeout)
        self._timer.start(self.interval_ms)

    @QtCore.Slot()
    def _on_timeout(self):
        try:
            self.fn()
            self.dataReady.emit()
        except Exception as e:
            print(f"[DataWorker] Erro em fn(): {e}")

    @QtCore.Slot()
    def stop(self):
        if self._timer is not None and self._timer.isActive():
            self._timer.stop()


class ControlGUI(QWidget):
    stopWorker = QtCore.Signal()

    def __init__(self, *, parent, app_mirror, x_label: str, y_label: str):
        super().__init__(parent)

        self.parent_gui = parent

        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.init_timestamp = None
        self.plot_seconds = []
        self.actuator_data = [[] for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [[] for _ in range(self.app_mirror.num_sensors)]

        self.sensor_labels = [chr(ord("A") + i) for i in range(self.app_mirror.num_sensors)]

        self.plot_views = ["ALL"] + self.sensor_labels
        self.current_mode = -1

        self.main_layout = QVBoxLayout()
        self.setLayout(self.main_layout)

        self.container = QWidget()
        self.container_layout = QVBoxLayout()
        self.container.setLayout(self.container_layout)

        self.plot_widget = PlotWidget(self.container_layout, Mode.PLOTTER)
        self.toggle_plot_view()

        self.temp_input = QLineEdit()
        self.temp_input.setPlaceholderText("Defina os setpoints desejados [separados por vírgulas ou espaços]")
        self.temp_input.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.container_layout.addWidget(self.temp_input)

        self.temp_input.returnPressed.connect(self.__on_return_pressed)
        self.main_layout.addWidget(self.container)

        self.container.keyPressEvent = partial(self.key_press_handle, self.container.keyPressEvent)

        log_path = "./temp_logs/"
        if not os.path.exists(log_path):
            os.makedirs(log_path)
        self.parent_gui.log.debug("Salvando dados em %s", log_path, extra={'method': 'init'})

        datetime = pd.Timestamp.now()
        sensor_columns = [f"sensor_{i}" for i in range(self.app_mirror.num_sensors)]
        actuator_columns = [f"actuator_{i}" for i in range(self.app_mirror.num_actuators)]

        columns = ["seconds"] + sensor_columns + actuator_columns + ["target"]

        self.df = pd.DataFrame(columns=columns)

        self.log_file_path = log_path + f"log_{datetime.year}-{datetime.month}-{datetime.day}-{datetime.hour}-{datetime.minute}-{datetime.second}.csv"
        self.df.to_csv(self.log_file_path, index=False)

        self.sensors_cache: list[list] = [[]]
        self.actuators_cache: list[list] = [[]]
        self.is_selected = True
        self.last_update = 0
        self.last_update_plot = 0

        self._data_thread = QtCore.QThread(self)
        self._data_worker = DataWorker(self.update_data, interval_ms=30)
        self._data_worker.moveToThread(self._data_thread)
        self._data_thread.started.connect(self._data_worker.start)
        self._data_thread.finished.connect(self._data_worker.stop)
        self.stopWorker.connect(self._data_worker.stop)
        self._data_worker.dataReady.connect(
            self.update_plots,
            QtCore.Qt.ConnectionType.QueuedConnection
        )
        self._data_thread.start()

        self.mutex = QtCore.QMutex()

    def __on_return_pressed(self):
        setpoint_string = self.temp_input.text()
        self.temp_input.clear()

        setpoint = re.split(r'[,\s]+', setpoint_string.strip())

        self.app_mirror.update_setpoint(setpoint)

        self.parent_gui.command_triggered.emit("update_setpoint", {"value": setpoint})

    # def update_setpoint_label(self):
    #     self.current_setpoint_line.setValue(self.app_mirror.get_setpoint())
    #     self.current_setpoint_line.label.setText(f"Temperatura desejada [{self.app_mirror.get_setpoint()}°C]")
    #     self.current_setpoint_line.update()
    #     try:
    #         return self.app_mirror.queue_to_gui.get(timeout=timeout)
    #     except queue.Empty:
    #         return None

    def __process_message(self, data):
        command = data.get('type')
        payload = data.get('payload')

        if command == "full_state":
            self.sensors_cache = payload.get('sensors')
            self.actuators_cache = payload.get('actuators')
            setpoints = payload.get('setpoints')
            running_instance = payload.get('running_instance')
            control_instances_data = payload.get('control_instances')
            self.cache_timestamp = payload.get('cache_timestamp')

            if self.init_timestamp is None:
                self.init_timestamp = self.cache_timestamp[0]

            self.app_mirror.running_instance = running_instance
            self.app_mirror.control_instances = control_instances_data
            self.app_mirror.update_setpoint(setpoints)

            return True
        else:
            return False

    def __append_plot_data(self):
        self.plot_seconds.extend([val - self.init_timestamp for val in self.cache_timestamp])

        for lista, cache_values in zip(self.sensor_data, self.sensors_cache):
            lista.extend(cache_values)

        for lista, cache_values in zip(self.actuator_data, self.actuators_cache):
            lista.extend(cache_values)

    def __write_csv(self):
        target_str = '"' + " ".join(map(str, self.app_mirror.setpoints)) + '"'
        for j in range(len(self.cache_timestamp)):
            rows = [{
                "seconds": f"{self.cache_timestamp[j] - self.init_timestamp:.4f}",

                **{f"sensor_{i}": f"{self.sensors_cache[i][j]:.4f}"
                   for i in range(self.app_mirror.num_sensors)},

                **{f"actuator_{i}": f"{self.actuators_cache[i][j]:.4f}"
                   for i in range(self.app_mirror.num_actuators)},

                "target": target_str
            }]

        with open(self.log_file_path, "a") as f:
            for row in rows:
                data = ",".join(map(str, row.values())) + "\n"
                f.write(data)

    def update_data(self):
        while True:
            try:
                data = self.app_mirror.queue_to_gui.get_nowait()
                if data is None:
                    break

                if not self.__process_message(data):
                    continue

                with QtCore.QMutexLocker(self.mutex):
                    self.__append_plot_data()

                self.__write_csv()

            except queue.Empty:
                break

    def update_plots(self):
        if not self.is_selected:
            return

        view = self.plot_views[self.current_mode]

        with QtCore.QMutexLocker(self.mutex):
            sensor_data_np = np.array(self.sensor_data)
            actuator_data_np = np.array(self.actuator_data)
            plot_seconds_np = np.array(self.plot_seconds)

        match view:
            case "ALL":
                for (i, sensor_data), (var_name, props) in zip(enumerate(sensor_data_np),
                                                               self.app_mirror.sensor_vars.items()):
                    self.plot_widget.update_curve(plot_seconds_np, sensor_data, 0, i)

                    legenda = f"{var_name}: {sensor_data[-1]:.4f} {props['unit']}"
                    self.plot_widget.update_legend(text=legenda, plot_n=0, idx=i)

                for (i, actuator_data), (var_name, props) in zip(enumerate(actuator_data_np),
                                                                 self.app_mirror.actuator_vars.items()):
                    self.plot_widget.update_curve(plot_seconds_np, actuator_data, 1, i)

                    legenda = f"{var_name}: {actuator_data[-1]:.4f} {props['unit']}"
                    self.plot_widget.update_legend(text=legenda, plot_n=1, idx=i)

            case _ if view in self.sensor_labels:
                letters = self.sensor_labels
                idx = letters.index(view)

                sensor_data = np.array(self.sensor_data[idx])
                self.plot_widget.update_curve(plot_seconds_np, sensor_data, plot_n=0, curve_n=0)

                var_name, props = list(self.app_mirror.sensor_vars.items())[idx]
                legenda = f"{var_name}: {sensor_data[-1]:.4f} {props['unit']}"
                self.plot_widget.update_legend(text=legenda, plot_n=0, idx=0)
            case _:
                self.parent_gui.log.warning("Visualização '%s' não reconhecida.", view, extra={'method': 'update plot'})

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
                self.parent_gui.log.warning("Visualização '%s' não reconhecida.", view, extra={'method': 'toggle plot'})

    def reset_data(self):
        self.plot_seconds = []
        self.actuator_data = [[] for _ in range(self.app_mirror.num_actuators)]
        self.sensor_data = [[] for _ in range(self.app_mirror.num_sensors)]
        self.init_timestamp = None

        self.df = pd.DataFrame(columns=self.df.columns)
        self.df.to_csv(self.log_file_path, index=False)

    def close(self):
        self.stopWorker.emit()
        self._data_thread.quit()
        self._data_thread.wait(1000)
        return True

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

        self.parent_gui = parent

        self.control_gui = control_gui
        self.current_control = None
        self.input_fields = {}

        self.main_layout = QVBoxLayout()
        self.setLayout(self.main_layout)

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

        self.main_layout.addWidget(self.controls_group)

        self.hbox = QHBoxLayout()
        self.hbox.addWidget(self.btn_activate_control)
        self.hbox.addWidget(self.btn_deactivate_control)

        self.main_layout.addLayout(self.hbox)

        self.settings_group = QGroupBox("Configurações do Controle")
        self.settings_group.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.settings_layout = QFormLayout()
        self.settings_group.setLayout(self.settings_layout)

        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFixedHeight(300)
        self.scroll_area.setWidget(self.settings_group)

        self.btn_update_settings = QPushButton("Atualizar Configurações")
        self.main_layout.addWidget(self.scroll_area)
        self.main_layout.addWidget(self.btn_update_settings)

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
            self.current_control: Controller = self.app_mirror.control_instances[control_name]

            for i in reversed(range(self.settings_layout.count())):
                self.settings_layout.itemAt(i).widget().deleteLater()
            self.input_fields.clear()

            for var_name, var_data in self.current_control.configurable_vars.items():
                value = var_data['value']
                var_type = var_data['type']

                label = QLabel(f"{var_name}")
                label.setStyleSheet("QLabel { font-size: 14px; }")

                if var_type is bool:
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

                    self.parent_gui.command_triggered.emit(command, payload)

                except ValueError:
                    self.parent_gui.log.error("Entrada inválida para '%s'", var_name,
                                              extra={'method': 'update control'})

            if (
                    self.app_mirror.running_instance and self.app_mirror.running_instance.label == self.current_control.label):
                self.app_mirror.update_setpoint(self.current_control.setpoints)
                self.parent_gui.command_triggered.emit("update_setpoint", {"value": self.current_control.setpoints})

    def activate_control(self):
        current_control = self.control_list.currentItem()

        if (current_control is not None):
            current_control_label = current_control.text()
            self.app_mirror.update_setpoint(self.current_control.setpoints)

            self.app_mirror.running_instance = self.app_mirror.get_instance(current_control_label)
            self.parent_gui.command_triggered.emit("start_controller", {"control_name": current_control_label})
            self.parent_gui.command_triggered.emit("update_setpoint", {"value": self.current_control.setpoints})

            # self.control_gui.update_setpoint_label()

            self.btn_deactivate_control.setEnabled(True)

    def deactivate_control(self):
        self.app_mirror.stop_controller()
        self.parent_gui.command_triggered.emit("stop_controller", {})

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

        self.main_layout = QHBoxLayout()
        self.setLayout(self.main_layout)

        self.plotter_gui = ControlGUI(parent=self, app_mirror=self.app_mirror, x_label="Tempo decorrido (s)",
                                      y_label="Temperatura (°C)")
        self.sidebar = SidebarGUI(parent=self, app_mirror=self.app_mirror, control_gui=self.plotter_gui)

        self.main_layout.addWidget(self.sidebar, 1)
        self.main_layout.addWidget(self.plotter_gui, 4)

        self.hide_mode = False

    def toggle_hide_mode(self):
        if self.hide_mode:
            self.sidebar.show()
            self.main_layout.insertWidget(0, self.sidebar, 1)
            self.main_layout.setStretchFactor(self.sidebar, 1)
            self.main_layout.setStretchFactor(self.plotter_gui, 4)
        else:
            self.sidebar.hide()
            self.main_layout.setStretchFactor(self, 5)

        self.hide_mode = not self.hide_mode

    def toggle_select(self, param):
        self.plotter_gui.is_selected = param

    def close(self):
        self.plotter_gui.close()

        return True
