# Backend System Explanation - Azure Production

**⚠️ IMPORTANT: Ini BUKAN guide untuk bikin ESP32 firmware. Ini penjelasan backend yang SUDAH JADI di Azure.**

**Purpose:** Dokumentasi untuk ESP32 developer agar tahu:
- Bagaimana backend bekerja (sudah jadi, sudah deploy)
- Format JSON apa yang harus dikirim ESP32
- Topic MQTT apa yang harus dipakai
- Endpoint API apa yang tersedia
- Credentials apa yang dibutuhkan

**Your Job:** Bikin ESP32 firmware yang **konek ke sistem ini**.

---

**Backend Location:** `C:\Users\User\dashboard_c\` (source code - jangan diubah)  
**Deployment:** Azure App Service (Cloud Production - sudah live)  
**Live Dashboard:** https://enose-dashboard-app.azurewebsites.net/dashboard  
**Type:** Rust Backend + Web Dashboard + MQTT Subscriber  
**Last Updated:** October 10, 2026  
**Status:** ✅ LIVE IN PRODUCTION (Azure)

---

## 📋 Table of Contents

**Ini dokumentasi BACKEND yang sudah jadi. Baca ini untuk tahu format data yang ESP32 harus kirim.**

1. [Backend Overview (What It Does)](#backend-overview)
2. [Architecture (How It Works)](#architecture)
3. [Credentials (What ESP32 Needs)](#production-credentials)
4. [API Endpoints (If ESP32 Want HTTP)](#api-endpoints)
5. [MQTT Integration (ESP32 Main Communication)](#mqtt-integration)
6. [Database Schema (What Gets Stored)](#database-schema)
7. [JSON Formats (CRITICAL: What ESP32 Must Send)](#json-formats)
8. [Troubleshooting (Common Issues)](#troubleshooting)

---

## 🎯 Backend Overview

**Backend `dashboard_c` adalah sistem yang SUDAH JADI dan SUDAH LIVE di Azure.**

### What Backend Does (SUDAH BERJALAN):
- ✅ **Listens** to MQTT broker (`test.mosquitto.org:1883`)
- ✅ **Subscribes** to topic `enose/+/measurement` (all devices)
- ✅ **Receives** JSON messages from ESP32
- ✅ **Validates** format (sample_id, score, accuracy)
- ✅ **Stores** data in Azure PostgreSQL database
- ✅ **Serves** web dashboard untuk visualisasi data
- ✅ **Provides** REST API untuk CRUD operations
- ✅ **Handles** OTA firmware uploads via web UI

### What ESP32 Must Do (YOUR JOB):
- 🎯 **Connect** to WiFi
- 🎯 **Connect** to MQTT broker `test.mosquitto.org:1883`
- 🎯 **Publish** JSON measurement ke topic `enose/ESP32-001/measurement`
- 🎯 **Subscribe** to OTA topics `enose/ESP32-001/ota/*`
- 🎯 **Receive** OTA chunks → write to flash → reboot

### What ESP32 Does NOT Need:
- ❌ Deploy backend (sudah di Azure)
- ❌ Setup database (sudah PostgreSQL)
- ❌ Create dashboard (sudah ada HTML)
- ❌ Manage MQTT broker (public broker)

### Current Backend Status:
- 🟢 **Backend:** LIVE at `enose-dashboard-app.azurewebsites.net`
- 🟢 **Dashboard:** Accessible, shows 0 data (waiting for ESP32)
- 🟢 **MQTT:** Listening and ready to receive
- 🟡 **Data:** 0 measurements (ESP32 belum kirim)

---

## 🏗️ Architecture (Azure Production)

```
┌─────────────┐        MQTT          ┌──────────────────────────┐
│   ESP32-S3  │  ───────────────────→ │  test.mosquitto.org      │
│   (Device)  │   Public MQTT Broker  │  Port: 1883 (no auth)    │
│   COM11     │                       └──────────────────────────┘
└─────────────┘                                    │
                                                   │ Subscribe
                                                   ↓
                                   ┌────────────────────────────────────┐
                                   │  Azure App Service                 │
                                   │  enose-dashboard-app               │
                                   │  (dashboard_c backend - Rust)      │
                                   │  - MQTT Client (subscribes)        │
                                   │  - HTTP Server (Axum port 8080)    │
                                   └────────────────────────────────────┘
                                                   │
                         ┌─────────────────────────┼──────────────────────┐
                         ↓                         ↓                      ↓
                ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
                │ Azure PostgreSQL│    │  Web Dashboard   │    │   REST API      │
                │  Database       │    │  /dashboard      │    │  /api/v1/*      │
                │  (Production)   │    │  (HTML/JS)       │    │  (JSON)         │
                └─────────────────┘    └──────────────────┘    └─────────────────┘
                         │                         │                      │
                         │                         ↓                      │
                         │              ┌──────────────────┐              │
                         │              │   Browser (You)  │              │
                         │              │   Dashboard UI   │              │
                         │              └──────────────────┘              │
                         │                                                │
                         └────────────────────────────────────────────────┘
                                  Stores & Retrieves Data
```

### Data Flow (ESP32 → Dashboard):
1. **ESP32** publish measurement → `enose/ESP32-001/measurement` topic (MQTT)
2. **MQTT Broker** (`test.mosquitto.org`) forward message → Azure backend
3. **Azure Backend** receive via MQTT subscriber → parse JSON → validate
4. **PostgreSQL** insert measurement dengan timestamp
5. **Dashboard** query database via API → display real-time data di browser

### URLs:
- **Dashboard:** https://enose-dashboard-app.azurewebsites.net/dashboard
- **API Base:** https://enose-dashboard-app.azurewebsites.net/api/v1
- **MQTT Broker:** test.mosquitto.org:1883 (public, no auth)

---

## 🔐 Production Credentials (Azure)

### Azure App Service Configuration

**App Service Name:** `enose-dashboard-app`  
**Resource Group:** (your resource group name)  
**Container Registry:** `enosedashboard2024.azurecr.io`  
**Docker Image:** `enosedashboard2024.azurecr.io/enose-dashboard:latest`

### Environment Variables (Azure App Settings)

```env
# Server Configuration
HOST=0.0.0.0
PORT=8080                            # ← Azure internal port

# Database (Azure PostgreSQL)
DATABASE_URL=postgres://USERNAME:PASSWORD@SERVER.postgres.database.azure.com:5432/enose?sslmode=require

# Logging
RUST_LOG=info

# MQTT Configuration (Production)
MQTT_ENABLED=true                    # ← MUST be true
MQTT_BROKER=test.mosquitto.org       # ← Public broker
MQTT_PORT=1883
MQTT_USERNAME=                       # ← Empty (no auth)
MQTT_PASSWORD=                       # ← Empty (no auth)
MQTT_CLIENT_ID=enose-cloud-azure    # ← Backend client ID
MQTT_TOPIC=enose/+/measurement      # ← Wildcard subscribes to all devices
MQTT_TLS=false                       # ← No TLS on port 1883

# OTA (Optional - not yet implemented)
OTA_BASE_URL=
OTA_API_TOKEN=
```

### ⚠️ CRITICAL Production Settings:

| Setting | Value | Why |
|---------|-------|-----|
| `MQTT_ENABLED` | `true` | Backend won't receive data if false |
| `MQTT_BROKER` | `test.mosquitto.org` | MUST match ESP32 broker |
| `MQTT_TOPIC` | `enose/+/measurement` | `+` wildcard = all device IDs |
| `PORT` | `8080` | Azure App Service internal port |
| `DATABASE_URL` | `postgres://...` | Azure PostgreSQL connection string |

### 🔒 Security Notes:
- ✅ PostgreSQL uses SSL (`sslmode=require`)
- ⚠️ MQTT broker is public (no authentication required)
- ✅ Dashboard accessible via HTTPS (Azure handles SSL termination)
- ⚠️ No API authentication yet (add if needed for production security)

---

## 🌐 API Endpoints (Azure Production)

### Base URL:
**Production:** `https://enose-dashboard-app.azurewebsites.net`

### Endpoints:

#### 1. **GET /** - Health Check
```
GET https://enose-dashboard-app.azurewebsites.net/
```
**Response:**
```
"E-Nose Cloud API v1.0 – Server is running!"
```

---

#### 2. **GET /dashboard** - Web Dashboard (HTML)
```
GET https://enose-dashboard-app.azurewebsites.net/dashboard
```
**Response:** HTML page dengan:
- Total samples / classes detected / confidence / accuracy
- Recent measurements table (last 20)
- Confidence trend chart
- Class distribution pie chart

**Current Status:** 0 measurements (ESP32 belum kirim data)

---

#### 3. **GET /api/v1/measurements** - List Measurements
```
GET https://enose-dashboard-app.azurewebsites.net/api/v1/measurements
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "sample_id": 5,
    "score": 92.0,
    "accuracy": 92.3,
    "predicted_class": "Arabica Gayo",
    "confidence": 0.95,
    "grade": "low_grade",
    "source": "mqtt",
    "device_id": "ESP32-001",
    "features_json": null,
    "captured_at": "2026-10-10T10:30:00Z",
    "created_at": "2026-10-10T10:30:05Z"
  }
]
```

**Limit:** Last 500 measurements  
**Order:** `captured_at DESC` (newest first)

---

#### 4. **POST /api/v1/measurements** - Create Measurement
```
POST https://enose-dashboard-app.azurewebsites.net/api/v1/measurements
Content-Type: application/json

{
  "sample_id": 5,
  "score": 92.0,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "low_grade",
  "device_id": "ESP32-001",
  "source": "api",
  "features": [1.2, 3.4, 5.6],
  "captured_at": "2026-10-10T10:30:00Z"
}
```

**Response:** `201 Created`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "sample_id": 5,
  "score": 92.0,
  "accuracy": 92.3,
  ...
}
```

**Validation Rules:**
- ✅ `sample_id` >= 1
- ✅ `score` between 0.0 and 100.0
- ✅ `accuracy` between 0.0 and 100.0
- ⚠️ `predicted_class`, `confidence`, `grade`, `device_id` = optional

---

#### 5. **DELETE /api/v1/measurements/{id}** - Delete Measurement
```
DELETE https://enose-dashboard-app.azurewebsites.net/api/v1/measurements/550e8400-e29b-41d4-a716-446655440000
```

**Response:** `204 No Content` (success) or `404 Not Found`

---

#### 6. **GET /api/v1/analytics/summary** - Analytics Summary
```
GET https://enose-dashboard-app.azurewebsites.net/api/v1/analytics/summary
```

**Response:**
```json
{
  "total_measurements": 0,
  "sample_count": 0,
  "average_score": 0.0,
  "average_accuracy": 0.0,
  "latest_captured_at": null,
  "by_sample": []
}
```

---

#### 7. **GET /api/v1/analytics/pdf** - Download PDF Report
```
GET https://enose-dashboard-app.azurewebsites.net/api/v1/analytics/pdf
```

**Response:** PDF file with:
- Overview cards (total samples, classes, confidence)
- Recent measurements table (last 12)
- Auto-generated from database

**Content-Type:** `application/pdf`  
**Filename:** `enose-analytics-YYYYMMDD-HHMMSS.pdf`

---

#### 8. **POST /api/ota/upload** - Upload Firmware (OTA)
```
POST https://enose-dashboard-app.azurewebsites.net/api/ota/upload
Content-Type: multipart/form-data

firmware: [binary file .bin]
version: "1.0.1"
```

**Response:** `200 OK`
```json
{
  "message": "Firmware uploaded successfully",
  "size": 526208,
  "checksum": "a1b2c3d4e5f6...",
  "version": "1.0.1"
}
```

**What Happens:**
1. Backend receives `.bin` file from web UI
2. Calculate MD5 checksum
3. Split into 1KB chunks (padded to 4-byte alignment)
4. Publish to MQTT:
   - `enose/ESP32-001/ota/start` → firmware metadata
   - `enose/ESP32-001/ota/chunk` → chunk 0, 1, 2, ... (1KB each)
   - `enose/ESP32-001/ota/complete` → checksum verification
5. ESP32 receives chunks → writes to flash → reboots

---

## 📡 MQTT Integration

### Broker Configuration

**Broker:** `test.mosquitto.org`  
**Port:** `1883` (no TLS)  
**Auth:** None (public broker)  
**QoS:** `1` (At Least Once)

### Topics

#### 1. **Measurement Topic** (ESP32 → Backend)
```
Topic: enose/{device_id}/measurement
Example: enose/ESP32-001/measurement
```

**Direction:** ESP32 publishes → Backend subscribes

**Payload (JSON):**
```json
{
  "device_id": "ESP32-001",
  "sample_id": 5,
  "score": 92.0,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "low_grade",
  "captured_at": "2026-10-10T10:30:00Z"
}
```

**Required Fields:**
- ✅ `sample_id` (integer >= 1)
- ✅ `score` (float 0-100)
- ✅ `accuracy` (float 0-100)

**Optional Fields:**
- `device_id` (string)
- `predicted_class` (string)
- `confidence` (float 0.0-1.0)
- `grade` ("high_grade" or "low_grade")
- `captured_at` (ISO 8601 timestamp)

---

#### 2. **OTA Start Topic** (Backend → ESP32)
```
Topic: enose/{device_id}/ota/start
Example: enose/ESP32-001/ota/start
```

**Direction:** Backend publishes → ESP32 subscribes

**Payload (JSON):**
```json
{
  "firmware_size": 526208,
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6789012345678901234567890",
  "version": "1.0.1"
}
```

---

#### 3. **OTA Chunk Topic** (Backend → ESP32)
```
Topic: enose/{device_id}/ota/chunk
Example: enose/ESP32-001/ota/chunk
```

**Direction:** Backend publishes → ESP32 subscribes

**Payload (Binary):**
```
[2 bytes: chunk_id (big-endian u16)][1024 bytes: firmware data (padded to 4-byte)]
```

**Example:**
- Chunk 0: `[0x00, 0x00][... 1024 bytes firmware ...]`
- Chunk 1: `[0x00, 0x01][... 1024 bytes firmware ...]`
- Chunk 515: `[0x02, 0x03][... last chunk padded with 0xFF ...]`

**Critical:**
- ✅ Chunk size: 1024 bytes (aligned)
- ✅ Padding: If last chunk < 1024, pad with `0xFF` to 4-byte boundary
- ✅ Delay: 150ms between chunks (avoid flooding)

---

#### 4. **OTA Complete Topic** (Backend → ESP32)
```
Topic: enose/{device_id}/ota/complete
Example: enose/ESP32-001/ota/complete
```

**Direction:** Backend publishes → ESP32 subscribes

**Payload (JSON):**
```json
{
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6789012345678901234567890"
}
```

**ESP32 Action:**
- Verify all chunks received
- Calculate MD5 checksum
- Compare with received checksum
- If match → set boot partition → reboot
- If mismatch → abort OTA

---

## 🗄️ Database Schema

### Table: `measurements`

```sql
CREATE TABLE measurements (
    id TEXT PRIMARY KEY,                      -- UUID (auto-generated)
    sample_id INTEGER NOT NULL,               -- Sample identifier (>= 1)
    score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
    accuracy REAL NOT NULL CHECK(accuracy BETWEEN 0 AND 100),
    predicted_class TEXT,                     -- ML prediction result
    confidence REAL,                          -- ML confidence (0.0-1.0)
    grade TEXT,                               -- "high_grade" or "low_grade"
    source TEXT NOT NULL DEFAULT 'api',       -- "api" or "mqtt"
    device_id TEXT,                           -- ESP32 device identifier
    features_json TEXT,                       -- JSON array of feature values
    captured_at TEXT NOT NULL,                -- ISO 8601 timestamp (measurement time)
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Field Details:

| Column | Type | Required | Example | Notes |
|--------|------|----------|---------|-------|
| `id` | TEXT | ✅ | `550e8400-...` | UUID v4 (auto) |
| `sample_id` | INTEGER | ✅ | `5` | Must be >= 1 |
| `score` | REAL | ✅ | `92.0` | 0.0 to 100.0 |
| `accuracy` | REAL | ✅ | `92.3` | 0.0 to 100.0 |
| `predicted_class` | TEXT | ❌ | `"Arabica Gayo"` | ML model output |
| `confidence` | REAL | ❌ | `0.95` | 0.0 to 1.0 |
| `grade` | TEXT | ❌ | `"low_grade"` | high_grade or low_grade |
| `source` | TEXT | ✅ | `"mqtt"` | "api" or "mqtt" |
| `device_id` | TEXT | ❌ | `"ESP32-001"` | Device identifier |
| `features_json` | TEXT | ❌ | `"[1.2,3.4,5.6]"` | JSON array |
| `captured_at` | TEXT | ✅ | `"2026-10-10T10:30:00Z"` | ISO 8601 |
| `created_at` | TEXT | ✅ | `"2026-10-10T10:30:05Z"` | Auto-generated |

### Database Migrations:

Backend auto-creates table on startup:
```rust
sqlx::query("CREATE TABLE IF NOT EXISTS measurements ...")
    .execute(&pool)
    .await?;
```

Existing columns auto-added if missing:
```rust
ALTER TABLE measurements ADD COLUMN source TEXT NOT NULL DEFAULT 'api';
ALTER TABLE measurements ADD COLUMN predicted_class TEXT;
ALTER TABLE measurements ADD COLUMN confidence REAL;
ALTER TABLE measurements ADD COLUMN grade TEXT;
```

---

## 📋 JSON Formats

### 1. Measurement Input (POST /api/v1/measurements OR MQTT)

```json
{
  "sample_id": 5,
  "score": 92.0,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "low_grade",
  "device_id": "ESP32-001",
  "source": "mqtt",
  "features": [1.2, 3.4, 5.6, 7.8, 9.0],
  "captured_at": "2026-10-10T10:30:00Z"
}
```

**Field Types:**
- `sample_id`: **integer** (NOT string!)
- `score`, `accuracy`: **float** (NOT string!)
- `confidence`: **float 0.0-1.0** (NOT 0-100!)
- `captured_at`: **ISO 8601** string with timezone

**Common Mistakes:**
```json
// ❌ WRONG
{
  "sample_id": "5",        // String (should be integer)
  "score": "92.0",         // String (should be float)
  "confidence": "95%",     // Percentage string (should be 0.95)
  "captured_at": "10/10/2026 10:30"  // Wrong format
}

// ✅ CORRECT
{
  "sample_id": 5,
  "score": 92.0,
  "confidence": 0.95,
  "captured_at": "2026-10-10T10:30:00Z"
}
```

---

### 2. Measurement Output (GET /api/v1/measurements)

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "sample_id": 5,
  "score": 92.0,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "low_grade",
  "source": "mqtt",
  "device_id": "ESP32-001",
  "features_json": "[1.2,3.4,5.6]",
  "captured_at": "2026-10-10T10:30:00Z",
  "created_at": "2026-10-10T10:30:05Z"
}
```

**Note:** `features_json` is a JSON **string** (not array).

---

### 3. OTA Start Message

```json
{
  "firmware_size": 526208,
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6789012345678901234567890",
  "version": "1.0.1"
}
```

**Calculation:**
- `total_chunks = (firmware_size + 1023) / 1024`
- `checksum = MD5(firmware_binary)`

---

### 4. OTA Complete Message

```json
{
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6789012345678901234567890"
}
```

---

## ☁️ Azure Deployment

### Current Production Setup:

**App Service:** `enose-dashboard-app`  
**Live URL:** https://enose-dashboard-app.azurewebsites.net  
**Dashboard:** https://enose-dashboard-app.azurewebsites.net/dashboard  
**Container Registry:** `enosedashboard2024.azurecr.io`  
**Image:** `enosedashboard2024.azurecr.io/enose-dashboard:latest`

### Update Deployment (When Code Changes):

**1. Build Docker Image:**
```powershell
cd C:\Users\User\dashboard_c
docker build -t enosedashboard2024.azurecr.io/enose-dashboard:latest .
```

**2. Login to Azure Container Registry:**
```powershell
az acr login --name enosedashboard2024
```

**3. Push Image:**
```powershell
docker push enosedashboard2024.azurecr.io/enose-dashboard:latest
```

**4. Restart Azure App Service:**
```powershell
az webapp restart --name enose-dashboard-app --resource-group <your-resource-group>
```

**5. View Live Logs:**
```powershell
az webapp log tail --name enose-dashboard-app --resource-group <your-resource-group>
```

### Check Deployment Status:

**View current image:**
```powershell
az acr repository show-tags --name enosedashboard2024 --repository enose-dashboard
```

**Test health endpoint:**
```powershell
curl https://enose-dashboard-app.azurewebsites.net/
# Should return: "E-Nose Cloud API v1.0 – Server is running!"
```

**Check MQTT connection:**
```
# Look for in logs:
[INFO] MQTT: Connected and subscribed to enose/+/measurement
```

---

## 🔧 Troubleshooting

### Problem 1: MQTT Not Receiving Data

**Symptoms:**
- Dashboard shows 0 measurements
- Logs: `MQTT: Connected` but no `Message received`

**Checklist:**
```powershell
# 1. Check MQTT_ENABLED
$env:MQTT_ENABLED
# Should be: true

# 2. Check broker reachable
Test-NetConnection test.mosquitto.org -Port 1883
# Should succeed

# 3. Check ESP32 publishing
# Use MQTT Explorer to see messages at enose/+/measurement

# 4. Check topic match
# Backend: enose/+/measurement
# ESP32:   enose/ESP32-001/measurement
# Must match pattern!
```

---

### Problem 2: Backend Not Starting

**Symptoms:**
- `cargo run` fails
- Error: `DATABASE_URL not found`

**Solution:**
```powershell
# Create .env file
Copy-Item .env.example .env

# Set DATABASE_URL
echo "DATABASE_URL=sqlite://./enose.db" >> .env

# Retry
cargo run --release --bin enose-cloud
```

---

### Problem 3: Port Already in Use

**Symptoms:**
- Error: `Address already in use (os error 10048)`

**Solution:**
```powershell
# Find process using port 3030
netstat -ano | findstr :3030

# Kill process (replace PID)
taskkill /F /PID 12345

# Or change port in .env
echo "PORT=8080" >> .env
```

---

### Problem 4: Database Locked

**Symptoms:**
- Error: `database is locked`

**Solution:**
```powershell
# Close any SQLite browser/editor
# Delete lock file
Remove-Item enose.db-wal -ErrorAction SilentlyContinue
Remove-Item enose.db-shm -ErrorAction SilentlyContinue

# Restart backend
cargo run --release --bin enose-cloud
```

---

### Problem 5: Invalid JSON from ESP32

**Symptoms:**
- Logs: `MQTT: Invalid JSON payload`
- Data not saved to database

**Debug:**
```powershell
# Check logs for actual payload
# Look for: payload = ?String::from_utf8_lossy(...)

# Common issues:
# - sample_id as string (should be integer)
# - score/accuracy as string (should be float)
# - confidence 0-100 (should be 0.0-1.0)
# - timestamp wrong format (should be ISO 8601)
```

**Fix ESP32 code to send correct format (see JSON Formats section above).**

---

## 📞 Support

**Project Owner:** Mahendra Dwi Fahreza  
**Location:** `C:\Users\User\dashboard_c\`  
**Documentation:** `PROJECT_OVERVIEW.md`, `DOCUMENTATION.md`, `MQTT_GUIDE_TIM_ESP32.md`  
**Dashboard URL (Local):** http://localhost:3030/dashboard  
**Dashboard URL (Azure):** https://enose-dashboard-app.azurewebsites.net/dashboard

---

## ✅ Quick Reference

### Production URLs:
```
Dashboard:  https://enose-dashboard-app.azurewebsites.net/dashboard
API Base:   https://enose-dashboard-app.azurewebsites.net/api/v1
Health:     https://enose-dashboard-app.azurewebsites.net/
```

### MQTT Configuration:
```
Broker:     test.mosquitto.org
Port:       1883
Topic:      enose/+/measurement (backend subscribes)
            enose/ESP32-001/measurement (ESP32 publishes)
QoS:        1 (At Least Once)
Auth:       None (public broker)
```

### Test ESP32 Connection:
```powershell
# Use MQTT Explorer to monitor
# Connect to: test.mosquitto.org:1883
# Subscribe to: enose/#
# Check if ESP32 messages appear
```

### Deploy Code Changes:
```powershell
cd C:\Users\User\dashboard_c
docker build -t enosedashboard2024.azurecr.io/enose-dashboard:latest .
az acr login --name enosedashboard2024
docker push enosedashboard2024.azurecr.io/enose-dashboard:latest
az webapp restart --name enose-dashboard-app --resource-group <your-rg>
```

### View Azure Logs:
```powershell
az webapp log tail --name enose-dashboard-app --resource-group <your-rg>
```

### Expected Logs (Production):
```
[INFO] Server listening on 0.0.0.0:8080
[INFO] MQTT: Connected and subscribed to enose/+/measurement
[INFO] MQTT: Message received from topic: enose/ESP32-001/measurement
[INFO] MQTT: Measurement saved (201)
```

---

**Document Version:** 1.0  
**Last Updated:** October 10, 2026  
**Status:** Production Ready ✅


## ✅ Quick Reference

### Production URLs:
```
Dashboard:  https://enose-dashboard-app.azurewebsites.net/dashboard
API Base:   https://enose-dashboard-app.azurewebsites.net/api/v1
Health:     https://enose-dashboard-app.azurewebsites.net/
```

### MQTT Configuration:
```
Broker:     test.mosquitto.org
Port:       1883
Topic:      enose/+/measurement (backend subscribes)
            enose/ESP32-001/measurement (ESP32 publishes)
QoS:        1 (At Least Once)
Auth:       None (public broker)
```

### Test ESP32 Connection:
```powershell
# Use MQTT Explorer to monitor
# Connect to: test.mosquitto.org:1883
# Subscribe to: enose/#
# Check if ESP32 messages appear
```

### Deploy Code Changes:
```powershell
cd C:\Users\User\dashboard_c
docker build -t enosedashboard2024.azurecr.io/enose-dashboard:latest .
az acr login --name enosedashboard2024
docker push enosedashboard2024.azurecr.io/enose-dashboard:latest
az webapp restart --name enose-dashboard-app --resource-group <your-rg>
```

### View Azure Logs:
```powershell
az webapp log tail --name enose-dashboard-app --resource-group <your-rg>
```

### Expected Logs (Production):
```
[INFO] Server listening on 0.0.0.0:8080
[INFO] MQTT: Connected and subscribed to enose/+/measurement
[INFO] MQTT: Message received from topic: enose/ESP32-001/measurement
[INFO] MQTT: Measurement saved (201)
```

---

**Document Version:** 1.1 (Azure Production)  
**Last Updated:** October 10, 2026  
**Status:** ✅ LIVE IN PRODUCTION (Azure)
