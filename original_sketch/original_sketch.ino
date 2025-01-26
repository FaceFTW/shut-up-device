// Test for minimum program size.

#include <Wire.h>
#include "SSD1306Ascii.h"
#include "SSD1306AsciiWire.h"

// 0X3C+SA0 - 0x3C or 0x3D
#define I2C_ADDRESS 0x3C

// Define proper RST_PIN if required.
#define RST_PIN -1

const int mic_pin = A0;
const int led_pin = 10;
const int buzzer = 9;
char buf[64];
char oled_buf1[64];
char oled_buf2[64];
char oled_buf3[64];



SSD1306AsciiWire oled;
//------------------------------------------------------------------------------
void setup() {
  Wire.begin();
  Wire.setClock(400000L);

#if RST_PIN >= 0
  oled.begin(&Adafruit128x64, I2C_ADDRESS, RST_PIN);
#else // RST_PIN >= 0
  oled.begin(&Adafruit128x64, I2C_ADDRESS);
#endif // RST_PIN >= 0

  oled.setFont(System5x7);
  oled.clear();
  oled.print("Hello world!");
  delay(2000);

  pinMode(buzzer, OUTPUT);
  noTone(buzzer);
  pinMode(led_pin, OUTPUT);
  digitalWrite(led_pin, LOW);
  Serial.begin(115200);
}
//------------------------------------------------------------------------------
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
    sprintf(oled_buf3, "TOO LOUD");
  } else {
    digitalWrite(led_pin, LOW);
    noTone(buzzer);
    sprintf(oled_buf3, "            ");
  }


  sprintf(buf,"Min:%d, Max:%d, Delta:%d, ref:512", mn, mx, delta);
  sprintf(oled_buf1,"Min:%d, Max:%d", mn, mx);
  sprintf(oled_buf2,"Delta:%d", delta);

  oled.home();
  oled.println(oled_buf1);
  oled.println(oled_buf2);
  oled.println(oled_buf3);

  Serial.println(buf);

}