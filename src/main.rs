use std::time::Duration;

use anyhow::{Context, Result};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::reset::restart;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration, QoS};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::ota::EspOta;
use esp_idf_svc::wifi::{BlockingWifi, Configuration, EspWifi};
use log::{error, info, warn};
use serde::Serialize;

const DEFAULT_DEVICE_ID: &str = "ESP32-001";
const DEFAULT_INTERVAL_SECONDS: u32 = 30;
const MAX_RETRIES: u8 = 3;
const DHT22_GPIO: i32 = 18;
const MQTT_URL: &str = match option_env!("MQTT_URL") {
    Some(value) => value,
    None => "mqtts://fb113b25.ala.asia-southeast1.emqxsl.com:8883",
};
const MQTT_TOPIC: &str = match option_env!("MQTT_TOPIC") {
    Some(value) => value,
    None => "enose/ESP32-001/measurement",
};
const OTA_URL: &str = match option_env!("OTA_URL") {
    Some(value) => value,
    None => "",
};

const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
    Some(value) => value,
    None => "lola",
};
const WIFI_PASSWORD: &str = match option_env!("WIFI_PASSWORD") {
    Some(value) => value,
    None => "12345678",
};
const DEVICE_ID: &str = match option_env!("DEVICE_ID") {
    Some(value) => value,
    None => DEFAULT_DEVICE_ID,
};
fn interval_seconds() -> u32 {
    option_env!("INTERVAL_SECONDS")
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECONDS)
}

#[derive(Debug, Serialize)]
struct Measurement {
    sample_id: u32,
    score: f32,
    accuracy: f32,
    source: &'static str,
    device_id: &'static str,
    features: [i32; 8],
}

