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
    y = temp_b_original

    sav = np.array(sig.savgol_filter(y, int(len(x) * 0.05), 5))
    sav_dt = np.gradient(sav, x, edge_order=1)

    max_dv_i = np.argmax(sav_dt)
    print(max_dv_i, x[max_dv_i], sav_dt[max_dv_i])
    print(sav[max_dv_i])

    neighborhood = np.linspace(x[max_dv_i] - 100, x[max_dv_i] + 100)

    fig, axs = plt.subplots(2,1, sharex='all')
    axs[0].grid()
    axs[0].plot(x, sav, label="Temperatura (°C)")
    axs[0].scatter(x[max_dv_i], sav[max_dv_i], color='C1', s=10, label="Ponto de maior derivada")
    axs[0].legend()
    axs[0].plot(neighborhood, [(((x0 - x[max_dv_i]) * sav_dt[max_dv_i]) + sav[max_dv_i]) for x0 in neighborhood], 'C1--')

    axs[0].title.set_text("Temperatura registrada")

    axs[1].grid()
    axs[1].plot(x, sav_dt)
    axs[1].axvline(x[max_dv_i], color="r", linestyle="--")
    axs[1].title.set_text("Derivada dT/dt")

    plt.show()


if __name__ == "__main__":
    main()
