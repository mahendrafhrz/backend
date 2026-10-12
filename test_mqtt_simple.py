#!/usr/bin/env python3
"""
Simple MQTT test - publish 1 message and exit
"""
import paho.mqtt.client as mqtt
import json
import time

# Configuration
BROKER = "test.mosquitto.org"
PORT = 1883
TOPIC = "enose/TEST-PYTHON/measurement"

# Message payload
payload = {
    "sample_id": 777,
    "score": 88.8,
    "accuracy": 89.9,
    "device_id": "TEST-PYTHON",
    "grade": "low_grade",
    "captured_at": "2026-10-11T13:00:00Z"
}

def on_connect(client, userdata, flags, rc):
    if rc == 0:
        print(f"✅ Connected to {BROKER}:{PORT}")
        print(f"📤 Publishing to topic: {TOPIC}")
        print(f"📦 Payload: {json.dumps(payload, indent=2)}")
        
        # Publish message
        result = client.publish(TOPIC, json.dumps(payload), qos=1)
        
        if result.rc == mqtt.MQTT_ERR_SUCCESS:
            print("✅ Message published successfully!")
        else:
            print(f"❌ Publish failed: {result.rc}")
        
        time.sleep(2)  # Wait for message to send
        client.disconnect()
    else:
        print(f"❌ Connection failed with code {rc}")

def on_publish(client, userdata, mid):
    print(f"✅ Message {mid} confirmed by broker")

def on_disconnect(client, userdata, rc):
    if rc == 0:
        print("🔌 Disconnected cleanly")
    else:
        print(f"⚠️ Disconnected unexpectedly: {rc}")

# Create client
client = mqtt.Client(client_id="python-test-client", clean_session=True)
client.on_connect = on_connect
client.on_publish = on_publish
client.on_disconnect = on_disconnect

print(f"🔄 Connecting to {BROKER}:{PORT}...")

try:
    client.connect(BROKER, PORT, keepalive=60)
    client.loop_forever()
    
    print("\n📊 Check backend:")
    print("https://enose-dashboard-app.azurewebsites.net/api/v1/analytics/summary")
    
except Exception as e:
    print(f"❌ Error: {e}")
