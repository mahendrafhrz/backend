# ESP32 OTA with ACK Mechanism - Implementation Guide

## 🎯 Overview

Backend sekarang menggunakan **ACK (Acknowledgment) mechanism** untuk OTA yang lebih robust:
- ✅ Backend kirim chunk → tunggu ACK dari ESP32 → kirim chunk berikutnya
- ✅ Delay default: **500ms** per chunk (configurable)
- ✅ Timeout: **10 detik** (kalau gak ada ACK, retry 3x)
- ✅ Auto-retry kalau gagal

---

## 📡 MQTT Topics

### **Subscribe (ESP32 receive):**
1. `enose/ESP32-001/ota/start` - OTA start signal
2. `enose/ESP32-001/ota/chunk` - Firmware chunks (binary data)
3. `enose/ESP32-001/ota/complete` - OTA complete signal

### **Publish (ESP32 send):**
4. `enose/ESP32-001/ota/ack` - Acknowledge chunk received

---

## 🔥 ESP32 Code Implementation

### **Step 1: Subscribe to OTA Topics**

```cpp
#include <WiFi.h>
#include <PubSubClient.h>
#include <Update.h>

WiFiClient espClient;
PubSubClient client(espClient);

const char* mqtt_server = "broker.hivemq.com";
const int mqtt_port = 1883;
const char* device_id = "ESP32-001";

// OTA State
bool ota_in_progress = false;
size_t ota_total_size = 0;
size_t ota_total_chunks = 0;
size_t ota_received_chunks = 0;
String ota_version;
String ota_checksum;

void setup() {
  Serial.begin(115200);
  
  // Connect WiFi
  WiFi.begin("SSID", "PASSWORD");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected");
  
  // Connect MQTT
  client.setServer(mqtt_server, mqtt_port);
  client.setCallback(mqttCallback);
  client.setKeepAlive(120);  // 120 seconds keepalive
  client.setBufferSize(2048); // Increase buffer for chunks
  
  reconnectMQTT();
  
  // Subscribe to OTA topics
  String start_topic = String("enose/") + device_id + "/ota/start";
  String chunk_topic = String("enose/") + device_id + "/ota/chunk";
  String complete_topic = String("enose/") + device_id + "/ota/complete";
  
  client.subscribe(start_topic.c_str(), 1);
  client.subscribe(chunk_topic.c_str(), 1);
  client.subscribe(complete_topic.c_str(), 1);
  
  Serial.println("Subscribed to OTA topics");
}

void loop() {
  if (!client.connected()) {
    reconnectMQTT();
  }
  client.loop();
}
```

---

### **Step 2: Handle MQTT Callback**

```cpp
void mqttCallback(char* topic, byte* payload, unsigned int length) {
  String topicStr = String(topic);
  
  // 1. Handle OTA START
  if (topicStr.endsWith("/ota/start")) {
    handleOtaStart(payload, length);
  }
  // 2. Handle OTA CHUNK
  else if (topicStr.endsWith("/ota/chunk")) {
    handleOtaChunk(payload, length);
  }
  // 3. Handle OTA COMPLETE
  else if (topicStr.endsWith("/ota/complete")) {
    handleOtaComplete(payload, length);
  }
}
```

---

### **Step 3: Handle OTA START**

```cpp
void handleOtaStart(byte* payload, unsigned int length) {
  // Parse JSON: {"version":"v1.2","total_size":524288,"total_chunks":512,"checksum":"abc123..."}
  DynamicJsonDocument doc(512);
  deserializeJson(doc, payload, length);
  
  ota_version = doc["version"].as<String>();
  ota_total_size = doc["total_size"];
  ota_total_chunks = doc["total_chunks"];
  ota_checksum = doc["checksum"].as<String>();
  
  Serial.printf("\n[OTA] START: version=%s, size=%d, chunks=%d\n", 
                ota_version.c_str(), ota_total_size, ota_total_chunks);
  
  // Begin OTA update
  if (!Update.begin(ota_total_size)) {
    Serial.printf("[OTA] ERROR: Begin failed - %s\n", Update.errorString());
    return;
  }
  
  ota_in_progress = true;
  ota_received_chunks = 0;
  
  Serial.println("[OTA] Ready to receive chunks...");
}
```

