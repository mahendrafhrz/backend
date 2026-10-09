# E-NOSE DASHBOARD - SYSTEM DOCUMENTATION

**Last Updated:** 2026-09-30  
**Version:** 1.0.0  
**Platform:** Azure App Service + MQTT  
**Status:** ✅ Production Ready

---

## 1. KESIMPULAN SISTEM

### Arsitektur Sistem

```
┌─────────────┐    MQTT     ┌─────────────┐    REST API    ┌─────────────┐
│  ESP32      │ ─────────> │   Backend   │ ─────────────> │  Dashboard  │
│  (Tugas A)  │  QoS 1     │   Azure     │    JSON        │     Web     │
│             │            │  (Tugas C)  │                │  (Tugas C)  │
└─────────────┘            └─────────────┘                └─────────────┘
      │                           │
      │                           │
   9 Sensor Gas              SQLite DB
   TinyML Edge              (ephemeral)
   Impulse
```

### Komponen Utama

1. **ESP32 (Tugas A)** - IoT Device dengan TinyML
2. **MQTT Broker** - test.mosquitto.org (message queue)
3. **Backend Azure** - Rust + Axum + SQLite
4. **Dashboard Web** - HTML + JavaScript + Chart.js

---

## 2. ARSITEKTUR SISTEM

### A. ESP32 (Tugas A) - IoT Device

**Hardware:**
- ESP32 microcontroller
- 9x sensor gas (MQ2, MQ3, MQ4, MQ5, MQ6, MQ7, MQ8, MQ9, MQ135)

**Software:**
- TinyML Edge Impulse (machine learning inference)
- MQTT Client (WiFi communication)

**Output Format:**
```json
{
  "device_id": "ESP32-001",
  "sample_id": 1,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade",
  "timestamp": "2026-09-30T10:00:00Z"
}
```

**Komunikasi:**
- Protocol: MQTT v3.1.1
- Broker: test.mosquitto.org:1883
- Topic: `enose/{device_id}/measurement`
- QoS: 1 (At least once delivery)

---

### B. MQTT Broker - Message Queue

**Configuration:**
- Host: `test.mosquitto.org`
- Port: `1883` (non-TLS)
- Protocol: MQTT v3.1.1
- Topic Pattern: `enose/+/measurement` (wildcard support)
- Keep-Alive: 60 seconds
- QoS: 1 (At least once)

**Fungsi:**
- Terima data dari ESP32 via publish
- Forward ke Backend via subscribe
- Auto-reconnect on connection loss

---

### C. Backend Azure (Tugas C) - Cloud Server

**Platform:**
- Service: Azure App Service
- Plan: F1 (Free Tier)
- Region: Indonesia Central
- Container: Docker (Azure Container Registry)

**Technology Stack:**
- Language: Rust 1.75+
- Framework: Axum 0.8
- Database: SQLite (ephemeral, resets on restart)
- MQTT Client: rumqttc

**URL:**
- Production: https://enose-dashboard-app.azurewebsites.net
- Health Check: https://enose-dashboard-app.azurewebsites.net/health
- Dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard

**Features:**
- ✅ MQTT Subscriber (auto-reconnect)
- ✅ REST API endpoints
- ✅ OTA firmware upload/download
- ✅ PDF report generation
- ✅ Real-time data processing

**Environment Variables:**
```
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
MQTT_ENABLED=true
MQTT_BROKER=test.mosquitto.org
MQTT_PORT=1883
MQTT_TOPIC=enose/+/measurement
BASE_URL=https://enose-dashboard-app.azurewebsites.net
```

---

### D. Dashboard Web (Tugas C) - Frontend

**Technology:**
- HTML5 + CSS3
- JavaScript (Vanilla)
- Chart.js 4.4.0 (data visualization)

**Features:**
- ✅ Real-time statistics
- ✅ Auto-refresh every 5 seconds
- ✅ Confidence Trend chart (line graph)
- ✅ Class Distribution chart (doughnut: high vs low grade)
- ✅ Recent Measurements table
- ✅ PDF report download
- ✅ OTA firmware upload interface
- ✅ Dark gradient glassmorphism UI

