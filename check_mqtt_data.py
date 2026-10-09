import psycopg2

conn = psycopg2.connect(
    host="thomas.proxy.rlwy.net",
    port=38687,
    user="postgres",
    password="ZbpsgrHnPNBLlRkxnjVhhWTFSibGrgWM",
    database="railway",
    sslmode="require"
)

cur = conn.cursor()
cur.execute("""
    SELECT device_id, predicted_class, confidence, grade, accuracy, created_at 
    FROM measurements 
    WHERE device_id = 'ESP32-TEST-MQTT' 
    ORDER BY created_at DESC 
    LIMIT 5;
""")

rows = cur.fetchall()

print("\n✓ Data MQTT di PostgreSQL:")
print("-" * 100)
print(f"{'Coffee Type':<25} | {'Confidence':>10} | {'Grade':<12} | {'Accuracy':>10} | {'Created At':<25}")
print("-" * 100)

for r in rows:
    conf = r[2] * 100 if r[2] else 0
    acc = r[4] if r[4] else 0
    print(f"{r[1]:<25} | {conf:>9.1f}% | {r[3]:<12} | {acc:>9.1f}% | {r[5]}")

print("-" * 100)
print(f"Total: {len(rows)} rows")

cur.close()
conn.close()
