# MQTT Integration Guide - E-Nose Dashboard
## Panduan untuk Tim ESP32 (Tugas A)

**Dibuat:** 4 Oktober 2026  
**Untuk:** Tim ESP32 - Pengiriman Data Training Model ke Dashboard  
**Dashboard URL:** https://enose-dashboard-app.azurewebsites.net/dashboard

---

## 📋 OVERVIEW

Dashboard E-Nose menerima data hasil training model dari ESP32 via protokol **MQTT**. Data yang dikirim akan:
- ✅ Otomatis masuk ke **PostgreSQL database**
- ✅ Tampil **real-time** di dashboard web
- ✅ Generate **PDF report** analytics
- ✅ Tersimpan permanent untuk analisis

---

## 🔧 MQTT BROKER CONFIGURATION

### Connection Details

```
Broker:   test.mosquitto.org
Port:     1883
Protocol: MQTT (non-TLS)
Auth:     None (public broker)
```

⚠️ **CATATAN:** Ini public broker untuk development. Untuk production, gunakan broker private dengan authentication.

### Topic Pattern

```
enose/{device_id}/measurement
```

**Contoh:**
- `enose/ESP32-001/measurement`
- `enose/ESP32-LAB-A/measurement`
- `enose/DEVICE-KOPI-01/measurement`

**Rules:**
- `{device_id}` harus **unique** per device
- Gunakan alphanumeric + dash/underscore (no space)
- Case-sensitive

---

## 📦 JSON PAYLOAD FORMAT

### Required Fields

```json
{
  "sample_id": 1,
  "score": 85.5,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.87,
  "grade": "high_grade",
  "device_id": "ESP32-001",
  "features": [450.2, 623.1, 789.5, 412.8, 556.3, 601.7, 445.9, 523.4],
  "captured_at": "2026-10-04T15:30:00Z"
}
```

### Field Specifications

| Field | Type | Required | Range/Format | Description |
|-------|------|----------|--------------|-------------|
| **sample_id** | integer | ✅ Yes | 1-1000 | ID sample kopi (1-18 untuk kelas training) |
| **score** | float | ✅ Yes | 0.0-100.0 | Score prediksi model (0-100%) |
| **accuracy** | float | ✅ Yes | 0.0-100.0 | Accuracy model (0-100%) |
| **predicted_class** | string | ✅ Yes | - | Nama kelas hasil prediksi (e.g., "Arabica Gayo") |
| **confidence** | float | ✅ Yes | 0.0-1.0 | Confidence model (0.0-1.0, NOT percentage!) |
| **grade** | string | ✅ Yes | `"high_grade"` or `"low_grade"` | Klasifikasi grade kopi |
| **device_id** | string | ✅ Yes | - | ID device ESP32 (harus match dengan topic) |
| **features** | array[float] | ⚠️ Optional | 8 values | Sensor features (MQ2-MQ9) |
| **captured_at** | string | ⚠️ Optional | ISO8601 | Timestamp capture (UTC format) |

### Field Details

#### `predicted_class` (Coffee Type)
Nama kelas hasil prediksi model training. Contoh:
- ✅ `"Arabica Gayo"`
- ✅ `"Arabica Toraja"`
- ✅ `"Arabica Jawa Tengah"`
- ✅ `"Robusta Lampung"`
- ✅ `"Robusta Bengkulu"`
- ❌ `null` atau kosong (wajib diisi!)

#### `confidence`
**PENTING:** Kirim dalam format **0.0-1.0**, BUKAN percentage!
- ✅ `0.87` (untuk 87%)
- ✅ `0.92` (untuk 92%)
- ❌ `87.0` (salah! ini akan jadi 8700%)

Dashboard otomatis convert ke percentage saat display.

#### `grade`
Klasifikasi kualitas kopi (hasil dari model):
- ✅ `"high_grade"` (untuk kopi grade tinggi)
- ✅ `"low_grade"` (untuk kopi grade rendah)
- ❌ `"High"` atau `"Low"` (salah format!)

#### `features`
Array 8 nilai sensor (MQ2-MQ9):
```json
[450.2, 623.1, 789.5, 412.8, 556.3, 601.7, 445.9, 523.4]
```
- Index 0: MQ2 (Methane, Butane, LPG)
- Index 1: MQ3 (Alcohol, Ethanol)
- Index 2: MQ4 (Natural Gas, Methane)
- Index 3: MQ5 (LPG, Natural Gas)
- Index 4: MQ6 (LPG, Butane)
- Index 5: MQ7 (Carbon Monoxide)
- Index 6: MQ8 (Hydrogen Gas)
- Index 7: MQ9 (CO, Methane)

---

## 🐛 TROUBLESHOOTING

### Data Tidak Masuk Dashboard

**1. Check MQTT Connection**
```cpp
if (client.connected()) {
  Serial.println("MQTT connected");
} else {
  Serial.print("MQTT disconnected, state: ");
  Serial.println(client.state());
}
```

**State codes:**
- `-4`: Connection timeout
- `-3`: Connection lost
- `-2`: Connect failed
- `-1`: Disconnected
- `0`: Connected
- `1-5`: Protocol errors

