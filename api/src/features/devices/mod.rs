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

use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    DrawingRoom,
    WorkRoom1,
    WorkRoom2,
}

#[allow(dead_code)]
impl Room {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DrawingRoom => "drawing_room",
            Self::WorkRoom1 => "work_room_1",
            Self::WorkRoom2 => "work_room_2",
        }
    }
}

impl FromStr for Room {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "drawing_room" => Ok(Self::DrawingRoom),
            "work_room_1" => Ok(Self::WorkRoom1),
            "work_room_2" => Ok(Self::WorkRoom2),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Fan,
    Light,
}

#[allow(dead_code)]
impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fan => "fan",
            Self::Light => "light",
        }
    }
}

impl FromStr for DeviceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fan" => Ok(Self::Fan),
            "light" => Ok(Self::Light),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceId {
    DrawingRoomFan1,
    DrawingRoomFan2,
    DrawingRoomLight1,
    DrawingRoomLight2,
    DrawingRoomLight3,
    WorkRoom1Fan1,
    WorkRoom1Fan2,
    WorkRoom1Light1,
    WorkRoom1Light2,
    WorkRoom1Light3,
    WorkRoom2Fan1,
    WorkRoom2Fan2,
    WorkRoom2Light1,
    WorkRoom2Light2,
    WorkRoom2Light3,
}

impl DeviceId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DrawingRoomFan1 => "drawing_room_fan_1",
            Self::DrawingRoomFan2 => "drawing_room_fan_2",
            Self::DrawingRoomLight1 => "drawing_room_light_1",
            Self::DrawingRoomLight2 => "drawing_room_light_2",
            Self::DrawingRoomLight3 => "drawing_room_light_3",
            Self::WorkRoom1Fan1 => "work_room_1_fan_1",
            Self::WorkRoom1Fan2 => "work_room_1_fan_2",
            Self::WorkRoom1Light1 => "work_room_1_light_1",
            Self::WorkRoom1Light2 => "work_room_1_light_2",
            Self::WorkRoom1Light3 => "work_room_1_light_3",
            Self::WorkRoom2Fan1 => "work_room_2_fan_1",
            Self::WorkRoom2Fan2 => "work_room_2_fan_2",
            Self::WorkRoom2Light1 => "work_room_2_light_1",
            Self::WorkRoom2Light2 => "work_room_2_light_2",
            Self::WorkRoom2Light3 => "work_room_2_light_3",
        }
    }
}

impl FromStr for DeviceId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "drawing_room_fan_1" => Ok(Self::DrawingRoomFan1),
            "drawing_room_fan_2" => Ok(Self::DrawingRoomFan2),
            "drawing_room_light_1" => Ok(Self::DrawingRoomLight1),
            "drawing_room_light_2" => Ok(Self::DrawingRoomLight2),
            "drawing_room_light_3" => Ok(Self::DrawingRoomLight3),
            "work_room_1_fan_1" => Ok(Self::WorkRoom1Fan1),
            "work_room_1_fan_2" => Ok(Self::WorkRoom1Fan2),
            "work_room_1_light_1" => Ok(Self::WorkRoom1Light1),
            "work_room_1_light_2" => Ok(Self::WorkRoom1Light2),
            "work_room_1_light_3" => Ok(Self::WorkRoom1Light3),
            "work_room_2_fan_1" => Ok(Self::WorkRoom2Fan1),
            "work_room_2_fan_2" => Ok(Self::WorkRoom2Fan2),
            "work_room_2_light_1" => Ok(Self::WorkRoom2Light1),
            "work_room_2_light_2" => Ok(Self::WorkRoom2Light2),
            "work_room_2_light_3" => Ok(Self::WorkRoom2Light3),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub room: String,
    pub device_type: String,
    pub status: bool,
    pub power_consumption: i32,
    pub last_changed: DateTime<Utc>,
}

#[allow(dead_code)]
impl Device {
    pub fn device_id(&self) -> Result<DeviceId, ()> {
        self.id.parse()
    }
    pub fn room_enum(&self) -> Result<Room, ()> {
        self.room.parse()
    }
    pub fn type_enum(&self) -> Result<DeviceType, ()> {
        self.device_type.parse()
    }
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
    match sqlx::query_as::<_, Device>("SELECT * FROM devices ORDER BY room, name")
        .fetch_all(pool)
        .await
    {
        Ok(devices) => {
            let mut cache = state.memory_devices.write().await;
            for d in &devices {
                cache.insert(d.id.clone(), d.clone());
            }
            Ok(Json(devices))
        }
        Err(e) => {
            error!("Database offline, fetching devices from memory cache: {}", e);
            let cache = state.memory_devices.read().await;
            let mut list: Vec<Device> = cache.values().cloned().collect();
            list.sort_by(|a, b| (&a.room, &a.name).cmp(&(&b.room, &b.name)));
            Ok(Json(list))
        }
    }
}

async fn toggle_device(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Device>, StatusCode> {
    let device_id = match DeviceId::from_str(&id) {
        Ok(d) => d,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let db_result = async {
        let mut tx = state.pool.begin().await.map_err(|e| {
            error!("Fail-safe: transaction begin failed: {}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

        let device: Option<Device> = sqlx::query_as(
            "SELECT * FROM devices WHERE id = $1 FOR UPDATE"
        )
        .bind(device_id.as_str())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

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
        .bind(device_id.as_str())
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

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

        tx.commit().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        Ok(updated_device)
    }.await;

    match db_result {
        Ok(updated_device) => {
            let mut cache = state.memory_devices.write().await;
            cache.insert(updated_device.id.clone(), updated_device.clone());
            info!("Device '{}' toggled via Database.", updated_device.id);
            Ok(Json(updated_device))
        }
        Err(err) => {
            if err == StatusCode::NOT_FOUND || err == StatusCode::TOO_MANY_REQUESTS {
                return Err(err);
            }
            
            error!("Database toggle failed: {:?}. Using in-memory fallback.", err);

            let mut cache = state.memory_devices.write().await;
            let device = cache.get_mut(device_id.as_str()).ok_or(StatusCode::NOT_FOUND)?;

            if Utc::now().signed_duration_since(device.last_changed).num_milliseconds() < 500 {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }

            device.status = !device.status;
            device.last_changed = Utc::now();
            let updated = device.clone();

            let mut history = state.memory_history.write().await;
            let next_id = history.len() as i32 + 1;
            history.push(DeviceHistory {
                id: next_id,
                device_id: updated.id.clone(),
                status: updated.status,
                timestamp: updated.last_changed,
            });

            info!("Device '{}' toggled via In-Memory.", updated.id);
            Ok(Json(updated))
        }
    }
}
