from enum import Enum
import numpy as np
import pyqtgraph as pg
from PySide6 import QtCore

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

class Mode(Enum):
    CLOSED = 'closed'
    OPEN = 'open'
    PLOTTER = 'plotter'

class PlotWidget:
    def __init__(self, layout, mode: int = None):
        self.plot_widget = pg.GraphicsLayoutWidget()
        self.plot_widget.setBackground('w')
        layout.addWidget(self.plot_widget)
        self.mode = mode
        self._init_containers()

    def _init_containers(self):
        # Initialize curves and legends storage based on modes
        self._curves = {
            Mode.CLOSED: [],
            Mode.OPEN: {0: [], 1: []},
            Mode.PLOTTER: {0: [], 1: []}
        }
        self._legends = {0: [], 1: []}

    def clear(self):
        # Clear all plots, curves, and legends
        curves = self._curves.get(self.mode)
        if isinstance(curves, dict):
            for lst in curves.values():
                lst.clear()
        else:
            curves.clear()
        for lst in self._legends.values():
            lst.clear()
        self.plot_widget.clear()

    def _get_plot_and_containers(self, plot_n: int = 0):
        if self.mode == Mode.CLOSED:
            return self.plot, self._curves[Mode.CLOSED], self._legends[plot_n]
        if self.mode == Mode.OPEN:
            key = 0 if plot_n == 0 else 1
            plot = self.plot_temp if key == 0 else self.plot_derivative
            return plot, self._curves[Mode.OPEN][key], self._legends[plot_n]
        if self.mode == Mode.PLOTTER:
            plot = self.plot_sensor if plot_n == 0 else self.plot_actuators
            return plot, self._curves[Mode.PLOTTER][plot_n], self._legends[plot_n]
        raise ValueError(f"Invalid mode: {self.mode}")

    def add_curve(self, x, y, color='black', width=1.5, plot_n: int = 0):
        if x is None or y is None:
            return
        plot, curves, _ = self._get_plot_and_containers(plot_n)
        curve = plot.plot(x, y, pen=pg.mkPen(color, width=width))
        curves.append(curve)

    def update_curve(self, x, y, plot_n: int, curve_n: int):
        if x is None or y is None:
            return
        _, curves, _ = self._get_plot_and_containers(plot_n)
        curves[curve_n].setData(x, y)

    def clear_plots(self, plot_n: int = 0):
        plot, curves, _ = self._get_plot_and_containers(plot_n)
        plot.clear()
        curves.clear()

    def add_legend(self, text: str = '', size: int = 11,
                   color: str = 'black', style=pg.QtCore.Qt.SolidLine, plot_n: int = 0):
        if not text:
            return
        plot, _, legends = self._get_plot_and_containers(plot_n)
        legend = plot.addLegend(labelTextSize=str(size))
        if style != 'dot':
            item = plot.plot([], [], pen=pg.mkPen(color, width=3, style=style))
        else:
            item = pg.ScatterPlotItem(size=3, brush=pg.mkBrush(color), pen=None, symbol='o')
        legend.addItem(item, text)
        legends.append((legend, item))

    def update_legend(self, text: str, idx: int, plot_n: int = 0):
        _, _, legends = self._get_plot_and_containers(plot_n)
        legend, item = legends[idx]
        label = legend.getLabel(item)
        label.setText(text)

    def add_item(self, item, plot_n: int = 0):
        if item is None:
            return
        plot, _, _ = self._get_plot_and_containers(plot_n)
        plot.addItem(item)

    def closed_loop_plot(self):
        self.mode = Mode.CLOSED
        self.clear()
        self.plot = self.plot_widget.addPlot(title='Análise de Malha Fechada')
        self._setup_plot(self.plot)
        self.marker_closed = MarkerPlot(self.plot)

    def open_loop_plot(self):
        self.mode = Mode.OPEN
        self.clear()
        self.plot_temp = self.plot_widget.addPlot(row=0, col=0, title='Análise de Malha Aberta')
        self.plot_derivative = self.plot_widget.addPlot(row=1, col=0, title='Derivada dT/t')
        self._setup_plot(self.plot_temp)
        self._setup_plot(self.plot_derivative)
        self.marker_temp = MarkerPlot(self.plot_temp)
        self.marker_derivative = MarkerPlot(self.plot_derivative)

    def plotter_plot(self):
        self.mode = Mode.PLOTTER

    def plotter_dual_plot(self, legend_1: str = '', legend_2: str = ''):
        self.mode = Mode.PLOTTER
        self.plot_sensor = self._init_subplot(0, title=legend_1)
        self.plot_actuators = self._init_subplot(1, title=legend_2)

    def plotter_single_plot(self, legend_1: str = ''):
        self.mode = Mode.PLOTTER
        self.plot_sensor = self._init_subplot(0, title=legend_1)

    def _init_subplot(self, row: int, title: str):
        plot = self.plot_widget.addPlot(row=row, col=0, title=title)
        self._setup_plot(plot)
        return plot

    def _setup_plot(self, plot):
        if self.mode == Mode.PLOTTER:
            self.plot_widget.setBackground('k')
            text_pen = pg.mkPen('w')
        else:
            text_pen = pg.mkPen('k')

        plot.showGrid(x=True, y=True, alpha=0.2)

        for axis in ('left', 'bottom'):
            ax = plot.getAxis(axis)
            ax.setPen(text_pen)
            ax.setTextPen(text_pen)

        plot.setLabel('bottom', 'Tempo', units='s', **{'color': text_pen.color().name()})