**Pages:**
1. Dashboard - Main monitoring page
2. Data Management - Full data table with search/delete
3. OTA Update - Firmware upload interface

---

## 3. DATABASE SCHEMA

### Table: `measurements`

```sql
CREATE TABLE measurements (
    id TEXT PRIMARY KEY,                    -- UUID
    sample_id INTEGER NOT NULL,             -- Sample number (>= 1)
    score REAL NOT NULL,                    -- Score 0-100
    accuracy REAL NOT NULL,                 -- Accuracy 0-100
    predicted_class TEXT,                   -- Class name from TinyML
    confidence REAL,                        -- Confidence 0.0-1.0
    grade TEXT,                             -- "high_grade" or "low_grade"
    source TEXT NOT NULL DEFAULT 'api',     -- "mqtt" or "api"
    device_id TEXT,                         -- ESP32 device ID
    features_json TEXT,                     -- JSON array of sensor readings
    captured_at TEXT NOT NULL,              -- ISO 8601 timestamp
    created_at TEXT NOT NULL                -- ISO 8601 timestamp
);
```

**Key Fields:**
- `predicted_class` - Nama kelas hasil TinyML (e.g., "Arabica Gayo")
- `confidence` - Confidence value dari TinyML (0.0-1.0)
- `grade` - Klasifikasi high/low grade (ditentukan ESP32)

**Notes:**
- Database SQLite bersifat **ephemeral** (reset on Azure restart)
- For persistent storage, upgrade to Azure Database for PostgreSQL

---

## 4. REST API ENDPOINTS

### Base URL
```
https://enose-dashboard-app.azurewebsites.net
```

### Endpoints

#### 1. Health Check
```http
GET /health
```
**Response:**
```json
{
  "status": "ok",
  "service": "enose-cloud"
}
```

#### 2. Get All Measurements
```http
GET /api/v1/measurements
```
**Response:**
```json
[
  {
    "id": "uuid",
    "sample_id": 1,
    "score": 0.95,
    "accuracy": 0.95,
    "predicted_class": "Arabica Gayo",
    "confidence": 0.95,
    "grade": "high_grade",
    "source": "mqtt",
    "device_id": "ESP32-001",
    "features_json": "[755,1108,960,...]",
    "captured_at": "2026-09-30T10:00:00Z",
    "created_at": "2026-09-30T10:00:00Z"
  }
]
```

#### 3. Create Measurement (Manual POST)
```http
POST /api/v1/measurements
Content-Type: application/json

{
  "sample_id": 1,
  "score": 0.95,
  "accuracy": 0.95,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade",
  "device_id": "ESP32-001"
}
```

#### 4. Delete Measurement
```http
DELETE /api/v1/measurements/{id}
```

#### 5. Analytics Summary
```http
GET /api/v1/analytics/summary
```

#### 6. Download PDF Report
```http
GET /api/v1/reports/analytics.pdf
```

#### 7. Upload Firmware (OTA)
```http
POST /api/v1/firmware/upload
Content-Type: multipart/form-data

firmware: [binary file]
version: "1.2.3"
```

#### 8. Download Firmware (OTA)
```http
GET /api/v1/firmware/{version}
```

---

## 5. MQTT CONFIGURATION

### Publisher (ESP32)

**Topic Pattern:**
```
enose/{device_id}/measurement
```

**Example:**
```
enose/ESP32-001/measurement
```

**Payload:**
```json
{
  "device_id": "ESP32-001",
  "sample_id": 1,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade",
  "timestamp": "2026-09-30T10:00:00Z"
}
```

### Subscriber (Backend)

**Topic Pattern:**
```
enose/+/measurement
```
(Wildcard `+` matches any device_id)

**Behavior:**
- Auto-reconnect on connection loss (10s delay)
- Payload validation (sample_id >= 1, score/accuracy 0-100)
- Auto-save to database
- Structured logging

---

## 6. OTA FIRMWARE UPDATE

### Flow

```
User (Browser)
    ↓ Upload firmware.bin + version
Dashboard (Tugas C)
    ↓ Save + Publish MQTT
MQTT Broker (enose/ota/update)
    ↓ Subscribe
ESP32 (Tugas A)
    ↓ Download via HTTP
Backend Azure
    ↓ Flash + Reboot
ESP32
```

