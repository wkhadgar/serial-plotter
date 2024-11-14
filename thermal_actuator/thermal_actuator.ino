#define CLAMP(val, min, max) (((val) > (max)) ? (max) : (((val) < (min)) ? (min) : (val)))

/** Termos de cálculo da formula de Steinhart para a temperatura do NTC. */
#define ZERO_K 273.0
#define NTC_BETA 3435.0
#define R0 10000.0
#define T0 (ZERO_K + 25.0)
#define R_A 9660.0
#define R_B 9400.0

/** Número de amostras a serem feitas para média de leitura dos NTCs. */
#define AVERAGE_AMOUNT 200
#define AMP 100

/** Definição dos pinos de controle da Peltier. */
#define PWM_POS_PIN 3
#define PWM_NEG_PIN 5

/** Definição dos pinos de leitura dos NTCs. */
#define NTC_A_PIN A0
#define NTC_B_PIN A1

/* Constantes do controlador PI, calculadas por Ziegler-Nichols. */
#define L   9.02
#define T   344.21
#define Ti  (2*L)
#define Td  (L/2)
#define Kp  (1.2 * (T / L))
#define Ki  (Kp / Ti)
#define Kd  (Kp * Td)

/** Variáveis de controle. */
static int pwm_pos_cyc = 0;
static int pwm_neg_cyc = 0;
static double duty_cycle = 0;
static const double RX = (R0 * exp(-NTC_BETA / T0));
static double I = 0;
static double err_prev = 0;
static double desired_temp = 25;

static long long int prev_ms = 0;

/** Função de leitura da temperatura do NTC A. */
double ntc_a_temp(double raw_reading) {
    return  NTC_BETA / log((((R_A * (1024.0 * AMP)) / raw_reading) - R_A) / RX);
}

/** Função de leitura da temperatura do NTC B. */
double ntc_b_temp(double raw_reading) {
	return  NTC_BETA / log((((R_B * (1024.0 * AMP)) / raw_reading) - R_B) / RX);

}/** Ajuste do PID, com medidas anti-windup. */
double P_I_D(double dt, double desired, double measured){
	double err = desired - measured;
	double P = Kp * err;
	double I_inc = (Ki * err * dt);
	double D = Kd * (err - err_prev)/dt;

	err_prev = err;

	const double windup_check = P + I + I_inc;

	if (windup_check > 100) {
		return 100;
	} else if (windup_check < -100) {
		return -100;
	} else {
		I += I_inc;
	}

	return P + I;
}

void setup() {
	Serial.begin(115200);

	pinMode(PWM_POS_PIN, OUTPUT);
	pinMode(PWM_NEG_PIN, OUTPUT);
}

void loop() {
	long long int ntc_a_raw = 0;
	long long int ntc_b_raw = 0;

	/** Média das leituras dos NTCs. */
	for (int i = 0; i < AVERAGE_AMOUNT; i++){
		ntc_a_raw += analogRead(NTC_A_PIN);
		ntc_b_raw += analogRead(NTC_B_PIN);
	}
	ntc_a_raw /= (AVERAGE_AMOUNT / AMP);
	ntc_b_raw /= (AVERAGE_AMOUNT / AMP);

	/** Temperaturas medidas. */
	double t_a = ntc_a_temp(ntc_a_raw) - ZERO_K;
	double t_b = ntc_a_temp(ntc_b_raw) - ZERO_K;

	/** Ajuste de temperatura da Peltier, via serial. */
	if (Serial.available() > 0) {
		String input = Serial.readStringUntil('\n');

		desired_temp = input.toFloat();
	}

	duty_cycle = P_I_D((millis() - prev_ms)/1000.0, desired_temp, (t_a + t_b) / 2.0);
  prev_ms = millis();
  
  duty_cycle = CLAMP(duty_cycle, -100, 100);

	if (duty_cycle > 0) {
		pwm_pos_cyc = map(duty_cycle, 0, 100, 0, 255);
		pwm_neg_cyc = 0;
	} else {
		pwm_neg_cyc = map(abs(duty_cycle), 0, 100, 0, 255);
		pwm_pos_cyc = 0;
	}

	analogWrite(PWM_POS_PIN, pwm_pos_cyc);
	analogWrite(PWM_NEG_PIN, pwm_neg_cyc);

	/** Escrita dos dados via serial. */
	Serial.print(">");        /*< Indica o ínicio dos dados. */
	Serial.print(t_a);        /*< Indica a temperatura em graus Celsius do NTC A. */
	Serial.print(";");        /*< Separador de dados. */
	Serial.print(t_b);        /*< Indica a temperatura em graus Celsius do NTC B. */
	Serial.print(";");        /*< Separador de dados. */
	Serial.print(duty_cycle); /*< Indica o ciclo de trabalho da peltier. */
	Serial.println("<");      /*< Indica o fim dos dados. */

  delay(25);
}