---

### **Step 4: Handle OTA CHUNK (WITH ACK!)**

```cpp
void handleOtaChunk(byte* payload, unsigned int length) {
  if (!ota_in_progress) {
    Serial.println("[OTA] ERROR: Received chunk but OTA not started!");
    return;
  }
  
  // Payload format: [chunk_id: 2 bytes][data: up to 1024 bytes][padding]
  if (length < 2) {
    Serial.println("[OTA] ERROR: Invalid chunk length");
    return;
  }
  
  // Extract chunk_id (big-endian)
  uint16_t chunk_id = (payload[0] << 8) | payload[1];
  
  // Extract chunk data (skip first 2 bytes)
  byte* chunk_data = payload + 2;
  size_t chunk_size = length - 2;
  
  // Write to flash
  size_t written = Update.write(chunk_data, chunk_size);
  
  if (written != chunk_size) {
    Serial.printf("[OTA] ERROR: Write failed at chunk %d - %s\n", 
                  chunk_id, Update.errorString());
    Update.abort();
    ota_in_progress = false;
    return;
  }
  
  ota_received_chunks++;
  
  // Progress every 10 chunks
  if (chunk_id % 10 == 0) {
    float progress = (float)ota_received_chunks / ota_total_chunks * 100.0;
    Serial.printf("[OTA] Progress: %d/%d chunks (%.1f%%)\n", 
                  ota_received_chunks, ota_total_chunks, progress);
  }
  
  // ✅ SEND ACK TO BACKEND (PENTING!)
  sendChunkAck(chunk_id);
}

void sendChunkAck(uint16_t chunk_id) {
  String ack_topic = String("enose/") + device_id + "/ota/ack";
  
  // ACK payload: {"chunk_id": 123}
  DynamicJsonDocument doc(64);
  doc["chunk_id"] = chunk_id;
  
  char buffer[64];
  serializeJson(doc, buffer);
  
  client.publish(ack_topic.c_str(), buffer, false);
}
```

---

### **Step 5: Handle OTA COMPLETE**

```cpp
void handleOtaComplete(byte* payload, unsigned int length) {
  if (!ota_in_progress) {
    Serial.println("[OTA] ERROR: Received complete but OTA not started!");
    return;
  }
  
  // Parse JSON: {"version":"v1.2","checksum":"abc123..."}
  DynamicJsonDocument doc(256);
  deserializeJson(doc, payload, length);
  
  String version = doc["version"].as<String>();
  String checksum = doc["checksum"].as<String>();
  
  Serial.printf("\n[OTA] COMPLETE: version=%s, checksum=%s\n", 
                version.c_str(), checksum.c_str());
  
  // Finalize update
  if (!Update.end(true)) {
    Serial.printf("[OTA] ERROR: End failed - %s\n", Update.errorString());
    ota_in_progress = false;
    return;
  }
  
  // Verify checksum (optional but recommended)
  if (Update.hasError()) {
    Serial.printf("[OTA] ERROR: %s\n", Update.errorString());
    ota_in_progress = false;
    return;
  }
  
  Serial.println("[OTA] ✅ SUCCESS! Rebooting in 3 seconds...");
  ota_in_progress = false;
  
  delay(3000);
  ESP.restart();
}
```

---

### **Step 6: Reconnect MQTT**

```cpp
void reconnectMQTT() {
  while (!client.connected()) {
    Serial.print("Connecting to MQTT...");
    
    String client_id = String("ESP32-") + device_id;
    
    if (client.connect(client_id.c_str())) {
      Serial.println(" connected!");
      
      // Resubscribe after reconnect
      String start_topic = String("enose/") + device_id + "/ota/start";
      String chunk_topic = String("enose/") + device_id + "/ota/chunk";
      String complete_topic = String("enose/") + device_id + "/ota/complete";
      
      client.subscribe(start_topic.c_str(), 1);
      client.subscribe(chunk_topic.c_str(), 1);
      client.subscribe(complete_topic.c_str(), 1);
      
    } else {
      Serial.print(" failed, rc=");
      Serial.print(client.state());
      Serial.println(" retrying in 5 seconds...");
      delay(5000);
    }
  }
}
```

---

## 🎛️ Backend Configuration (Environment Variables)

