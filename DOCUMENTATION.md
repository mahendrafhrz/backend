# E-Nose Dashboard - Full Documentation

**Last Updated:** 2026-10-01  
**Status:** ✅ FULLY OPERATIONAL

---

## 🎯 System Overview

Complete IoT E-Nose (Electronic Nose) dashboard system for coffee quality classification deployed on Azure with persistent PostgreSQL database.

**Live Dashboard:** https://enose-dashboard-app.azurewebsites.net/dashboard

---

## 🏗️ Architecture

```
┌─────────────────┐
│   ESP32 Device  │ (Coffee sensor + ML model)
│   (Tugas A)     │
└────────┬────────┘
         │ MQTT Publish
         │ (test.mosquitto.org:1883)
         │ Topic: enose/ESP32-XXX/measurement
         ↓
┌─────────────────────────────────┐
│   Azure App Service             │
│   (Indonesia Central)           │
│   - Rust Backend (Actix-Web)    │
│   - MQTT Subscriber             │
│   - REST API                    │
│   - PDF Report Generator        │
│   - OTA Firmware Manager        │
└────────┬────────────────────────┘
         │ SQL Connection
         │ (DATABASE_URL)
         ↓
┌─────────────────────────────────┐
│   Railway PostgreSQL            │
│   (External, Free Tier)         │
│   - Persistent Storage          │
│   - 500 hours/month runtime     │
│   - 100GB egress/month          │
└─────────────────────────────────┘
         ↑
         │ HTTP GET
┌────────┴────────┐
│  Web Dashboard  │ (Served by Azure)
│  (dashboard.html)│
└─────────────────┘
```

---

## ✅ Test Results (2026-10-01)

### **Persistence Test - PASSED ✅**

**Test Scenario:** Verify data persists after Azure restart

1. ✅ **Sent 10 test measurements** via MQTT
   - 6 high_grade samples
   - 4 low_grade samples
   
2. ✅ **Azure App Service restarted** (simulate crash/redeploy)

3. ✅ **Data verification:**
   - All 10 measurements still exist
   - No data loss
   - Database connection restored automatically

**Conclusion:** Railway PostgreSQL successfully provides persistent storage for Azure backend.

---

## 🔧 Azure Configuration

### Resource Details
- **Resource Group:** `enose-dashboard-rg`
- **App Service:** `enose-dashboard-app`
- **Container Registry:** `enosedashboard2024.azurecr.io`
- **Region:** Indonesia Central
- **Runtime:** Docker Container (Rust + PostgreSQL client)

### Environment Variables

```bash
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
MQTT_ENABLED=true
MQTT_BROKER=test.mosquitto.org
MQTT_PORT=1883
MQTT_TOPIC=enose/+/measurement
BASE_URL=https://enose-dashboard-app.azurewebsites.net
DATABASE_URL=postgresql://postgres:ZbpsgrHnPNBLlRkxnjVhhWTFSibGrgWM@thomas.proxy.rlwy.net:38687/railway?sslmode=require
```

**Important Notes:**
- `DATABASE_URL` points to Railway PostgreSQL (external)
- `MQTT_BROKER` uses public test.mosquitto.org (stable, no 6s disconnect issue)
- `BASE_URL` used for OTA firmware download links

---

## 🗄️ Database Schema

### Railway PostgreSQL Connection
```
Host: thomas.proxy.rlwy.net
Port: 38687
Database: railway
User: postgres
Password: ZbpsgrHnPNBLlRkxnjVhhWTFSibGrgWM
SSL Mode: require
```

### Table: `measurements`

```sql
CREATE TABLE measurements (
    id TEXT PRIMARY KEY,              -- UUID from ESP32
    sample_id INTEGER NOT NULL,       -- Sample identifier
    score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
    accuracy REAL NOT NULL CHECK(accuracy BETWEEN 0 AND 100),
    predicted_class TEXT,             -- e.g., "Arabica Gayo"
    confidence REAL,                  -- ML model confidence (0-1)
    grade TEXT,                       -- "high_grade" or "low_grade"
    source TEXT NOT NULL DEFAULT 'api',  -- "api" or "mqtt"
    device_id TEXT,                   -- e.g., "ESP32-001"
    features_json TEXT,               -- Sensor readings (JSON string)
    captured_at TEXT NOT NULL,        -- ISO8601 timestamp from device
    created_at TEXT NOT NULL          -- Server timestamp
);
```

