use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use csv::StringRecord;
use serde_json::json;
use sqlx::{AnyPool, any::AnyPoolOptions};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const CHANNELS: [&str; 8] = [
    "tgs2600", "mq135", "mq3", "mq6", "mq7", "tgs2602", "tgs2611", "tgs2620",
];

#[derive(Default)]
struct Accumulator {
    sums: [f64; 8],
    count: u64,
    last_timestamp: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    sqlx::any::install_default_drivers();
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./enose.db".into());
    let dataset_root = env::var("DATASET_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("Raw Data Kopi Rafi/Raw Data Kopi Rafi"));
    let pool = connect_database(&database_url).await?;
    prepare_schema(&pool).await?;

    sqlx::query("DELETE FROM measurements WHERE device_id LIKE 'dummy-dataset-%'")
        .execute(&pool)
        .await?;

    let mut inserted = 0;
    for (grade, grade_offset) in [("high_grade", 0), ("low_grade", 6)] {
        for sample_number in 1..=6 {
            let sample_dir = dataset_root
                .join(grade)
                .join(format!("sample{}", sample_number));
            let features = aggregate_folder(&sample_dir)
                .with_context(|| format!("cannot aggregate {}", sample_dir.display()))?;
            let score = demo_score(grade, sample_number);
            let accuracy = demo_accuracy(grade, sample_number);
            let captured_at =
                Utc::now() - Duration::minutes((12 - (grade_offset + sample_number)) as i64);
            let device_id = format!("dummy-dataset-{}", grade.trim_end_matches("_grade"));

            let insert_sql = if is_postgres() {
                "INSERT INTO measurements
                 (id, sample_id, score, accuracy, source, device_id, features_json, captured_at, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            } else {
                "INSERT INTO measurements
                 (id, sample_id, score, accuracy, source, device_id, features_json, captured_at, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            };
            sqlx::query(insert_sql)
                .bind(Uuid::new_v4().to_string())
                .bind(grade_offset + sample_number)
                .bind(score)
                .bind(accuracy)
                .bind("dummy")
                .bind(device_id)
                .bind(serde_json::to_string(&features)?)
                .bind(captured_at.to_rfc3339())
                .bind(Utc::now().to_rfc3339())
                .execute(&pool)
                .await?;
            inserted += 1;
        }
    }

    println!(
        "Seeded {} dummy measurements from {}",
        inserted,
        dataset_root.display()
    );
    println!("Values are dashboard demo aggregates, not Edge Impulse predictions.");
    Ok(())
}

async fn connect_database(database_url: &str) -> Result<AnyPool> {
    if let Some(path) = database_url.strip_prefix("sqlite://./") {
        if !Path::new(path).exists() {
            fs::File::create(path)?;
        }
    }
    Ok(AnyPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?)
}

async fn prepare_schema(pool: &AnyPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS measurements (
            id TEXT PRIMARY KEY,
            sample_id INTEGER NOT NULL CHECK(sample_id BETWEEN 1 AND 18),
            score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
            accuracy REAL NOT NULL CHECK(accuracy BETWEEN 0 AND 100),
            source TEXT NOT NULL DEFAULT 'api',
            device_id TEXT,
            features_json TEXT,
            captured_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN source TEXT NOT NULL DEFAULT 'api'")
        .execute(pool)
        .await;
    Ok(())
}

fn aggregate_folder(folder: &Path) -> Result<Vec<f64>> {
    let mut aggregate = Accumulator::default();
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("csv") {
            continue;
        }
        let mut reader = csv::Reader::from_path(&path)?;
        let headers = reader.headers()?.clone();
        let indexes = CHANNELS.map(|channel| headers.iter().position(|header| header == channel));
        for record in reader.records() {
            let record = record?;
            accumulate_record(&mut aggregate, &record, &indexes);
        }
    }
    if aggregate.count == 0 {
        anyhow::bail!("no numeric sensor rows found in {}", folder.display());
    }
    Ok(aggregate
        .sums
        .iter()
        .map(|value| value / aggregate.count as f64)
        .collect())
}

fn accumulate_record(
    aggregate: &mut Accumulator,
    record: &StringRecord,
    indexes: &[Option<usize>; 8],
) {
    let mut values = [0.0; 8];
    for (position, index) in indexes.iter().enumerate() {
        let Some(index) = index else { return };
        let Ok(value) = record.get(*index).unwrap_or_default().parse::<f64>() else {
            return;
        };
        values[position] = value;
    }
    aggregate
        .sums
        .iter_mut()
        .zip(values)
        .for_each(|(sum, value)| *sum += value);
    aggregate.count += 1;
    aggregate.last_timestamp = record
        .get(0)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0);
}

fn demo_score(grade: &str, sample_number: i32) -> f64 {
    if grade == "high_grade" {
        88.0 + sample_number as f64 * 1.1
    } else {
        62.0 + sample_number as f64 * 1.3
    }
}

fn demo_accuracy(grade: &str, sample_number: i32) -> f64 {
    if grade == "high_grade" {
        91.0 + sample_number as f64 * 0.7
    } else {
        78.0 + sample_number as f64 * 0.8
    }
}

fn is_postgres() -> bool {
    env::var("DATABASE_URL")
        .map(|url| url.starts_with("postgres://") || url.starts_with("postgresql://"))
        .unwrap_or(false)
}

#[allow(dead_code)]
fn _metadata_example() -> serde_json::Value {
    json!({"source": "Raw Data Kopi Rafi", "channels": CHANNELS})
}
