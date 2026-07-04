#include <WiFi.h>

const int LIGHT1_PIN = 15;
const int LIGHT2_PIN = 2;
const int LIGHT3_PIN = 4;
const int FAN1_PIN = 16;
const int FAN2_PIN = 17;

// Variables to store previous state for edge detection (toggling)
int prev_l1 = LOW;
int prev_l2 = LOW;
int prev_l3 = LOW;
int prev_f1 = LOW;
int prev_f2 = LOW;

void setup() {
  Serial.begin(115200);
  
  pinMode(LIGHT1_PIN, INPUT);
  pinMode(LIGHT2_PIN, INPUT);
  pinMode(LIGHT3_PIN, INPUT);
  pinMode(FAN1_PIN, INPUT);
  pinMode(FAN2_PIN, INPUT);
  
  Serial.println("ESP32 Office Monitor Booted. Watching physical switches...");
}

// Helper function to simulate the exact Rust Axum Backend API call
void triggerToggle(String deviceId) {
  Serial.printf("[HTTP POST] -> /api/devices/%s/toggle\n", deviceId.c_str());
}

void loop() {
  int l1 = digitalRead(LIGHT1_PIN);
  int l2 = digitalRead(LIGHT2_PIN);
  int l3 = digitalRead(LIGHT3_PIN);
  int f1 = digitalRead(FAN1_PIN);
  int f2 = digitalRead(FAN2_PIN);
  
  // Detect state changes (if someone flipped a switch) and fire the exact backend API endpoint
  if (l1 != prev_l1) { prev_l1 = l1; triggerToggle("work_room_1_light_1"); }
  if (l2 != prev_l2) { prev_l2 = l2; triggerToggle("work_room_1_light_2"); }
  if (l3 != prev_l3) { prev_l3 = l3; triggerToggle("work_room_1_light_3"); }
  if (f1 != prev_f1) { prev_f1 = f1; triggerToggle("work_room_1_fan_1"); }
  if (f2 != prev_f2) { prev_f2 = f2; triggerToggle("work_room_1_fan_2"); }
  
  delay(100); // 100ms debounce / poll rate
}