fn read_dht22() -> Result<(f32, f32)> {
    // Retry up to 3 times with 500ms delay between attempts
    for attempt in 1..=3 {
        match read_dht22_once() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt < 3 {
                    warn!("DHT22 read attempt {} failed: {}; retrying in 500ms", attempt, e);
                    FreeRtos::delay_ms(500);
                } else {
                    error!("DHT22 read failed after {} attempts: {}", attempt, e);
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

fn read_dht22_once() -> Result<(f32, f32)> {
    critical_section::with(|_| unsafe {
        esp_idf_sys::gpio_set_pull_mode(DHT22_GPIO, esp_idf_sys::gpio_pull_mode_t_GPIO_PULLUP_ONLY);
        esp_idf_sys::gpio_set_direction(DHT22_GPIO, esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT);

        esp_idf_sys::gpio_set_level(DHT22_GPIO, 0);
        esp_idf_sys::ets_delay_us(18_000);
        esp_idf_sys::gpio_set_direction(DHT22_GPIO, esp_idf_sys::gpio_mode_t_GPIO_MODE_INPUT);

        wait_for_level(0, 100).context("DHT22 response start")?;
        wait_for_level(1, 100).context("DHT22 response ready")?;
        wait_for_level(0, 100).context("DHT22 data start")?;

        let mut data = [0u8; 5];
        for byte_idx in 0..5 {
            for bit_idx in 0..8 {
                wait_for_level(1, 100).with_context(|| {
                    format!("DHT22 bit start timeout at byte {byte_idx} bit {bit_idx}")
                })?;
                let high_start = esp_idf_sys::esp_timer_get_time();
                wait_for_level(0, 100).with_context(|| {
                    format!("DHT22 bit high timeout at byte {byte_idx} bit {bit_idx}")
                })?;
                let high_duration = esp_idf_sys::esp_timer_get_time() - high_start;

                data[byte_idx] <<= 1;
                if high_duration > 45 {
                    data[byte_idx] |= 1;
                }
            }
        }

        let checksum = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);

        if data[4] != checksum {
            anyhow::bail!("DHT22 checksum error: calc={}, recv={}", checksum, data[4]);
        }

        let humidity = f32::from(u16::from_be_bytes([data[0], data[1]])) / 10.0;
        let raw_temp = u16::from_be_bytes([data[2] & 0x7F, data[3]]);
        let temperature = f32::from(raw_temp) / 10.0 * if data[2] & 0x80 != 0 { -1.0 } else { 1.0 };

        Ok((temperature, humidity))
    })
}

fn wait_for_level(level: i32, timeout_us: i32) -> Result<()> {
    let start = unsafe { esp_idf_sys::esp_timer_get_time() };
    let timeout_i64 = i64::from(timeout_us);
    while unsafe { esp_idf_sys::gpio_get_level(DHT22_GPIO) } != level {
        if unsafe { esp_idf_sys::esp_timer_get_time() } - start > timeout_i64 {
            anyhow::bail!("DHT22 timeout waiting for level {} (waited {}us)", level, timeout_us);
        }
        // Yield to prevent tight loop
        unsafe { esp_idf_sys::ets_delay_us(1); }
    }
    Ok(())
}

fn dht22_measurement(sample_id: u32) -> Result<Measurement> {
    let (temperature, humidity) = read_dht22()?;
    Ok(Measurement {
        sample_id,
        score: temperature,
        accuracy: humidity,
        source: "dht22",
        device_id: DEVICE_ID,
        features: [
            (temperature * 10.0) as i32,
            (humidity * 10.0) as i32,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    })
}

fn publish_measurement(client: &mut EspMqttClient<'_>, measurement: &Measurement) -> Result<()> {
    let body = serde_json::to_string(measurement).context("serialize measurement")?;
    client
        .enqueue(MQTT_TOPIC, QoS::AtLeastOnce, false, body.as_bytes())
        .context("publish MQTT measurement")?;
    info!("measurement {} published to MQTT", measurement.sample_id);
    Ok(())
}

fn send_with_retries(client: &mut EspMqttClient<'_>, measurement: &Measurement) {
    for attempt in 1..=MAX_RETRIES {
        match publish_measurement(client, measurement) {
            Ok(()) => return,
            Err(error) if attempt < MAX_RETRIES => {
                warn!("POST attempt {attempt}/{MAX_RETRIES} failed: {error:#}");
                FreeRtos::delay_ms(u32::from(attempt) * 2_000);
            }
            Err(error) => error!("measurement failed after {MAX_RETRIES} attempts: {error:#}"),
        }
    }
}

fn start_mqtt_client() -> Result<(EspMqttClient<'static>, EspMqttConnection)> {
    // MQTT credentials for EMQX Cloud authentication
    let mqtt_username = option_env!("MQTT_USERNAME").unwrap_or("enose_device");
    let mqtt_password = option_env!("MQTT_PASSWORD").unwrap_or("11223344");
    
    let (client, connection) = EspMqttClient::new(
        MQTT_URL,
        &MqttClientConfiguration {
            client_id: Some(DEVICE_ID),
            username: Some(mqtt_username),
            password: Some(mqtt_password),
            ..Default::default()
        },
    )
    .context("create MQTT client")?;
    info!("MQTT connecting with username: {}", mqtt_username);
    Ok((client, connection))
}

fn pump_mqtt_connection(mut connection: EspMqttConnection) {
    std::thread::Builder::new()
        .name("mqtt-connection".to_owned())
        .stack_size(6_000)
        .spawn(move || {
            while let Ok(event) = connection.next() {
                info!("MQTT event: {}", event.payload());
            }
            warn!("MQTT connection closed");
        })
        .expect("spawn MQTT connection thread");
}

fn apply_ota_update() -> Result<()> {
    if OTA_URL.is_empty() {
        return Ok(());
    }

    info!("starting OTA update from {OTA_URL}");
    let http_config = HttpConfiguration {
        timeout: Some(Duration::from_secs(60)),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    let connection = EspHttpConnection::new(&http_config).context("create OTA connection")?;
    let mut client = Client::wrap(connection);
    let mut response = client
        .request(Method::Get, OTA_URL, &[])
        .context("create OTA request")?
        .submit()
        .context("submit OTA request")?;

    if response.status() != 200 {
        anyhow::bail!("OTA server returned HTTP {}", response.status());
    }

    let mut ota = EspOta::new().context("open OTA service")?;
    let mut update = ota
        .initiate_update()
        .context("begin OTA partition update")?;
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = response.read(&mut buffer).context("read OTA response")?;
        if bytes_read == 0 {
            break;
        }
        update
            .write(&buffer[..bytes_read])
            .context("write OTA image")?;
    }

    update.complete().context("complete OTA image")?;
    info!("OTA image accepted; restarting into the new firmware");
    restart();
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Ok(mut ota) = EspOta::new() {
        if let Err(error) = ota.mark_running_slot_valid() {
            warn!("could not confirm running firmware slot: {error}");
        }
    }

    let peripherals = Peripherals::take().context("take ESP32 peripherals")?;
    let sys_loop = EspSystemEventLoop::take().context("take system event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("take NVS partition")?;
    let mut led = PinDriver::output(peripherals.pins.gpio2).context("configure GPIO2 LED")?;
    led.set_low().context("turn LED off")?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))
            .context("create Wi-Fi driver")?,
        sys_loop,
    )?;
    
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID
            .try_into()
            .map_err(|_| anyhow::anyhow!("WIFI_SSID is too long"))?,
        password: WIFI_PASSWORD
            .try_into()
            .map_err(|_| anyhow::anyhow!("WIFI_PASSWORD is too long"))?,
        bssid: None,
        channel: None,
        ..Default::default()
    }))?;

    info!("starting {DEVICE_ID}; MQTT broker: {MQTT_URL}; topic: {MQTT_TOPIC}");
    info!("WiFi credentials: SSID='{}' Password='{}'", WIFI_SSID, WIFI_PASSWORD);
    let mut mqtt_client: Option<EspMqttClient<'static>> = None;
    let mut sample_id = 1;

    loop {
        if !wifi.is_connected().unwrap_or(false) {
            led.set_low()?;
            info!("connecting to Wi-Fi network {WIFI_SSID}");
            
            if let Err(error) = wifi
                .start()
                .and_then(|_| wifi.connect())
                .and_then(|_| wifi.wait_netif_up())
            {
                error!("Wi-Fi connection failed: {error:#}; retrying in 5 seconds");
                FreeRtos::delay_ms(5_000);
                continue;
            }
            led.set_high()?;
            info!("Wi-Fi connected");
            if mqtt_client.is_none() {
                let (client, connection) = start_mqtt_client()?;
                pump_mqtt_connection(connection);
                mqtt_client = Some(client);
                info!("MQTT client connected");
            }
            if let Err(error) = apply_ota_update() {
                error!("OTA update failed: {error:#}");
            }
        }

        match dht22_measurement(sample_id) {
            Ok(measurement) => {
                if let Some(client) = mqtt_client.as_mut() {
                    send_with_retries(client, &measurement);
                }
            }
            Err(error) => {
                error!("DHT22 read failed: {error:#}");
                // Send dummy data to test MQTT connection
                warn!("Sending dummy measurement data for testing");
                let dummy_measurement = Measurement {
                    sample_id,
                    score: 75.5,
                    accuracy: 0.85,
                    source: "ESP32-DHT22-DUMMY",
                    device_id: DEVICE_ID,
                    features: [100, 200, 150, 180, 220, 190, 210, 170],
                };
                if let Some(client) = mqtt_client.as_mut() {
                    send_with_retries(client, &dummy_measurement);
                }
            }
        }
        sample_id = sample_id.wrapping_add(1);
        FreeRtos::delay_ms(interval_seconds().saturating_mul(1_000));
    }
}
