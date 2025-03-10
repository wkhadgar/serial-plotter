import matplotlib.pyplot as plt
import numpy as np

# Valores base para calculo das leituras do ADC
ZERO_K_C = 273.0
NTC_BETA = 3435.0
R0 = 10000.0
T0 = (ZERO_K_C + 25.0)
R_A = 9880.0
R_B = 9990.0
RX = (R0 * np.exp(-NTC_BETA / T0))


# Função de leitura da temperatura do NTC A. #
def ntc_a_temp(raw_reading):
    t_k = NTC_BETA / np.log((((R_A * 4096.0) / (raw_reading + 0.0001)) - R_A) / RX)
    return t_k - ZERO_K_C


# Função de leitura da temperatura do NTC B. #
def ntc_b_temp(raw_reading):
    t_k = NTC_BETA / np.log((((R_B * 4096.0) / (raw_reading + 0.0001)) - R_B) / RX)
    return t_k - ZERO_K_C


def main():
    ntc_a_temperatures = []
    ntc_b_temperatures = []
    for r in range(2 ** 12):
        sim_temp_a = round(ntc_a_temp(r), 3)
        sim_temp_b = round(ntc_b_temp(r), 3)

        if sim_temp_a < -20:
            sim_temp_a = -20
        elif sim_temp_a > 100:
            sim_temp_a = 100

        if sim_temp_b < -20:
            sim_temp_b = -20
        elif sim_temp_b > 100:
            sim_temp_b = 100

        ntc_a_temperatures.append(sim_temp_a)
        ntc_b_temperatures.append(sim_temp_b)

    plt.plot(list(range(len(ntc_a_temperatures))), ntc_a_temperatures)
    plt.plot(list(range(len(ntc_b_temperatures))), ntc_b_temperatures)
    plt.show()

    with open("./luts.txt", "w") as lut_file:
        lut_file.write(
            f"NTC_A LUT [0->{len(ntc_a_temperatures)}]->[{ntc_a_temperatures[0]}->{ntc_a_temperatures[-1]}]:\n")
        lut_file.write("{\n\t")
        count = 0
        for value in ntc_a_temperatures:
            count += 1
            lut_file.write(f"{value}, ")
            if count % 15 == 0:
                lut_file.write("\n\t")
        lut_file.write("\n}\n\n")

        lut_file.write(
            f"NTC_B LUT [0->{len(ntc_b_temperatures)}]->[{ntc_b_temperatures[0]}->{ntc_b_temperatures[-1]}]:\n")
        lut_file.write("{\n\t")
        count = 0
        for value in ntc_b_temperatures:
            count += 1
            lut_file.write(f"{value}, ")
            if count % 15 == 0:
                lut_file.write("\n\t")
        lut_file.write("\n}\n")


if __name__ == "__main__":
    main()
