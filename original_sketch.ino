
const int mic_pin = A0;
const int led_pin = 10;
const int buzzer = 9;
char buf[64];

void setup() {
  pinMode(buzzer, OUTPUT);
  noTone(buzzer);
  pinMode(led_pin, OUTPUT);
  digitalWrite(led_pin, LOW);
  Serial.begin(115200);

}
void loop() {
  // put your main code here, to run repeatedly:
  int mn = 1024;
  int mx = 0;

  for (int i = 0; i < 1000; ++i) {

    int val = analogRead(mic_pin);
    
    mn = min(mn, val);
    mx = max(mx, val);
  }

  int delta = mx - mn;

  if (delta >=20){
    digitalWrite(led_pin, HIGH);
    tone(buzzer, 1000);

  } else {
    digitalWrite(led_pin, LOW);
    noTone(buzzer);
  }


  sprintf(buf,"Min:%d, Max:%d, Delta:%d, ref:512", mn, mx, delta);
  Serial.println(buf);

}