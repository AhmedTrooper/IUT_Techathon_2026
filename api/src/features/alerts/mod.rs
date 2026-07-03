use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, FixedOffset, Timelike, Utc};
use serde::Serialize;
use tracing::error;
use std::collections::HashMap;

use crate::AppState;
use crate::features::devices::Device;

#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: String,
}

#[derive(Debug, Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<Alert>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/alerts", get(get_alerts))
}

pub async fn check_alerts(state: &AppState) -> Vec<Alert> {
    let pool = &state.pool;
    
    let devices = match sqlx::query_as::<_, Device>("SELECT * FROM devices").fetch_all(pool).await {
        Ok(devs) => devs,
        Err(e) => {
            error!("Database offline, using memory cache for alerts: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values().cloned().collect()
        }
    };

    let mut alerts = Vec::new();
    let now_utc = Utc::now();
    let dhaka_offset = FixedOffset::east_opt(6 * 3600).unwrap();
    let now_dhaka = now_utc.with_timezone(&dhaka_offset);
    let hour = now_dhaka.hour();
    
    // Check office hours (assume 9 AM to 5 PM is 9 to 16, outside is < 9 or >= 17)
    let outside_office_hours = hour < 9 || hour >= 17;

    let mut room_devices: HashMap<String, Vec<Device>> = HashMap::new();
    for d in devices {
        room_devices.entry(d.room.clone()).or_default().push(d);
    }

    for (room_id, devs) in room_devices {
        let on_devs: Vec<_> = devs.iter().filter(|d| d.status).collect();
        if on_devs.is_empty() {
            continue;
        }

        let room_name = room_id
            .split('_')
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        // Rule 1: A room where all devices have been on for more than 2 hours continuously.
        let mut all_on_and_over_2h = true;
        let mut any_over_2h = false;
        
        for d in &devs {
            if !d.status {
                all_on_and_over_2h = false;
            } else {
                let duration_on = now_utc.signed_duration_since(d.last_changed);
                if duration_on.num_hours() >= 2 {
                    any_over_2h = true;
                } else {
                    all_on_and_over_2h = false;
                }
            }
        }

        if all_on_and_over_2h && !devs.is_empty() {
            alerts.push(Alert {
                id: format!("alert_{}_all_2h_{}", room_id, now_utc.timestamp()),
                timestamp: now_utc,
                message: format!("CRITICAL: ALL devices in {} have been ON for over 2 hours continuously!", room_name),
                level: "CRITICAL".to_string(),
            });
        } else if any_over_2h {
            alerts.push(Alert {
                id: format!("alert_{}_some_2h_{}", room_id, now_utc.timestamp()),
                timestamp: now_utc,
                message: format!("Warning: Some devices in {} have been ON for over 2 hours.", room_name),
                level: "WARNING".to_string(),
            });
        }

        // Rule 2: Devices left on after office hours.
        if outside_office_hours {
            let on_count = on_devs.len();
            let total = devs.len();
            alerts.push(Alert {
                id: format!("alert_{}_after_hours_{}", room_id, now_utc.timestamp()),
                timestamp: now_utc,
                message: format!("{} has {}/{} devices ON outside of office hours ({}).", room_name, on_count, total, now_dhaka.format("%I:%M %p")),
                level: "WARNING".to_string(),
            });
        }
    }

    alerts
}

async fn get_alerts(State(state): State<AppState>) -> Result<Json<AlertsResponse>, StatusCode> {
    let alerts = check_alerts(&state).await;
    Ok(Json(AlertsResponse { alerts }))
}
