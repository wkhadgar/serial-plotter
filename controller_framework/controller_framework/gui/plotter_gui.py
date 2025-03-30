from collections.abc import Callable
from functools import partial
import os

from controller_framework.core.controller import Controller
import numpy as np
import pandas as pd

from PySide6 import QtCore
from PySide6.QtWidgets import ( QGroupBox, QFormLayout, QVBoxLayout, QWidget, QLabel, QScrollArea,
                             QPushButton, QHBoxLayout, QLineEdit, QGraphicsProxyWidget, QListWidget, QCheckBox )
import pyqtgraph as pg
import scipy.signal as sig

class ControlGUI(QWidget):
    def __init__(self, *, parent, app_mirror, x_label: str, y_label: str):
        super().__init__(parent)

        self.parent = parent

        from controller_framework.core import AppManager
        assert isinstance(app_mirror, AppManager)
        self.app_mirror = app_mirror

        self.fullscreen = False
        
        self.init_timestamp = pd.Timestamp.now()
        self.plot_seconds = np.array([])
        self.duty_data = np.array([])
        self.temp_a_data = np.array([])
        self.temp_b_data = np.array([])
        self.temp_m_data = np.array([])
        
        self.plot_views = ["C", "A", "B", "M"]
        self.current_mode = -1

        self.win = pg.GraphicsLayoutWidget(show=False, title="Plotter Serial SC")
        layout = QVBoxLayout()
        layout.addWidget(self.win)
        self.setLayout(layout)
        
        self.current_setpoint_line = pg.InfiniteLine(pos=app_mirror.get_setpoint(), angle=0, movable=False,
                                                     label="Temperatura desejada [-]",
                                                     pen=pg.mkPen("orange", width=2))

        self.plot_combined = self.win.addPlot(title="Temperatura A e B", col=0, row=0)
        self.plot_combined.setLabel('bottom', x_label)
        self.plot_combined.setLabel('left', y_label)
        self.plot_combined.showGrid(x=True, y=True, alpha=0.5)
        self.combined_leg = self.plot_combined.addLegend()
        self.curve_a_combined = self.plot_combined.plot(pen="c", name="Temperatura A")
        self.curve_b_combined = self.plot_combined.plot(pen="g", name="Temperatura B")
        self.curve_m_combined = self.plot_combined.plot(pen="y", name="Temperatura média")
        self.curve_a_combined_label = self.combined_leg.getLabel(self.curve_a_combined)
        self.curve_b_combined_label = self.combined_leg.getLabel(self.curve_b_combined)
        self.curve_m_combined_label = self.combined_leg.getLabel(self.curve_m_combined)

        self.plot_d = self.win.addPlot(title="Duty", col=0, row=1)
        self.plot_d.addItem(pg.InfiniteLine(pos=100, angle=0, movable=False, pen=pg.mkPen("red", width=2)))
        self.plot_d.addItem(pg.InfiniteLine(pos=0, angle=0, movable=False, pen=pg.mkPen("white", width=2)))
        self.plot_d.addItem(pg.InfiniteLine(pos=-100, angle=0, movable=False, pen=pg.mkPen("cyan", width=2)))
        self.plot_d.setLabel('bottom', x_label)
        self.plot_d.setLabel('left', "Duty Cycle (%)")
        self.plot_d.showGrid(x=True, y=True, alpha=0.5)
        self.d_leg = self.plot_d.addLegend()
        self.curve_d = self.plot_d.plot(pen="yellow", name="Duty")
        self.curve_d_label = self.d_leg.getLabel(self.curve_d)

        # Plot individual dos dados
        self.plot_a = self.win.addPlot(title="Temperatura A", col=1, row=0)
        self.plot_a.setLabel('bottom', x_label)
        self.plot_a.setLabel('left', y_label)
        self.plot_a.showGrid(x=True, y=True, alpha=0.5)
        self.a_leg = self.plot_a.addLegend()
        self.curve_a = self.plot_a.plot(pen="c", name="Temperatura A")
        self.curve_a_label = self.a_leg.getLabel(self.curve_a)

        self.plot_b = self.win.addPlot(title="Temperatura B", col=1, row=1)
        self.plot_b.setLabel('bottom', x_label)
        self.plot_b.setLabel('left', y_label)
        self.plot_b.showGrid(x=True, y=True, alpha=0.5)
        self.b_leg = self.plot_b.addLegend()
        self.curve_b = self.plot_b.plot(pen="g", name="Temperatura B")
        self.curve_b_label = self.b_leg.getLabel(self.curve_b)

        self.plot_m = self.win.addPlot(title="Temperatura Média", col=0, row=3)
        self.plot_m.setLabel('bottom', x_label)
        self.plot_m.setLabel('left', y_label)
        self.plot_m.showGrid(x=True, y=True, alpha=0.5)
        self.m_leg = self.plot_m.addLegend()
        self.curve_m = self.plot_m.plot(pen="y", name="Temperatura Média")
        self.curve_m_label = self.m_leg.getLabel(self.curve_m)

        self.temp_input = QLineEdit()
        self.temp_input.setPlaceholderText("Defina a temperatura desejada (°C). [Entre 20°C e 50°C]...")
        self.temp_input.setAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)

        self.proxy = QGraphicsProxyWidget()
        self.proxy.setWidget(self.temp_input)
        self.win.addItem(self.proxy, col=0, row=2)

        self.temp_input.returnPressed.connect(self.__on_return_pressed)

        self.win.keyPressEvent = partial(self.key_press_handle, self.win.keyPressEvent)
        
        log_path = "./temp_logs/"
        if not os.path.exists(log_path):
            os.makedirs(log_path)
        print(f"Salvando dados em {log_path}")
        
        datetime = pd.Timestamp.now()
        df = pd.DataFrame(columns=["timestamp", "seconds", "temp_a", "temp_b", "duty", "target"])
        
        log_file_path = log_path + f"log_{datetime.year}-{datetime.month}-{datetime.day}-{datetime.hour}-{datetime.minute}-{datetime.second}.csv"
        df.to_csv(log_file_path, index=False)
        
        self.update_delay = 15
        self.plot_timer = QtCore.QTimer()
        self.plot_timer.timeout.connect(partial(self.update_plots, log_file_path))
        self.plot_timer.start(self.update_delay)
        self.toggle_plot_view()

    def key_press_handle(self, super_press_handler: Callable, ev):
        if self.temp_input.hasFocus():
            super_press_handler(ev)
        else:
            if ev.key() == QtCore.Qt.Key.Key_Space:
                self.toggle_plot_view()
            elif ev.key() == QtCore.Qt.Key.Key_Escape:
                super_press_handler(ev)
            elif ev.key() == QtCore.Qt.Key.Key_F:
                super_press_handler(ev)

    def __on_return_pressed(self):
        setpoint = float(self.temp_input.text())

        self.app_mirror.update_setpoint(setpoint)

        self.parent.command_triggered.emit("update_setpoint", setpoint)
        
        self.temp_input.clear()
        self.update_setpoint_label()

    def update_setpoint_label(self):
        self.current_setpoint_line.setValue(self.app_mirror.get_setpoint())
        self.current_setpoint_line.label.setText(f"Temperatura desejada [{self.app_mirror.get_setpoint()}°C]")
        self.current_setpoint_line.update()

    def update_plots(self, log_f_path: str):
        temp_a, temp_b, duty = 0,0,0

        try:
            data = self.app_mirror.gui_data_queue.get(timeout=0.01)  # Espera até 10ms
            temp_a, temp_b, duty, _ = data

            self.update_setpoint_label()

            timestamp = pd.Timestamp.now()
            self.plot_seconds = np.append(self.plot_seconds, (timestamp - self.init_timestamp).total_seconds())
            self.duty_data = np.append(self.duty_data, duty)
            self.temp_a_data = np.append(self.temp_a_data, temp_a)
            self.temp_b_data = np.append(self.temp_b_data, temp_b)
            self.temp_m_data = np.append(self.temp_m_data, (temp_b + temp_a) / 2)

            with open(log_f_path, "a") as f:
                f.write(
                    f"{timestamp.strftime('%H:%M:%S')},"
                    f"{self.plot_seconds[-1]:.4f},"
                    f"{temp_a},"
                    f"{temp_b},"
                    f"{duty},"
                    f"{self.app_mirror.setpoint}\n")

            if len(self.temp_m_data) > 400:
                f_temps = np.array(sig.savgol_filter(self.temp_m_data, int(len(self.temp_m_data) * 0.02), 6))
            else:
                f_temps = self.temp_m_data.copy()

            match self.plot_views[self.current_mode]:
                case "C":
                    self.curve_a_combined.setData(self.plot_seconds, self.temp_a_data)
                    self.curve_b_combined.setData(self.plot_seconds, self.temp_b_data)
                    self.curve_m_combined.setData(self.plot_seconds, f_temps)
                    self.curve_d.setData(self.plot_seconds, self.duty_data)

                    self.curve_a_combined_label.setText(f"Temperatura A: {temp_a}")
                    self.curve_b_combined_label.setText(f"Temperatura B: {temp_b}")
                    self.curve_m_combined_label.setText(f"Temperatura Média: {(temp_b + temp_a) / 2:.2f}")
                    self.curve_d_label.setText(f"Duty Cycle (%): {duty}")
                case "A":
                    self.curve_a.setData(self.plot_seconds, self.temp_a_data)
                    self.curve_a_label.setText(f"Temperatura A: {temp_a}")
                case "B":
                    self.curve_b.setData(self.plot_seconds, self.temp_b_data)
                    self.curve_b_label.setText(f"Temperatura B: {temp_b}")
                case "M":
                    self.curve_m.setData(self.plot_seconds, f_temps)
                    self.curve_m_label.setText(f"Temperatura Média: {(temp_b + temp_a) / 2:.2f}")
        except:
            pass

    def toggle_plot_view(self):
        self.current_mode = (self.current_mode + 1) % len(self.plot_views)
        match self.plot_views[self.current_mode]:
            case "C":
                self.plot_m.removeItem(self.current_setpoint_line)
                self.plot_combined.addItem(self.current_setpoint_line)
                self.plot_combined.show()
                self.plot_a.hide()
                self.plot_b.hide()
                self.plot_m.hide()
                self.plot_d.show()
                self.proxy.show()

            case "A":
                self.plot_combined.hide()
                self.plot_combined.removeItem(self.current_setpoint_line)
                self.plot_a.addItem(self.current_setpoint_line)
                self.plot_a.show()
                self.plot_b.hide()
                self.plot_m.hide()
                self.plot_d.hide()
                self.proxy.hide()
            case "B":
                self.plot_combined.hide()
                self.plot_a.hide()
                self.plot_a.removeItem(self.current_setpoint_line)
                self.plot_b.addItem(self.current_setpoint_line)
                self.plot_b.show()
                self.plot_m.hide()
                self.plot_d.hide()
                self.proxy.hide()
            case "M":
                self.plot_combined.hide()
                self.plot_a.hide()
                self.plot_b.hide()
                self.plot_b.removeItem(self.current_setpoint_line)
                self.plot_m.addItem(self.current_setpoint_line)
                self.plot_m.show()
                self.plot_d.hide()
                self.proxy.hide()

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
        self.settings_group.setStyleSheet("QGroupBox { background: white; font-size: 16px; font-weight: bold; }")
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
                    self.parent.command_triggered.emit("update_variable", [self.current_control.label, var_name, new_value])
                    
                except ValueError:
                    print(f"Entrada inválida para '{var_name}'")
            
            if(self.app_mirror.running_instance == self.current_control):
                self.app_mirror.update_setpoint(self.current_control.setpoint)
                self.parent.command_triggered.emit("update_setpoint", self.current_control.setpoint)
                self.control_gui.update_setpoint_label()
                    
    def activate_control(self):
        current_control = self.control_list.currentItem()
        
        if(current_control != None):
            current_control_label = current_control.text()
            self.app_mirror.update_setpoint(self.current_control.setpoint)

            self.app_mirror.running_instance = self.app_mirror.get_instance(current_control_label)
            self.parent.command_triggered.emit("start_controller", current_control_label)
            self.parent.command_triggered.emit("update_setpoint", self.current_control.setpoint)

            self.control_gui.update_setpoint_label()
            
            self.btn_deactivate_control.setEnabled(True)
    
    def deactivate_control(self):
        self.app_mirror.stop_controller()
        
        self.btn_deactivate_control.setEnabled(False)
        # self.btn_activate_control.

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
