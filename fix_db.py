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

print("Altering columns to DOUBLE PRECISION...")
cur.execute("""
    ALTER TABLE measurements 
    ALTER COLUMN score TYPE DOUBLE PRECISION,
    ALTER COLUMN accuracy TYPE DOUBLE PRECISION,
    ALTER COLUMN confidence TYPE DOUBLE PRECISION;
""")

conn.commit()
print("✓ Database columns updated successfully!")

cur.close()
conn.close()
