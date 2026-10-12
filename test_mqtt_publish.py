import paho.mqtt.publish as publish
import json

# Test payload
payload = {
    "sample_id": 1,
    "score": 90.0,
    "accuracy": 92.0,
    "device_id": "ESP32-001",
    "predicted_class": "Arabica Gayo",
    "confidence": 0.95,
    "grade": "low_grade",
    "captured_at": "2026-10-11T12:35:00Z"
}

# Publish to MQTT
topic = "enose/ESP32-001/measurement"
broker = "test.mosquitto.org"
port = 1883

print(f"Publishing to {broker}:{port}")
print(f"Topic: {topic}")
print(f"Payload: {json.dumps(payload, indent=2)}")

try:
    publish.single(
        topic=topic,
        payload=json.dumps(payload),
        hostname=broker,
        port=port,
        qos=1,
        keepalive=60
    )
    print("\n✅ Message published successfully!")
    print("\nWait 10 seconds, then check dashboard...")
    print("Dashboard: https://enose-dashboard-app.azurewebsites.net/dashboard")
    
    import time
    time.sleep(10)
    
    # Check if data arrived
    import requests
    response = requests.get("https://enose-dashboard-app.azurewebsites.net/api/v1/analytics/summary")
    summary = response.json()
    print(f"\nTotal measurements: {summary['total_measurements']}")
    if summary['total_measurements'] > 0:
        print("🎉 SUCCESS! Data arrived in backend!")
    else:
        print("⚠️ Data not yet visible (may take a few seconds)")
        
except Exception as e:
    print(f"\n❌ Error: {e}")
