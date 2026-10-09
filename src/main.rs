use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Multipart, Path as AxumPath, State},
    http::{HeaderValue, StatusCode, header},
    response::Html,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{BuiltinFont, Color, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Rgb};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, FromRow, any::AnyPoolOptions};
use std::{
    borrow::Borrow, env, fs::OpenOptions, io::BufWriter, path::Path, sync::Arc, time::Duration,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: AnyPool,
}

#[derive(Debug, Deserialize)]
struct CreateMeasurement {
    sample_id: i32,
    score: f64,
    accuracy: f64,
    predicted_class: Option<String>,
    confidence: Option<f64>,
    grade: Option<String>,
    source: Option<String>,
    device_id: Option<String>,
    features: Option<Vec<f64>>,
    captured_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
struct Measurement {
    id: String,
    sample_id: i32,
    score: f64,
    accuracy: f64,
    predicted_class: Option<String>,
    confidence: Option<f64>,
    grade: Option<String>,
    source: String,
    device_id: Option<String>,
    features_json: Option<String>,
    captured_at: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct AnalyticsSummary {
    total_measurements: i64,
    sample_count: i64,
    average_score: f32,
    average_accuracy: f32,
    latest_captured_at: Option<String>,
    by_sample: Vec<SampleSummary>,
}

#[derive(Debug, Serialize, FromRow)]
struct SampleSummary {
    sample_id: i32,
    measurement_count: i64,
    average_score: f32,
    average_accuracy: f32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(env("RUST_LOG", "info"))
        .init();

    let database_url = env("DATABASE_URL", "sqlite://./enose.db");
    if let Some(path) = database_url.strip_prefix("sqlite://./") {
        if !Path::new(path).exists() {
            OpenOptions::new().create(true).write(true).open(path)?;
        }
    }
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS measurements (
            id TEXT PRIMARY KEY,
            sample_id INTEGER NOT NULL,
            score REAL NOT NULL CHECK(score BETWEEN 0 AND 100),
            accuracy REAL NOT NULL CHECK(accuracy BETWEEN 0 AND 100),
            predicted_class TEXT,
            confidence REAL,
            grade TEXT,
            source TEXT NOT NULL DEFAULT 'api',
            device_id TEXT,
            features_json TEXT,
            captured_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;
    
    // Add columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN source TEXT NOT NULL DEFAULT 'api'")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN predicted_class TEXT")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN confidence REAL")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE measurements ADD COLUMN grade TEXT")
        .execute(&pool)
        .await;

    let state = AppState { pool };
    
    let state_arc = Arc::new(state);
    
    if env("MQTT_ENABLED", "false").eq_ignore_ascii_case("true") {
        tokio::spawn(mqtt_worker((*state_arc).clone()));
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/dashboard", get(dashboard))

        .route(
            "/api/v1/measurements",
            get(list_measurements).post(create_measurement),
        )
        .route("/api/v1/measurements/{id}", axum::routing::delete(delete_measurement))

        .route("/api/v1/analytics/summary", get(summary))
        .route("/api/v1/reports/analytics.pdf", get(analytics_pdf))
        
        .route("/api/v1/firmware/upload", axum::routing::post(upload_firmware))
        .route("/api/v1/firmware/{version}", get(download_firmware))
        
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state_arc);

    let address = format!("{}:{}", env("HOST", "0.0.0.0"), env("PORT", "8080"));
    let listener = TcpListener::bind(&address).await?;
    info!("E-Nose cloud backend listening on http://{}", address);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "enose-cloud"}))
}

async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../dashboard.html"))
}

