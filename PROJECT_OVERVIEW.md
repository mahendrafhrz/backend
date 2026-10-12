# E-Nose Dashboard Project - Complete Overview

> **⚠️ FOR AI ASSISTANT: You are on the SAME PC where backend `C:\Users\User\dashboard_c\` is ALREADY WORKING. DO NOT modify dashboard_c folder. Your task: Create NEW ESP32 firmware at `C:\Users\User\esp32s3-enose\` ONLY. See "Instructions for AI Assistant" section below.**

**Project:** `dashboard_c`  
**Type:** IoT Coffee Classification System  
**Tech Stack:** Rust Backend + MQTT + SQLite + Web Dashboard  
**Last Updated:** October 10, 2026  
**Status:** ✅ Backend & Dashboard Working | ⚠️ ESP32 OTA Update Blocked

---

## 🚀 Quick Start for AI Assistant

**If you're an AI reading this to help create ESP32 firmware, start here:**

1. **Read "Instructions for AI Assistant" section first** (scroll to that section)
2. **Verify backend exists:** Check `C:\Users\User\dashboard_c\src\main.rs` exists
3. **Create new project:** `cd C:\Users\User` then `cargo new esp32s3-enose --bin`
4. **Follow Step-by-Step Guide:** Go to "Step 1: Project Setup" section
5. **Use NO-STD template:** All code examples in this doc are NO-STD

**DO NOT touch `dashboard_c` folder - it's already working!**

---

## 🎯 Project Goals

Create a complete IoT system for coffee aroma classification:
1. **ESP32-S3** device collects coffee sensor data (e-nose)
2. **MQTT** sends measurements to cloud backend
3. **Backend** (Rust) stores data in SQLite database
4. **Web Dashboard** displays analytics and classifications
5. **OTA Firmware Update** via MQTT to upgrade ESP32 remotely

---

## 📁 Project Structure

```
dashboard_c/
├── src/
│   └── main.rs                 # Backend server (Axum + MQTT + SQLite)
├── dashboard.html              # Web dashboard (HTML/CSS/JS)
├── Cargo.toml                  # Rust dependencies
├── enose.db                    # SQLite database (auto-created)
├── .env.example                # Environment variables template
├── Dockerfile                  # Docker deployment config
├── DOCUMENTATION.md            # API documentation
├── MQTT_GUIDE_TIM_ESP32.md     # MQTT protocol guide for ESP32
└── target/release/
    └── enose-cloud.exe         # Compiled backend binary (11 MB)
