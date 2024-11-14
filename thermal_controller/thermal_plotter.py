import numpy as np
import pandas as pd
import pyqtgraph as pg
import argparse
import os
import random
import serial
import sys
from collections.abc import Callable
from functools import partial
from PyQt5.QtWidgets import QApplication, QLineEdit, QGraphicsProxyWidget, QWidget
from PyQt5 import QtCore


class MainWindow(QWidget):
    def __init__(self):
        super().__init__()

        self.desired_temp = 25

        self.win = pg.GraphicsLayoutWidget(show=False, title="Plotter Serial SC")
        self.win.showFullScreen()

        # Plot combinado dos dados
        self.plot_combined = self.win.addPlot(title="Temperatura A e B", col=0, row=0)
        self.plot_combined.setLabel('bottom', "Tempo decorrido (s)")
        self.plot_combined.setLabel('left', "Temperatura (°C)")
        self.plot_combined.showGrid(x=True, y=True, alpha=0.5)
        self.combined_leg = self.plot_combined.addLegend()
        self.curve_a_combined = self.plot_combined.plot(pen="c", name="Temperatura A")
        self.curve_b_combined = self.plot_combined.plot(pen="g", name="Temperatura B")
        self.curve_m_combined = self.plot_combined.plot(pen="y", name="Temperatura média")
        self.curve_a_combined_label = self.combined_leg.getLabel(self.curve_a_combined)
        self.curve_b_combined_label = self.combined_leg.getLabel(self.curve_b_combined)
        self.curve_m_combined_label = self.combined_leg.getLabel(self.curve_m_combined)

        self.desired_temp_line = pg.InfiniteLine(pos=self.desired_temp, angle=0, movable=False, label="Temperatura desejada [-]",
                                                 pen=pg.mkPen("orange", width=2))
        self.plot_combined.addItem(self.desired_temp_line)

        # Plot individual dos dados
        self.plot_a = self.win.addPlot(title="Temperatura A", col=1, row=0)
        self.plot_a.setLabel('bottom', "Tempo decorrido (s)")
        self.plot_a.setLabel('left', "Temperatura (°C)")
        self.plot_a.showGrid(x=True, y=True, alpha=0.5)
        self.a_leg = self.plot_a.addLegend()
        self.curve_a = self.plot_a.plot(pen="c", name="Temperatura A")
        self.curve_a_label = self.a_leg.getLabel(self.curve_a)

        self.plot_b = self.win.addPlot(title="Temperatura B", col=1, row=1)
        self.plot_b.setLabel('bottom', "Tempo decorrido (s)")
        self.plot_b.setLabel('left', "Temperatura (°C)")
        self.plot_b.showGrid(x=True, y=True, alpha=0.5)
        self.b_leg = self.plot_b.addLegend()
        self.curve_b = self.plot_b.plot(pen="g", name="Temperatura B")
        self.curve_b_label = self.b_leg.getLabel(self.curve_b)

        self.plot_m = self.win.addPlot(title="Temperatura Média", col=0, row=3)
        self.plot_m.setLabel('bottom', "Tempo decorrido (s)")
        self.plot_m.setLabel('left', "Temperatura (°C)")
        self.plot_m.showGrid(x=True, y=True, alpha=0.5)
        self.m_leg = self.plot_m.addLegend()
        self.curve_m = self.plot_m.plot(pen="y", name="Temperatura Média")
        self.curve_m_label = self.b_leg.getLabel(self.curve_m)

        self.plot_d = self.win.addPlot(title="Duty", col=0, row=1)
        self.plot_d.setLabel('bottom', "Tempo decorrido (s)")
        self.plot_d.setLabel('left', "Duty Cycle (%)")
        self.plot_d.showGrid(x=True, y=True, alpha=0.5)
        self.d_leg = self.plot_d.addLegend()
        self.curve_d = self.plot_d.plot(pen="r", name="Duty")
        self.curve_d_label = self.d_leg.getLabel(self.curve_d)

        self.temp_input = QLineEdit()
        self.temp_input.setPlaceholderText("Defina a temperatura desejada (°C). [Entre 20°C e 50°C]...")
        self.temp_input.setAlignment(QtCore.Qt.AlignCenter)

        # Use QGraphicsProxyWidget to overlay the QLineEdit on the plot
        self.proxy = QGraphicsProxyWidget()
        self.proxy.setWidget(self.temp_input)
        self.win.addItem(self.proxy, col=0, row=2)

        self.temp_input.returnPressed.connect(self.on_return_pressed)

        self.init_timestamp = pd.Timestamp.now()
        self.plot_seconds = np.array([])
        self.duty_data = np.array([])
        self.temp_a_data = np.array([])
        self.temp_b_data = np.array([])
        self.temp_m_data = np.array([])

        self.plot_views = ["C", "IC", "A", "B", "M"]
        self.current_mode = -1

        self.ser: serial.Serial | None = None

    def toggle_plot_view(self):

        self.current_mode = (self.current_mode + 1) % len(self.plot_views)
        match self.plot_views[self.current_mode]:
            case "C":
                self.plot_combined.show()
                self.plot_a.hide()
                self.plot_b.hide()
                self.plot_m.hide()
                self.plot_d.show()
                self.proxy.show()
            case "IC":
                self.plot_combined.hide()
                self.plot_a.show()
                self.plot_b.show()
                self.plot_m.hide()
                self.plot_d.hide()
                self.proxy.hide()
            case "A":
                self.plot_combined.hide()
                self.plot_a.show()
                self.plot_b.hide()
                self.plot_m.hide()
                self.plot_d.hide()
                self.proxy.hide()
            case "B":
                self.plot_combined.hide()
                self.plot_a.hide()
                self.plot_b.show()
                self.plot_m.hide()
                self.plot_d.hide()
                self.proxy.hide()
            case "M":
                self.plot_combined.hide()
                self.plot_a.hide()
                self.plot_b.hide()
                self.plot_m.show()
                self.plot_d.hide()
                self.proxy.hide()

    def set_serial(self, ser: serial.Serial):
        self.ser = ser

    def get_data_serial(self):
        if not self.ser.in_waiting:
            return ""

        data = self.ser.read().decode("utf-8")
        if data != ">":
            self.ser.reset_input_buffer();
            return ""

        while data[-1] != "<":
            data += self.ser.read().decode("utf-8")
        return data

    def send_data_serial(self, duty: float):
        self.ser.write(str(duty).encode('utf-8'))

    def on_return_pressed(self):
        self.desired_temp = float(self.temp_input.text())

        self.temp_input.clear()
        self.send_data_serial(self.desired_temp)
        self.desired_temp_line.setValue(self.desired_temp)
        self.desired_temp_line.label.setText(f"Temperatura desejada [{self.desired_temp}°C]")
        self.desired_temp_line.update()

    # Function to print the input text on 'Enter'
    def update_plots(self, get_data: Callable, log_f_path: str):
        data = get_data()

        if data != "":
            try:
                temp_a, temp_b, duty = [float(t) for t in data[1:-1].split(";")]
            except ValueError:
                return

            timestamp = pd.Timestamp.now()

            self.temp_a_data = np.append(self.temp_a_data, temp_a)
            self.temp_b_data = np.append(self.temp_b_data, temp_b)
            self.temp_m_data = np.append(self.temp_m_data, (temp_b + temp_a) / 2)
            self.duty_data = np.append(self.duty_data, duty)

            self.plot_seconds = np.append(self.plot_seconds, (timestamp - self.init_timestamp).total_seconds())

            with open(log_f_path, "a") as f:
                f.write(
                    f"{timestamp.strftime("%H:%M:%S")},"
                    f"{self.plot_seconds[-1]:.4f},"
                    f"{temp_a},"
                    f"{temp_b},"
                    f"{duty}\n")

            match self.plot_views[self.current_mode]:
                case "C":
                    self.curve_a_combined.setData(self.plot_seconds, self.temp_a_data)
                    self.curve_a_combined_label.setText(f"Temperatura A: {temp_a}")

                    self.curve_b_combined.setData(self.plot_seconds, self.temp_b_data)
                    self.curve_b_combined_label.setText(f"Temperatura B: {temp_b}")

                    self.curve_m_combined.setData(self.plot_seconds, self.temp_m_data)
                    self.curve_m_combined_label.setText(f"Temperatura Média: {(temp_b + temp_a) / 2:.2f}")

                    self.curve_d.setData(self.plot_seconds, self.duty_data)
                    self.curve_d_label.setText(f"Duty Cycle (%): {duty}")
                case "IC":
                    self.curve_a.setData(self.plot_seconds, self.temp_a_data)
                    self.curve_a_label.setText(f"Temperatura A: {temp_a}")

                    self.curve_b.setData(self.plot_seconds, self.temp_b_data)
                    self.curve_b_label.setText(f"Temperatura B: {temp_b}")
                case "A":
                    self.curve_a.setData(self.plot_seconds, self.temp_a_data)
                    self.curve_a_label.setText(f"Temperatura A: {temp_a}")
                case "B":
                    self.curve_b.setData(self.plot_seconds, self.temp_b_data)
                    self.curve_b_label.setText(f"Temperatura B: {temp_b}")
                case "M":
                    self.curve_m.setData(self.plot_seconds, self.temp_m_data)
                    self.curve_m_label.setText(f"Temperatura Média: {(temp_b + temp_a) / 2:.2f}")

    def key_press_handle(self, super_press_handler: Callable, ev):
        if self.temp_input.hasFocus():
            super_press_handler(ev)
        else:
            if ev.key() == QtCore.Qt.Key_Space:
                self.toggle_plot_view()
            elif ev.key() == QtCore.Qt.Key_Escape:
                sys.exit(0)


