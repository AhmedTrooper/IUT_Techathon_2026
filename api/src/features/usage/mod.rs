use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{Utc, TimeZone, Datelike, FixedOffset};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::error;
use redis::AsyncCommands;

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

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/usage", get(get_usage))
}

async fn get_usage(
    State(state): State<AppState>,
) -> Result<Json<UsageResponse>, StatusCode> {
    let pool = &state.pool;
    let devices_result = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await;

    let devices = match devices_result {
        Ok(devs) => {
            let mut cache = state.memory_devices.write().await;
            for d in &devs {
                cache.insert(d.id.clone(), d.clone());
            }
            devs
        }
        Err(e) => {
            error!("Database offline, using in-memory devices for usage calculation: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values().cloned().collect()
        }
    };

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

    let mut cached_kwh: Option<f64> = None;
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(val) = con.get::<_, String>("today_kwh").await {
            if let Ok(kwh) = val.parse::<f64>() {
                cached_kwh = Some(kwh);
            }
        }
    }

    let today_kwh = match cached_kwh {
        Some(kwh) => kwh,
        None => {
            let kwh = match calculate_today_kwh(pool, &devices).await {
                Ok(val) => val,
                Err(e) => {
                    error!("Database error calculating today's kWh: {}. Falling back to in-memory history.", e);
                    let history = state.memory_history.read().await;
                    calculate_today_kwh_in_memory(&devices, &history).await
                }
            };
            if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
                let _: Result<(), _> = con.set_ex("today_kwh", kwh.to_string(), 3).await;
            }
            kwh
        }
    };

    Ok(Json(UsageResponse {
        total_current_watts,
        room_breakdown,
        today_kwh,
    }))
}

pub async fn calculate_today_kwh(pool: &PgPool, devices: &[Device]) -> Result<f64, sqlx::Error> {
    let now = Utc::now();
    let dhaka_offset = FixedOffset::east_opt(6 * 3600).unwrap();
    let now_dhaka = now.with_timezone(&dhaka_offset);

    let today_start_dhaka = dhaka_offset.with_ymd_and_hms(
        now_dhaka.year(),
        now_dhaka.month(),
        now_dhaka.day(),
        0, 0, 0
    )
    .single()
    .unwrap_or(now_dhaka);

    let today_start_utc = today_start_dhaka.with_timezone(&Utc);
    let mut total_watt_seconds = 0.0;

    for device in devices {
        let history = sqlx::query_as::<_, DeviceHistory>(
            r#"
            SELECT * FROM device_history 
            WHERE device_id = $1 AND timestamp >= $2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(&device.id)
        .bind(today_start_utc)
        .fetch_all(pool)
        .await?;

        let last_before: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT status FROM device_history 
            WHERE device_id = $1 AND timestamp < $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(&device.id)
        .bind(today_start_utc)
        .fetch_optional(pool)
        .await?;

        let mut current_state = last_before.map(|(status,)| status).unwrap_or(false);
        let mut current_time = today_start_utc;

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

    Ok(total_watt_seconds / 3_600_000.0)
}

pub async fn calculate_today_kwh_in_memory(
    devices: &[Device],
    history_list: &[DeviceHistory],
) -> f64 {
    let now = Utc::now();
    let dhaka_offset = FixedOffset::east_opt(6 * 3600).unwrap();
    let now_dhaka = now.with_timezone(&dhaka_offset);

    let today_start_dhaka = dhaka_offset.with_ymd_and_hms(
        now_dhaka.year(),
        now_dhaka.month(),
        now_dhaka.day(),
        0, 0, 0
    )
    .single()
    .unwrap_or(now_dhaka);

    let today_start_utc = today_start_dhaka.with_timezone(&Utc);
    let mut total_watt_seconds = 0.0;

    for device in devices {
        let mut history: Vec<&DeviceHistory> = history_list.iter()
            .filter(|h| h.device_id == device.id && h.timestamp >= today_start_utc)
            .collect();
        history.sort_by_key(|h| h.timestamp);

        let last_before = history_list.iter()
            .filter(|h| h.device_id == device.id && h.timestamp < today_start_utc)
            .max_by_key(|h| h.timestamp)
            .map(|h| h.status)
            .unwrap_or(false);

        let mut current_state = last_before;
        let mut current_time = today_start_utc;

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

    total_watt_seconds / 3_600_000.0
}