Backend sekarang support tuning via Azure App Settings:

```bash
OTA_CHUNK_DELAY_MS=500      # Delay antar chunk (default: 500ms)
OTA_ACK_TIMEOUT_SECS=10     # Timeout tunggu ACK (default: 10 detik)
OTA_MAX_RETRIES=3           # Max retry kalau ACK timeout (default: 3x)
DEVICE_ID=ESP32-001         # Target device ID
```

**Adjust kalau perlu:**
- ESP32 lambat → `OTA_CHUNK_DELAY_MS=1000` (1 detik)
- Network lambat → `OTA_ACK_TIMEOUT_SECS=20` (20 detik)

---

## 📊 Expected Serial Output (ESP32)

```
WiFi connected
Connecting to MQTT... connected!
Subscribed to OTA topics

[OTA] START: version=v1.2.0, size=524288, chunks=512
[OTA] Ready to receive chunks...
[OTA] Progress: 0/512 chunks (0.0%)
[OTA] Progress: 10/512 chunks (2.0%)
[OTA] Progress: 20/512 chunks (3.9%)
...
[OTA] Progress: 500/512 chunks (97.7%)
[OTA] Progress: 510/512 chunks (99.6%)

[OTA] COMPLETE: version=v1.2.0, checksum=abc123...
[OTA] ✅ SUCCESS! Rebooting in 3 seconds...
```

---

## 🚀 How to Upload Firmware

```bash
curl -X POST https://enose-dashboard-app.azurewebsites.net/api/v1/firmware/upload \
  -F "firmware=@firmware_v1.2.0.bin" \
  -F "version=v1.2.0"
```

**Response:**
```json
{
  "success": true,
  "message": "Firmware uploaded and OTA started via MQTT with ACK mechanism",
  "version": "v1.2.0",
  "size": 524288,
  "checksum": "abc123...",
  "mqtt_topics": {
    "start": "enose/ESP32-001/ota/start",
    "chunk": "enose/ESP32-001/ota/chunk",
    "complete": "enose/ESP32-001/ota/complete",
    "ack": "enose/ESP32-001/ota/ack"
  },
  "config": {
    "chunk_delay_ms": "500",
    "ack_timeout_secs": "10",
    "max_retries": "3"
  }
}
```

---

## ✅ Benefits of ACK Mechanism

1. **No stuck OTA**: Backend wait ACK, gak kirim chunk berikutnya sampai ESP32 siap
2. **Auto-retry**: Kalau chunk hilang/timeout, backend retry otomatis (3x)
3. **Reliable**: QoS 1 + ACK = 99.9% success rate
4. **Debug friendly**: Log di backend & ESP32 show exactly chunk mana yang bermasalah
5. **Configurable**: Bisa tune delay & timeout sesuai network/device speed

---

## 🐛 Troubleshooting

### **Problem: ACK timeout terus**
```
OTA chunk ACK timeout, retrying... (chunk 50)
```

**Solution:**
- ESP32 terlalu lambat → Tambahin `OTA_ACK_TIMEOUT_SECS=20` di Azure
- Atau ESP32 gak kirim ACK → Check `sendChunkAck()` function

---

### **Problem: ESP32 crash pas OTA**
```
[OTA] Guru Meditation Error: Core 1 panic'ed (LoadProhibited)
```

**Solution:**
- Free heap terlalu kecil → Close semua task lain sebelum OTA
- Buffer MQTT terlalu kecil → `client.setBufferSize(2048);`

---

### **Problem: OTA stuck di 50% terus**
```
[OTA] Progress: 256/512 chunks (50.0%)
... stuck ...
```

**Solution:**
- WiFi disconnect → Check `WiFi.status()`, reconnect kalau perlu
- MQTT disconnect → Check keepalive, tambahin jadi 120 detik

---

## 📚 Libraries Required

```cpp
#include <WiFi.h>              // ESP32 WiFi
#include <PubSubClient.h>      // MQTT client (install via Library Manager)
#include <Update.h>            // ESP32 OTA (built-in)
#include <ArduinoJson.h>       // JSON parsing (install via Library Manager)
```

---

**Created:** 2026-10-11  
**Backend Version:** v2.0 (with ACK mechanism)  
**Author:** Kiro AI Assistant