def arg_parse():
    parser = argparse.ArgumentParser(description="Plotter serial para sensores.")
    parser.add_argument(
        "port",
        type=str,
        help="Porta serial a ser plotada.")
    parser.add_argument(
        "baud",
        type=int,
        help="Baud rate da porta serial.")
    parser.add_argument(
        "--closed",
        "-c",
        action="store_true",
        help="Indica se o controle de malha deve ser feito."
    )
    parser.add_argument(
        "--update-delay",
        "-u",
        metavar="<delay_ms>",
        type=int,
        default=1,
        help="Tempo entre atualizações do plot, em milissegundos."
    )
    parser.add_argument(
        "--output-log-path",
        "-o",
        metavar="<path/to/out>",
        type=str,
        default="./temp_logs/",
        help="Diretório de saída dos logs de gravação."
    )

    return parser.parse_args()


def run_func_forever(func):
    while True:
        func()


def main():
    args = arg_parse()
    port = args.port
    baud = args.baud
    update_delay = args.update_delay
    log_path = args.output_log_path

    dt = pd.Timestamp.now()
    df = pd.DataFrame(columns=["timestamp", "seconds", "temp_a", "temp_b", "duty"])

    app = QApplication([])
    main_w = MainWindow()
    main_w.win.keyPressEvent = partial(main_w.key_press_handle, main_w.win.keyPressEvent)

    if port != "sim":
        try:
            ser = serial.Serial(port, baud, timeout=0)
            main_w.set_serial(ser)
            get_data = main_w.get_data_serial
        except serial.SerialException:
            print(f"Não foi possível abrir a porta serial {port}@{baud}")
            sys.exit(1)
    else:
        get_data = main_w.get_data_dummy

    try:
        os.mkdir(log_path)
    except FileExistsError:
        pass

    print(f"Salvando dados em {log_path}")
    log_file_path = log_path + f"log_{dt.year}-{dt.month}-{dt.day}-{dt.hour}-{dt.minute}-{dt.second}.csv"
    df.to_csv(log_file_path, index=False)

    timer = QtCore.QTimer()
    timer.timeout.connect(partial(main_w.update_plots, get_data, log_file_path))
    main_w.toggle_plot_view()

    timer.start(update_delay)
    try:
        app.exec_()
    except KeyboardInterrupt:
        print("Exiting...")
        sys.exit()


if __name__ == "__main__":
    main()