async fn list_measurements(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Measurement>>, ApiError> {
    let rows = sqlx::query_as::<_, Measurement>(
        "SELECT id, sample_id, CAST(score AS DOUBLE PRECISION) AS score,
         CAST(accuracy AS DOUBLE PRECISION) AS accuracy, predicted_class, 
         CAST(confidence AS DOUBLE PRECISION) AS confidence, grade, source, device_id, 
         features_json, captured_at, created_at
         FROM measurements ORDER BY captured_at DESC LIMIT 500",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_measurement(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateMeasurement>,
) -> Result<(StatusCode, Json<Measurement>), ApiError> {
    validate(&payload)?;
    let id = Uuid::new_v4().to_string();
    let captured_at = payload.captured_at.unwrap_or_else(Utc::now).to_rfc3339();
    let created_at = Utc::now().to_rfc3339();
    let features_json = payload
        .features
        .map(|features| serde_json::to_string(&features))
        .transpose()?;

    let insert_sql = if is_postgres() {
        "INSERT INTO measurements (id, sample_id, score, accuracy, predicted_class, confidence, grade, source, device_id, features_json, captured_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
    } else {
        "INSERT INTO measurements (id, sample_id, score, accuracy, predicted_class, confidence, grade, source, device_id, features_json, captured_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    };
    sqlx::query(insert_sql)
        .bind(&id)
        .bind(payload.sample_id)
        .bind(payload.score)
        .bind(payload.accuracy)
        .bind(&payload.predicted_class)
        .bind(payload.confidence)
        .bind(&payload.grade)
        .bind(payload.source.unwrap_or_else(|| "api".to_string()))
        .bind(&payload.device_id)
        .bind(&features_json)
        .bind(captured_at)
        .bind(created_at)
        .execute(&state.pool)
        .await?;

    let select_sql = if is_postgres() {
        "SELECT id, sample_id, CAST(score AS DOUBLE PRECISION) AS score,
         CAST(accuracy AS DOUBLE PRECISION) AS accuracy, predicted_class, 
         CAST(confidence AS DOUBLE PRECISION) AS confidence, grade, source, device_id, 
         features_json, captured_at, created_at
         FROM measurements WHERE id = $1"
    } else {
        "SELECT id, sample_id, score, accuracy, predicted_class, confidence, grade, source, device_id, features_json, captured_at, created_at FROM measurements WHERE id = ?"
    };
    let measurement = sqlx::query_as::<_, Measurement>(select_sql)
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(measurement)))
}

async fn delete_measurement(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, ApiError> {
    let delete_sql = if is_postgres() {
        "DELETE FROM measurements WHERE id = $1"
    } else {
        "DELETE FROM measurements WHERE id = ?"
    };
    
    let result = sqlx::query(delete_sql)
        .bind(&id)
        .execute(&state.pool)
        .await?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

async fn summary(State(state): State<Arc<AppState>>) -> Result<Json<AnalyticsSummary>, ApiError> {
    let total_measurements = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM measurements")
    .fetch_one(&state.pool)
    .await?;
    let average_score = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(score), 0.0) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let average_accuracy = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(accuracy), 0.0) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let latest_captured_at = sqlx::query_scalar::<_, Option<String>>(
        "SELECT MAX(captured_at) FROM measurements",
    )
    .fetch_one(&state.pool)
    .await?;
    let sample_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT sample_id) FROM measurements")
            .fetch_one(&state.pool)
            .await?;
    let by_sample = sqlx::query_as::<_, SampleSummary>(
        "SELECT sample_id, COUNT(*) AS measurement_count, AVG(score) AS average_score, AVG(accuracy) AS average_accuracy
         FROM measurements GROUP BY sample_id ORDER BY sample_id",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(AnalyticsSummary {
        total_measurements,
        sample_count,
        average_score: average_score as f32,
        average_accuracy: average_accuracy as f32,
        latest_captured_at,
        by_sample,
    }))
}

