import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import scipy.signal as sig

temp_log = "temp_logs/log_2024-11-4-18-10-29.csv"


def main():
    df = pd.DataFrame(pd.read_csv(temp_log))

    temp_a_original = np.array(df["temp_a"])
    temp_b_original = np.array(df["temp_b"])
    np.array(df["temp_b"])

    x = [float(row.iloc[0].split(':')[1]) for (_, row) in df.iterrows()]
    temps = (temp_a_original + temp_b_original) / 2

    f_temps = np.array(sig.savgol_filter(temps, int(len(x) * 0.02), 3))
    f_temps_dt = np.gradient(f_temps, x, edge_order=1)

    max_dv_i = np.argmax(f_temps_dt)

    fig, axs = plt.subplots(2, 1, sharex='all')

    axs[1].grid()
    axs[1].plot(x, f_temps, label="Temperatura (°C)")
    axs[1].scatter(x[max_dv_i], f_temps[max_dv_i], color='C1', s=10, label="Ponto de maior derivada")

    L = x[max_dv_i] - ((f_temps[max_dv_i] - f_temps[0]) / f_temps_dt[max_dv_i])
    T = f_temps[-1] / f_temps_dt[max_dv_i]
    axs[1].set_xticks([L, T])
    axs[1].set_xticklabels([f"L = {L:.2f}", f"T = (L/tan(a)) = {T:.2f}"])

    axs[1].legend()
    axs[1].axvline(0, color="black", linestyle="--")
    axs[1].axhline(f_temps[0], color="black", linestyle="--")
    axs[1].axhline(f_temps[-1], color="gray", linestyle="--")
    neighborhood = np.linspace(x[max_dv_i] - 100, x[max_dv_i] + 100)
    axs[1].plot(neighborhood,
                [(((x0 - x[max_dv_i]) * f_temps_dt[max_dv_i]) + f_temps[max_dv_i]) for x0 in neighborhood],
                'C1--')

    axs[1].title.set_text("Temperatura registrada")

    axs[0].grid()
    axs[0].plot(x, f_temps_dt)
    axs[0].axvline(x[max_dv_i], color="r", linestyle="--")
    axs[0].title.set_text("Derivada dT/dt")

    plt.show()


if __name__ == "__main__":
    main()
