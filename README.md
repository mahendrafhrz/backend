# ESP32 E-Nose Firmware

Rust firmware for an ESP32-S3 DevKit. It reads a DHT22 on GPIO5, publishes measurements to the Railway backend through MQTT, indicates connectivity on GPIO2, and keeps OTA support for firmware updates.

## Architecture choice

This uses **ESP-IDF**, through `esp-idf-hal` and `esp-idf-svc`, because it is the most mature Rust path for the original ESP32 Xtensa chip's Wi-Fi, TLS, and HTTP support. It is not strict `#![no_std]`: ESP-IDF supplies the C runtime and networking stack, and Rust uses `std` on top of it. A strict `no_std` Embassy port is a separate project and is not the stable/simple choice for this MVP.

## Prerequisites on Windows

1. Install Rust with [rustup](https://rustup.rs/).
2. Install the ESP-IDF toolchain using [espup](https://github.com/esp-rs/espup):

   ```powershell
   cargo install espup
   espup install
   ```

3. Install the flashing tool:

   ```powershell
    cargo install cargo-espflash
   ```

4. Install Python 3 and Git if they are not already installed. `espup` installs the Xtensa Rust toolchain and ESP-IDF tools; follow its printed instruction to load `export-esp.ps1` into each new PowerShell session.
5. Connect the board with a data-capable USB cable and identify its COM port in Device Manager.

The detected board is ESP32-S3, using target `xtensa-esp32s3-espidf`. ESP32-C3/C6/RISC-V boards require a different target and HAL configuration.

## Configure credentials

Credentials are compile-time environment variables so they are not stored in source control. In PowerShell, from the project directory:

```powershell
$env:WIFI_SSID = "your-network"
$env:WIFI_PASSWORD = "your-password"
$env:DEVICE_ID = "ESP32-001"
$env:INTERVAL_SECONDS = "30"
$env:MQTT_URL = "mqtt://broker.hivemq.com:1883"
$env:MQTT_TOPIC = "enose/ESP32-001/measurement"
```

The firmware falls back to `CHANGE_ME` for Wi-Fi credentials and the documented backend URL/device ID for the other values. Do not flash while either Wi-Fi value is still `CHANGE_ME`.

## Build and flash

Load the ESP-IDF environment first, then run:

```powershell
. $HOME\export-esp.ps1
$env:IDF_PATH = "C:\.embuild\espressif\esp-idf\v5.2.3"
$env:ESP_IDF_SDKCONFIG_DEFAULTS = "$PWD\sdkconfig.defaults"
cargo +esp espflash flash --release --port COM14
```

If more than one serial port exists, specify it:

```powershell
cargo +esp espflash flash --release --port COM14
```

The `.cargo/config.toml` also registers `espflash flash --monitor` as Cargo's runner, so `cargo run` can be used after the board is connected. A standalone monitor is:

```powershell
espflash monitor
```

## Expected behavior


The JSON payload contains `sample_id`, `score`, `accuracy`, `source`, `device_id`, and eight integer `features`, matching the backend contract. `sample_id` starts at 1 and wraps only after a 32-bit overflow; it is not persisted across reboot.

## Verify the backend

1. Watch the serial monitor for `Wi-Fi connected` and `measurement published to MQTT`.
2. Check the Railway application logs for `MQTT: Message received` and `MQTT: Measurement saved`.
3. Open the [dashboard](https://enose-cloud-backend-production-facd.up.railway.app/dashboard) and wait for its refresh (30 seconds).
4. To test error handling, temporarily use an invalid SSID or broker URL, then restore the values and rebuild. The firmware should log retries rather than panic.

**📖 Complete backend integration guide:** See `MQTT_INTEGRATION_SUMMARY.md` for:
- Backend MQTT configuration
- Payload format validation
- Testing procedures
- Troubleshooting guide

## MQTT and certificates

The firmware publishes JSON to `MQTT_TOPIC` using `MQTT_URL`. Testing defaults are `mqtt://broker.hivemq.com:1883` and `enose/ESP32-001/measurement`, without TLS or credentials. 

**⚠️ IMPORTANT: Backend MQTT subscriber is READY!**
Enable `MQTT_ENABLED=true` in Railway Variables so the backend subscribes to `enose/+/measurement`.

**Railway Variables to set:**
```
MQTT_ENABLED=true
MQTT_BROKER=broker.hivemq.com
MQTT_PORT=1883
MQTT_TOPIC=enose/+/measurement
```

**Backend will:**
- Auto-subscribe to all devices: `enose/ESP32-001/measurement`, `enose/ESP32-002/measurement`, etc.
- Save data to PostgreSQL (same database as HTTP API)
- Display data in dashboard: https://enose-cloud-backend-production-facd.up.railway.app/dashboard

**Public brokers are for testing only; use an authenticated TLS broker in production.**

TLS is still used by OTA through ESP-IDF's built-in certificate bundle. Use an `mqtts://` broker with matching certificate configuration before production MQTT deployment.

**Documentation:** See `MQTT_INTEGRATION_SUMMARY.md` for complete backend integration details.

## OTA update

OTA uses two app slots (`ota_0` and `ota_1`) with rollback support. The update URL must point directly to a compatible ESP32-S3 `.bin` file over HTTPS. The firmware downloads it after Wi-Fi connects, writes it to the inactive slot, validates it, switches the boot slot, and restarts. The running firmware confirms its slot as valid at startup.

For a one-time update, host the new image and set `OTA_URL` before building the firmware currently installed on the board:

```powershell
$env:OTA_URL = "https://your-host.example.com/lancar-esp32-v2.bin"
cargo +esp build --release
```

Flash that intermediary firmware once over USB. It downloads the URL on the next boot. After the update succeeds, remove `OTA_URL` and build normally; otherwise every boot will try the same URL again. Signed images, version checks, and an authenticated update endpoint are still required before production use.

## Sensor and maintenance roadmap

The firmware now reads a DHT22 on **GPIO5**. DHT22 is a digital sensor, not an analog ADC sensor. Wire `VCC` to `3.3V`, `GND` to `GND`, and `DATA` to GPIO5; add a `4.7k-10k` pull-up resistor from `DATA` to `3.3V` if the module does not already include one. Each payload uses temperature and humidity scaled by 10: `[temperature_x10, humidity_x10, 0, 0, 0, 0, 0, 0]`.

The next larger features should be added independently:


## Project layout

```text
Cargo.toml
build.rs
src/main.rs
.cargo/config.toml
sdkconfig.defaults
partitions.csv
.env.example
README.md
```
