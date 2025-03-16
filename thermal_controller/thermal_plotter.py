import struct
import threading
import time

import numpy as np
import pandas as pd
import pyqtgraph as pg
import argparse
import os
import sys

from pyocd.core.helpers import ConnectHelper, Session
from collections.abc import Callable
from functools import partial
from PyQt5.QtWidgets import QApplication, QLineEdit, QGraphicsProxyWidget, QWidget
from PyQt5 import QtCore
import scipy.signal as sig

L = 9.02
T = 344.21


class PIDController:
    def __init__(self, *, setpoint: float, l: float, t: float):
        ## /** Constantes do controlador PI, calculadas por Ziegler-Nichols. */

        self.target_setpoint = setpoint

        ti = (l / 0.3)
        td = 0
        self.Kp = (0.9 * (t / l))
        self.Ki = (self.Kp / ti)
        self.Kd = (self.Kp * td)

        self.error = 0
        self.accumulated_I = 0

        self.last_timestamp = pd.Timestamp.now()

        self.ser: Session | None = None
        self.ram = None
        self.control_block_addr = 0x0

        self.ntc_a_temp = 0
        self.ntc_b_temp = 0
        self.duty = 0

    def __PID(self, dt_us: float, desired: float, measured: float):
        ## /** Ajuste do PID, com medidas anti-windup. */
        dt_s = dt_us / 10 ** 6

        err = desired - measured
        P = self.Kp * err
        I_inc = self.Ki * err * dt_s
        D = self.Kd * (err - self.error) / (dt_s + 0.000001)

        self.error = err

        windup_check = P + self.accumulated_I + I_inc + D

        if windup_check > 100:
            return 100

        if windup_check < -100:
            return -100

        self.accumulated_I += I_inc
        return windup_check

    def set_serial(self, ser: Session):
        self.ser = ser
        self.ram = self.ser.target.get_memory_map()[1]

        print("Finding control block area...")
        key = [ord(c) for c in "!CTR"]
        for addr in range(self.ram.start, self.ram.end):
            byte = self.ser.target.read8(addr)
            if byte != key[0]:
                continue

            if self.ser.target.read_memory_block8(addr, len(key)) == key:
                print(f"Control block area found at 0x{addr:X}!")
                self.control_block_addr = addr + len(key)
                break
        else:
            print("Block control area not found!!!")
            sys.exit(1)

    def __get_data_serial(self):
        def __read_float(_from: int) -> float:
            data_bytes = struct.pack("<I", self.ser.target.read32(_from))
            return struct.unpack("<f", data_bytes)[0]

        control_floats = []
        for i in range(3):
            control_floats.append(__read_float(self.control_block_addr + (i * 4)))

        self.ntc_a_temp = control_floats[0]
        self.ntc_b_temp = control_floats[1]
        self.duty = control_floats[2]

        return self.ntc_a_temp, self.ntc_b_temp, self.duty

    def __feedback(self, out: float):
        data_bytes = struct.pack("<f", out)
        data = struct.unpack("<I", data_bytes)[0]
        self.ser.target.write32(self.control_block_addr + (2 * 4), data)

    def update_setpoint(self, new_setpoint: float):
        self.target_setpoint = new_setpoint

    def control(self):
        data = self.__get_data_serial()
        measure = (data[0] + data[1]) / 2  # Média das temperaturas
        timestamp = pd.Timestamp.now()
        dt_t = timestamp - self.last_timestamp
        out = self.__PID(dt_us=dt_t.microseconds, desired=self.target_setpoint, measured=measure)
        self.last_timestamp = timestamp
        self.__feedback(out)