async fn analytics_pdf(State(state): State<Arc<AppState>>) -> Result<Response, ApiError> {
    // Fetch all measurements with grade field
    let all_measurements = sqlx::query_as::<_, Measurement>(
        "SELECT id, sample_id, score, accuracy, predicted_class, confidence, grade, source, device_id, features_json, captured_at, created_at
         FROM measurements ORDER BY captured_at DESC"
    )
    .fetch_all(&state.pool)
    .await?;
    
    let summary = summary(State(state)).await?.0;
    let (document, page, layer) = PdfDocument::new(
        "E-Nose Dashboard Report",
        Mm(210.0),
        Mm(297.0),
        "Dashboard Report",
    );
    let regular = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let bold = document.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let current_layer = document.get_page(page).get_layer(layer);
    
    // Modern color palette - Green theme matching dashboard
    let green = Color::Rgb(Rgb::new(0.086, 0.639, 0.290, None)); // #16a34a
    let _dark_green = Color::Rgb(Rgb::new(0.082, 0.502, 0.239, None)); // #15803d
    let light_bg = Color::Rgb(Rgb::new(0.961, 0.969, 0.980, None)); // #f5f7fa
    let white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));
    let gray_text = Color::Rgb(Rgb::new(0.176, 0.216, 0.282, None)); // #2d3748
    let muted_text = Color::Rgb(Rgb::new(0.392, 0.455, 0.545, None)); // #64748b
    let border = Color::Rgb(Rgb::new(0.886, 0.910, 0.941, None)); // #e2e8f0

    // Clean header - no navy background, just simple green accent
    draw_rect(&current_layer, 0.0, 272.0, 210.0, 25.0, &white);
    text(&current_layer, "E-Nose Dashboard", 22.0, 20.0, 282.0, &bold, &gray_text);
    text(
        &current_layer,
        "Real-time Coffee Quality Monitoring Report",
        11.0,
        20.0,
        274.0,
        &regular,
        &muted_text,
    );
    
    // Date/time in top right
    text(
        &current_layer,
        &format!("{}", Utc::now().format("%B %d, %Y")),
        9.0,
        150.0,
        284.0,
        &regular,
        &gray_text,
    );
    text(
        &current_layer,
        &format!("{}", Utc::now().format("%H:%M UTC")),
        8.0,
        150.0,
        277.0,
        &regular,
        &muted_text,
    );
    
    // Green divider line
    draw_rect(&current_layer, 20.0, 268.0, 170.0, 1.0, &green);

    // Stats cards - matching dashboard layout
    text(&current_layer, "Overview", 14.0, 20.0, 258.0, &bold, &gray_text);
    
    // Count high/low grade from actual measurements
    let high_count = all_measurements.iter().filter(|m| m.grade.as_deref() == Some("high_grade")).count();
    let low_count = all_measurements.iter().filter(|m| m.grade.as_deref() == Some("low_grade")).count();
    
    // Calculate average confidence from confidence field
    let avg_confidence = if !all_measurements.is_empty() {
        all_measurements.iter()
            .map(|m| m.confidence.unwrap_or(m.score / 100.0) as f32)
            .sum::<f32>() / all_measurements.len() as f32
    } else {
        0.0f32
    };
    
    let cards = [
        ("Total Samples", summary.total_measurements.to_string(), "measurements received"),
        ("Classes Detected", "2".to_string(), &format!("High: {} | Low: {}", high_count, low_count)),
        ("Avg Confidence", format!("{:.1}%", avg_confidence * 100.0), "model certainty"),
        ("Accuracy", format!("{:.1}%", summary.average_accuracy), "prediction accuracy"),
    ];
    
    let mut card_x = 20.0;
    for (label, value, subtitle) in &cards {
        // White card with subtle shadow
        draw_rect(&current_layer, card_x, 227.0, 40.0, 24.0, &white);
        draw_rect(&current_layer, card_x, 226.5, 40.0, 0.5, &border);
        
        text(&current_layer, label, 8.0, card_x + 3.0, 246.0, &regular, &muted_text);
        text(&current_layer, value, 16.0, card_x + 3.0, 237.0, &bold, &green);
        text(&current_layer, subtitle, 7.0, card_x + 3.0, 229.0, &regular, &muted_text);
        
        card_x += 42.5;
    }

    // Recent Measurements Table
    text(&current_layer, "Recent Measurements", 14.0, 20.0, 215.0, &bold, &gray_text);
    
    // Table header
    draw_rect(&current_layer, 20.0, 199.0, 170.0, 10.0, &light_bg);
    text(&current_layer, "Sample", 8.5, 24.0, 203.0, &bold, &muted_text);
    text(&current_layer, "Count", 8.5, 75.0, 203.0, &bold, &muted_text);
    text(&current_layer, "Confidence", 8.5, 105.0, 203.0, &bold, &muted_text);
    text(&current_layer, "Accuracy", 8.5, 145.0, 203.0, &bold, &muted_text);
    text(&current_layer, "Class", 8.5, 172.0, 203.0, &bold, &muted_text);

    let mut y = 192.0;
    // Show latest 12 measurements (not grouped by sample_id)
    let measurements_to_show: Vec<_> = all_measurements.iter().take(12).collect();
    
    for (idx, measurement) in measurements_to_show.iter().enumerate() {
        // Alternating row background
        if idx % 2 == 1 {
            draw_rect(
                &current_layer,
                20.0,
                y - 1.0,
                170.0,
                7.0,
                Color::Rgb(Rgb::new(0.98, 0.988, 0.992, None)),
            );
        }
        
        // Use grade field from database
        let grade = measurement.grade.as_deref().unwrap_or("unknown");
        let class_label = if grade == "high_grade" { "High Grade" } else { "Low Grade" };
        let class_color = if grade == "high_grade" { &green } else { &Color::Rgb(Rgb::new(0.863, 0.106, 0.106, None)) };
        
        // Use predicted_class if available, otherwise use sample_id
        let sample_name = measurement.predicted_class.clone()
            .unwrap_or_else(|| format!("Sample {}", measurement.sample_id));
        
        text(
            &current_layer,
            &sample_name,
            8.0,
            24.0,
            y,
            &regular,
            &gray_text,
        );
        text(
            &current_layer,
            "1", // Individual measurement, count = 1
            8.0,
            78.0,
            y,
            &regular,
            &gray_text,
        );
        text(
            &current_layer,
            &format!("{:.1}%", (measurement.confidence.unwrap_or(measurement.score / 100.0) * 100.0)),
            8.0,
            108.0,
            y,
            &regular,
            &gray_text,
        );
        text(
            &current_layer,
            &format!("{:.1}%", measurement.accuracy),
            8.0,
            148.0,
            y,
            &regular,
            &gray_text,
        );
        text(
            &current_layer,
            class_label,
            8.0,
            172.0,
            y,
            &bold,
            class_color,
        );
        
        // Subtle separator line
        draw_rect(&current_layer, 20.0, y - 2.0, 170.0, 0.3, &border);
        y -= 7.0;
    }

    // Calculate where the table ends
    let table_end_y = y.max(80.0);
    
    // Dataset Summary section
    let info_y = table_end_y - 10.0;
    text(&current_layer, "Dataset Summary", 14.0, 20.0, info_y, &bold, &gray_text);
    
    let box_y = info_y - 18.0;
    
    // Single info box
    draw_rect(&current_layer, 20.0, box_y, 170.0, 16.0, &white);
    draw_rect(&current_layer, 20.0, box_y - 0.5, 170.0, 0.5, &border);
    
    text(
        &current_layer,
        &format!("High Grade: {} samples", high_count),
        9.0,
        24.0,
        box_y + 9.0,
        &regular,
        &gray_text,
    );
    text(
        &current_layer,
        &format!("Low Grade: {} samples", low_count),
        9.0,
        24.0,
        box_y + 3.0,
        &regular,
        &gray_text,
    );
    
    // Footer
    text(
        &current_layer,
        "Generated from E-Nose Dashboard - Real-time coffee quality monitoring system",
        7.5,
        20.0,
        15.0,
        &regular,
        &muted_text,
    );
    text(
        &current_layer,
        "Confidence values represent model certainty in classification results",
        7.0,
        20.0,
        10.0,
        &regular,
        &muted_text,
    );
    let mut bytes = Vec::new();
    document.save(&mut BufWriter::new(std::io::Cursor::new(&mut bytes)))?;
    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=enose-dashboard-report.pdf"),
    );
    Ok(response)
}

