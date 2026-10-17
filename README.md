# E-Nose IoT System

<div align="center">

## Electronic Nose System for Coffee Quality Classification

![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)
![PostgreSQL](https://img.shields.io/badge/postgresql-14+-blue.svg)
![MQTT](https://img.shields.io/badge/mqtt-HiveMQ-green.svg)
![Docker](https://img.shields.io/badge/docker-ready-2496ED.svg)
![Azure](https://img.shields.io/badge/azure-deployed-0078D4.svg)
![ESP32](https://img.shields.io/badge/esp32--s3-supported-red.svg)
![License](https://img.shields.io/badge/license-MIT-yellow.svg)
![Status](https://img.shields.io/badge/status-production-brightgreen.svg)

**Group 12 - C Class**

Evan Javier Firdausi Malik - 2042241010  
Mahendra Dwi Fahreza - 2042241023

</div>

---

## 📋 Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Key Features](#key-features)
- [Technology Stack](#technology-stack)
- [Screenshots](#screenshots)
- [Configuration](#configuration)
- [API Endpoints](#api-endpoints)
- [Deployment](#deployment)
- [Contributors](#contributors)

---

## 🎯 Overview

E-Nose IoT System is a real-time coffee quality monitoring solution that combines embedded systems (ESP32), cloud infrastructure (Azure), and machine learning analytics to classify coffee samples based on electronic nose sensor data.

**Project Goals:**
- Real-time sensor data collection from ESP32 devices
- Cloud-based data storage and processing
- Over-The-Air (OTA) firmware updates for remote ESP32 management
- Analytics dashboard with confidence trends and grade classification
- PDF report generation for quality assurance

**Use Cases:**
- Coffee quality control in production facilities
- Research and development for coffee grading systems
- Remote monitoring of multiple coffee processing stations
- Automated classification of coffee types (Arabica, Robusta, Liberica)

---

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         IoT Devices                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   ESP32-001  │  │   ESP32-002  │  │   ESP32-N    │         │
│  │  (E-Nose)    │  │  (E-Nose)    │  │  (E-Nose)    │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
└─────────┼──────────────────┼──────────────────┼────────────────┘
          │                  │                  │
          │  MQTT Publish    │                  │
          └──────────────────┴──────────────────┘
                             │
                             ▼
          ┌──────────────────────────────────┐
          │     MQTT Broker (HiveMQ)         │
          │  broker.hivemq.com:1883          │
          └──────────┬───────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────────────────────────┐
│                    Cloud Backend (Azure)                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Rust Backend (Axum Framework)                  │  │
│  │  ┌─────────────────┐         ┌──────────────────┐       │  │
│  │  │ MQTT Subscriber │◄────────┤  MQTT Publisher  │       │  │
│  │  │ (Measurements)  │         │  (OTA Updates)   │       │  │
│  │  └────────┬────────┘         └──────────────────┘       │  │
│  │           │                                              │  │
│  │           ▼                                              │  │
│  │  ┌────────────────────────────────────────────┐         │  │
│  │  │        PostgreSQL Database                 │         │  │
│  │  │  (Railway - Persistent Storage)            │         │  │
│  │  │  • measurements table                      │         │  │
│  │  │  • device tracking                         │         │  │
│  │  │  • analytics data                          │         │  │
│  │  └────────────────────────────────────────────┘         │  │
│  │           │                                              │  │
│  │           ▼                                              │  │
│  │  ┌────────────────────────────────────────────┐         │  │
│  │  │         REST API Endpoints                 │         │  │
│  │  │  /api/v1/measurements                      │         │  │
│  │  │  /api/v1/analytics/summary                 │         │  │
│  │  │  /api/v1/reports/analytics.pdf             │         │  │
│  │  │  /api/v1/firmware/upload                   │         │  │
│  │  └────────────────────────────────────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                             │
                             ▼
          ┌──────────────────────────────────┐
          │       Web Dashboard (UI)         │
          │  • Real-time charts              │
          │  • Data management               │
          │  • OTA firmware upload           │
          │  • PDF report download           │
          └──────────────────────────────────┘
```

**Data Flow:**
1. ESP32 sensors collect coffee aroma data
2. ESP32 publishes measurements to MQTT broker
3. Backend subscribes and stores data in PostgreSQL
4. Dashboard queries REST API for visualization
5. Admin uploads firmware → Backend broadcasts via MQTT
6. ESP32 receives OTA update and flashes new firmware

---

## ✨ Key Features

### 📡 Real-Time Data Collection
- Multi-device support (ESP32-001, ESP32-002, etc.)
- MQTT-based communication with QoS 1
- Automatic reconnection and retry mechanism
- Device identification and tracking

### 📊 Analytics Dashboard
- Live confidence trend visualization
- Class distribution (High/Low grade)
- Summary statistics (total samples, accuracy, confidence)
- Recent measurements table with filtering

### 🔄 OTA Firmware Updates
- Over-The-Air firmware deployment to ESP32 devices
- Chunked transfer (1KB chunks) with ACK mechanism
- MD5 checksum verification
- No physical access required for updates
- Automatic device reboot after successful flash

### 📈 Reporting & Export
- PDF report generation with analytics
- Sample data table with classification results
- Dataset summary (high/low grade distribution)
- Timestamp and metadata tracking

### 🗄️ Database Management
- PostgreSQL for persistent storage
- Full CRUD operations on measurements
- Device-level data filtering
- Automatic schema management

---

## 🛠️ Technology Stack

### Backend
- **Language:** Rust 1.83+
- **Web Framework:** Axum (async Tokio-based)
- **Database:** PostgreSQL 14+ (Railway hosted)
- **ORM:** SQLx (compile-time verified queries)
- **MQTT Client:** Rumqttc (async MQTT 3.1.1)
- **PDF Generation:** printpdf
- **Serialization:** Serde JSON

### Frontend
- **UI:** HTML5, CSS3, JavaScript (ES6+)
- **Charts:** Custom SVG-based visualizations
- **Styling:** Modern dark theme with responsive design

### Infrastructure
- **Containerization:** Docker
- **Cloud Platform:** Azure Web Apps
- **Container Registry:** Azure Container Registry
- **Database Hosting:** Railway PostgreSQL
- **MQTT Broker:** HiveMQ (public broker)

### IoT Devices
- **Hardware:** ESP32-S3 microcontroller
- **Firmware:** Rust (no_std, esp-hal)
- **Communication:** WiFi + MQTT
- **OTA:** Dual partition with rollback support

---

## 📸 Screenshots

### 1. Dashboard - Real-Time Analytics
![Dashboard](docs/images/dashboard.png)
*Live monitoring with confidence trends, class distribution, and summary statistics*

### 2. Data Management
![Data Management](docs/images/data-management.png)
*View, filter, and manage measurement data with device tracking*

### 3. OTA Firmware Update
![OTA Update](docs/images/ota-update.png)
*Upload firmware binaries for over-the-air updates to ESP32 devices*

### 4. PDF Report
![PDF Report](docs/images/pdf-report.png)
*Generated analytics report with measurement data and grade classification*

### 5. Railway PostgreSQL Database
![Railway Database](docs/images/railway-database.png)
*Cloud-hosted PostgreSQL database with measurements table*

---

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host/db` |
| `MQTT_BROKER` | MQTT broker hostname | `broker.hivemq.com` |
| `MQTT_TOPIC` | Topic pattern for measurements | `enose/+/measurement` |
| `DEVICE_ID` | Target ESP32 device for OTA | `ESP32-001` |
| `BASE_URL` | Public backend URL | `https://your-domain.com` |
| `PORT` | HTTP server port | `8080` |

### MQTT Topics

| Topic Pattern | Direction | Purpose |
|---------------|-----------|---------|
| `enose/{device_id}/measurement` | ESP32 → Backend | Sensor measurements |
| `enose/{device_id}/ota/start` | Backend → ESP32 | OTA start signal |
| `enose/{device_id}/ota/chunk` | Backend → ESP32 | Firmware chunks |
| `enose/{device_id}/ota/complete` | Backend → ESP32 | OTA completion |
| `enose/{device_id}/ota/ack` | ESP32 → Backend | Chunk acknowledgment |

---

## 🔌 API Endpoints

### Measurements
- `GET /api/v1/measurements` - Retrieve all measurements
- `DELETE /api/v1/measurements/{id}` - Delete specific measurement

### Analytics
- `GET /api/v1/analytics/summary` - Get summary statistics
- `GET /api/v1/reports/analytics.pdf` - Download PDF report

### OTA
- `POST /api/v1/firmware/upload` - Upload firmware for OTA update

### Dashboard
- `GET /` - Homepage
- `GET /dashboard` - Main dashboard interface

---

## 🚀 Deployment

### Production Stack
- **Backend:** Azure Web Apps (Docker container)
- **Database:** Railway PostgreSQL (cloud-hosted)
- **MQTT Broker:** HiveMQ public broker
- **Container Registry:** Azure Container Registry

### Deployment Architecture
```
┌──────────────────────────────────────────────┐
│         Azure Container Registry             │
│  enosedashboard.azurecr.io                   │
│  └─ enose-dashboard:latest                   │
└──────────────────┬───────────────────────────┘
                   │
                   │ Pull Image
                   ▼
┌──────────────────────────────────────────────┐
│         Azure Web App                        │
│  enose-dashboard-app                         │
│  └─ Docker Container (Rust Backend)          │
│     ├─ Axum HTTP Server :8080                │
│     ├─ MQTT Subscriber                       │
│     └─ OTA Publisher                         │
└──────────────────┬───────────────────────────┘
                   │
                   │ Database Connection
                   ▼
┌──────────────────────────────────────────────┐
│         Railway PostgreSQL                   │
│  └─ measurements table                       │
│  └─ Persistent storage                       │
└──────────────────────────────────────────────┘
```

### Key Deployment Features
- Automatic container restart on failure
- Environment variable injection via Azure configuration
- Secure database connection with SSL
- Scalable architecture with horizontal scaling capability
- Centralized logging via Azure Application Insights

---

## 👥 Contributors

<div align="center">

### Group 12 - C Class

| Name | Student ID | Role |
|------|------------|------|
| **Evan Javier Firdausi Malik** | 2042241010 | IoT System & Embedded Development |
| **Mahendra Dwi Fahreza** | 2042241023 | Backend & Cloud Infrastructure |

</div>

---

## 📄 License

This project is licensed under the MIT License.

---

## 🔗 Links

- **GitHub Repository (Backend):** [mahendrafhrz/backend](https://github.com/mahendrafhrz/backend)
- **GitHub Repository (ESP32):** [mahendrafhrz/esp32-rust](https://github.com/mahendrafhrz/esp32-rust)
- **Live Dashboard:** [enose-dashboard-app.azurewebsites.net](https://enose-dashboard-app.azurewebsites.net)

---

<div align="center">

**E-Nose IoT System** - Real-Time Coffee Quality Classification  
© 2026 Group 12 - C Class

</div>
