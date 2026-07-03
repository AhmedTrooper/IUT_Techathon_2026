use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

pub fn start_simulator(pool: PgPool) {
    tokio::spawn(async move {
        info!("Starting background device simulator...");
        loop {
            let seconds = rand::random::<u64>() % 5 + 3;
            sleep(Duration::from_secs(seconds)).await;

            if let Err(e) = toggle_random_device(&pool).await {
                error!("Simulator error: {}", e);
            }
        }
    });
}

async fn toggle_random_device(pool: &PgPool) -> Result<(), sqlx::Error> {
    let devices: Vec<String> = sqlx::query_scalar("SELECT id FROM devices")
        .fetch_all(pool)
        .await?;

    if devices.is_empty() {
        return Ok(());
    }

    let random_index = rand::random::<usize>() % devices.len();
    let device_id = &devices[random_index];

    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        UPDATE devices 
        SET status = NOT status, last_changed = NOW()
        WHERE id = $1
        RETURNING status
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    if let Some((new_status,)) = row {
        sqlx::query(
            r#"
            INSERT INTO device_history (device_id, status, timestamp)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(device_id)
        .bind(new_status)
        .execute(pool)
        .await?;

        info!(
            "Simulator toggled device '{}' to {}",
            device_id,
            if new_status { "ON" } else { "OFF" }
        );
    }

    Ok(())
}
