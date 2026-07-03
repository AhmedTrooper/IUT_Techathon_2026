use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use tracing::error;
use chrono::{DateTime, Utc};
use std::time::Duration;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use sqlx::FromRow;
use redis::AsyncCommands;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub download_url: String,
}

#[derive(Debug, FromRow)]
struct HistoryRow {
    timestamp: DateTime<Utc>,
    device_id: String,
    name: String,
    room: String,
    status: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/reports/export", post(export_report))
}

async fn export_report(
    State(state): State<AppState>,
) -> Result<Json<ExportResponse>, StatusCode> {
    let mut cached_url: Option<String> = None;
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(val) = con.get::<_, String>("cached_export_url").await {
            cached_url = Some(val);
        }
    }

    if let Some(url) = cached_url {
        return Ok(Json(ExportResponse { download_url: url }));
    }

    let rows = sqlx::query_as::<_, HistoryRow>(
        r#"
        SELECT h.timestamp, h.device_id, d.name, d.room, h.status 
        FROM device_history h
        JOIN devices d ON h.device_id = d.id
        ORDER BY h.timestamp DESC
        LIMIT 1000
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!("Error fetching history for report: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut csv_content = String::from("Timestamp,Device ID,Device Name,Room,Status\n");
    for row in rows {
        csv_content.push_str(&format!(
            "{},{},{},{},{}\n",
            row.timestamp.to_rfc3339(),
            row.device_id,
            row.name,
            row.room,
            if row.status { "ON" } else { "OFF" }
        ));
    }

    let file_key = format!("reports/power_report_{}.csv", Utc::now().timestamp());

    state.s3_client
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .body(ByteStream::from(csv_content.into_bytes()))
        .content_type("text/csv")
        .send()
        .await
        .map_err(|e| {
            error!("Error uploading report to S3: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let presigned = state.s3_client
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .presigned(PresigningConfig::expires_in(Duration::from_secs(900)).map_err(|e| {
            error!("Error building presigning config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .await
        .map_err(|e| {
            error!("Error generating presigned S3 url: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let download_url = presigned.uri().to_string();
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = con.set_ex("cached_export_url", &download_url, 300).await;
    }

    Ok(Json(ExportResponse {
        download_url,
    }))
}
