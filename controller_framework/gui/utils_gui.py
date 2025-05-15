import numpy as np
import pyqtgraph as pg

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
        
        self.curves_sensors = []
        self.curves_actuators = []

    def clear(self):
        self.curves.clear()
        self.curves_dt.clear()
        self.curves_sensors.clear()
        self.curves_actuators.clear()
        self.plot_widget.clear()

    def add_legend(self, legenda="", size = 11, color="black", type=pg.QtCore.Qt.SolidLine, plot_n=0):
        if legenda == "":
            return
        
        plot = None

        if self.mode == "closed":
            plot = self.plot
        elif self.mode == "open":
            if plot_n == 0:
                plot = self.plot_temp
            elif plot_n == 1:
                plot = self.plot_derivative
        if self.mode == "plotter":
            if plot_n == 0:
                plot = self.plot_sensor
            elif plot_n == 1:
                plot = self.plot_actuators
        else:
            return

        legend = plot.addLegend(labelTextSize=str(size))
        symbol = None

        if type != 'dot':
            symbol = plot.plot([], [], pen=pg.mkPen(color, width=15, style=type))
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
            if plot_n == 0:
                plot = self.plot_temp
                lista = self.curves
            elif plot_n == 1:
                plot = self.plot_derivative
                lista = self.curves_dt
        elif self.mode == 'plotter':
            if plot_n == 0:
                plot = self.plot_sensor
                lista = self.curves_sensors
            elif plot_n == 1:
                plot = self.plot_actuators
                lista = self.curves_actuators
        else:
            return

        curve = plot.plot(x, y, pen = pg.mkPen(color, width=width))
        lista.append(curve)

        print('depois do add curve')

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
        elif self.mode == "plotter":
            if plot_n == 0:
                plot = self.plot_sensor
            elif plot_n == 1:
                plot = self.plot_actuators
        else:
            return
    
        plot.addItem(item)

    def clear_plots(self, plot_n):
        lista = None

        if self.mode == "closed":
            plot = self.plot
            lista = self.curves
        elif self.mode == "open":
            if plot_n == 0:
                plot = self.plot_temp
                lista = self.curves
            elif plot_n == 1:
                plot = self.plot_derivative
                lista = self.curves_dt
        elif self.mode == 'plotter':
            if plot_n == 0:
                plot = self.plot_sensor
                lista = self.curves_sensors
            elif plot_n == 1:
                plot = self.plot_actuators
                lista = self.curves_actuators
        else:
            return

        plot.clear()
        lista.clear()

    def update_curve(self, x, y, plot_n, curve_n):
        if x is None or y is None:
            return
        
        lista = None

        if self.mode == "closed":
            lista = self.curves
        elif self.mode == "open":
            if plot_n == 0:
                lista = self.curves
            elif plot_n == 1:
                lista = self.curves_dt
        elif self.mode == 'plotter':
            if plot_n == 0:
                lista = self.curves_sensors
            elif plot_n == 1:
                lista = self.curves_actuators
        else:
            return

        curve = lista[curve_n]
        curve.setData(x, y)

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

    def plotter_plot(self):
        self.mode = "plotter"
        self.plot_widget.setBackground('k')

    def plotter_dual_plott(self, legend_1='', legend_2=''):
        self.plot_sensor = self.plot_widget.addPlot(row=0, col=0, title=legend_1)
        self.plot_actuators = self.plot_widget.addPlot(row=1, col=0, title=legend_2)

        self.plot_sensor.showGrid(x=True, y=True, alpha=0.5)
        self.plot_actuators.showGrid(x=True, y=True, alpha=0.5)

        self.plot_sensor.setLabel('bottom', 'Tempo', units='s')
        self.plot_actuators.setLabel('bottom', 'Tempo', units='s')
    
    def plotter_single_plot(self, legend_1=''):
        self.plot_sensor = self.plot_widget.addPlot(row=0, col=0, title=legend_1)
        self.plot_sensor.showGrid(x=True, y=True, alpha=0.5)
        self.plot_sensor.setLabel('bottom', 'Tempo', units='s')