**2. Check WiFi Connection**
```cpp
if (WiFi.status() == WL_CONNECTED) {
  Serial.println("WiFi OK");
} else {
  Serial.println("WiFi disconnected!");
  WiFi.reconnect();
}
```

**3. Validate JSON Payload**
```cpp
// Print payload before sending
Serial.println(payload);

// Check JSON size (must be < 1024 bytes)
Serial.print("JSON size: ");
Serial.println(payload.length());
```

**4. Check Topic Format**
- ✅ `enose/ESP32-001/measurement` (correct)
- ❌ `enose/ESP32-001` (missing `/measurement`)
- ❌ `Enose/ESP32-001/measurement` (case-sensitive!)

### Confidence Tampil 8700%

**Masalah:** Kirim confidence dalam format percentage (87.0) bukan decimal (0.87)

**Fix:**
```cpp
// ❌ SALAH
doc["confidence"] = 87.0;  // Akan tampil 8700%!

// ✅ BENAR
doc["confidence"] = 0.87;  // Tampil 87.0%
```

### Grade Tidak Tampil

**Masalah:** Format grade salah

**Fix:**
```cpp
// ❌ SALAH
doc["grade"] = "High";
doc["grade"] = "high";
doc["grade"] = "HIGH_GRADE";

// ✅ BENAR
doc["grade"] = "high_grade";  // lowercase + underscore
doc["grade"] = "low_grade";
```

---

## 📊 DATA FLOW DIAGRAM

```
┌─────────────┐
│   ESP32     │
│  (Tugas A)  │
│             │
│  - Sensor   │
│  - Model    │
│  - MQTT     │
└──────┬──────┘
       │
       │ MQTT Publish
       │ Topic: enose/{device_id}/measurement
       │ Payload: JSON (predicted_class, confidence, grade)
       │
       ↓
┌──────────────────────────────────┐
│   test.mosquitto.org (Broker)    │
│   Port: 1883                     │
└──────┬───────────────────────────┘
       │
       │ Subscribe
       │
       ↓
┌──────────────────────────────────┐
│  Azure App Service               │
│  (Dashboard Backend - Rust)      │
│                                  │
│  - MQTT Subscriber               │
│  - JSON Validation               │
│  - Database Insert               │
└──────┬───────────────────────────┘
       │
       │ SQL INSERT
       │
       ↓
┌──────────────────────────────────┐
│  PostgreSQL Database (Railway)   │
│                                  │
│  Table: measurements             │
│  - predicted_class               │
│  - confidence                    │
│  - grade                         │
│  - score, accuracy, features     │
└──────┬───────────────────────────┘
       │
       │ REST API
       │ GET /api/v1/measurements
       │
       ↓
┌──────────────────────────────────┐
│  Dashboard Web (HTML/JS)         │
│  https://enose-dashboard-app.    │
│  azurewebsites.net/dashboard     │
│                                  │
│  - Real-time table               │
│  - KPI cards                     │
│  - Charts                        │
│  - PDF download                  │
└──────────────────────────────────┘
```

---

## 📝 CHECKLIST SEBELUM DEPLOY

- [ ] WiFi SSID & password sudah benar
- [ ] `device_id` unique per ESP32
- [ ] Topic format: `enose/{device_id}/measurement`
- [ ] JSON fields lengkap (sample_id, score, accuracy, predicted_class, confidence, grade)
- [ ] **Confidence format 0.0-1.0** (BUKAN percentage!)
- [ ] **Grade format "high_grade" atau "low_grade"** (lowercase + underscore)
- [ ] Features array punya 8 nilai
- [ ] Test MQTT connection sebelum kirim data
- [ ] Serial Monitor aktif untuk debugging
- [ ] Dashboard dapat diakses & data masuk

---

## 🆘 KONTAK & SUPPORT

**Dashboard URL:**  
https://enose-dashboard-app.azurewebsites.net/dashboard

**Database:** PostgreSQL (Railway - managed)

**Support:**
- Email: mahendrafhrz1202@gmail.com
- Check dashboard logs untuk error details

**Testing MQTT:**
- Public broker: `test.mosquitto.org:1883`
- Test tool: MQTT Explorer, MQTTX, atau Python script

---

## 📚 REFERENSI

### MQTT Resources
- MQTT.org: https://mqtt.org/
- PubSubClient Library: https://github.com/knolleary/pubsubclient
- Mosquitto Public Broker: https://test.mosquitto.org/

### JSON Resources
- ArduinoJson Documentation: https://arduinojson.org/
- JSON Format Validator: https://jsonlint.com/

### ESP32 Resources
- ESP32 Arduino Core: https://docs.espressif.com/projects/arduino-esp32/
- WiFi Library: https://arduino-esp32.readthedocs.io/en/latest/api/wifi.html

---

**Dibuat oleh:** Tim Backend (Tugas C)  
**Last Updated:** 4 Oktober 2026  
**Version:** 1.0

**Good luck, Tim ESP32! 🚀☕**
