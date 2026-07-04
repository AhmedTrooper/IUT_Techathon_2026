use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::time::sleep;
use tracing::{error, info};
use chrono::Utc;

use crate::features::devices::{Device, DeviceHistory};

pub fn start_simulator(
    pool: PgPool,
    memory_devices: Arc<RwLock<HashMap<String, Device>>>,
    memory_history: Arc<RwLock<Vec<DeviceHistory>>>,
) {
    tokio::spawn(async move {
        info!("Starting background device simulator...");
        loop {
            let seconds = rand::random::<u64>() % 30 + 30;
            sleep(Duration::from_secs(seconds)).await;

            if let Err(e) = toggle_random_device(&pool, &memory_devices, &memory_history).await {
                error!("Simulator error: {}", e);
            }
        }
    });
}

async fn toggle_random_device(
    pool: &PgPool,
    memory_devices: &Arc<RwLock<HashMap<String, Device>>>,
    memory_history: &Arc<RwLock<Vec<DeviceHistory>>>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let devices: Vec<String> = sqlx::query_scalar("SELECT id FROM devices")
        .fetch_all(&mut *tx)
        .await?;

    if devices.is_empty() {
        return Ok(());
    }

    let random_index = rand::random::<usize>() % devices.len();
    let device_id = &devices[random_index];

    let status_row: Option<(bool,)> = sqlx::query_as(
        "SELECT status FROM devices WHERE id = $1 FOR UPDATE"
    )
    .bind(device_id)
    .fetch_optional(&mut *tx)
    .await?;

    if status_row.is_some() {
        let updated: (bool,) = sqlx::query_as(
            r#"
            UPDATE devices 
            SET status = NOT status, last_changed = NOW()
            WHERE id = $1
            RETURNING status
            "#,
        )
        .bind(device_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO device_history (device_id, status, timestamp)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(device_id)
        .bind(updated.0)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Sync in-memory cache so fail-safe mode stays current
        let now = Utc::now();
        {
            let mut cache = memory_devices.write().await;
            if let Some(device) = cache.get_mut(device_id) {
                device.status = updated.0;
                device.last_changed = now;
            }
        }
        {
            let mut hist = memory_history.write().await;
            let next_id = hist.len() as i32 + 1;
            hist.push(DeviceHistory {
                id: next_id,
                device_id: device_id.clone(),
                status: updated.0,
                timestamp: now,
            });
        }

        info!(
            "Simulator toggled device '{}' to {}",
            device_id,
            if updated.0 { "ON" } else { "OFF" }
        );
    } else {
        tx.rollback().await?;
    }

    Ok(())
}
