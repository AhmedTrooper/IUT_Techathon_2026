use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

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