fn text<C: Borrow<Color>>(
    layer: &PdfLayerReference,
    value: &str,
    size: f64,
    x: f64,
    y: f64,
    font: &printpdf::IndirectFontRef,
    color: C,
) {
    layer.set_fill_color(color.borrow().clone());
    layer.use_text(value, size as f32, Mm(x as f32), Mm(y as f32), font);
}

fn draw_rect<C: Borrow<Color>>(
    layer: &PdfLayerReference,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: C,
) {
    let polygon = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x as f32), Mm(y as f32)), false),
            (Point::new(Mm(x as f32), Mm((y + height) as f32)), false),
            (
                Point::new(Mm((x + width) as f32), Mm((y + height) as f32)),
                false,
            ),
            (Point::new(Mm((x + width) as f32), Mm(y as f32)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.set_fill_color(color.borrow().clone());
    layer.add_polygon(polygon);
}

fn draw_rule<C: Borrow<Color>>(layer: &PdfLayerReference, x1: f64, y: f64, x2: f64, color: C) {
    let polygon = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x1 as f32), Mm(y as f32)), false),
            (Point::new(Mm(x2 as f32), Mm(y as f32)), false),
            (Point::new(Mm(x2 as f32), Mm((y + 0.2) as f32)), false),
            (Point::new(Mm(x1 as f32), Mm((y + 0.2) as f32)), false),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.set_fill_color(color.borrow().clone());
    layer.add_polygon(polygon);
}

