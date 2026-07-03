use sqlx::PgPool;
use tracing::info;

pub async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id VARCHAR(100) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            room VARCHAR(100) NOT NULL,
            device_type VARCHAR(100) NOT NULL,
            status BOOLEAN NOT NULL DEFAULT FALSE,
            power_consumption INT NOT NULL,
            last_changed TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS device_history (
            id SERIAL PRIMARY KEY,
            device_id VARCHAR(100) NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
            status BOOLEAN NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM devices")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        info!("Seeding initial 15 devices into the database...");

        let rooms = vec![
            ("drawing_room", "Drawing Room"),
            ("work_room_1", "Work Room 1"),
            ("work_room_2", "Work Room 2"),
        ];

        for (room_id, _room_name) in rooms {
            for fan_num in 1..=2 {
                let id = format!("{}_fan_{}", room_id, fan_num);
                let name = format!("Fan {}", fan_num);
                sqlx::query(
                    r#"
                    INSERT INTO devices (id, name, room, device_type, status, power_consumption)
                    VALUES ($1, $2, $3, 'fan', FALSE, 60)
                    "#,
                )
                .bind(id)
                .bind(name)
                .bind(room_id)
                .execute(pool)
                .await?;
            }

            for light_num in 1..=3 {
                let id = format!("{}_light_{}", room_id, light_num);
                let name = format!("Light {}", light_num);
                sqlx::query(
                    r#"
                    INSERT INTO devices (id, name, room, device_type, status, power_consumption)
                    VALUES ($1, $2, $3, 'light', FALSE, 15)
                    "#,
                )
                .bind(id)
                .bind(name)
                .bind(room_id)
                .execute(pool)
                .await?;
            }
        }
        info!("Seeding completed successfully.");
    }

    Ok(())
}
