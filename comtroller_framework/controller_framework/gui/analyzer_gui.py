import os
from PyQt5.QtWidgets import (
    QWidget, QPushButton, QVBoxLayout, QFileDialog,
    QRadioButton, QButtonGroup, QListWidget, QHBoxLayout
)

import numpy as np
import pandas as pd
import scipy.signal as sig
import pyqtgraph as pg

class SidebarAnalyzer(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.parent_gui = parent
        self.setFixedWidth(300)

        self.layout = QVBoxLayout()
        self.setLayout(self.layout)

        self.file_list = QListWidget()
        self.file_list.itemClicked.connect(self.file_selected)
        self.layout.addWidget(self.file_list)

        self.refresh_button = QPushButton("Atualizar Lista de Logs")
        self.refresh_button.clicked.connect(self.update_file_list)
        self.layout.addWidget(self.refresh_button)

        self.analyze_button = QPushButton("Analisar")
        self.analyze_button.setEnabled(False)
        self.analyze_button.clicked.connect(self.start_analysis)
        self.layout.addWidget(self.analyze_button)

        self.option_group = QButtonGroup(self)

        self.radio_closed = QRadioButton("Análise de malha fechada")
        self.radio_closed.setChecked(True)
        self.option_group.addButton(self.radio_closed)

        self.radio_open = QRadioButton("Análise de malha aberta")
        self.option_group.addButton(self.radio_open)

        self.layout.addWidget(self.radio_closed)
        self.layout.addWidget(self.radio_open)

        self.update_file_list()

    def update_file_list(self):
        self.file_list.clear()
        log_dir = "logs"
        if not os.path.exists(log_dir):
            os.makedirs(log_dir)

        for file in os.listdir(log_dir):
            if file.endswith(".csv"):
                self.file_list.addItem(file)

    def file_selected(self, item):
        self.parent_gui.selected_file = os.path.join("logs", item.text())
        self.analyze_button.setEnabled(True)

    def start_analysis(self):
        mode = "closed" if self.radio_closed.isChecked() else "open"
        self.parent_gui.start_analysis(self.parent_gui.selected_file, mode)

class PlotterAnalyzer(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.layout = QVBoxLayout()
        self.setLayout(self.layout)
        self.parent = parent

        self.plot_widget = pg.GraphicsLayoutWidget()
        self.layout.addWidget(self.plot_widget)

        self.plot = self.plot_widget.addPlot(title="Temperatura ao longo do tempo")
        self.plot.setLabel('left', "Temperatura (°C)")
        self.plot.setLabel('bottom', "Tempo (s)")
        self.plot.showGrid(x=True, y=True, alpha=0.3)
        self.curve = self.plot.plot(pen='y')
        
        self.plot_widget.setBackground("w")
        self.plot.getAxis("left").setPen(pg.mkPen("black"))
        self.plot.getAxis("bottom").setPen(pg.mkPen("black"))
        self.plot.showGrid(x=True, y=True, alpha=0.2)
        
        self.curve = self.plot.plot(pen=pg.mkPen("b", width=1))
        self.curve_temp = self.plot.plot(pen=pg.mkPen("b", width=1))
        self.curve_derivative = self.plot.plot(pen=pg.mkPen("b", width=1))
        
        self.df = None
        self.reference_lines = []
        self.x_data = []
        self.y_data = []
        
        self.threshold = 10
        self.scatter = pg.ScatterPlotItem()
        self.plot.addItem(self.scatter)
        self.marker = pg.ScatterPlotItem(size=10, brush=pg.mkBrush("r"))
        self.plot.addItem(self.marker)
        self.plot.scene().sigMouseMoved.connect(self.on_mouse_moved)

    def update_plot(self, x_data, y_data, mode):
        self.plot.clear()
        
        self.x_data = x_data
        self.y_data = y_data

        if mode == "closed":
            self.plot.setTitle("Análise de Malha Fechada")
            self.closed_loop_plot()
        else:
            self.plot.setTitle("Análise de Malha Aberta")
            self.open_loop_plot()
    
    def closed_loop_plot(self):
        self.curve = self.plot.plot(self.x_data, self.y_data, pen = pg.mkPen("black", width=1.5))
        
        temp_inicial = self.y_data[0]
        h_line_init = pg.InfiniteLine(pos=temp_inicial, angle=0, pen=pg.mkPen("green", width=2, style=pg.QtCore.Qt.DashLine))
        self.plot.addItem(h_line_init)
        self.reference_lines.append(h_line_init)

        max_over_signal = np.max(self.y_data)
        h_line_max = pg.InfiniteLine(pos=max_over_signal, angle=0, pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.DashLine))
        self.plot.addItem(h_line_max)
        self.reference_lines.append(h_line_max)

        self.plot.addLegend()
        self.plot.plot([], [], pen="green", name=f"Temperatura inicial ({temp_inicial:.2f}ºC)")
        self.plot.plot([], [], pen="red", name=f"Máximo Sobressinal ({max_over_signal:.2f}ºC)")
        
        targets = np.array(self.parent.df["target"])
        label_targets = ""
        last_target = targets[0] - 1
        for t in targets:
            if last_target != t:
                last_target = t
                label_targets += f"{t}ºC, "
        label_targets = label_targets[:-2]
        
        self.plot.addLegend()
        self.plot.plot([], [], pen="gray", name=f"Temperaturas desejadas\n[{label_targets}]")
        
        self.curve2 = self.plot.plot(self.x_data, targets, pen=pg.mkPen("blue", width=1))
        
        points = [{'pos': (x, y), 'data': (x, y)} for x, y in zip(self.x_data, self.y_data)]
        self.scatter.setData(points)
        
    def open_loop_plot(self):
        temp_a_original = np.array(self.parent.df["temp_a"])
        temp_b_original = np.array(self.parent.df["temp_b"])

        x = [float(row.iloc[1]) - self.parent.df.at[self.parent.df.index[0], 'seconds'] for (_, row) in self.parent.df.iterrows()]
        temps = (temp_a_original + temp_b_original) / 2

        f_temps = np.array(sig.savgol_filter(temps, int(len(x) * 0.02), 6))
        f_temps_dt = np.gradient(f_temps, x, edge_order=1)

        max_dv_i = np.argmax(f_temps_dt)

        self.curve_temp.setData(x, f_temps)
        self.curve_derivative.setData(x, f_temps_dt)

        self.scatter_temp.setData([x[max_dv_i]], [f_temps[max_dv_i]])
        self.scatter_derivative.setData([x[max_dv_i]], [f_temps_dt[max_dv_i]])

        h_line_max = pg.InfiniteLine(pos=x[max_dv_i], angle=90, pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.DashLine))
        self.plot_derivative.addItem(h_line_max)
        self.plot_temp.addItem(h_line_max)

        L = x[max_dv_i] - ((f_temps[max_dv_i] - f_temps[0]) / f_temps_dt[max_dv_i])
        T = ((f_temps[-1] - f_temps[0]) / f_temps_dt[max_dv_i])

        self.plot_temp.setTitle(f"Temperatura registrada - L = {L:.2f}, T (+L) = {T:.2f} (+{L:.2f})")
    
    def on_mouse_moved(self, event):
        pos = self.plot.vb.mapSceneToView(event)

        if len(self.x_data) == 0:
            return

        x_array = np.array(self.x_data, dtype=np.float64)

        idx = np.abs(x_array - pos.x()).argmin()
        x_val, y_val = self.x_data[idx], self.y_data[idx]
        
        distance = np.sqrt((pos.x() - x_val) ** 2 + (pos.y() - y_val) ** 2) * 100
        print(x_val, y_val, distance)

        if distance > self.threshold:
            self.marker.clear()
            self.setToolTip("")
            return

        self.marker.setData([x_val], [y_val])

        tooltip = f"Tempo: {x_val:.2f}s\nTemp: {y_val:.2f}°C"
        self.setToolTip(tooltip)
        self.repaint()
        
class AnalyzerGUI(QWidget):
    def __init__(self, app_manager):
        super().__init__()
        self.app_manager = app_manager
        self.selected_file = None

        self.setWindowTitle("Thermal Analyzer")
        self.setMinimumSize(900, 600)

        self.main_layout = QHBoxLayout()
        self.setLayout(self.main_layout)

        self.sidebar = SidebarAnalyzer(self)
        self.plotter = PlotterAnalyzer(self)

        self.main_layout.addWidget(self.sidebar)
        self.main_layout.addWidget(self.plotter, 1)

    def start_analysis(self, file_path, mode):
        if file_path:
            self.df = pd.read_csv(file_path)
            x_data = [float(row["seconds"]) - self.df.iloc[0]["seconds"] for _, row in self.df.iterrows()]
            temps = (self.df["temp_a"] + self.df["temp_b"]) / 2

            if mode == "open":
                temps = np.array(sig.savgol_filter(temps, int(len(x_data) * 0.02), 6))

            self.plotter.update_plot(x_data, temps, mode)