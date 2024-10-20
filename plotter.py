import numpy as np
import pandas as pd
import pyqtgraph as pg
import os
import serial
import sys
from PyQt5.QtWidgets import QApplication
from PyQt5 import QtCore

ser = serial.Serial("COM3", 9600, timeout=1)

app = QApplication([])
win = pg.GraphicsLayoutWidget(show=True, title="Plotter Serial SC")
win.showFullScreen()

# Plot combinado dos dados
plot_combined = win.addPlot(title="Temperatura A e B", col=0, row=0)
curve_a_combined = plot_combined.plot(pen="c", name="Temperatura A")
curve_b_combined = plot_combined.plot(pen="g", name="Temperatura B")

# Plot individual dos dados
plot_a = win.addPlot(title="Temperatura A", col=0, row=1)
curve_a = plot_a.plot(pen="c")

plot_b = win.addPlot(title="Temperatura B", col=1, row=1)
curve_b = plot_b.plot(pen="g")

temp_a_data = np.array([])
temp_b_data = np.array([])
time_data = []

log_path = "temp_logs/"

try:
    os.mkdir(log_path)
except FileExistsError:
    pass

dt = pd.Timestamp.now()
log_file_path = log_path + f"log_{dt.year}-{dt.month}-{dt.day}-{dt.hour}-{dt.minute}-{dt.second}.csv"
df = pd.DataFrame(columns=["time", "temp_a", "temp_b"])
df.to_csv(log_file_path, index=False)

plot_views = ["C", "IC", "A", "B"]
current_mode = -1


def toggle_plot_view():
    global current_mode

    current_mode = (current_mode + 1) % len(plot_views)
    match plot_views[current_mode]:
        case "C":
            plot_combined.show()
            plot_a.hide()
            plot_b.hide()
        case "IC":
            plot_combined.hide()
            plot_a.show()
            plot_b.show()
        case "A":
            plot_combined.hide()
            plot_a.show()
            plot_b.hide()
        case "B":
            plot_combined.hide()
            plot_a.hide()
            plot_b.show()


def update_plots():
    global temp_a_data, temp_b_data, time_data

    # data = ser.readline().decode("utf-8").strip()
    import random
    t = random.randint(20, int(20+5*random.random())) + random.random()
    data = f"> {t:.2f};{t -  random.random():.2f}"

    if data.startswith("> "):
        temp_a, temp_b = [float(t) for t in data[2:].split(";")]

        timestamp = pd.Timestamp.now()

        temp_a_data = np.append(temp_a_data, temp_a)
        temp_b_data = np.append(temp_b_data, temp_b)

        time_data.append(timestamp)
        plot_seconds = [(t - time_data[0]).total_seconds() for t in time_data]

        with open(log_file_path, "a") as f:
            f.write(f"{int(plot_seconds[-1] / 60):02d}:{plot_seconds[-1]:06.3f},{temp_a},{temp_b}\n")

        curve_a_combined.setData(plot_seconds, temp_a_data)
        curve_b_combined.setData(plot_seconds, temp_b_data)
        curve_a.setData(plot_seconds, temp_a_data)
        curve_b.setData(plot_seconds, temp_b_data)


def key_press_event(event):
    if event.key() == QtCore.Qt.Key_Space:
        toggle_plot_view()
    elif event.key() == QtCore.Qt.Key_Escape:
        ser.close()
        sys.exit()


if __name__ == "__main__":
    timer = QtCore.QTimer()
    timer.timeout.connect(update_plots)

    win.keyPressEvent = key_press_event
    toggle_plot_view()

    # TODO: ajustar o delay
    timer.start(100)
    try:
        QApplication.instance().exec_()
    except KeyboardInterrupt:
        print("Exiting...")
        ser.close()
        sys.exit()