### MQTT Topic
```
enose/ota/update
```

### Notification Payload
```json
{
  "version": "1.2.3",
  "url": "https://enose-dashboard-app.azurewebsites.net/api/v1/firmware/1.2.3",
  "size": 123456,
  "checksum": "abc123...",
  "timestamp": "2026-09-30T10:00:00Z"
}
```

### ESP32 Implementation
1. Subscribe to `enose/ota/update`
2. Receive notification
3. Download firmware from `url` via HTTP GET
4. Verify MD5 checksum
5. Flash firmware
6. Reboot

---

## 7. ESP32 INTEGRATION GUIDE

### Required Payload Structure

```json
{
  "device_id": "ESP32-001",
  "sample_id": 1,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade",
  "timestamp": "2026-09-30T10:00:00Z"
}
```

### Field Explanation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `device_id` | String | ✅ Yes | Unique ESP32 ID |
| `sample_id` | Integer | ✅ Yes | Sample number (>= 1) |
| `predicted_class` | String | ✅ Yes | Class name from TinyML |
| `confidence` | Float | ✅ Yes | Confidence 0.0-1.0 |
| `grade` | String | ✅ Yes | "high_grade" or "low_grade" |
| `timestamp` | String | ⚠️ Optional | ISO 8601 format |

### How to Determine `grade`

**Option 1: Based on Class Name (Recommended)**
```cpp
String predicted_class = result.classification[0].label;
float confidence = result.classification[0].value;

String grade;
if (predicted_class == "Arabica Gayo" || 
    predicted_class == "Arabica Toraja" || 
    predicted_class == "Arabica Jawa Tengah" ||
    predicted_class == "Arabica Bali" ||
    predicted_class == "Arabica Ijen" ||
    predicted_class == "Arabica Papua Wamena") {
  grade = "high_grade";
} else {
  grade = "low_grade";
}
```

**Option 2: Based on Confidence Threshold (Not Recommended)**
```cpp
String grade;
if (confidence > 0.85) {
  grade = "high_grade";
} else {
  grade = "low_grade";
}
```

**⚠️ Warning:** Option 2 tidak disarankan karena confidence != quality.

### Arduino Code Example

```cpp
#include <WiFi.h>
#include <PubSubClient.h>
#include <ArduinoJson.h>

const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";
const char* mqtt_server = "test.mosquitto.org";
const int mqtt_port = 1883;
const char* device_id = "ESP32-001";

WiFiClient espClient;
PubSubClient client(espClient);

void setup() {
  Serial.begin(115200);
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  
  client.setServer(mqtt_server, mqtt_port);
  while (!client.connected()) {
    if (client.connect(device_id)) {
      Serial.println("MQTT connected");
    } else {
      delay(5000);
    }
  }
}

void sendMeasurement(String predicted_class, float confidence, String grade) {
  char topic[100];
  sprintf(topic, "enose/%s/measurement", device_id);
  
  StaticJsonDocument<512> doc;
  doc["device_id"] = device_id;
  doc["sample_id"] = 1;
  doc["predicted_class"] = predicted_class;
  doc["confidence"] = confidence;
  doc["grade"] = grade;
  doc["timestamp"] = getISO8601Timestamp();
  
  String payload;
  serializeJson(doc, payload);
  client.publish(topic, payload.c_str(), true);
}

void loop() {
  client.loop();
  // Run TinyML inference
  // ...
  sendMeasurement("Arabica Gayo", 0.95, "high_grade");
  delay(5000);
}
```

---

## 8. DEPLOYMENT & MAINTENANCE

### Azure Deployment

**Resources:**
- Resource Group: `enose-dashboard-rg`
- Container Registry: `enosedashboard2024`
- App Service: `enose-dashboard-app`
- Plan: F1 (Free Tier)
- Region: Indonesia Central

**Deployment Steps:**
```powershell
# 1. Build Docker image
docker build -t enosedashboard2024.azurecr.io/enose-dashboard:latest .

# 2. Login to ACR
az acr login --name enosedashboard2024

# 3. Push image
docker push enosedashboard2024.azurecr.io/enose-dashboard:latest

# 4. Restart App Service
az webapp restart --name enose-dashboard-app --resource-group enose-dashboard-rg
```