fn section_title<C1: Borrow<Color>, C2: Borrow<Color>>(
    layer: &PdfLayerReference,
    title: &str,
    x: f64,
    y: f64,
    width: f64,
    color: C1,
    font: &printpdf::IndirectFontRef,
    text_color: C2,
) {
    draw_rect(layer, x, y, width, 8.0, color);
    text(layer, title, 9.5, x + 3.0, y + 2.3, font, text_color);
}

fn validate(payload: &CreateMeasurement) -> Result<(), ApiError> {
    if payload.sample_id < 1
        || !(0.0..=100.0).contains(&payload.score)
        || !(0.0..=100.0).contains(&payload.accuracy)
    {
        return Err(ApiError::BadRequest(
            "sample_id must be >= 1 and score/accuracy must be 0..100".into(),
        ));
    }
    Ok(())
}

async fn mqtt_worker(state: AppState) {
    loop {
        info!("MQTT: Initializing connection...");
        
        let broker = env("MQTT_BROKER", "test.mosquitto.org");
        let port = env("MQTT_PORT", "1883").parse().unwrap_or(1883);
        let username = env::var("MQTT_USERNAME").ok();
        let password = env::var("MQTT_PASSWORD").ok();
        let topic = env("MQTT_TOPIC", "enose/+/measurement");
        let _tls_enabled = env("MQTT_TLS", "false").eq_ignore_ascii_case("true");
        
        let mut options = MqttOptions::new(
            env("MQTT_CLIENT_ID", "enose-cloud"),
            broker.clone(),
            port,
        );
        options.set_keep_alive(Duration::from_secs(60));
        
        // Set credentials if provided
        if let (Some(user), Some(pass)) = (username.as_ref(), password.as_ref()) {
            options.set_credentials(user, pass);
            info!("MQTT: Using authentication");
        }
        
        // TODO: TLS support can be added here when needed
        // if tls_enabled {
        //     use rumqttc::TlsConfiguration;
        //     let tls_config = TlsConfiguration::Simple { ... };
        //     options.set_tls_configuration(tls_config);
        // }
        
        let (client, mut event_loop) = AsyncClient::new(options, 10);
        
        // Subscribe to topic
        match client.subscribe(&topic, QoS::AtLeastOnce).await {
            Ok(_) => {
                info!(%topic, %broker, %port, "MQTT: Connected and subscribed");
            }
            Err(error) => {
                error!(%error, "MQTT: Subscribe failed, retrying in 10s");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
        
        // Event loop
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(message))) => {
                    info!(
                        topic = %message.topic,
                        payload_len = message.payload.len(),
                        "MQTT: Message received"
                    );
                    
                    match serde_json::from_slice::<CreateMeasurement>(&message.payload) {
                        Ok(mut payload) => {
                            payload.source = Some("mqtt".to_string());
                            
                            if let Err(e) = validate(&payload) {
                                warn!(?e, "MQTT: Payload validation failed");
                                continue;
                            }
                            
                            let state_arc = Arc::new(state.clone());
                            match create_measurement(State(state_arc), Json(payload)).await {
                                Ok((status, json)) => {
                                    info!(
                                        measurement_id = %json.id,
                                        sample_id = json.sample_id,
                                        device_id = ?json.device_id,
                                        "MQTT: Measurement saved ({})",
                                        status.as_u16()
                                    );
                                }
                                Err(error) => {
                                    warn!(?error, "MQTT: Database insert failed");
                                }
                            }
                        }
                        Err(error) => {
                            warn!(
                                %error,
                                payload = ?String::from_utf8_lossy(&message.payload),
                                "MQTT: Invalid JSON payload"
                            );
                        }
                    }
                }
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    info!("MQTT: Connection acknowledged");
                }
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    info!("MQTT: Subscription acknowledged");
                }
                Err(error) => {
                    error!(%error, "MQTT: Connection error, reconnecting in 10s");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    break; // Break inner loop to reconnect
                }
                _ => {}
            }
        }
        
        // Reconnect delay before outer loop retries
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn env(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn is_postgres() -> bool {
    env::var("DATABASE_URL")
        .map(|url| url.starts_with("postgres://") || url.starts_with("postgresql://"))
        .unwrap_or(false)
}

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound,
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl From<printpdf::Error> for ApiError {
    fn from(error: printpdf::Error) -> Self {
        Self::Internal(error.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": message})),
            )
                .into_response(),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "not found"})),
            )
                .into_response(),
            Self::Internal(error) => {
                error!(%error, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal server error"})),
                )
                    .into_response()
            }
        }
    }
}

