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
    let is_valid = matches!(id.as_str(),
        "drawing_room_fan_1" | "drawing_room_fan_2" |
        "drawing_room_light_1" | "drawing_room_light_2" | "drawing_room_light_3" |
        "work_room_1_fan_1" | "work_room_1_fan_2" |
        "work_room_1_light_1" | "work_room_1_light_2" | "work_room_1_light_3" |
        "work_room_2_fan_1" | "work_room_2_fan_2" |
        "work_room_2_light_1" | "work_room_2_light_2" | "work_room_2_light_3"
    );

    if !is_valid {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let device: Option<Device> = sqlx::query_as(
        "SELECT * FROM devices WHERE id = $1 FOR UPDATE"
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error fetching device for update: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let device = match device {
        Some(d) => d,
        None => return Err(StatusCode::NOT_FOUND),
    };

    if Utc::now().signed_duration_since(device.last_changed).num_milliseconds() < 500 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let updated_device: Device = sqlx::query_as(
        r#"
        UPDATE devices 
        SET status = NOT status, last_changed = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(&id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Error toggling device: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _ = sqlx::query(
        r#"
        INSERT INTO device_history (device_id, status, timestamp)
        VALUES ($1, $2, NOW())
        "#,
    )
    .bind(&updated_device.id)
    .bind(updated_device.status)
    .execute(&mut *tx)
    .await;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Device '{}' toggled to {}", updated_device.id, if updated_device.status { "ON" } else { "OFF" });
    Ok(Json(updated_device))
}
