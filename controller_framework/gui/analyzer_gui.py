import os
from PySide6.QtWidgets import (
    QWidget, QPushButton, QVBoxLayout,
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
        log_dir = "./temp_logs"
        if not os.path.exists(log_dir):
            os.makedirs(log_dir)

        for file in os.listdir(log_dir):
            if file.endswith(".csv"):
                self.file_list.addItem(file)

    def file_selected(self, item):
        self.parent_gui.selected_file = os.path.join("temp_logs", item.text())
        self.analyze_button.setEnabled(True)

    def start_analysis(self):
        mode = "closed" if self.radio_closed.isChecked() else "open"
        self.parent_gui.start_analysis(self.parent_gui.selected_file, mode)

class MarkerPlot:
    def __init__(self, plot, x_data=None, y_data=None, threshold=10):
        self.plot = plot
        self.x_data = np.array(x_data, dtype=np.float64) if x_data is not None else np.array([])
        self.y_data = np.array(y_data, dtype=np.float64) if y_data is not None else np.array([])
        self.threshold = threshold

        self.marker = pg.ScatterPlotItem(size=5, brush=pg.mkBrush("r"), pen=pg.mkPen(None), symbol='o')
        self.marker.setZValue(10)
        self.marker.setData([], [])
        self.plot.addItem(self.marker)

        self.plot.scene().sigMouseMoved.connect(self.on_mouse_moved)

    def set_data(self, x_data, y_data):
        self.x_data = np.array(x_data, dtype=np.float64)
        self.y_data = np.array(y_data, dtype=np.float64)

    def on_mouse_moved(self, event):
        if self.x_data.size == 0:
            return

        pos = self.plot.vb.mapSceneToView(event)
        idx = np.abs(self.x_data - pos.x()).argmin()
        x_val, y_val = self.x_data[idx], self.y_data[idx]
        dist = np.hypot(pos.x() - x_val, pos.y() - y_val) * 100

        if dist > self.threshold:
            self.marker.setData([], [])
            self.marker.setToolTip('')
            return
        
        self.marker.setData([x_val], [y_val])
        tooltip = f"Tempo: {x_val:.4f}s\nTemp: {y_val:.4f}°C"
        self.marker.setToolTip(tooltip)

class PlotWidget:
    def __init__(self, layout, mode = None):
        self.plot_widget = pg.GraphicsLayoutWidget()
        self.plot_widget.setBackground("w")

        layout.addWidget(self.plot_widget)

        self.curves = []
        self.curves_dt = []

    def clear(self):
        self.plot_widget.clear()

    def closed_loop_plot(self):
        self.mode = "closed"

        self.plot = self.plot_widget.addPlot(title="Análise de Malha Fechada")
        self.plot.setLabel('left', "Temperatura (°C)")
        self.plot.setLabel('bottom', "Tempo (s)")
        self.plot.getAxis("left").setPen(pg.mkPen("black"))
        self.plot.getAxis("bottom").setPen(pg.mkPen("black"))
        self.plot.showGrid(x=True, y=True, alpha=0.05)

        self.marker_closed = MarkerPlot(self.plot)

    def open_loop_plot(self):
        self.mode = "open"
        
        self.plot_temp = self.plot_widget.addPlot(row=0, col=0, title="Análise de Malha Aberta")
        self.plot_derivative = self.plot_widget.addPlot(row=1, col=0, title="Derivada dT/t")

        self.marker_temp = MarkerPlot(self.plot_temp)
        self.marker_derivative = MarkerPlot(self.plot_derivative)

    def add_legend(self, legenda="", color="black", type=pg.QtCore.Qt.SolidLine, plot_n=0):
        if legenda == "":
            return
        
        plot = None

        if self.mode == "closed":
            plot = self.plot
        elif self.mode == "open":
            plot = None

            if plot_n == 0:
                plot = self.plot_temp
            elif plot_n == 1:
                plot = self.plot_derivative
        else:
            return

        legend = plot.addLegend()
        symbol = None

        if type != 'dot':
            symbol = plot.plot([], [], pen=pg.mkPen(color, width=2, style=type))
        else:
            symbol = pg.ScatterPlotItem(size=7, brush=pg.mkBrush(color), pen=pg.mkPen(None), symbol='o')

        legend.addItem(symbol, legenda)

    def add_curve(self, x, y, color='black', width=1.5, plot_n = 0):
        if x is None or y is None:
            return
        
        plot = None
        lista = None

        if self.mode == "closed":
            plot = self.plot
            lista = self.curves
        elif self.mode == "open":
            plot = None
            lista = None

            if plot_n == 0:
                plot = self.plot_temp
                lista = self.curves
            elif plot_n == 1:
                plot = self.plot_derivative
                lista = self.curves_dt
        else:
            return

        curve = plot.plot(x, y, pen = pg.mkPen(color, width=width))
        lista.append(curve)

    def add_item(self, item, plot_n):
        if item == None:
            return
        
        if self.mode == "closed":
            plot = self.plot
        elif self.mode == "open":
            plot = None

            if plot_n == 0:
                plot = self.plot_temp
            elif plot_n == 1:
                plot = self.plot_derivative
        else:
            return
    
        plot.addItem(item)

class PlotterAnalyzer(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.parent = parent
        self.layout = QVBoxLayout()
        self.setLayout(self.layout)

        self.plot_widget: PlotWidget = PlotWidget(self.layout)
        self.plot_widget.clear()
        
        self.df = None
        self.x_data = []
        self.y_data = []
        self.reference_lines = []  

    def update_analyzer(self, x_data, y_data, mode):
        self.plot_widget.clear()

        self.x_data = x_data
        self.y_data = y_data

        if mode == "closed":
            self.plot_widget.closed_loop_plot()
            self.closed_loop_analyzer()
        else:
            self.plot_widget.open_loop_plot()
            self.open_loop_analyzer()
    
    def closed_loop_analyzer(self):
        self.plot_widget.marker_closed.set_data(self.x_data, self.y_data)
        self.plot_widget.add_curve(self.x_data, self.y_data, 'blue', 1.5, 0)
        self.plot_widget.add_legend("Temperatura", 'blue')

        temp_inicial = self.y_data[0]
        h_line_init = pg.InfiniteLine(pos=temp_inicial, angle=0, pen=pg.mkPen("green", width=2, style=pg.QtCore.Qt.DashLine))
        self.plot_widget.plot.addItem(h_line_init)
        self.reference_lines.append(h_line_init)

        max_over_signal = np.max(self.y_data)
        h_line_max = pg.InfiniteLine(pos=max_over_signal, angle=0, pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.DashLine))
        self.plot_widget.plot.addItem(h_line_max)
        self.reference_lines.append(h_line_max)
        
        self.plot_widget.add_legend(f"Temperatura inicial ({temp_inicial:.2f}ºC)", "green", type=pg.QtCore.Qt.DashLine)
        self.plot_widget.add_legend(f"Máximo Sobressinal ({max_over_signal:.2f}ºC)", "red", type=pg.QtCore.Qt.DashLine)
        
        targets = np.array(self.parent.df["target"])
        label_targets = ""
        last_target = targets[0] - 1
        for t in targets:
            if last_target != t:
                last_target = t
                label_targets += f"{t}ºC, "
        label_targets = label_targets[:-2]
        
        self.plot_widget.add_legend(f"Temperaturas desejadas\n[{label_targets}]", "orange")
        self.plot_widget.add_curve(self.x_data, targets, 'orange', 1)
                
    def open_loop_analyzer(self):
        temp_a_original = np.array(self.parent.df["temp_a"])
        temp_b_original = np.array(self.parent.df["temp_b"])

        x = [float(row.iloc[1]) - self.parent.df.at[self.parent.df.index[0], 'seconds'] for (_, row) in self.parent.df.iterrows()]
        temps = (temp_a_original + temp_b_original) / 2

        f_temps = np.array(sig.savgol_filter(temps, int(len(x) * 0.02), 6))
        f_temps_dt = np.gradient(f_temps, x, edge_order=1)

        max_dv_i = np.argmax(f_temps_dt)
        self.y_data = f_temps
        self.x_data = x

        self.plot_widget.marker_temp.set_data(self.x_data, self.y_data)
        self.plot_widget.marker_derivative.set_data(self.x_data, f_temps_dt)

        self.plot_widget.add_curve(x, f_temps, 'blue', 1.5, 0)
        self.plot_widget.add_curve(x, f_temps_dt, 'blue', 1.5, 1)

        h_line_max = pg.InfiniteLine(pos=x[max_dv_i], angle=90, pen=pg.mkPen("red", width=2, style=pg.QtCore.Qt.DashLine))

        self.plot_widget.add_item(h_line_max, 1)
        
        L = x[max_dv_i] - ((f_temps[max_dv_i] - f_temps[0]) / f_temps_dt[max_dv_i])
        T = ((f_temps[-1] - f_temps[0]) / f_temps_dt[max_dv_i])

        neighborhood = np.linspace(L, T + L)
        max_dv_scarter = pg.ScatterPlotItem(size=5, brush=pg.mkBrush("r"), pen=pg.mkPen(None), symbol='o')
        max_dv_scarter.setData(pos=[(x[max_dv_i], f_temps[max_dv_i])])

        self.plot_widget.add_curve(neighborhood,
                                   [(((x0 - x[max_dv_i]) * f_temps_dt[max_dv_i]) + f_temps[max_dv_i]) for x0 in neighborhood],
                                   'red')
        
        v_initial_line = pg.InfiniteLine(pos=x[0], angle=90, pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.DashLine))
        h_initial_line = pg.InfiniteLine(pos=f_temps[0], angle=0, pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.DashLine))

        v_final_line = pg.InfiniteLine(pos=x[-1], angle=90, pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.DashLine))
        h_final_line = pg.InfiniteLine(pos=f_temps[-1], angle=0, pen=pg.mkPen("black", width=2, style=pg.QtCore.Qt.DashLine))

        self.plot_widget.add_item(max_dv_scarter, 0)
        self.plot_widget.add_item(h_initial_line, 0)
        self.plot_widget.add_item(v_initial_line, 0)
        self.plot_widget.add_item(v_final_line, 0)
        self.plot_widget.add_item(h_final_line, 0)

        self.plot_widget.add_legend('Temperatura (ºC)', 'blue', plot_n=0)
        self.plot_widget.add_legend('Ponto de maior derivada', 'red', 'dot', plot_n=0)
        self.plot_widget.add_legend(f'Temperatura inicial ({f_temps[0]:.3f}ºC)', 'black', pg.QtCore.Qt.DashLine, plot_n=0)
        self.plot_widget.add_legend(f'Temperatura final ({f_temps[-1]:.3f}ºC)', 'black', pg.QtCore.Qt.DashLine, plot_n=0)

        self.plot_widget.plot_temp.setTitle(f"Temperatura registrada - L = {L:.2f}, T (+L) = {T:.2f} (+{L:.2f})")

class AnalyzerGUI(QWidget):
    def __init__(self, app_mirror):
        super().__init__()
        self.app_mirror = app_mirror
        self.selected_file = None

        self.setWindowTitle("Thermal Analyzer")
        self.setMinimumSize(900, 600)

        self.layout = QHBoxLayout()
        self.setLayout(self.layout)

        self.sidebar = SidebarAnalyzer(self)
        self.plotter_gui = PlotterAnalyzer(self)

        self.layout.addWidget(self.sidebar, 1)
        self.layout.addWidget(self.plotter_gui, 4)

    def start_analysis(self, file_path, mode):
        if file_path:
            self.mode = mode
            print(file_path)
            self.df = pd.read_csv(file_path)
            x_data = [float(row["seconds"]) - self.df.iloc[0]["seconds"] for _, row in self.df.iterrows()]
            #temps = (self.df["temp_a"] + self.df["temp_b"]) / 2
            temps = self.df["temp_a"]

            if mode == "open":
                temps = np.array(sig.savgol_filter(temps, int(len(x_data) * 0.02), 6))

            self.plotter_gui.update_analyzer(x_data, temps, mode)