**Key Fields:**
- `grade`: Determined by ESP32 during inference (not backend)
- `predicted_class`: Coffee type classification
- `confidence`: ML model confidence score
- `source`: Tracks whether data came from API or MQTT

---

## 📡 MQTT Integration

### Configuration
- **Broker:** test.mosquitto.org (public, no auth)
- **Port:** 1883 (TCP)
- **Topic Pattern:** `enose/+/measurement` (wildcard for device_id)
- **QoS:** 0 (Fire and forget)

### ESP32 Payload Format

ESP32 should publish to: `enose/{device_id}/measurement`

```json
{
  "device_id": "ESP32-001",
  "sample_id": 1,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade",
  "timestamp": "2026-10-01T10:30:00Z"
}
```

**Required Fields:**
- `device_id`: ESP32 identifier
- `sample_id`: Sequential sample number
- `predicted_class`: Coffee classification result
- `confidence`: Model confidence (0.0 - 1.0)
- `grade`: **"high_grade"** or **"low_grade"** (determined by ESP32)
- `timestamp`: ISO8601 format

**Important:** The `grade` field is sent by ESP32, not calculated by backend. This ensures grade determination logic stays with ML inference team.

---

## 🔄 OTA Firmware System

### Upload Firmware (Web Dashboard)
1. Click "Upload Firmware" button
2. Select `.bin` file
3. System generates unique download URL
4. ESP32 can download via HTTP GET

### ESP32 Download Process
**Endpoint:** `GET /api/firmware/download/{filename}`

```cpp
// Example ESP32 code
String firmwareUrl = "https://enose-dashboard-app.azurewebsites.net/api/firmware/download/firmware_v1.0.0.bin";
http.begin(firmwareUrl);
int httpCode = http.GET();
if (httpCode == 200) {
    Update.begin(UPDATE_SIZE_UNKNOWN);
    // ... write firmware
}
```

**Firmware Storage:**
- Location: Azure App Service `/tmp/firmware/` (ephemeral)
- Auto-cleanup: Old files cleared on restart
- Max size: 5MB per file

---

## 📊 REST API Endpoints

### 1. Get All Measurements
```http
GET /api/measurements
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "sample_id": 1,
    "score": 85.5,
    "accuracy": 92.3,
    "predicted_class": "Arabica Gayo",
    "confidence": 0.95,
    "grade": "high_grade",
    "source": "mqtt",
    "device_id": "ESP32-001",
    "captured_at": "2026-10-01T10:30:00Z",
    "created_at": "2026-10-01T10:30:05Z"
  }
]
```

### 2. Create Measurement (Manual)
```http
POST /api/measurements
Content-Type: application/json

{
  "sample_id": 1,
  "score": 85.5,
  "accuracy": 92.3,
  "predicted_class": "Arabica Gayo",
  "confidence": 0.95,
  "grade": "high_grade"
}
```

### 3. Delete Measurement
```http
DELETE /api/measurements/{id}
```

### 4. Generate PDF Report
```http
GET /api/report/pdf
```
- Returns PDF file with all measurements
- Includes grade classification (high/low)
- Auto-download in browser

### 5. Upload Firmware
```http
POST /api/firmware/upload
Content-Type: multipart/form-data

firmware: <binary_file>
```

### 6. Download Firmware
```http
GET /api/firmware/download/{filename}
```

### 7. List Firmware Files
```http
GET /api/firmware/list
```

---

## 🎨 Web Dashboard Features

**URL:** https://enose-dashboard-app.azurewebsites.net/dashboard

### Features:
1. ✅ **Real-time data table** (auto-refresh every 5s)
2. ✅ **Grade badges** (Green: high_grade, Red: low_grade)
3. ✅ **Confidence display** (percentage with color coding)
4. ✅ **Delete measurements** (trash icon per row)
5. ✅ **PDF report generation** (download button)
6. ✅ **OTA firmware upload** (drag & drop or click)
7. ✅ **Responsive design** (works on mobile)

### UI Components:
- Bootstrap 5 styling
- Font Awesome icons
- Auto-refresh with visual indicator
- Toast notifications for actions

