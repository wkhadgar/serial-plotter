const uint8_t TEMP_PINS[] = {A0, A2};
const uint8_t HEATER_PINS[] = {3, 5};
const uint8_t LED_PIN = 9;

const uint16_t BAUD_RATE = 9600;
const uint8_t READINGS_TO_AVERAGE = 5;
const uint16_t SERIAL_TIMEOUT = 100;

enum sensor_id {
	SENSOR_TEMPERATURE_A = 0,
	SENSOR_TEMPERATURE_B,
	SENSOR_COUNT,
};

uint32_t readings[SENSOR_COUNT] = {};
uint8_t pwm_values[SENSOR_COUNT] = {};

void setup() {
	Serial.begin(BAUD_RATE);
	for (uint8_t i = 0; i < SENSOR_COUNT; i++) {
		pinMode(HEATER_PINS[i], OUTPUT);
	}
	pinMode(LED_PIN, OUTPUT);
    analogReference(EXTERNAL);
}

void loop() {
	read_temperatures();
	process_serial();
	update_outputs();
}

void read_temperatures() {
	for (uint8_t s = 0; s < SENSOR_COUNT; s+=1) {
		readings[s] = 0;
		for (uint8_t reading_index = 0; reading_index < READINGS_TO_AVERAGE; reading_index++) {
			readings[s] += analogRead(TEMP_PINS[s]);
		}
		readings[s] /= READINGS_TO_AVERAGE;
	}

	// 10 bits temp to 8 bits LED value
	uint8_t led_value = max(readings[SENSOR_TEMPERATURE_A], readings[SENSOR_TEMPERATURE_B]) >> 2;
	analogWrite(LED_PIN, led_value);
}

/**
 * Handle serial communication with Python controller
 */
void process_serial() {
	if (Serial.available()) {
		String command = Serial.readStringUntil('\n');
		command.trim();

		if (command == "GET_TEMP") {
			Serial.print(readings[SENSOR_TEMPERATURE_A]);
			Serial.print(",");
			Serial.println(readings[SENSOR_TEMPERATURE_B]);
		} else if (command.startsWith("SET_PWM:")) {
			int commaPos = command.indexOf(',');
			if (commaPos > 0) {
				pwm_values[0] = command.substring(8, commaPos).toInt();
				pwm_values[1] = command.substring(commaPos + 1).toInt();
			}
		} else if (command.startsWith("GET_PWM")) {
			Serial.print(pwm_values[0]);
			Serial.print(",");
			Serial.println(pwm_values[1]);
    }
	}
}

/**
 * Update PWM outputs for heaters and LED
 */
void update_outputs() {
	for (uint8_t i = 0; i < SENSOR_COUNT; i++) {
		analogWrite(HEATER_PINS[i], pwm_values[i]);
	}
}
