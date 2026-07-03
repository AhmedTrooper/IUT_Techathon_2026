#include <WiFi.h>

const int LIGHT1_PIN = 15;
const int LIGHT2_PIN = 2;
const int LIGHT3_PIN = 4;
const int FAN1_PIN = 16;
const int FAN2_PIN = 17;

void setup() {
  Serial.begin(115200);
  pinMode(LIGHT1_PIN, INPUT);
  pinMode(LIGHT2_PIN, INPUT);
  pinMode(LIGHT3_PIN, INPUT);
  pinMode(FAN1_PIN, INPUT);
  pinMode(FAN2_PIN, INPUT);
  
  Serial.println("ESP32 Office Monitor Booted");
}

void loop() {
  int l1 = digitalRead(LIGHT1_PIN);
  int l2 = digitalRead(LIGHT2_PIN);
  int l3 = digitalRead(LIGHT3_PIN);
  int f1 = digitalRead(FAN1_PIN);
  int f2 = digitalRead(FAN2_PIN);
  
  // In a real scenario, we would send this via HTTP POST to our Rust Backend API
  // e.g., POST /api/devices/update 
  
  Serial.printf("{\"room\":\"Work Room 1\", \"light1\":%d, \"light2\":%d, \"light3\":%d, \"fan1\":%d, \"fan2\":%d}\n", l1, l2, l3, f1, f2);
  
  delay(2000);
}
