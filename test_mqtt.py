import paho.mqtt.client as mqtt
import json
import time

broker = "test.mosquitto.org"
port = 1883
topic = "enose/ESP32-TEST-MQTT/measurement"

# Test data
test_data = [
    {
        "sample_id": 1,
        "score": 88.5,
        "accuracy": 94.2,
        "predicted_class": "Arabica Toraja",
        "confidence": 0.92,
        "grade": "high_grade",
        "device_id": "ESP32-TEST-MQTT",
        "features": [450.2, 623.1, 789.5, 412.8, 556.3, 601.7, 445.9, 523.4],
        "captured_at": "2026-10-04T06:45:00Z"
    },
    {
        "sample_id": 2,
        "score": 76.3,
        "accuracy": 82.1,
        "predicted_class": "Robusta Lampung",
        "confidence": 0.78,
        "grade": "low_grade",
        "device_id": "ESP32-TEST-MQTT",
        "features": [380.1, 490.2, 610.3, 320.4, 440.5, 550.6, 360.7, 470.8],
        "captured_at": "2026-10-04T06:46:00Z"
    },
    {
        "sample_id": 3,
        "score": 91.2,
        "accuracy": 96.5,
        "predicted_class": "Arabica Gayo",
        "confidence": 0.95,
        "grade": "high_grade",
        "device_id": "ESP32-TEST-MQTT",
        "features": [520.3, 680.4, 820.5, 460.6, 590.7, 710.8, 490.9, 610.1],
        "captured_at": "2026-10-04T06:47:00Z"
    },
    {
        "sample_id": 4,
        "score": 73.8,
        "accuracy": 79.4,
        "predicted_class": "Robusta Bengkulu",
        "confidence": 0.74,
        "grade": "low_grade",
        "device_id": "ESP32-TEST-MQTT",
        "features": [360.2, 470.3, 580.4, 310.5, 420.6, 530.7, 340.8, 450.9],
        "captured_at": "2026-10-04T06:48:00Z"
    },
    {
        "sample_id": 5,
        "score": 89.7,
        "accuracy": 93.8,
        "predicted_class": "Arabica Jawa Tengah",
        "confidence": 0.91,
        "grade": "high_grade",
        "device_id": "ESP32-TEST-MQTT",
        "features": [510.4, 670.5, 810.6, 450.7, 580.8, 700.9, 480.1, 600.2],
        "captured_at": "2026-10-04T06:49:00Z"
    }
]

def on_connect(client, userdata, flags, rc):
    if rc == 0:
        print("✓ Connected to MQTT broker")
    else:
        print(f"✗ Connection failed: {rc}")

def on_publish(client, userdata, mid):
    print(f"  Message {mid} published")

client = mqtt.Client()
client.on_connect = on_connect
client.on_publish = on_publish

print(f"Connecting to {broker}:{port}...")
client.connect(broker, port, 60)
client.loop_start()

time.sleep(2)  # Wait for connection

print(f"\nPublishing 5 test messages to topic: {topic}\n")

for i, data in enumerate(test_data, 1):
    payload = json.dumps(data)
    result = client.publish(topic, payload)
    print(f"{i}. Published: {data['predicted_class']} (confidence: {data['confidence']*100:.1f}%, grade: {data['grade']})")
    time.sleep(1)

time.sleep(2)
client.loop_stop()
client.disconnect()

print("\n✓ All 5 messages sent!")
print("\nCheck:")
print("- Dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard")
print("- Database: SELECT * FROM measurements WHERE device_id = 'ESP32-TEST-MQTT' ORDER BY created_at DESC LIMIT 5;")
