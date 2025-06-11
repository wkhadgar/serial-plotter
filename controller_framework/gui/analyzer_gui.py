import os
from PySide6.QtWidgets import (
    QWidget, QPushButton, QVBoxLayout,
    QRadioButton, QButtonGroup, QListWidget, QHBoxLayout
)

import numpy as np
import pandas as pd
import scipy.signal as sig
import pyqtgraph as pg

from .utils_gui import PlotWidget


class SidebarAnalyzer(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.parent_gui = parent
        self.setFixedWidth(300)

        self.main_layout = QVBoxLayout()
        self.setLayout(self.main_layout)

        self.file_list = QListWidget()
        self.file_list.itemClicked.connect(self.file_selected)
        self.main_layout.addWidget(self.file_list)

        self.variable_list = QListWidget()
        self.main_layout.addWidget(self.variable_list)

        self.refresh_button = QPushButton("Atualizar Lista de Logs")
        self.refresh_button.clicked.connect(self.update_file_list)
        self.main_layout.addWidget(self.refresh_button)

        self.analyze_button = QPushButton("Analisar")
        self.analyze_button.setEnabled(False)
        self.analyze_button.clicked.connect(self.start_analysis)
        self.main_layout.addWidget(self.analyze_button)

        self.option_group = QButtonGroup(self)

        self.radio_closed = QRadioButton("Análise de malha fechada")
        self.radio_closed.setChecked(True)
        self.option_group.addButton(self.radio_closed)

        self.radio_open = QRadioButton("Análise de malha aberta")
        self.option_group.addButton(self.radio_open)

        self.main_layout.addWidget(self.radio_closed)
        self.main_layout.addWidget(self.radio_open)

        self.update_file_list()

    def update_file_list(self):
        self.file_list.clear()
        log_dir = "./temp_logs"
        if not os.path.exists(log_dir):
            os.makedirs(log_dir)

        for file in os.listdir(log_dir):
            if file.endswith(".csv"):
                self.file_list.addItem(file)

    def file_selected(self, item):
        self.variable_list.clear()
        self.selected_file = os.path.join("temp_logs", item.text())

        self.columns = pd.read_csv(self.selected_file).columns

        for string in self.columns:
            if string.startswith('sensor_'):
                string_formatada = string.replace('_', ' ').capitalize()
                self.variable_list.addItem(string_formatada)

        if self.variable_list.count() > 0:
            self.variable_list.setCurrentRow(0)
            self.analyze_button.setEnabled(True)
        else:
            self.analyze_button.setEnabled(False)

    def start_analysis(self):
        mode = "closed" if self.radio_closed.isChecked() else "open"
        selected_var = self.variable_list.selectedItems()[0] \
            .text() \
            .lower() \
            .replace(' ', '_')

        self.parent_gui.start_analysis(self.selected_file, selected_var, mode)


class PlotterAnalyzer(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.parent_gui = parent
        self.main_layout = QVBoxLayout()
        self.setLayout(self.main_layout)

        self.plot_widget: PlotWidget = PlotWidget(self.main_layout)

        self.df = None
        self.x_data = []
        self.y_data = []
        self.reference_lines = []

    def update_analyzer(self, x_data, y_data, mode):
        self.x_data = x_data
        self.y_data = y_data

        if mode == "closed":
            self.plot_widget.closed_loop_plot()
            self.closed_loop_analyzer()
        elif mode == "open":
            self.plot_widget.open_loop_plot()
            self.open_loop_analyzer()

    def closed_loop_analyzer(self):
        self.plot_widget.marker_closed.set_data(self.x_data, self.y_data)
        self.plot_widget.add_curve(self.x_data, self.y_data, 'blue', 1.5, 0)
        self.plot_widget.add_legend(text="Temperatura", color='blue', size=16)

        temp_inicial = self.y_data[0]
        h_line_init = pg.InfiniteLine(pos=temp_inicial, angle=0,
                                      pen=pg.mkPen("green", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))
        self.plot_widget.add_item(h_line_init)
        self.reference_lines.append(h_line_init)

        max_over_signal = np.max(self.y_data)
        h_line_max = pg.InfiniteLine(pos=max_over_signal, angle=0,
                                     pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))
        self.plot_widget.add_item(h_line_max)
        self.reference_lines.append(h_line_max)

        self.plot_widget.add_legend(size=16,text=f"Temperatura inicial ({temp_inicial:.2f}ºC)", color="green",
                                    style=pg.QtCore.Qt.PenStyle.DashLine)
        self.plot_widget.add_legend(size=16,text=f"Máximo Sobressinal ({max_over_signal:.2f}ºC)", color="red",
                                    style=pg.QtCore.Qt.PenStyle.DashLine)
        targets = np.array(self.parent_gui.df["target"])
        label_targets = ""
        last_target = targets[0] - 1
        for t in targets:
            if last_target != t:
                last_target = t
                label_targets += f"{t}ºC, "
        label_targets = label_targets[:-2]

        self.plot_widget.add_legend(size=16,text=f"Temperaturas desejadas\n[{label_targets}]", color="orange")
        self.plot_widget.add_curve(self.x_data, targets, 'orange', plot_n=1, width=2)
    def open_loop_analyzer(self):
        f_temps = np.array(sig.savgol_filter(self.y_data, 100, 3))
        f_temps = np.array(sig.savgol_filter(f_temps, 2000, 3))
        f_temps_dt = np.gradient(f_temps, self.x_data, edge_order=1)
        f_temps_dt = np.array(sig.savgol_filter(f_temps_dt, 2000, 3)) - 0.005

        max_dv_i = np.argmax(f_temps_dt)
        self.y_data = f_temps
        f_temps = np.array(sig.savgol_filter(f_temps, 20, 3))

        self.plot_widget.marker_temp.set_data(self.x_data, self.y_data)
        self.plot_widget.marker_derivative.set_data(self.x_data, f_temps_dt)

        self.plot_widget.add_curve(self.x_data, f_temps, 'blue', 1.5, 0)
        self.plot_widget.add_curve(self.x_data, f_temps_dt, 'blue', 1.5, 1)

        h_line_max = pg.InfiniteLine(pos=self.x_data[max_dv_i], angle=90,
                                     pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))

        self.plot_widget.add_item(h_line_max, 1)

        L = self.x_data[max_dv_i] - ((f_temps[max_dv_i] - f_temps[0]) / f_temps_dt[max_dv_i])
        T = ((f_temps[-1] - f_temps[0]) / f_temps_dt[max_dv_i])

        neighborhood = np.linspace(L, T + L)
        max_dv_scarter = pg.ScatterPlotItem(size=5, brush=pg.mkBrush("r"), pen=pg.mkPen(None), symbol='o')
        max_dv_scarter.setData(pos=[(self.x_data[max_dv_i], f_temps[max_dv_i])])

        self.plot_widget.add_curve(neighborhood,
                                   [(((x0 - self.x_data[max_dv_i]) * f_temps_dt[max_dv_i]) + f_temps[max_dv_i]) for x0
                                    in neighborhood],
                                   'red')

        v_initial_line = pg.InfiniteLine(pos=self.x_data[0], angle=90,
                                         pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))
        h_initial_line = pg.InfiniteLine(pos=f_temps[0], angle=0,
                                         pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))

        v_final_line = pg.InfiniteLine(pos=self.x_data[-1], angle=90,
                                       pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))
        h_final_line = pg.InfiniteLine(pos=f_temps[-1], angle=0,
                                       pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.PenStyle.DashLine))

        self.plot_widget.add_item(max_dv_scarter, 0)
        self.plot_widget.add_item(h_initial_line, 0)
        self.plot_widget.add_item(v_initial_line, 0)
        self.plot_widget.add_item(v_final_line, 0)
        self.plot_widget.add_item(h_final_line, 0)

        self.plot_widget.add_legend(text='Temperatura (ºC)', color='blue', plot_n=0, size=10)
        self.plot_widget.add_legend(text='Ponto de maior derivada', color='red', style='dot', plot_n=0, size=10)
        self.plot_widget.add_legend(text=f'Temperatura inicial ({f_temps[0]:.3f}ºC)', color='black',
                                    style=pg.QtCore.Qt.PenStyle.DashLine, plot_n=0, size=10)
        self.plot_widget.add_legend(text=f'Temperatura final ({f_temps[-1]:.3f}ºC)', color='black',
                                    style=pg.QtCore.Qt.PenStyle.DashLine, plot_n=0, size="15pt")

        self.plot_widget.plot_temp.setTitle(
            f"L = {L:.2f}, T (+L) = {T:.2f} (+{L:.2f}); Temperatura registrada", size="20pt", color="#000000")


class AnalyzerGUI(QWidget):
    def __init__(self, app_mirror):
        super().__init__()
        self.app_mirror = app_mirror
        self.selected_file = None

        self.setWindowTitle("Thermal Analyzer")
        self.setMinimumSize(900, 600)

        self.main_layout = QHBoxLayout()
        self.setLayout(self.main_layout)

        self.sidebar = SidebarAnalyzer(self)
        self.plotter_gui = PlotterAnalyzer(self)

        self.main_layout.addWidget(self.sidebar, 1)
        self.main_layout.addWidget(self.plotter_gui, 4)

    def start_analysis(self, file_path, sensor_variable, mode):
        if file_path:
            self.mode = mode
            self.df = pd.read_csv(file_path)
            x_data = [float(row["seconds"]) - self.df.iloc[0]["seconds"] for _, row in self.df.iterrows()]
            # temps = (self.df["temp_a"] + self.df["temp_b"]) / 2
            temps = self.df[sensor_variable]

            self.plotter_gui.update_analyzer(x_data=x_data, y_data=temps, mode=mode)
