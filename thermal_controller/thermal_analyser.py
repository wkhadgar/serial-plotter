import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import scipy.signal as sig
import tkinter as tk
from tkinter import filedialog as fd

is_open_loop = False
temp_log = ""

def analyse_closed(df: pd.DataFrame):
    temp_a_original = np.array(df["temp_a"])
    temp_b_original = np.array(df["temp_b"])
    targets = np.array(df["target"])
    np.array(df["temp_b"])

    x = [float(row.iloc[1]) - df.at[df.index[0], 'seconds'] for (_, row) in df.iterrows()]
    temps = (temp_a_original + temp_b_original) / 2

    f_temps = temps
    fig, axs = plt.subplots(1, 1, sharex='all')

    axs.grid()
    axs.plot(x, f_temps, label="Temperatura (°C)")
    axs.set_xlabel("Tempo (s)")
    axs.set_ylabel("Temperatura (°C)")

    label_targets = ""
    last_target = targets[0] - 1
    for t in targets:
        if last_target != t:
            last_target = t
            label_targets += f"{t}ºC, "
    label_targets = label_targets[:-2]

    max_over_signal = f_temps[np.argmin(f_temps)] if f_temps[0] > f_temps[-1] else f_temps[np.argmax(f_temps)]

    if len(label_targets.split(",")) < 2:
        tgt = targets[-1] - f_temps[0]
        t10 = 0
        t90 = 0
        for i in range(len(f_temps)):   
            if (f_temps[i] - f_temps[0]) < tgt * 0.1:
                t10 = x[i]
            if (f_temps[i] - f_temps[0]) < tgt * 0.9 :
                t90 = x[i]

        axs.axvline(t10, color="orange", linestyle="--")
        axs.axvline(t90, color="orange", linestyle="--")
        print(f"tr = {t90} - {t10} = {t90-t10}")

    axs.axvline(0, color="black", linestyle="--")
    axs.axhline(f_temps[0], color="black", linestyle="--", label=f"Temperatura inicial ({f_temps[0]:.2f}ºC)")
    axs.axhline(max_over_signal, color="red", linestyle="--", label=f"Máximo sobressinal ({max_over_signal:.2f}ºC)")
    axs.plot(x, targets, color="gray", linestyle="--", label=f"Temperaturas desejadas\n[{label_targets}]")
    axs.legend()

    axs.title.set_text("Temperatura registrada")

    fig.tight_layout()
    plt.show()

def analyse_open(df: pd.DataFrame):
    temp_a_original = np.array(df["temp_a"])
    temp_b_original = np.array(df["temp_b"])
    np.array(df["temp_b"])

    x = [float(row.iloc[1]) - df.at[df.index[0], 'seconds'] for (_, row) in df.iterrows()]
    temps = (temp_a_original + temp_b_original) / 2

    f_temps = np.array(sig.savgol_filter(temps, int(len(x) * 0.02), 6))
    f_temps_dt = np.gradient(f_temps, x, edge_order=1)

    max_dv_i = np.argmax(f_temps_dt)

    fig, axs = plt.subplots(2, 1, sharex='all')

    axs[1].grid()
    axs[1].plot(x, f_temps, label="Temperatura (°C)")
    axs[1].set_xlabel("Tempo (s)")
    axs[1].set_ylabel("Temperatura (°C)")
    axs[1].scatter(x[max_dv_i], f_temps[max_dv_i], color='C1', s=10, label="Ponto de maior derivada")

    L = x[max_dv_i] - ((f_temps[max_dv_i] - f_temps[0]) / f_temps_dt[max_dv_i])
    T = ((f_temps[-1] - f_temps[0]) / f_temps_dt[max_dv_i])
    axs[1].set_xticks([L, T + L])
    axs[1].set_xticklabels([f"L = {L:.2f}", f"T (+L) = {T:.2f} (+{L:.2f})"])

    axs[1].axvline(0, color="black", linestyle="--")
    axs[1].axhline(f_temps[0], color="black", linestyle="--", label=f"Temperatura inicial ({f_temps[0]:.2f}ºC)")
    axs[1].axhline(f_temps[-1], color="gray", linestyle="--", label=f"Temperaturas final ({f_temps[-1]:.2f}ºC)")
    axs[1].legend()
    neighborhood = np.linspace(L, T + L)
    axs[1].plot(neighborhood,
                [(((x0 - x[max_dv_i]) * f_temps_dt[max_dv_i]) + f_temps[max_dv_i]) for x0 in neighborhood],
                'C1--')

    axs[1].title.set_text("Temperatura registrada")

    axs[0].grid()
    axs[0].plot(x, f_temps_dt)
    axs[0].axvline(x[max_dv_i], color="r", linestyle="--")
    axs[0].title.set_text("Derivada dT/dt")

    fig.tight_layout()
    plt.show()


def main():
    global temp_log
    df = pd.DataFrame(pd.read_csv(temp_log))

    if is_open_loop:
        analyse_open(df)
    else:
        analyse_closed(df)


def select_file():
    global temp_log

    temp_log = fd.askopenfilename(defaultextension="csv", title="Escolha o log gerado para análise",
                                  initialdir="temp_logs")

    if temp_log:
        file_label.config(text=f"Log selecionado: {temp_log.split('/')[-1]}")
        open_button.config(state=tk.NORMAL)  # Enable the "Open" button
    else:
        file_label.config(text="Log selecionado: -")
        open_button.config(state=tk.DISABLED)  # Disable the "Open" button


def on_option_change():
    global is_open_loop
    selected_option = option_var.get()
    is_open_loop = selected_option == "open"


if __name__ == "__main__":
    # Create the main window
    root = tk.Tk()
    root.title("Thermal Analyser")
    screen_width = root.winfo_screenwidth()  # Get the width of the screen
    screen_height = root.winfo_screenheight()  # Get the height of the screen

    height = 300
    width = 700

    # Calculate the position to center the window
    position_top = int(screen_height / 2 - height / 2)
    position_right = int(screen_width / 2 - width / 2)

    # Set the geometry of the window with width, height, and position
    root.geometry(f'{width}x{height}+{position_right}+{position_top}')

    # Create a label for file selection
    file_label = tk.Label(root, text="Log selecionado: -")
    file_label.pack(pady=10)

    # Create a button to open the file dialog
    file_button = tk.Button(root, text="Selecione um log", command=select_file)
    file_button.pack(pady=5)

    # Create an "Open" button, initially disabled
    open_button = tk.Button(root, text="Analisar", command=main, state=tk.DISABLED)
    open_button.pack(pady=5)

    # Create a variable for the radio buttons
    option_var = tk.StringVar(value="closed")

    # Create two radio buttons for selecting between options
    radio2 = tk.Radiobutton(root, text="Análise de malha fechada", variable=option_var, value="closed",
                            command=on_option_change)
    radio2.pack(pady=5)
    radio1 = tk.Radiobutton(root, text="Análise de malha aberta   ", variable=option_var, value="open",
                            command=on_option_change)
    radio1.pack(pady=5)

    # Start the Tkinter event loop
    root.mainloop()
