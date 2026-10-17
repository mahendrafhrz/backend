# E-Nose Dashboard Backend

> Real-time coffee quality monitoring system with MQTT data ingestion, OTA firmware updates, and analytics dashboard.

![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Status](https://img.shields.io/badge/status-production-green.svg)

---

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Endpoints](#api-endpoints)
- [OTA Updates](#ota-updates)
- [Deployment](#deployment)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

---

## 🎯 Overview

The E-Nose Dashboard Backend is a Rust-based web service that:

- **Receives** real-time sensor measurements from ESP32 devices via MQTT
- **Stores** data in PostgreSQL database with full CRUD operations
- **Provides** RESTful API for dashboard and analytics
- **Manages** Over-The-Air (OTA) firmware updates for ESP32 devices
- **Generates** PDF reports with analytics and visualizations

**Tech Stack:**
- **Backend Framework:** Axum (async Rust web framework)
- **Database:** PostgreSQL (Railway hosted) with SQLx
- **MQTT:** Rumqttc (async MQTT client)
- **Deployment:** Docker + Azure Web Apps
- **Monitoring:** Structured logging with tracing

---

## ✨ Features

### 📊 Data Management
- Real-time measurement ingestion via MQTT (`enose/+/measurement` topic)
- RESTful API for querying measurements
- Support for multiple coffee types (Arabica, Robusta, Liberica)
- Grade classification (high/low grade)
- Device tracking (by device_id)

### 📈 Analytics & Reporting
- Summary statistics (total samples, average accuracy, confidence trends)
- Class distribution visualization
- PDF report generation with charts
- Time-series data export

### 🔄 OTA Firmware Updates
- Upload firmware binary via web interface
- Automatic chunking (1KB chunks) with ACK mechanism
- MD5 checksum verification
- Real-time progress monitoring
- Supports ESP32-S3 devices

### 🖥️ Dashboard
- Modern web UI with real-time updates
- Confidence trend charts
- Class distribution pie charts
- Recent measurements table
- One-click report download

---

## 🏗️ Architecture

```
┌─────────────────┐
│   ESP32 Devices │ ──MQTT (pub)──> enose/+/measurement
│   (Sensors)     │ <──MQTT (sub)── enose/{id}/ota/*
└─────────────────┘
         │
         │ MQTT Broker
         │ (HiveMQ)
         ▼
┌─────────────────────────────────────────┐
│     Backend (Rust + Axum)               │
│  ┌───────────────┐  ┌──────────────┐   │
│  │ MQTT Listener │  │  HTTP Server │   │
│  └───────┬───────┘  └──────┬───────┘   │
│          │                  │           │
│          ▼                  ▼           │
│    ┌─────────────────────────────┐     │
│    │   PostgreSQL Database       │     │
│    │   (Railway)                 │     │
│    └─────────────────────────────┘     │
└─────────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│  Dashboard UI   │
│  (HTML/JS)      │
└─────────────────┘
```

**Data Flow:**
1. ESP32 publishes sensor data → MQTT broker
2. Backend subscribes to MQTT → receives measurements
3. Backend stores data → PostgreSQL database
4. Dashboard fetches data → REST API
5. User uploads firmware → Backend broadcasts OTA via MQTT
6. ESP32 receives OTA → Downloads and flashes firmware

---

## 📦 Prerequisites

- **Rust:** 1.83 or later
- **Docker:** (optional, for containerized deployment)
- **PostgreSQL:** 14+ (Railway or local)
- **MQTT Broker:** (e.g., HiveMQ, Mosquitto)

---

## 🚀 Installation

### Local Development

1. **Clone the repository:**
```bash
git clone https://github.com/mahendrafhrz/backend.git
cd backend
```

2. **Set up environment variables:**
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Run database migrations:**
```bash
cargo install sqlx-cli
sqlx migrate run
```

4. **Build and run:**
```bash
cargo build --release
cargo run --release
```

Server will start on `http://localhost:8080`

### Docker Deployment

```bash
docker build -t enose-dashboard .
docker run -p 8080:8080 \
  -e DATABASE_URL="postgresql://user:pass@host/db" \
  -e MQTT_BROKER="broker.hivemq.com" \
  enose-dashboard
```

---

## ⚙️ Configuration

### Environment Variables

Create a `.env` file or set environment variables:

```bash
# Database
DATABASE_URL=postgresql://postgres:password@host:5432/enose?sslmode=require

# MQTT Configuration
MQTT_BROKER=broker.hivemq.com
MQTT_PORT=1883
MQTT_TOPIC=enose/+/measurement
MQTT_ENABLED=true

# OTA Configuration
DEVICE_ID=ESP32-001
BASE_URL=https://your-domain.com
OTA_CHUNK_DELAY_MS=500
OTA_ACK_TIMEOUT_SECS=10
OTA_MAX_RETRIES=3

# Server
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
```

### Configuration Details

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | ✅ |
| `MQTT_BROKER` | MQTT broker hostname | `broker.hivemq.com` | ✅ |
| `MQTT_PORT` | MQTT broker port | `1883` | ❌ |
| `MQTT_TOPIC` | Topic pattern for measurements | `enose/+/measurement` | ❌ |
| `MQTT_ENABLED` | Enable MQTT listener | `true` | ❌ |
| `DEVICE_ID` | Target device for OTA | `ESP32-001` | ✅ |
| `BASE_URL` | Public URL for firmware downloads | - | ✅ |
| `PORT` | HTTP server port | `8080` | ❌ |
| `RUST_LOG` | Logging level | `info` | ❌ |

---

## 📖 Usage

### Starting the Server

```bash
# Development mode
cargo run

# Production mode
cargo run --release

# With custom port
PORT=3000 cargo run --release
```

### Accessing the Dashboard

Open browser: `http://localhost:8080/dashboard`

### API Testing

```bash
# Get all measurements
curl http://localhost:8080/api/v1/measurements

# Get summary statistics
curl http://localhost:8080/api/v1/analytics/summary

# Download PDF report
curl -O http://localhost:8080/api/v1/reports/analytics.pdf
```

---

## 🔌 API Endpoints

### Measurements

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/measurements` | List all measurements |
| `POST` | `/api/v1/measurements` | Create measurement (manual) |
| `DELETE` | `/api/v1/measurements/{id}` | Delete measurement |

### Analytics

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/analytics/summary` | Get summary statistics |
| `GET` | `/api/v1/reports/analytics.pdf` | Download PDF report |

### OTA Updates

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/firmware/upload` | Upload firmware for OTA |

### Dashboard

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Dashboard homepage |
| `GET` | `/dashboard` | Dashboard UI |

---

## 📡 OTA Updates

### Upload Firmware

**Via Dashboard:**
1. Navigate to `/dashboard`
2. Click "OTA Update" in sidebar
3. Select `.bin` file
4. Enter version (e.g., `0.1.2`)
5. Click "Upload Firmware"

**Via API:**
```bash
curl -X POST http://localhost:8080/api/v1/firmware/upload \
  -F "firmware=@firmware_v0.1.2.bin" \
  -F "version=0.1.2"
```

### OTA Process

1. Backend receives firmware file
2. Calculates MD5 checksum
3. Splits into 1KB chunks
4. Publishes to MQTT:
   - `enose/{device_id}/ota/start` - Start message with metadata
   - `enose/{device_id}/ota/chunk` - Firmware chunks (1-620)
   - `enose/{device_id}/ota/complete` - Complete message
5. ESP32 receives, verifies, flashes, and reboots

### OTA Topics

| Topic | Direction | Purpose |
|-------|-----------|---------|
| `enose/{id}/ota/start` | Backend → ESP32 | Initiate OTA with metadata |
| `enose/{id}/ota/chunk` | Backend → ESP32 | Firmware data chunks |
| `enose/{id}/ota/complete` | Backend → ESP32 | Finalize and verify |
| `enose/{id}/ota/ack` | ESP32 → Backend | Acknowledge chunk received |

---

## 🐳 Deployment

### Azure Web Apps

1. **Build Docker image:**
```bash
docker build -t enosedashboard.azurecr.io/enose-dashboard:latest .
```

2. **Push to Azure Container Registry:**
```bash
az acr login --name enosedashboard
docker push enosedashboard.azurecr.io/enose-dashboard:latest
```

3. **Deploy to Web App:**
```bash
az webapp config container set \
  --name enose-dashboard-app \
  --resource-group enose-rg \
  --container-image-name enosedashboard.azurecr.io/enose-dashboard:latest

az webapp restart \
  --name enose-dashboard-app \
  --resource-group enose-rg
```

4. **Set environment variables:**
```bash
az webapp config appsettings set \
  --name enose-dashboard-app \
  --resource-group enose-rg \
  --settings \
    DATABASE_URL="postgresql://..." \
    MQTT_BROKER="broker.hivemq.com" \
    DEVICE_ID="ESP32-001"
```

### Docker Compose

```yaml
version: '3.8'
services:
  backend:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgresql://postgres:password@db:5432/enose
      MQTT_BROKER: broker.hivemq.com
      MQTT_ENABLED: true
    depends_on:
      - db
  
  db:
    image: postgres:14
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: enose
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

---

## 📊 Monitoring

### Logging

Structured logging with `tracing`:

```bash
# Set log level
RUST_LOG=debug cargo run

# Log levels: trace, debug, info, warn, error
```

### Health Check

```bash
curl http://localhost:8080/api/v1/measurements
# Should return 200 OK with measurements array
```

### Database Connection

```bash
# Check PostgreSQL connection
psql $DATABASE_URL -c "SELECT COUNT(*) FROM measurements;"
```

### MQTT Status

Check backend logs for:
```
INFO - MQTT: TCP connected to {broker}
INFO - MQTT: connected as {client_id}
INFO - MQTT: subscribed to topic: enose/+/measurement
INFO - MQTT: received measurement from device: ESP32-001
```

---

## 🐛 Troubleshooting

### Common Issues

**1. MQTT Connection Failed**
```
ERROR - MQTT connection error: Connection refused
```
**Solution:** Check `MQTT_BROKER` is reachable and port 1883 is open.

**2. Database Connection Error**
```
ERROR - Failed to connect to database
```
**Solution:** Verify `DATABASE_URL` is correct and PostgreSQL is running.

**3. OTA Upload Success but ESP32 Not Receiving**
```
INFO - Firmware uploaded successfully
# But ESP32 serial shows no OTA activity
```
**Solution:** Check `DEVICE_ID` environment variable matches ESP32 device ID.

**4. PDF Download Returns Error**
```
{"error":"internal server error"}
```
**Solution:** Railway database connection may be slow. Check backend logs for timeout errors.

**5. Dashboard Shows 0 Measurements**
```
Total Samples: 0
```
**Solution:** 
- Verify MQTT_ENABLED=true
- Check ESP32 is publishing to correct topic
- Verify MQTT_TOPIC wildcard matches ESP32 topic pattern

---

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy before committing (`cargo clippy`)
- Add tests for new features

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 👥 Authors

- **Mahendra Dwi Fahreza** - [GitHub](https://github.com/mahendrafhrz)

---

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum)
- MQTT client: [rumqttc](https://github.com/bytebeamio/rumqtt)
- Database: [SQLx](https://github.com/launchbadge/sqlx)
- PDF generation: [printpdf](https://github.com/fschutt/printpdf)

---

## 📞 Support

For issues and questions:
- **GitHub Issues:** [Report a bug](https://github.com/mahendrafhrz/backend/issues)
- **Email:** mahendrafahreza@example.com

---

**Last Updated:** December 2026  
**Version:** 1.0.0  
**Status:** ✅ Production Ready