class ControlGUI(QWidget):
    def __init__(self, *, controller: PIDController, x_label: str, y_label: str):
        super().__init__()

        self.controller = controller

        self.init_timestamp = pd.Timestamp.now()
        self.plot_seconds = np.array([])
        self.duty_data = np.array([])
        self.temp_a_data = np.array([])
        self.temp_b_data = np.array([])
        self.temp_m_data = np.array([])

        self.plot_views = ["C", "A", "B", "M"]
        self.current_mode = -1

        self.win = pg.GraphicsLayoutWidget(show=False, title="Plotter Serial SC")
        self.win.showFullScreen()
        self.current_setpoint_line = pg.InfiniteLine(pos=self.controller.target_setpoint, angle=0, movable=False,
                                                     label="Temperatura desejada [-]",
                                                     pen=pg.mkPen("orange", width=2))

        # Plot combinado dos dados
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
        self.temp_input.setAlignment(QtCore.Qt.AlignCenter)

        self.proxy = QGraphicsProxyWidget()
        self.proxy.setWidget(self.temp_input)
        self.win.addItem(self.proxy, col=0, row=2)

        self.temp_input.returnPressed.connect(self.__on_return_pressed)

        self.win.keyPressEvent = partial(self.key_press_handle, self.win.keyPressEvent)

    def key_press_handle(self, super_press_handler: Callable, ev):
        if self.temp_input.hasFocus():
            super_press_handler(ev)
        else:
            if ev.key() == QtCore.Qt.Key_Space:
                self.toggle_plot_view()
            elif ev.key() == QtCore.Qt.Key_Escape:
                sys.exit(0)

    def __on_return_pressed(self):
        self.controller.update_setpoint(float(self.temp_input.text()))
        self.temp_input.clear()

        self.current_setpoint_line.setValue(self.controller.target_setpoint)
        self.current_setpoint_line.label.setText(f"Temperatura desejada [{self.controller.target_setpoint}°C]")
        self.current_setpoint_line.update()

    def update_plots(self, log_f_path: str):
        temp_a, temp_b, duty = self.controller.ntc_a_temp, self.controller.ntc_b_temp, self.controller.duty

        timestamp = pd.Timestamp.now()
        self.plot_seconds = np.append(self.plot_seconds, (timestamp - self.init_timestamp).total_seconds())
        self.duty_data = np.append(self.duty_data, duty)
        self.temp_a_data = np.append(self.temp_a_data, temp_a)
        self.temp_b_data = np.append(self.temp_b_data, temp_b)
        self.temp_m_data = np.append(self.temp_m_data, (temp_b + temp_a) / 2)

        with open(log_f_path, "a") as f:
            f.write(
                f"{timestamp.strftime("%H:%M:%S")},"
                f"{self.plot_seconds[-1]:.4f},"
                f"{temp_a},"
                f"{temp_b},"
                f"{duty},"
                f"{self.current_setpoint}\n")

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


def arg_parse():
    parser = argparse.ArgumentParser(description="Plotter serial para sensores.")
    parser.add_argument(
        "--setpoint",
        "-s",
        type=float,
        default=25,
        help="Valor desejado inicial."
    )
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


def control_thread(controller: PIDController):
    while True:
        controller.control()
        time.sleep(0.01)


def main():
    args = arg_parse()
    update_delay = args.update_delay
    log_path = args.output_log_path

    datetime = pd.Timestamp.now()
    df = pd.DataFrame(columns=["timestamp", "seconds", "temp_a", "temp_b", "duty", "target"])

    controller = PIDController(setpoint=args.setpoint, l=L, t=T)
    main_w = ControlGUI(controller=controller, x_label="Tempo decorrido (s)", y_label="Temperatura (°C)")
    main_w.toggle_plot_view()

    print(f"Salvando dados em {log_path}")
    log_file_path = log_path + f"log_{datetime.year}-{datetime.month}-{datetime.day}-{datetime.hour}-{datetime.minute}-{datetime.second}.csv"
    df.to_csv(log_file_path, index=False)

    timer = QtCore.QTimer()
    timer.timeout.connect(partial(main_w.update_plots, log_file_path))
    timer.start(update_delay)

    with ConnectHelper.session_with_chosen_probe(target_override="stm32f103c8", connect_mode="attach") as session:
        controller.set_serial(session)
        control_t = threading.Thread(target=partial(control_thread, controller))

        try:
            os.mkdir(log_path)
        except FileExistsError:
            pass

        try:
            app = QApplication([])
            control_t.start()
            app.exec_()
        except KeyboardInterrupt:
            print("Exiting...")
            sys.exit()


if __name__ == "__main__":
    main()