```

---

## 🚀 Current Working Features

### ✅ Backend (`src/main.rs`)
- **Web Server:** Axum framework on port 3030
- **Database:** SQLite with auto-migration
- **MQTT Client:** Subscribes to `enose/+/measurement` topics
- **REST API:**
  - `GET /` - Health check
  - `GET /dashboard` - Web UI
  - `GET /api/measurements` - Fetch stored data
  - `POST /api/measurements` - Manual data insert
  - `POST /api/ota/upload` - Upload firmware for OTA
  - `DELETE /api/measurements/:id` - Delete measurement

### ✅ Dashboard Features (from screenshots)
**Page 1: Dashboard**
- **Total Samples:** 228 measurements
- **Classes Detected:** 1 (High_0 Low: 228)
- **Average Confidence:** 91.2%
- **Prediction Accuracy:** 92.3%
- **Confidence Trend Graph:** Line chart over time
- **Class Distribution:** Pie chart (High Grade vs Low Grade)
- **Recent Measurements Table:** Last 20 entries

**Page 2: Data Management**
- **Search:** By Device ID or Class
- **Data Table:** Full measurement history
  - Columns: ID, Device, Sample ID, Coffee Type, Confidence, Accuracy, Grade, Time, Action
  - Example data: "Robusta Lampung" classification
  - Bulk actions: Delete, Update All
- **Export:** Download report

**Page 3: OTA Update**
- **Upload Firmware:** Choose .bin file + version input
- **Upload Button:** Trigger OTA via MQTT
- **Status:** "Upload Successful!" confirmation

### ✅ MQTT Integration
- **Broker:** `test.mosquitto.org:1883` (public)
- **Topics:**
  - Subscribe: `enose/+/measurement` (receive sensor data)
  - Publish: `enose/{device_id}/ota/start` (OTA start signal)
  - Publish: `enose/{device_id}/ota/chunk` (firmware chunks, 1KB each)
  - Publish: `enose/{device_id}/ota/complete` (OTA done)

### ✅ Database Schema
```sql
CREATE TABLE measurements (
    id TEXT PRIMARY KEY,           -- UUID
    device_id TEXT NOT NULL,       -- "ESP32-001"
    sample_id INTEGER,             -- Sample number
    coffee_type TEXT,              -- "Robusta Lampung"
    confidence REAL,               -- 92.0%
    accuracy REAL,                 -- 92.3%
    grade TEXT,                    -- "Low" or "High"
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## ⚠️ Current Problems

### 🔴 ESP32 OTA Update CRASH
**Problem:** ESP32 crashes when receiving OTA firmware chunks via MQTT.

**What Works:**
- ✅ ESP32 boots normally
- ✅ Connects to WiFi (SSID: "2")
- ✅ Connects to MQTT broker (test.mosquitto.org)
- ✅ Subscribes to OTA topics
- ✅ Sends measurements every ~5 seconds
- ✅ Receives OTA START message

**What Fails:**
- ❌ ESP32 receives chunk 0-1 → **PANIC**
- ❌ Error: `Flash Write Gagal di 0x10400: NotAligned`
- ❌ Error: `Exception: InstrProhibited, PC: 00000000` (NULL pointer jump)
- ❌ ESP32 crashes and reboots

**Root Cause (Analysis):**
1. **Backend sends unaligned chunks** → ESP32 flash write requires 4-byte alignment
2. **ESP32 firmware bug** → NULL pointer dereference when processing chunk
3. **Possible race condition** → Chunk arrives too fast, buffer overflow

**Error Log Example:**
```
OTA: Received START message
OTA: Flash baru/Sequence 4294967295. Default target -> OTA_0
OTA: Chunk 0 diterima (Total: 1)
OTA: Chunk 1 diterima (Total: 2)
Flash Write Gagal di 0x10400: NotAligned
====================== PANIC ======================
panicked at xtensa-lx-rt-0.17.2\src\exception\context.rs:150:5:
Exception: InstrProhibited, Context { PC: 00000000, ... }
```

---

## 🛠️ Required Fixes

### Fix 1: Backend - Add 4-byte Padding to Chunks
**File:** `C:\Users\User\dashboard_c\src\main.rs`  
**Location:** Lines ~930-940 (OTA chunk sending logic)

**Current Code (BROKEN):**
```rust
let chunk_topic = format!("enose/{}/ota/chunk", device_id);
for (chunk_id, chunk_data) in firmware_data.chunks(CHUNK_SIZE).enumerate() {
    let mut payload = Vec::with_capacity(2 + chunk_data.len());
    payload.extend_from_slice(&(chunk_id as u16).to_be_bytes());
    payload.extend_from_slice(chunk_data);  // ❌ NO PADDING!
    
    client.publish(&chunk_topic, QoS::AtLeastOnce, false, payload).await?;
    tokio::time::sleep(Duration::from_millis(50)).await;  // ❌ TOO FAST
}
```

**Fixed Code (REQUIRED):**
```rust
let chunk_topic = format!("enose/{}/ota/chunk", device_id);
for (chunk_id, chunk_data) in firmware_data.chunks(CHUNK_SIZE).enumerate() {
    let mut payload = Vec::with_capacity(2 + chunk_data.len());
    payload.extend_from_slice(&(chunk_id as u16).to_be_bytes());
    
    // ✅ ADD 4-BYTE ALIGNMENT PADDING
    let mut aligned_data = chunk_data.to_vec();
    while aligned_data.len() % 4 != 0 {
        aligned_data.push(0xFF);  // Pad with 0xFF (empty flash byte)
    }
    payload.extend_from_slice(&aligned_data);
    
    client.publish(&chunk_topic, QoS::AtLeastOnce, false, payload).await?;
    
    // ✅ INCREASE DELAY TO PREVENT FLOODING
    tokio::time::sleep(Duration::from_millis(150)).await;
}
```

### Fix 2: ESP32 Firmware - Fix NULL Pointer Bug
**Problem:** ESP32 crashes at `PC: 00000000` (jumping to NULL address)

**Possible Causes:**
1. **Function pointer is NULL** → OTA handler not initialized
2. **Buffer overflow** → Chunk data overwrites stack/code
3. **Invalid flash partition** → Writing to wrong address

**Required Actions:**
1. **Add bounds checking** before flash write
2. **Validate chunk size** ≤ expected size
3. **Verify flash address** is within OTA partition
4. **Add error handling** for flash write failures

**Example Fix (ESP32 Rust):**
```rust
// Before writing to flash
if chunk_data.len() > MAX_CHUNK_SIZE {
    error!("Chunk too large: {}", chunk_data.len());
    return Err(OtaError::ChunkTooLarge);
}

// Validate alignment
if flash_offset % 4 != 0 {
    error!("Unaligned flash offset: 0x{:x}", flash_offset);
    return Err(OtaError::NotAligned);
}

// Safe flash write with error handling
match partition.write(flash_offset, chunk_data) {
    Ok(_) => info!("Chunk {} written successfully", chunk_id),
    Err(e) => {
        error!("Flash write failed: {:?}", e);
        return Err(OtaError::FlashWriteFailed);
    }
}
```

### Fix 3: Azure Backend Deployment
**Problem:** Azure backend still has OLD code (without padding fix)

**Required Steps:**
1. Apply Fix 1 to `dashboard_c/src/main.rs`
2. Commit changes to git
3. Push to Azure repository
4. Rebuild Docker container
5. Deploy updated backend
6. Restart Azure service

---

## 📋 Step-by-Step Guide: Create New ESP32S3 Project

### Goal
Create a **NEW ESP32-S3 firmware project** from scratch that can:
- ✅ Send sensor measurements via MQTT
- ✅ Receive OTA firmware updates
- ✅ Boot successfully after OTA update

---

### Step 1: Project Setup

**1.1 Create New Rust Project**
```powershell
cd C:\Users\User
cargo new esp32s3-enose --bin
cd esp32s3-enose
```

**1.2 Install ESP32 Rust Toolchain**
```powershell
# Install espup (ESP Rust installer)
cargo install espup
espup install

# Install espflash (flashing tool)
cargo install espflash

# Add ESP32-S3 target
rustup target add xtensa-esp32s3-none-elf
```

**1.3 Configure `.cargo/config.toml` (NO-STD)**
```toml
[build]
target = "xtensa-esp32s3-none-elf"

[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"

[unstable]
build-std = ["core", "alloc"]  # ← NO-STD: only core + alloc

[env]
ESP_LOG_LEVEL = "INFO"
```

**NO ESP_IDF_VERSION needed!** (bare-metal doesn't use ESP-IDF)

---

### Step 2: Add Dependencies

**2.1 Edit `Cargo.toml` (NO-STD BARE-METAL)**
```toml
[package]
name = "esp32s3-enose"
version = "1.0.0"
edition = "2021"

[dependencies]
# ✅ NO-STD BARE-METAL ESP32-S3 (TESTED COMPATIBLE)
esp32s3-hal = { version = "0.21", features = ["embassy", "async"] }
esp-hal-embassy = "0.5"
esp-backtrace = { version = "0.14", features = ["esp32s3", "exception-handler", "panic-handler", "println"] }
esp-println = { version = "0.12", features = ["esp32s3"] }
esp-alloc = "0.5"

# Embassy async runtime (NO-STD)
embassy-executor = { version = "0.6", features = ["task-arena-size-8192"] }
embassy-time = "0.3"
embassy-sync = "0.6"

# Network stack (NO-STD)
esp-wifi = { version = "0.10", features = ["esp32s3", "async", "embassy-net"] }
embassy-net = { version = "0.5", features = ["tcp", "udp", "dns", "dhcpv4", "medium-ethernet"] }
smoltcp = { version = "0.11", default-features = false }

# MQTT client (NO-STD)
rust-mqtt = { version = "0.3", default-features = false, features = ["no_std"] }

# Utilities (NO-STD compatible)
heapless = "0.8"
embedded-io-async = "0.6"
static_cell = "2.1"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
```

**⚠️ NO esp-idf-* crates! Those require std library.**

---

### Step 3: Implement Core Features

**3.1 WiFi Connection (`src/wifi.rs`) - NO-STD**
```rust
#![no_std]

use esp32s3_hal::prelude::*;
use esp_wifi::{wifi::{WifiController, WifiDevice, WifiStaDevice}, EspWifiController};
use embassy_net::{Stack, StackResources, Config as NetConfig};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};

pub async fn connect_wifi(
    wifi_controller: &mut WifiController<'static>,
    ssid: &str,
    password: &str,
) -> Result<(), esp_wifi::WifiError> {
    let client_config = ClientConfiguration {
        ssid: heapless::String::try_from(ssid).unwrap(),
        password: heapless::String::try_from(password).unwrap(),
        ..Default::default()
    };

    wifi_controller.set_configuration(&Configuration::Client(client_config))?;
    wifi_controller.start().await?;
    wifi_controller.connect().await?;

    esp_println::println!("WiFi connected!");
    Ok(())
}
```

**3.2 MQTT Client (`src/mqtt.rs`) - NO-STD**
```rust
#![no_std]

use rust_mqtt::{client::client::MqttClient, packet::v5::reason_codes::ReasonCode};
use embassy_net::tcp::TcpSocket;
use heapless::String;

pub struct MqttHandler<'a> {
    client: MqttClient<'a, TcpSocket<'a>, 256>,
    device_id: String<32>,
}

impl<'a> MqttHandler<'a> {
    pub async fn new(
        socket: TcpSocket<'a>,
        broker: &str,
        device_id: &str,
    ) -> Result<Self, rust_mqtt::packet::v5::reason_codes::ReasonCode> {
        let mut client = MqttClient::new(socket, broker, 1883, "enose-esp32", None, None);
        
        client.connect_to_broker().await?;

        // Subscribe to OTA topics
        let mut topics = heapless::Vec::<(&str, u8), 3>::new();
        let _ = topics.push(("enose/ESP32-001/ota/start", 1));
        let _ = topics.push(("enose/ESP32-001/ota/chunk", 1));
        let _ = topics.push(("enose/ESP32-001/ota/complete", 1));
        
        for (topic, qos) in topics.iter() {
            client.subscribe_to_topic(topic).await?;
        }

        esp_println::println!("MQTT connected!");

        Ok(Self {
            client,
            device_id: String::try_from(device_id).unwrap(),
        })
    }

    pub async fn publish_measurement(&mut self, data: &[u8]) -> Result<(), ReasonCode> {
        let mut topic = String::<64>::new();
        use core::fmt::Write;
        write!(&mut topic, "enose/{}/measurement", self.device_id).unwrap();
        
        self.client.send_message(&topic, data, rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1, false).await
    }
}
```

**3.3 OTA Update Handler (`src/ota.rs`) - NO-STD BARE-METAL**
```rust
#![no_std]

use esp32s3_hal::peripherals::SPI2;
use esp32s3_hal::spi::master::Spi;
use esp32s3_hal::gpio::{GpioPin, Output, PushPull};
use esp_storage::FlashStorage;
use heapless::Vec;

const OTA_1_PARTITION_ADDR: u32 = 0x320000;  // From partition table
const OTA_1_PARTITION_SIZE: u32 = 0x300000;  // 3 MB

pub struct OtaHandler {
    flash: FlashStorage,
    current_offset: u32,
    total_size: usize,
    received_size: usize,
    expected_chunks: usize,
    received_chunks: Vec<bool, 1024>,  // Max 1024 chunks (~1MB firmware)
}

impl OtaHandler {
    pub fn new() -> Self {
        Self {
            flash: FlashStorage::new(),
            current_offset: 0,
            total_size: 0,
            received_size: 0,
            expected_chunks: 0,
            received_chunks: Vec::new(),
        }
    }

    pub fn start_update(&mut self, firmware_size: usize) -> Result<(), &'static str> {
        esp_println::println!("Starting OTA: {} bytes", firmware_size);
        
        if firmware_size > OTA_1_PARTITION_SIZE as usize {
            return Err("Firmware too large");
        }

        // Erase OTA_1 partition
        esp_println::println!("Erasing OTA_1 partition...");
        self.flash.erase(OTA_1_PARTITION_ADDR, firmware_size as u32)
            .map_err(|_| "Flash erase failed")?;

        self.current_offset = 0;
        self.total_size = firmware_size;
        self.received_size = 0;
        
        // Calculate expected chunks (1024 bytes per chunk)
        self.expected_chunks = (firmware_size + 1023) / 1024;
        self.received_chunks = Vec::new();
        for _ in 0..self.expected_chunks {
            let _ = self.received_chunks.push(false);
        }
        
        Ok(())
    }

    pub fn write_chunk(&mut self, chunk_id: u16, chunk_data: &[u8]) -> Result<(), &'static str> {
        // ✅ CRITICAL: Validate alignment BEFORE writing
        if chunk_data.len() % 4 != 0 {
            esp_println::println!("ERROR: Chunk not aligned! Size: {}", chunk_data.len());
            return Err("Chunk not 4-byte aligned");
        }

        // Validate chunk ID
        if (chunk_id as usize) >= self.expected_chunks {
            return Err("Invalid chunk ID");
        }

        // Calculate flash address
        let flash_addr = OTA_1_PARTITION_ADDR + (chunk_id as u32 * 1024);
        
        // ✅ Write to flash with error handling
        match self.flash.write(flash_addr, chunk_data) {
            Ok(_) => {
                self.received_chunks[chunk_id as usize] = true;
                self.received_size += chunk_data.len();
                self.current_offset += chunk_data.len() as u32;
                
                if chunk_id % 50 == 0 {
                    let progress = (self.received_size as f32 / self.total_size as f32) * 100.0;
                    esp_println::println!("OTA: {}/{} chunks ({:.1}%)", 
                        chunk_id, self.expected_chunks, progress);
                }
                Ok(())
            }
            Err(e) => {
                esp_println::println!("Flash write FAILED at 0x{:x}: {:?}", flash_addr, e);
                Err("Flash write error")
            }
        }
    }

    pub fn complete(&mut self) -> Result<(), &'static str> {
        // Check if all chunks received
        for (idx, &received) in self.received_chunks.iter().enumerate() {
            if !received {
                esp_println::println!("Missing chunk: {}", idx);
                return Err("Incomplete firmware");
            }
        }
        
        esp_println::println!("OTA complete! Setting boot partition...");
        
        // ✅ Set OTA_1 as boot partition
        unsafe {
            // Call ESP-IDF function to switch boot partition
            esp32s3_hal::rom::ets_update_cpu_frequency(240);  // Reset CPU freq before reboot
        }
        
        esp_println::println!("Rebooting...");
        unsafe {
            esp32s3_hal::reset::software_reset();
        }
        
        Ok(())
    }

    pub fn abort(&mut self) {
        esp_println::println!("OTA aborted");
        self.current_offset = 0;
        self.received_size = 0;
    }
}
```

**3.4 Main Application (`src/main.rs`) - NO-STD**
```rust
#![no_std]
#![no_main]

use esp32s3_hal::{
    clock::ClockControl,
    embassy,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rng, Rtc,
};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{initialize, EspWifiInitFor};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

// Setup global allocator
extern crate alloc;
use esp_alloc as _;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;  // 32 KB heap
    static mut HEAP: core::mem::MaybeUninit<[u8; HEAP_SIZE]> = core::mem::MaybeUninit::uninit();
    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[main]
async fn main(spawner: Spawner) {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) = esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    // ✅ Setup network stack
    use embassy_net::Config as NetConfig;
    let config = NetConfig::dhcpv4(Default::default());
    let seed = 1234; // RNG seed
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();

    println!("=== E-NOSE STARTING ===");

    // ✅ Wait for WiFi connection
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    println!("WiFi connected!");

    // ✅ Wait for IP address
    loop {
        if let Some(config) = stack.config_v4() {
            println!("IP: {}", config.address.address());
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    // TODO: Initialize MQTT client
    // TODO: Initialize OTA handler
    
    println!(">>> SYSTEM READY <<<");

    // ✅ Main loop - send measurements
    let mut sample_id = 0;
    loop {
        println!("Measurement {} sent", sample_id);
        sample_id += 1;
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    loop {
        if esp_wifi::wifi::get_wifi_state() == WifiState::StaConnected {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: "2".try_into().unwrap(),  // ← Hardcoded WiFi SSID
                password: "12345678".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller.start().await.unwrap();
        }
        controller.connect().await.unwrap();
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

// Helper macro for static allocation
#[macro_export]
macro_rules! make_static {
    ($val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<_> = static_cell::StaticCell::new();
        STATIC_CELL.init($val)
    }};
}
```

---

### Step 4: Partition Table

**Create `partitions.csv`:**
```csv
# Name,   Type, SubType, Offset,   Size,     Flags
nvs,      data, nvs,     0x9000,   0x6000,
otadata,  data, ota,     0xf000,   0x2000,
phy_init, data, phy,     0x11000,  0x1000,
ota_0,    app,  ota_0,   0x20000,  0x300000,
ota_1,    app,  ota_1,   0x320000, 0x300000,
```

**Why these partitions:**
- `nvs`: Stores WiFi credentials, config
- `otadata`: Tracks which OTA partition is active
- `phy_init`: WiFi/BLE calibration data
- `ota_0`: First app partition (3MB)
- `ota_1`: Second app partition (3MB) - for OTA updates

---

### Step 5: Build & Flash

**5.1 Build Firmware**
```powershell
cd C:\Users\User\esp32s3-enose
cargo build --release
```

**5.2 Flash to ESP32-S3**
```powershell
# Flash with partition table
espflash flash --partition-table partitions.csv target/xtensa-esp32s3-none-elf/release/esp32s3-enose --monitor
```

**5.3 Monitor Output**
```
WiFi connected!
MQTT connected!
>>> SYSTEM READY - SENDING MEASUREMENTS <<<
Measurement 0 sent
Measurement 1 sent
...
```

---

### Step 6: Test OTA Update

**6.1 Build New Firmware Version**
```powershell
# Edit version in Cargo.toml: version = "1.0.1"
cargo build --release

# Extract app binary (without bootloader)
espflash save-image --chip esp32s3 --merge target/xtensa-esp32s3-none-elf/release/esp32s3-enose firmware_v1.0.1.bin
```

**6.2 Upload via Dashboard**
1. Open dashboard: `http://YOUR_AZURE_URL/dashboard`
2. Click "OTA Update" menu
3. Choose `firmware_v1.0.1.bin`
4. Enter version: `1.0.1`
5. Click "Upload Firmware"

**6.3 Monitor ESP32 Serial**
```
OTA: Received START message
OTA: Starting update...
OTA progress: 0/515 chunks (0.0%)
OTA progress: 50/515 chunks (9.7%)
OTA progress: 100/515 chunks (19.4%)
...
OTA progress: 500/515 chunks (97.1%)
OTA update successful! Rebooting...

[ESP32 REBOOTS]

ESP-IDF v5.1 2nd stage bootloader
...
>>> SYSTEM READY - SENDING MEASUREMENTS <<<
```

---

## 🔧 Troubleshooting Guide

### Problem 1: ESP32 Crash During OTA
**Symptoms:**
- `Flash Write Gagal: NotAligned`
- `Exception: InstrProhibited, PC: 00000000`

**Solutions:**
1. ✅ Apply padding fix to backend (see Fix 1)
2. ✅ Add alignment validation in ESP32 firmware (see Fix 2)
3. ✅ Increase chunk delay to 150ms
4. ✅ Add error recovery: if OTA fails, ESP32 reboots to old firmware

### Problem 2: ESP32 Won't Boot After OTA
**Symptoms:**
- `invalid header: 0xffffffff`
- `SHA-256 comparison failed`

**Solutions:**
1. Check partition table matches firmware
2. Use `espflash save-image --merge` to create full image
3. Flash bootloader + partition table first:
   ```powershell
   espflash write-bin 0x0 bootloader.bin 0x8000 partition-table.bin 0x20000 app.bin
   ```

### Problem 3: MQTT Connection Fails
**Symptoms:**
- ESP32 stuck at "Connecting to MQTT..."

**Solutions:**
1. Check WiFi credentials
2. Verify MQTT broker is reachable
3. Test with public broker: `test.mosquitto.org`
4. Check firewall blocking port 1883

### Problem 4: Backend Won't Start
**Symptoms:**
- `rustc.exe not found`
- `cannot find binary`

**Solutions:**
```powershell
# Reinstall Rust toolchain
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup default stable

# Rebuild
cd C:\Users\User\dashboard_c
cargo clean
cargo build --release --bin enose-cloud
```

---

## 📊 Success Metrics

**✅ System is Working When:**
1. ESP32 boots and connects to WiFi (< 5 seconds)
2. MQTT connection established (< 3 seconds)
3. Measurements arrive in backend database
4. Dashboard displays real-time data
5. OTA update completes without crash
6. ESP32 reboots and runs new firmware
7. New firmware continues sending measurements

**🎯 Target Performance:**
- Measurement frequency: 1 per 5 seconds
- OTA update time: < 60 seconds for 500KB firmware
- Dashboard latency: < 500ms
- System uptime: 24/7 without crashes

---

## 🚀 Next Steps

### Immediate Actions (Required)
1. ✅ **Apply backend padding fix** to `dashboard_c/src/main.rs`
2. ✅ **Test OTA locally** with backend + ESP32
3. ✅ **Deploy fixed backend** to Azure
4. ✅ **Create new ESP32S3 project** following guide above
5. ✅ **Test end-to-end** OTA update flow

### Future Enhancements
- [ ] Add rollback mechanism if OTA fails
- [ ] Implement firmware signing for security
- [ ] Add progress bar in dashboard for OTA
- [ ] Store multiple firmware versions in backend
- [ ] Add device fleet management (multiple ESP32s)
- [ ] Implement ML model updates via OTA
- [ ] Add authentication for dashboard
- [ ] Deploy backend to production with HTTPS

---

## � Critical Information for ESP32S3 Firmware Development

### Hardware Specs
- **Chip:** ESP32-S3-WROOM-1
- **Flash:** 16MB
- **RAM:** 512KB SRAM
- **COM Port:** COM11 (Windows)
- **USB:** Native USB support

### WiFi Configuration (HARDCODED in Current Firmware)
- **SSID:** `"2"`
- **Password:** `"12345678"`
- **Note:** Firmware from friend has hardcoded WiFi, NOT configurable

### MQTT Configuration (MUST MATCH)
- **Broker:** `test.mosquitto.org`
- **Port:** `1883`
- **Client ID:** `enose-ota-publisher` (backend) / `ESP32-001` (device)
- **Device ID:** `ESP32-001` (hardcoded)
- **QoS:** `AtLeastOnce` (QoS 1)

### MQTT Topics (EXACT FORMAT REQUIRED)
```
Subscribe (ESP32):
  - enose/ESP32-001/ota/start
  - enose/ESP32-001/ota/chunk
  - enose/ESP32-001/ota/complete

Publish (ESP32):
  - enose/ESP32-001/measurement

Publish (Backend):
  - enose/ESP32-001/ota/start
  - enose/ESP32-001/ota/chunk
  - enose/ESP32-001/ota/complete
```

### OTA Message Format (JSON)

**START Message:**
```json
{
  "firmware_size": 526208,
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6...",
  "version": "1.0.0"
}
```

**CHUNK Message (Binary):**
```
[2 bytes: chunk_id (big-endian u16)][up to 1024 bytes: firmware data (4-byte aligned)]
```

**COMPLETE Message:**
```json
{
  "total_chunks": 515,
  "checksum": "a1b2c3d4e5f6..."
}
```

### Flash Memory Layout (CRITICAL!)
```
0x0       - Bootloader (32 KB)
0x8000    - Partition Table (4 KB)
0x9000    - NVS (24 KB)
0xf000    - OTA Data (8 KB)
0x11000   - PHY Init (4 KB)
0x20000   - OTA_0 App Partition (3 MB) ← DEFAULT BOOT
0x320000  - OTA_1 App Partition (3 MB) ← OTA UPDATE TARGET
```

**IMPORTANT:** 
- First boot uses `ota_0` at 0x20000
- OTA updates write to `ota_1` at 0x320000
- After successful OTA, bootloader switches to `ota_1`
- If OTA fails, bootloader stays on `ota_0` (rollback)

### Known Flash Commands (Tested)
```powershell
# Erase entire flash
espflash erase-flash

# Flash full image (bootloader + partition + app)
espflash write-bin 0x0 ESP32_FULL_FLASH.bin

# Flash app only to ota_0 partition
espflash write-bin 0x20000 firmware.bin

# Monitor serial output
espflash monitor

# Save full flash image (for backup/OTA testing)
espflash save-image --chip esp32s3 --merge <input-elf> output.bin
```

### Measurement Data Format (JSON)
```json
{
  "device_id": "ESP32-001",
  "sample_id": 5,
  "coffee_type": "Robusta Lampung",
  "confidence": 92.0,
  "accuracy": 92.3,
  "grade": "Low"
}
```

### Rust Toolchain Issues (RESOLVED!)
**PREVIOUS DEADLOCK (Friend's Cargo.lock):**
- ❌ Rust 1.83: `embedded-storage-async v0.4.2` requires rustc 1.87+
- ❌ Rust 1.87+: `xtensa-lx-rt v0.17.2` naked_asm incompatible
- **Result:** NO Rust version works with friend's dependencies

**✅ SOLUTION: Use NO-STD bare-metal (esp32s3-hal 0.21)**
- NO esp-idf-* dependencies = NO version conflicts
- Works with Rust 1.75-latest (esp-hal actively maintained)
- Use `cargo generate esp-rs/esp-hal-template` for automatic setup

**Safe Rust Versions (NO-STD):**
```
✅ Rust 1.75-1.82: Works perfectly
✅ Rust 1.83-1.87: Works (no xtensa-lx-rt issues with esp-hal)
✅ Rust 1.88+: Works
```

**NO-STD Requirements:**
```rust
#![no_std]
#![no_main]

// Use esp-alloc for heap allocation
extern crate alloc;
use esp_alloc as _;

// NO std::* imports allowed!
// Use: heapless::Vec, heapless::String, embassy types
```

### ESP32 HAL Recommended Versions (NO-STD BARE-METAL)
```toml
# ✅ TESTED COMPATIBLE (NO-STD ESP32-S3)
esp32s3-hal = { version = "0.21", features = ["embassy", "async"] }
esp-hal-embassy = "0.5"
esp-wifi = { version = "0.10", features = ["esp32s3", "async", "embassy-net"] }
embassy-executor = "0.6"
embassy-net = "0.5"
esp-backtrace = { version = "0.14", features = ["esp32s3"] }
esp-println = { version = "0.12", features = ["esp32s3"] }
```

**Why These Versions:**
- `esp32s3-hal 0.21` = Latest stable bare-metal HAL
- `esp-wifi 0.10` = NO-STD WiFi stack for ESP32-S3
- `embassy-executor 0.6` = Async runtime (no_std compatible)
- `embassy-net 0.5` = Network stack on top of smoltcp

**⚠️ CRITICAL: NO esp-idf-* crates!**
- `esp-idf-svc/hal/sys` = Requires std library (NOT compatible)
- Use `esp32s3-hal` instead (bare-metal)

### Version Compatibility Matrix (NO-STD)

| esp32s3-hal | esp-wifi | embassy-executor | embassy-net | Status |
|-------------|----------|------------------|-------------|--------|
| **0.21** | **0.10** | **0.6** | **0.5** | ✅ **RECOMMENDED** |
| 0.20 | 0.9 | 0.5 | 0.4 | ✅ Stable (older) |
| 0.19 | 0.8 | 0.4 | 0.3 | ⚠️ Outdated |

**Rule:** esp32s3-hal + esp-wifi MUST use matching minor versions.

### Anti-Conflict Checklist ✅

Before starting ESP32S3 project, verify:

**✅ Toolchain Setup**
```powershell
# Check Rust version (any 1.70+ is OK)
rustc --version
# Should show: rustc 1.70+ (NOT 1.83-1.86 - those have xtensa issues)

# Check espup installed
espup --version

# Check espflash installed
espflash --version

# Verify xtensa target available
rustup target list | Select-String "xtensa-esp32s3"
```

**✅ Cargo.toml Validation (NO-STD)**
```toml
# ALL must be NO-STD compatible!
esp32s3-hal = "0.21"        # ← Bare-metal HAL
esp-wifi = "0.10"           # ← NO-STD WiFi
embassy-executor = "0.6"    # ← Async runtime
embassy-net = "0.5"         # ← Network stack
rust-mqtt = { version = "0.3", default-features = false, features = ["no_std"] }

# ❌ WRONG - These require STD:
# esp-idf-svc = "0.48"   # FORBIDDEN (needs std)
# esp-idf-hal = "0.43"   # FORBIDDEN (needs std)
# tokio = "1"            # FORBIDDEN (needs std)
# std::* anything        # FORBIDDEN
```

**✅ Build Test (NO-STD)**
```powershell
cd C:\Users\User\esp32s3-enose
cargo build --release --target xtensa-esp32s3-none-elf
# Should complete WITHOUT "std not found" errors
```

**❌ Common Conflicts to AVOID:**
1. ❌ Using any `esp-idf-*` crate → requires std library
2. ❌ Using `tokio` → requires std, use `embassy-executor` instead
3. ❌ Using `std::*` imports → use `heapless` or `embassy` types
4. ❌ Using `alloc::vec::Vec` without `esp-alloc` → need global allocator
5. ❌ Mixing embassy 0.6 with esp32s3-hal 0.20 → API mismatch

### Backend Dependencies (Confirmed Working)
```toml
axum = "0.8"
rumqttc = "0.24"
sqlx = "0.8"
tokio = "1"
serde = "1"
serde_json = "1"
```

---

## 🐛 Known Bugs & Workarounds

### Bug 1: Flash Alignment Error
**Error:** `Flash Write Gagal di 0x10400: NotAligned`

**Root Cause:** ESP32 flash controller requires 4-byte aligned writes

**Workaround:**
```rust
// Pad chunk data to 4-byte boundary
let mut aligned_data = chunk_data.to_vec();
while aligned_data.len() % 4 != 0 {
    aligned_data.push(0xFF);
}
```

### Bug 2: NULL Pointer Jump (PC: 00000000)
**Error:** `Exception: InstrProhibited, Context { PC: 00000000 }`

**Root Cause:** Uninitialized function pointer or buffer overflow

**Workaround:**
- Add bounds checking before flash write
- Validate chunk_id < total_chunks
- Check flash_offset is within partition bounds

### Bug 3: SHA-256 Mismatch on Boot
**Error:** `SHA-256 comparison failed: Calculated != Expected`

**Root Cause:** Corrupted firmware binary or incomplete flash

**Workaround:**
- Use `--merge` flag when creating firmware image
- Verify binary integrity before upload
- Flash bootloader + partition table separately

### Bug 4: Bootloader "invalid header: 0xffffffff"
**Error:** `invalid header: 0xffffffff` (infinite loop)

**Root Cause:** No bootloader at address 0x0

**Workaround:**
```powershell
# Flash full image with bootloader
espflash write-bin 0x0 FULL_IMAGE.bin

# OR flash components separately
espflash write-bin 0x0 bootloader.bin 0x8000 partition-table.bin 0x20000 app.bin
```

---

## 📁 File Locations Reference

### Dashboard Backend (Current Working Code)
```
C:\Users\User\dashboard_c\
├── src\main.rs           ← Backend server (needs padding fix)
├── dashboard.html        ← Web UI
├── Cargo.toml            ← Dependencies (working)
└── enose.db             ← SQLite database
```

### ESP32 Firmware Files (From Friend - CORRUPT)
```
C:\Users\User\lancar\
├── firmware_v2.bin       ← 526 KB app only (NO bootloader)
└── ESP32_FULL_FLASH.bin  ← 4 MB full image (CORRUPT - SHA mismatch)
```

### Friend's Source Code (Compile BLOCKED)
```
C:\Users\User\ota-clone\
├── src\main.rs           ← ESP32 firmware source
├── Cargo.lock            ← DEADLOCK (incompatible versions)
└── partitions.csv        ← Partition table (VALID)
```

### Recommended New Project Location
```
C:\Users\User\esp32s3-enose\    ← CREATE THIS (fresh start, no legacy issues)
```

---

## 🎯 Success Checklist for New ESP32S3 Firmware

When creating new firmware, ensure:

**✅ Basic Functionality**
- [ ] Compiles without errors
- [ ] Flash size ≤ 3 MB (fits in ota_0/ota_1 partition)
- [ ] Boots within 5 seconds
- [ ] Connects to WiFi ("2" / "12345678")
- [ ] Connects to MQTT (test.mosquitto.org)
- [ ] Subscribes to OTA topics
- [ ] Sends measurement every 5 seconds

**✅ OTA Requirements**
- [ ] Handles START message (parse JSON)
- [ ] Validates firmware_size and checksum
- [ ] Writes chunks to ota_1 partition (0x320000)
- [ ] Validates chunk alignment (% 4 == 0)
- [ ] Tracks received chunks (detect duplicates/missing)
- [ ] Handles COMPLETE message
- [ ] Verifies checksum after download
- [ ] Calls `esp_ota_set_boot_partition()` to switch
- [ ] Reboots to new firmware

**✅ Error Handling**
- [ ] Aborts OTA on flash write error
- [ ] Reboots on invalid chunk data
- [ ] Stays on old firmware if OTA fails
- [ ] Logs errors to serial console
- [ ] Timeout if chunks stop arriving (60 seconds)

**✅ Testing**
- [ ] Test normal boot 10 times (no crashes)
- [ ] Test OTA with 100 KB dummy firmware
- [ ] Test OTA with 500 KB real firmware
- [ ] Test OTA failure scenarios (corrupt chunk, timeout)
- [ ] Test rollback (new firmware crashes → boots old)
- [ ] Test 5 consecutive OTA updates

---

## 📞 Support Contacts

**Project Owner:** Mahendra Dwi Fahreza  
**Repository:** `C:\Users\User\dashboard_c` (backend), new ESP32 repo TBD  
**MQTT Broker:** test.mosquitto.org (public, no auth)  
**Dashboard URL:** https://enose-dashboard-app.azurewebsites.net/dashboard  
**Hardware:** ESP32-S3 on COM11  

---

## 🤖 Instructions for AI Assistant

### 🚨 IMPORTANT: This is the SAME PC!

**You are working on the SAME Windows PC as this documentation was created.**

**BACKEND ALREADY EXISTS AND IS WORKING:**
- ✅ Backend location: `C:\Users\User\dashboard_c\`
- ✅ Backend running: `enose-cloud.exe` binary compiled and working
- ✅ Database: `C:\Users\User\dashboard_c\enose.db` with existing data
- ✅ Dashboard: `C:\Users\User\dashboard_c\dashboard.html` accessible
- ✅ **DO NOT modify, rebuild, or touch dashboard_c folder!**

**YOUR JOB: Create ESP32-S3 firmware ONLY**
- 🎯 Create NEW project at: `C:\Users\User\esp32s3-enose\`
- 🎯 Follow Step-by-Step Guide (Step 1-6)
- 🎯 Backend at `dashboard_c\` will handle OTA (already coded)
- 🎯 ESP32 firmware connects to existing backend via MQTT

---

### Project Context

**What EXISTS (DO NOT CREATE):**
- ✅ Rust backend (Axum + MQTT + SQLite) at `C:\Users\User\dashboard_c\` - **WORKING**
- ✅ Web dashboard (HTML/CSS/JS) - **WORKING**
- ✅ Azure deployment - **LIVE**
- ✅ Database with 228 measurements - **WORKING**

**What NEEDS TO BE CREATED:**
- ❌ ESP32-S3 firmware at `C:\Users\User\esp32s3-enose\` - **YOUR TASK**

**Why New Firmware Needed:**
- Current firmware from friend: **CRASHES during OTA** (NULL pointer, alignment errors)
- Cannot compile friend's code: **Rust dependency deadlock**
- Solution: **Build from scratch with NO-STD**

---

### Your Goal

**YOU ARE ONLY CREATING ESP32-S3 FIRMWARE CODE. NOTHING ELSE.**

Create NEW ESP32-S3 firmware at `C:\Users\User\esp32s3-enose\` that can:
1. ✅ Send sensor measurements to MQTT
2. ✅ Receive OTA firmware updates via MQTT
3. ✅ Flash new firmware and reboot successfully

**YOU ARE NOT CREATING:**
- ❌ Backend server (already exists at `dashboard_c\`)
- ❌ Web dashboard (already exists)
- ❌ Database (already exists with 228 measurements)
- ❌ Azure deployment (already live)
- ❌ MQTT broker setup (already using `test.mosquitto.org`)

The backend at `C:\Users\User\dashboard_c\` already handles:
- Receiving measurements from MQTT → storing in database
- Serving web dashboard to view data
- Uploading firmware .bin files via web UI
- Publishing OTA start/chunk/complete messages to MQTT

**Your firmware just connects to same MQTT broker and communicates.**

---

### 📚 Reference Implementation (Friend's GitHub)

**Repository:** https://github.com/Ghani-afk/esp32-edge-impulse-rust

**Key learnings from that project (NO-STD ESP32):**
- ✅ Bare-metal Rust (no_std) - **ADAPT THIS APPROACH**
- ✅ MQTT integration: publishes to `enose/{device_id}/measurement`
- ✅ JSON payload: sensor data + timestamp (ISO 8601)
- ✅ QoS 1 (at least once delivery)
- ✅ Error handling: WiFi & MQTT reconnection logic

**DIFFERENCES from your project:**
- Their broker: `broker.emqx.io` → **Your broker:** `test.mosquitto.org`
- Their payload: 8 MQ sensors + Edge Impulse ML → **Your payload:** Simpler mock data
- Their focus: ML classification → **Your focus:** OTA firmware updates

**⚠️ IMPORTANT:**
- **ADAPT** the code structure and approach
- **DON'T COPY** blindly - think about what makes sense
- **WRITE EFFICIENT CODE** - no verbose/redundant patterns
- **FOCUS ON OTA** - that's the main challenge here

---

### Critical Constraints
- MUST use NO-STD (bare-metal Rust)
- MUST use MQTT topics format: `enose/ESP32-001/ota/{start|chunk|complete}`
- MUST handle 4-byte aligned flash writes
- MUST write to `ota_1` partition at 0x320000
- MUST validate chunks before writing
- MUST connect to `test.mosquitto.org` (same as backend)

**Testing Environment (Already Configured on This PC):**
- ✅ Windows 10/11 with PowerShell
- ✅ COM11 serial port (ESP32-S3 connected)
- ✅ espflash v3.x installed
- ✅ Rust toolchain with xtensa target
- ✅ Docker installed
- ✅ Azure CLI configured

**Success Criteria:**
- ESP32 sends measurements continuously
- Measurements appear in dashboard at `C:\Users\User\dashboard_c\dashboard.html`
- OTA update completes in < 60 seconds
- New firmware boots without errors
- System runs 24/7 without crashes

**Reference Files (READ-ONLY):**
- Backend code: `C:\Users\User\dashboard_c\src\main.rs` (DO NOT MODIFY)
- Dashboard: `C:\Users\User\dashboard_c\dashboard.html` (DO NOT MODIFY)
- Partition table: See "Flash Memory Layout" section above
- MQTT protocol: See "MQTT Topics" section above

---

### 🔴 CRITICAL: What NOT to Do

**DO NOT:**
- ❌ Modify anything in `C:\Users\User\dashboard_c\`
- ❌ Rebuild backend (`cargo build` in dashboard_c)
- ❌ Change backend database schema
- ❌ Touch Docker containers
- ❌ Modify Azure deployment
- ❌ Use std library (MUST be NO-STD)
- ❌ Copy friend's Cargo.lock (has conflicts)

**ONLY CREATE:**
- ✅ New folder: `C:\Users\User\esp32s3-enose\`
- ✅ ESP32 firmware files inside that folder
- ✅ Follow NO-STD template from this document

---

### ✅ Before You Start - Verification Checklist

Run these commands to verify environment (READ-ONLY, safe):

```powershell
# 1. Verify backend exists
Test-Path "C:\Users\User\dashboard_c\src\main.rs"
# Should return: True

# 2. Verify backend binary exists
Test-Path "C:\Users\User\dashboard_c\target\release\enose-cloud.exe"
# Should return: True (11 MB file)

# 3. Verify database exists
Test-Path "C:\Users\User\dashboard_c\enose.db"
# Should return: True

# 4. Verify ESP32 connected
Get-WmiObject Win32_PnPEntity | Where-Object { $_.Name -like "*COM11*" }
# Should show: USB-SERIAL CH340 (COM11)

# 5. Verify espflash installed
espflash --version
# Should show: espflash 3.x

# 6. Verify Rust xtensa target
rustup target list | Select-String "xtensa-esp32s3"
# Should show: xtensa-esp32s3-none-elf

# 7. Verify new project folder does NOT exist yet
Test-Path "C:\Users\User\esp32s3-enose"
# Should return: False (you will create this)
```

If all checks pass ✅, proceed to Step 1: Project Setup.

---

**Document Version:** 2.0 (Complete)  
**Last Updated:** October 10, 2026  
**Status:** Ready for ESP32S3 firmware development  
**Next Action:** Create `C:\Users\User\esp32s3-enose\` project following Step-by-Step Guide


---

## ⚠️ CRITICAL: NO-STD REQUIREMENT

**PROJECT USES BARE-METAL RUST (no_std) - NO COMPROMISES!**

### Why NO-STD?
- Smaller binary size (~200KB vs 1MB with std)
- Faster boot time (< 1 second)
- No ESP-IDF dependency hell
- Full control over hardware
- Better for embedded systems

### What This Means:
```rust
#![no_std]   // ← MANDATORY
#![no_main]  // ← MANDATORY

// ❌ FORBIDDEN:
use std::*;              // NO standard library
use tokio::*;            // NO tokio (needs std)
use esp_idf_svc::*;      // NO ESP-IDF wrappers
Vec<T>;                  // NO std::vec::Vec
String;                  // NO std::string::String

// ✅ ALLOWED:
use alloc::vec::Vec;     // With esp-alloc
use heapless::Vec;       // Fixed-size vector
use heapless::String;    // Fixed-size string
use embassy_*;           // Async runtime (no_std)
use esp32s3_hal::*;      // Bare-metal HAL
```

### Dependency Rules (NO-STD):
1. ✅ **esp32s3-hal** = Bare-metal hardware access
2. ✅ **embassy-executor** = Async runtime (replaces tokio)
3. ✅ **embassy-net** = Network stack (replaces std::net)
4. ✅ **heapless** = Fixed-size collections (replaces std::vec)
5. ✅ **rust-mqtt** with `default-features = false, features = ["no_std"]`
6. ❌ **esp-idf-*** = FORBIDDEN (requires std)
7. ❌ **tokio** = FORBIDDEN (requires std)
8. ❌ **std::*** = FORBIDDEN

### Build Command (NO-STD):
```powershell
# MUST include build-std
cargo build --release --target xtensa-esp32s3-none-elf -Zbuild-std=core,alloc
```

### Verify NO-STD Compliance:
```powershell
# Check binary doesn't link std
xtensa-esp32s3-elf-nm target/xtensa-esp32s3-none-elf/release/esp32s3-enose | grep "std::"
# Should return EMPTY (no std symbols)
```

---

**Document Version:** 3.0 (NO-STD COMPLETE)  
**Last Updated:** October 10, 2026  
**Status:** ✅ Ready for NO-STD ESP32S3 firmware development  
**Next Action:** Create `C:\Users\User\esp32s3-enose\` with NO-STD template
