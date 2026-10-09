import paho.mqtt.client as mqtt
import json
import time
from datetime import datetime
import random

BROKER = "test.mosquitto.org"
PORT = 1883
TOPIC = "enose/esp32_001/measurement"

def on_publish(client, userdata, mid, properties=None, reason_code=None):
    print(f"✅ Published")

client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
client.on_publish = on_publish
client.connect(BROKER, PORT, 60)
client.loop_start()
time.sleep(1)

high_classes = ["Arabica Gayo", "Arabica Toraja", "Arabica Bali"]
low_classes = ["Robusta Lampung", "Liberica Jambi"]

print("Sending 10 test data to Azure...\n")

for i in range(1, 11):
    if i <= 6:
        predicted_class = random.choice(high_classes)
        grade = "high_grade"
        emoji = "🟢"
    else:
        predicted_class = random.choice(low_classes)
        grade = "low_grade"
        emoji = "🔴"
    
    confidence = round(random.uniform(0.80, 0.98), 4)
    
    payload = {
        "device_id": "esp32_001",
        "sample_id": i,
        "predicted_class": predicted_class,
        "confidence": confidence,
        "grade": grade,
        "score": confidence,
        "accuracy": confidence,
        "timestamp": datetime.utcnow().isoformat() + "Z"
    }
    
    client.publish(TOPIC, json.dumps(payload), qos=1)
    print(f"{emoji} {i:2d}. {predicted_class:20s} ({grade:10s}) {confidence*100:5.1f}%")
    time.sleep(0.3)

print("\n✅ Done! Check dashboard:")
print("https://enose-dashboard-app.azurewebsites.net/dashboard")

client.loop_stop()
client.disconnect()