---

## 🧪 Testing

### Test MQTT Publishing

**File:** `test_mqtt_quick.py` (in workspace root)

```bash
# Install requirements
pip install paho-mqtt

# Run test (sends 10 samples)
python test_mqtt_quick.py
```

**What it does:**
- Publishes 10 measurements (6 high, 4 low)
- Alternates between different coffee types
- Uses realistic confidence scores
- Publishes to test.mosquitto.org

### Manual MQTT Test

```bash
# Install mosquitto_pub
mosquitto_pub -h test.mosquitto.org -p 1883 \
  -t "enose/ESP32-TEST/measurement" \
  -m '{"device_id":"ESP32-TEST","sample_id":1,"predicted_class":"Arabica Gayo","confidence":0.95,"grade":"high_grade","timestamp":"2026-10-01T10:00:00Z"}'
```

---

## 📁 Project Structure

```
lancar/
├── src/
│   └── main.rs                    # Rust backend (Actix-Web)
├── dashboard.html                 # Web dashboard (single file)
├── Dockerfile                     # Azure container config
├── Cargo.toml                     # Rust dependencies
├── .dockerignore                  # Docker build exclusions
├── .gitignore                     # Git exclusions
├── test_mqtt_quick.py             # MQTT test script
└── DOCUMENTATION.md               # This file
```

**Deleted Files (cleanup):**
- ❌ `enose.db` (SQLite, not used anymore)
- ❌ `send_dummy_mqtt.py` (replaced by test_mqtt_quick.py)
- ❌ `test_5_data.py`, `test_15_data.py`, `test_mqtt.py` (old tests)
- ❌ `CREDENTIALS.txt` (moved to Azure env vars)
- ❌ `README.md`, `ESP32_INTEGRATION.md`, `AZURE_DEPLOYMENT.md` (consolidated here)
- ❌ `LAPORAN_PROGRESS_BACKEND.tex` (outdated)

---

## 🚀 Deployment Process

### Initial Setup (Done)
1. ✅ Created Azure Resource Group
2. ✅ Created Azure Container Registry
3. ✅ Created Azure App Service (Docker)
4. ✅ Set up Railway PostgreSQL
5. ✅ Configured environment variables
6. ✅ Deployed application

### Update Deployment

```powershell
# Build and push new version
docker build -t enosedashboard2024.azurecr.io/enose-dashboard:latest .
docker push enosedashboard2024.azurecr.io/enose-dashboard:latest

# Azure auto-deploys on push (webhook configured)
# Or restart manually:
az webapp restart --name enose-dashboard-app --resource-group enose-dashboard-rg
```

---

## 🔐 Security Notes

1. **Database Credentials:** Stored in Azure environment variables (not in code)
2. **MQTT:** Public broker (test.mosquitto.org) - no authentication required
3. **Firmware Upload:** No auth currently - consider adding for production
4. **HTTPS:** Azure provides SSL certificate automatically
5. **CORS:** Enabled for all origins (restrict for production)

---

## 💰 Cost Analysis

### Current Setup (FREE)

| Service | Tier | Cost | Notes |
|---------|------|------|-------|
| Azure App Service | Free F1 | $0 | 60 min/day limit |
| Azure Container Registry | Basic | ~$5/mo | First month free |
| Railway PostgreSQL | Free | $0 | 500h runtime, 100GB egress |
| MQTT Broker | Public | $0 | test.mosquitto.org |
| **Total** | | **~$5/mo** | ACR only paid service |

### Production Recommendations:
- Upgrade Azure App Service to Basic B1 (~$13/mo) for 24/7 uptime
- Keep Railway PostgreSQL free tier (sufficient for IoT use case)
- Consider private MQTT broker if needed (~$5-10/mo)

---

## 📈 Usage Limits (Free Tier)

### Azure App Service (F1)
- ✅ 1 GB RAM
- ✅ 1 GB storage
- ⚠️ 60 minutes CPU time per day
- ⚠️ App sleeps after 20 min idle

### Railway PostgreSQL (Free)
- ✅ 100 GB outbound bandwidth/month
- ✅ 500 hours runtime/month (~16h/day)
- ✅ 512 MB RAM
- ✅ 1 GB storage

