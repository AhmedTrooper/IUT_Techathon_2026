use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub room: String,
    pub device_type: String,
    pub status: bool,
    pub power_consumption: i32, // in Watts
    pub last_changed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceHistory {
    pub id: i32,
    pub device_id: String,
    pub status: bool,
    pub timestamp: DateTime<Utc>,
}

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices", get(get_devices))
        .route("/devices/:id/toggle", post(toggle_device))
}

async fn get_devices(
    State(state): State<AppState>,
) -> Result<Json<Vec<Device>>, StatusCode> {
    let pool = &state.pool;
    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices ORDER BY room, name")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!("Error fetching devices: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(devices))
}

async fn toggle_device(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Device>, StatusCode> {
    let pool = &state.pool;
    let current: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "SELECT last_changed FROM devices WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("Error fetching last changed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some((last,)) = current {
        if Utc::now().signed_duration_since(last).num_milliseconds() < 500 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    let row: Option<Device> = sqlx::query_as(
        r#"
        UPDATE devices 
        SET status = NOT status, last_changed = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("Error updating device status {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(device) => {
            let _ = sqlx::query(
                r#"
                INSERT INTO device_history (device_id, status, timestamp)
                VALUES ($1, $2, NOW())
                "#,
            )
            .bind(&device.id)
            .bind(device.status)
            .execute(pool)
            .await;

            info!("Device '{}' toggled to {}", device.id, if device.status { "ON" } else { "OFF" });
            Ok(Json(device))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