// OTA Firmware Upload Handler
async fn upload_firmware(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut firmware_data: Option<Vec<u8>> = None;
    let mut version: Option<String> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Multipart error: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "firmware" {
            let data = field.bytes().await.map_err(|e| {
                ApiError::BadRequest(format!("Read firmware error: {}", e))
            })?;
            firmware_data = Some(data.to_vec());
        } else if name == "version" {
            let text = field.text().await.map_err(|e| {
                ApiError::BadRequest(format!("Read version error: {}", e))
            })?;
            version = Some(text);
        }
    }

    let firmware_data = firmware_data.ok_or_else(|| {
        ApiError::BadRequest("Missing firmware file".to_string())
    })?;
    
    let version = version.ok_or_else(|| {
        ApiError::BadRequest("Missing version".to_string())
    })?;

    // Create firmware directory if not exists
    let firmware_dir = std::path::Path::new("./firmware");
    if !firmware_dir.exists() {
        tokio::fs::create_dir_all(firmware_dir).await.map_err(|e| {
            ApiError::Internal(e.into())
        })?;
    }

    // Save firmware file
    let filename = format!("firmware_{}.bin", version);
    let filepath = firmware_dir.join(&filename);
    
    let mut file = tokio::fs::File::create(&filepath).await.map_err(|e| {
        ApiError::Internal(e.into())
    })?;
    
    file.write_all(&firmware_data).await.map_err(|e| {
        ApiError::Internal(e.into())
    })?;

    // Calculate checksum (MD5)
    let checksum = format!("{:x}", md5::compute(&firmware_data));
    
    // Get base URL
    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://enose-dashboard-app.azurewebsites.net".to_string());
    let download_url = format!("{}/api/v1/firmware/{}", base_url, version);

    info!(
        version = %version,
        size = firmware_data.len(),
        checksum = %checksum,
        "Firmware uploaded successfully"
    );

    // Clone for response
    let response_version = version.clone();
    let response_size = firmware_data.len();
    let response_checksum = checksum.clone();

    // Publish OTA update via MQTT with 1KB chunks (NO HTTP!)
    tokio::spawn(async move {
        let broker = env("MQTT_BROKER", "test.mosquitto.org");
        let port = env("MQTT_PORT", "1883").parse().unwrap_or(1883);
        let device_id = env("DEVICE_ID", "ESP32-001");
        
        let mut options = MqttOptions::new("enose-ota-publisher", broker.clone(), port);
        options.set_keep_alive(Duration::from_secs(60));
        
        let (client, mut event_loop) = AsyncClient::new(options, 10);
        tokio::spawn(async move {
            loop {
                if event_loop.poll().await.is_err() {
                    break;
                }
            }
        });
        
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        // Split firmware into 1KB chunks
        const CHUNK_SIZE: usize = 1024;
        let total_chunks = (firmware_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        
        info!(
            version = %version,
            size = firmware_data.len(),
            chunks = total_chunks,
            "Starting OTA via MQTT chunks"
        );
        
        // 1. Send OTA START message
        let start_topic = format!("enose/{}/ota/start", device_id);
        let start_message = serde_json::json!({
            "version": version,
            "total_size": firmware_data.len(),
            "total_chunks": total_chunks,
            "chunk_size": CHUNK_SIZE,
            "checksum": checksum,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        if let Err(e) = client.publish(
            &start_topic,
            QoS::AtLeastOnce,
            false,
            serde_json::to_vec(&start_message).unwrap_or_default()
        ).await {
            error!(?e, "Failed to publish OTA START");
            return;
        }
        
        info!(%start_topic, "OTA START sent");
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 2. Send firmware chunks
        let chunk_topic = format!("enose/{}/ota/chunk", device_id);
        for (chunk_id, chunk_data) in firmware_data.chunks(CHUNK_SIZE).enumerate() {
            // Format: [chunk_id: 2 bytes][data: up to 1024 bytes]
            let mut payload = Vec::with_capacity(2 + chunk_data.len());
            payload.extend_from_slice(&(chunk_id as u16).to_be_bytes());
            payload.extend_from_slice(chunk_data);
            
            if let Err(e) = client.publish(&chunk_topic, QoS::AtLeastOnce, false, payload).await {
                error!(?e, chunk_id, "Failed to publish chunk");
                return;
            }
            
            if chunk_id % 10 == 0 {
                info!(chunk_id, total_chunks, "OTA chunk progress");
            }
            
            // Small delay between chunks to prevent flooding
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        info!("All OTA chunks sent");
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // 3. Send OTA COMPLETE message
        let complete_topic = format!("enose/{}/ota/complete", device_id);
        let complete_message = serde_json::json!({
            "version": version,
            "checksum": checksum,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        if let Err(e) = client.publish(
            &complete_topic,
            QoS::AtLeastOnce,
            false,
            serde_json::to_vec(&complete_message).unwrap_or_default()
        ).await {
            error!(?e, "Failed to publish OTA COMPLETE");
            return;
        }
        
        info!(%complete_topic, "OTA COMPLETE sent - firmware published via MQTT!");
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Firmware uploaded and OTA started via MQTT",
        "version": response_version,
        "size": response_size,
        "checksum": response_checksum,
        "mqtt_topics": {
            "start": format!("enose/{}/ota/start", env("DEVICE_ID", "ESP32-001")),
            "chunk": format!("enose/{}/ota/chunk", env("DEVICE_ID", "ESP32-001")),
            "complete": format!("enose/{}/ota/complete", env("DEVICE_ID", "ESP32-001")),
        }
    })))
}te", env("DEVICE_ID", "ESP32-001")),
        }
    })))
}

// OTA Firmware Download Handler
async fn download_firmware(
    AxumPath(version): AxumPath<String>,
) -> Result<Response, ApiError> {
    let filename = format!("firmware_{}.bin", version);
    let filepath = std::path::Path::new("./firmware").join(&filename);

    if !filepath.exists() {
        return Err(ApiError::NotFound);
    }

    let data = tokio::fs::read(&filepath).await.map_err(|e| {
        ApiError::Internal(e.into())
    })?;

    info!(version = %version, size = data.len(), "Firmware downloaded");

    let mut response = data.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename={}", filename))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );

    Ok(response)
}