**Estimated Usage (IoT):**
- 1 measurement = ~500 bytes
- 1000 measurements/month = 0.5 MB
- Well within 100GB limit

---

## 🐛 Troubleshooting

### Dashboard shows no data
1. Check Azure App Service is running: https://enose-dashboard-app.azurewebsites.net/api/measurements
2. Verify Railway PostgreSQL connection in Azure logs
3. Test MQTT: `python test_mqtt_quick.py`

### MQTT not receiving
1. Check broker: `test.mosquitto.org` (should be stable)
2. Verify topic format: `enose/{device_id}/measurement`
3. Check Azure environment variable: `MQTT_ENABLED=true`

### Database connection failed
1. Verify DATABASE_URL in Azure environment variables
2. Check Railway PostgreSQL is running (railway.app dashboard)
3. Ensure SSL mode: `?sslmode=require` in connection string

### OTA firmware upload fails
1. Check file size < 5MB
2. Verify `/tmp/firmware/` directory writable
3. Check Azure App Service has enough storage

---

## 🎯 Design Decisions

### 1. Why Railway PostgreSQL instead of Azure Database?
**Decision:** Use external Railway PostgreSQL  
**Reason:** Azure Database for PostgreSQL costs $15-20/month, Railway offers free tier sufficient for IoT use case  
**Trade-off:** Extra network latency (~50-100ms), but acceptable for dashboard

### 2. Why ESP32 sends `grade` field?
**Decision:** ESP32 determines grade during inference  
**Reason:** Grade classification logic stays with ML team, backend just stores/displays  
**Alternative rejected:** Backend mapping based on confidence thresholds (not reliable)

### 3. Why test.mosquitto.org broker?
**Decision:** Use public test.mosquitto.org  
**Reason:** Stable, no authentication needed, free  
**Alternative rejected:** broker.emqx.io (disconnects every 6s on free tier)

### 4. Why single DOCUMENTATION.md?
**Decision:** One file for all updates  
**Reason:** User requested centralized documentation that gets updated, not new files  
**Alternative rejected:** Multiple README/guide files (scattered information)

---

## 🔄 Next Steps / Future Improvements

### Immediate (ESP32 Team - Tugas A)
- [ ] Implement proper MQTT payload with `grade` field
- [ ] Test OTA firmware download from ESP32
- [ ] Verify `predicted_class` naming conventions

### Short-term
- [ ] Add authentication for firmware upload
- [ ] Implement data pagination (if >1000 measurements)
- [ ] Add measurement charts (line/bar graphs)
- [ ] Email notifications for low-grade coffee

### Long-term (Production)
- [ ] Upgrade to Azure Basic tier for 24/7 uptime
- [ ] Set up private MQTT broker with auth
- [ ] Add user login system
- [ ] Implement data export (CSV/Excel)
- [ ] Add device management dashboard

---

## 📞 Support & Contacts

**System Owner:** C12 IoT Group  
**Backend Developer:** [Your name]  
**ESP32 Team (Tugas A):** Coffee sensor + ML inference  

**Resources:**
- Azure Portal: https://portal.azure.com
- Railway Dashboard: https://railway.app
- MQTT Broker: test.mosquitto.org (public)
- Live Dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard

---

## 📝 Changelog

### 2026-10-01 - Persistence Test Success ✅
- ✅ Verified Railway PostgreSQL persistence after Azure restart
- ✅ Sent 10 test measurements (6 high, 4 low grade)
- ✅ Confirmed data survives app service restart
- ✅ System fully operational
- 📄 Updated documentation with test results

### 2026-09-30 - Production Deployment
- ✅ Deployed to Azure Indonesia Central
- ✅ Connected Railway PostgreSQL
- ✅ MQTT integration working
- ✅ OTA firmware endpoints functional
- ✅ PDF generation using `grade` field

### 2026-09-29 - Initial Development
- ✅ Rust backend (Actix-Web)
- ✅ PostgreSQL schema design
- ✅ Dashboard UI (Bootstrap 5)
- ✅ MQTT subscriber implementation

---

**Document Version:** 1.2  
**Last Updated:** 2026-10-01 15:45 UTC+7  
**Status:** ✅ All Systems Operational