**Build Time:**
- Local build: ~15 seconds (with cache)
- Docker build: ~3-4 minutes
- Push to ACR: ~1-2 minutes
- Azure restart: ~30 seconds

### Configuration Management

**Environment Variables (Azure):**
```bash
# View all settings
az webapp config appsettings list --name enose-dashboard-app --resource-group enose-dashboard-rg

# Set new variable
az webapp config appsettings set --name enose-dashboard-app --resource-group enose-dashboard-rg --settings KEY=VALUE
```

### Monitoring

**Health Check:**
```bash
curl https://enose-dashboard-app.azurewebsites.net/health
```

**Expected Response:**
```json
{
  "status": "ok",
  "service": "enose-cloud"
}
```

**Azure Logs:**
```bash
az webapp log tail --name enose-dashboard-app --resource-group enose-dashboard-rg
```

### Database Maintenance

**⚠️ SQLite Ephemeral Storage:**
- Database resets on Azure restart
- Data not persistent across deployments
- For production, migrate to Azure Database for PostgreSQL

**Backup Strategy (if using PostgreSQL):**
```bash
# Export data
pg_dump DATABASE_URL > backup.sql

# Import data
psql DATABASE_URL < backup.sql
```

### Cost Estimation

**Current Setup (Free Tier):**
- Azure App Service F1: $0/month (free tier)
- Azure Container Registry: ~$5/month (prorated)
- MQTT Broker: $0/month (public broker)
- **Total:** ~$5/month

**Azure Student Credit:**
- Initial: $100
- Used: ~$2-3 (prorated)
- Remaining: ~$97-98

### Known Limitations

1. **SQLite Ephemeral Storage**
   - Database resets on restart
   - No data persistence across deployments
   - Solution: Upgrade to PostgreSQL

2. **No Authentication**
   - MQTT broker is public (test.mosquitto.org)
   - API endpoints are open
   - Solution: Implement API key auth + private MQTT broker

3. **Single Instance**
   - No load balancing
   - No auto-scaling
   - Limited to F1 tier resources

4. **Arabica Gayo Ambiguity**
   - Appears in both high_grade and low_grade datasets
   - ESP32 must determine grade explicitly

### Troubleshooting

**Problem: Dashboard tidak menerima data**

**Solution:**
1. Check Azure logs: `az webapp log tail ...`
2. Verify MQTT broker: `test.mosquitto.org:1883`
3. Check topic format: `enose/{device_id}/measurement`
4. Verify JSON payload structure
5. Test with Python script: `send_dummy_mqtt.py`

**Problem: Azure App Service restart**

**Solution:**
1. Check deployment status in Azure Portal
2. Verify Docker image exists in ACR
3. Check environment variables are set
4. Review application logs

**Problem: OTA firmware not working**

**Solution:**
1. Verify MQTT_BROKER env variable
2. Check firmware file exists in `./firmware/` directory
3. Verify ESP32 subscribes to `enose/ota/update`
4. Check BASE_URL env variable

---

## VERSION HISTORY

### v1.0.0 (2026-09-30)
- ✅ Initial production deployment
- ✅ MQTT integration with test.mosquitto.org
- ✅ Dashboard with real-time charts
- ✅ OTA firmware update support
- ✅ PDF report generation
- ✅ Database schema with predicted_class, confidence, grade fields
- ✅ Fixed OTA MQTT broker configuration
- ✅ Updated PDF generation to use grade field

---

## CONTACT & SUPPORT

**Maintainer:** Tugas C (Dashboard/Backend)  
**Platform:** Azure App Service  
**Documentation:** This file (DOCUMENTATION.md)

**URLs:**
- Production: https://enose-dashboard-app.azurewebsites.net
- Dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard
- Health: https://enose-dashboard-app.azurewebsites.net/health

**For Updates:**
This file will be updated with any configuration changes, new features, or system modifications. Do not create separate documentation files.

---

**Last Updated:** 2026-09-30  
**Status:** ✅ Production Ready  
**Next Review:** When system changes occur
