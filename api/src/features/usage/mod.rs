use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{Utc, TimeZone, Datelike};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::error;

use crate::features::devices::{Device, DeviceHistory};

#[derive(Debug, Serialize)]
pub struct RoomBreakdown {
    pub room: String,
    pub current_watts: i32,
}

#[derive(Debug, Serialize)]
pub struct UsageResponse {
    pub total_current_watts: i32,
    pub room_breakdown: Vec<RoomBreakdown>,
    pub today_kwh: f64,
}

pub fn router() -> Router<PgPool> {
    Router::new().route("/usage", get(get_usage))
}

// GET /api/usage
async fn get_usage(
    State(pool): State<PgPool>,
) -> Result<Json<UsageResponse>, StatusCode> {
    // 1. Fetch all devices to compute current usage
    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            error!("Database error fetching devices for usage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut total_current_watts = 0;
    let mut room_watts_map: HashMap<String, i32> = HashMap::new();

    for d in &devices {
        let watts = if d.status { d.power_consumption } else { 0 };
        total_current_watts += watts;
        *room_watts_map.entry(d.room.clone()).or_insert(0) += watts;
    }

    let room_breakdown = room_watts_map
        .into_iter()
        .map(|(room, current_watts)| RoomBreakdown { room, current_watts })
        .collect::<Vec<_>>();

    // 2. Compute today's estimated usage in kWh
    let today_kwh = calculate_today_kwh(&pool, &devices).await.map_err(|e| {
        error!("Error calculating today's kWh: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(UsageResponse {
        total_current_watts,
        room_breakdown,
        today_kwh,
    }))
}

pub async fn calculate_today_kwh(pool: &PgPool, devices: &[Device]) -> Result<f64, sqlx::Error> {
    let now = Utc::now();
    // Start of today: 00:00:00 UTC
    let today_start = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .unwrap_or(now);

    let mut total_watt_seconds = 0.0;

    for device in devices {
        // Fetch all transitions for this device today
        let history = sqlx::query_as::<_, DeviceHistory>(
            r#"
            SELECT * FROM device_history 
            WHERE device_id = $1 AND timestamp >= $2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(&device.id)
        .bind(today_start)
        .fetch_all(pool)
        .await?;

        // Determine initial state at 00:00:00 today.
        let last_before: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT status FROM device_history 
            WHERE device_id = $1 AND timestamp < $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(&device.id)
        .bind(today_start)
        .fetch_optional(pool)
        .await?;

        let mut current_state = last_before.map(|(status,)| status).unwrap_or(false);
        let mut current_time = today_start;

        for event in history {
            if current_state {
                let duration_secs = (event.timestamp - current_time).num_seconds() as f64;
                total_watt_seconds += duration_secs * (device.power_consumption as f64);
            }
            current_state = event.status;
            current_time = event.timestamp;
        }

        if current_state {
            let duration_secs = (now - current_time).num_seconds() as f64;
            total_watt_seconds += duration_secs * (device.power_consumption as f64);
        }
    }

    let total_kwh = total_watt_seconds / 3_600_000.0;
    Ok(total_kwh)
}
