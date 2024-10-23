
#include <LiquidCrystal.h>

const int pinthermistor1 = A0;
const int pinthermistor2 = A1;
const int potenciometer = A2;
const int rs = 12, en = 11, d4 = 6, d5 = 4, d6 = 7, d7 = 2;
LiquidCrystal lcd(rs, en, d4, d5, d6, d7);

const int pwm_pos = 3;
const int pwm_neg = 5;

const double beta = 3435.0;
const double r0 = 10000.0;
const double t0 = 273.0 +25.0;
const double rx = r0*exp(-beta/t0);


const double vcc = 5.0;
const double Ra = 9660.0;
const double Rb = 9400.0;

const int nAmostras = 100;

int pwm_pos_cyc = 0;
int pwm_neg_cyc = 0;

void setup(){
  Serial.begin(9600);
  pinMode(pwm_pos, OUTPUT);
  pinMode(pwm_neg, OUTPUT);
  lcd.begin(16,2);
  lcd.print("Hello world");
}

void loop(){
  lcd.setCursor(0,1);
  lcd.print(millis() / 1000);
  long int soma_a = 0;
  long int soma_b = 0;
  long int pot_sum = 0;
  for (int i = 0; i<nAmostras; i++){
    soma_a += analogRead(pinthermistor1);
    soma_b += analogRead(pinthermistor2);
    pot_sum += analogRead(potenciometer);
    delay(1);
  }

  int pot_8 = (pot_sum/nAmostras) >> 2;
  
  double va = (vcc*soma_a)/(nAmostras*1024.0);
  double rta = (vcc*Ra)/va - Ra;
  
  double vb = (vcc*soma_b)/(nAmostras*1024.0);
  double rtb= (vcc*Rb)/vb - Rb;
  
  double ta = beta / log(rta/rx);
  double tb = beta / log(rtb/rx);
  
  ta -= 273;
  tb -= 273;

  Serial.print("> ");
  Serial.print(ta);
  Serial.print(";");
  Serial.print(tb);
  Serial.print(";");
  
// Verifica se há dados disponíveis na porta serial
  if (Serial.available() > 0) {
    String input = Serial.readStringUntil('\n');  // Lê a linha enviada via terminal
    float dutyCycle = input.toFloat();
    // Ajusta o duty cycle com base na entrada
    if (dutyCycle > 0) {
      pwm_pos_cyc = map(dutyCycle, 0, 100, 0, 255);  // Mapeia de 0 a 100% para 0 a 255
      pwm_neg_cyc = 0;
    } else {
      pwm_neg_cyc = map(abs(dutyCycle), 0, 100, 0, 255);  // Mapeia negativo de 0 a -100% para 0 a 255
      pwm_pos_cyc = 0;
    }
    Serial.println(dutyCycle);   
  }

  analogWrite(pwm_pos, pwm_pos_cyc);
  analogWrite(pwm_neg, pwm_neg_cyc);
}

