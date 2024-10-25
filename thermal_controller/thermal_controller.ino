#include <LiquidCrystal.h>

/** Termos de cálculo da formula de Steinhart para a temperatura do NTC. */
#define NTC_BETA 3435.0
#define R0 10000.0
#define T0 (273.0 + 25.0)
#define R_A 9660.0
#define R_B 9400.0

/** Número de amostras a serem feitas para média de leitura dos NTCs. */
#define AVERAGE_AMOUNT 100

/** Definição dos pinos de controle da Peltier. */
#define PWM_POS_PIN 5
#define PWM_NEG_PIN 3

/** Definição dos pinos de leitura dos NTCs. */
#define NTC_A_PIN A0
#define NTC_B_PIN A1

/** Definição dos pinos de controle do display LCD. */
#define LCD_RS 12
#define LCD_EN 11
#define LCD_D4 6
#define LCD_D5 4
#define LCD_D6 7
#define LCD_D7 2

/** Variáveis de controle. */
static int pwm_pos_cyc = 0;
static int pwm_neg_cyc = 0;
static double duty_cycle = 0;
const int RX = (R0 * exp(-NTC_BETA / T0));
LiquidCrystal lcd(rs, en, d4, d5, d6, d7);

/** Função de leitura da temperatura do NTC A. */
double ntc_a_temp(int raw_reading) {
    return  BETA / log((((R_A * 1024.0) / raw_reading) - R_A) / RX);
}

/** Função de leitura da temperatura do NTC B. */
double ntc_b_temp(int raw_reading) {
	return  BETA / log((((R_B * 1024.0) / raw_reading) - R_B) / RX);
}

void setup() {
	Serial.begin(9600);

	pinMode(PWM_POS_PIN, OUTPUT);
	pinMode(PWM_NEG_PIN, OUTPUT);

	lcd.begin(16,2);
	lcd.print("Hello world");
}

void loop() {
	long int ntc_a_raw = 0;
	long int ntc_b_raw = 0;

	/** Média das leituras dos NTCs. */
	for (int i = 0; i < AVERAGE_AMOUNT; i++){
		ntc_a_raw += analogRead(NTC_A_PIN);
		ntc_b_raw += analogRead(NTC_B_PIN);
		delay(1);
	}
	ntc_a_raw /= AVERAGE_AMOUNT;
	ntc_b_raw /= AVERAGE_AMOUNT;

	/** Temperaturas medidas. */
	double t_a = ntc_a_temp(ntc_a_raw) - 273;
	double t_b = ntc_a_temp(ntc_b_raw) - 273;

	/** Ajuste de tensão da Peltier, via serial. */
	if (Serial.available() > 0) {
		String input = Serial.readStringUntil('\n');

		duty_cycle = input.toFloat();
		if (duty_dycle > 100) {
		  duty_cycle = 100;
		} else if (duty_dycle < -100) {
		  duty_dycle = -100;
		}

		if (duty_dycle > 0) {
		  pwm_pos_cyc = map(duty_dycle, 0, 100, 0, 255);
		  pwm_neg_cyc = 0;
		} else {
		  pwm_neg_cyc = map(abs(duty_dycle), 0, 100, 0, 255);
		  pwm_pos_cyc = 0;
	    }

		analogWrite(PWM_POS_PIN, pwm_pos_cyc);
		analogWrite(PWM_NEG_PIN, pwm_neg_cyc);
	}

	/** Escrita dos dados via serial. */
	Serial.print("> ");      /*< Indica o ínicio dos dados. */
	Serial.print(t_a);       /*< Indica a temperatura em graus Celsius do NTC A. */
	Serial.print(";");       /*< Separador de dados. */
	Serial.print(t_b);       /*< Indica a temperatura em graus Celsius do NTC B. */
	Serial.print(";");       /*< Separador de dados. */
	Serial.print(dutyCycle); /*< Indica o ciclo de trabalho da peltier. */
	Serial.print("\n");      /*< Indica o fim dos dados. */
}

