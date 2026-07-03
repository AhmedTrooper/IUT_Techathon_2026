use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};

use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use chrono::{Utc, Timelike, DateTime};
use std::collections::HashSet;
use tokio::sync::Mutex;
use rig::client::ProviderClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use redis::AsyncCommands;

use crate::features::devices::Device;
use crate::features::usage::calculate_today_kwh;
use crate::AppState;

struct Handler {
    state: AppState,
    alert_channel_id: Option<u64>,
    sent_alerts: Arc<Mutex<HashSet<String>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim();

        if content.starts_with("!") {
            let author_id = msg.author.id.get();
            if let Ok(mut con) = self.state.redis_client.get_multiplexed_async_connection().await {
                let redis_key = format!("discord_ratelimit:{}", author_id);
                if let Ok(count) = con.incr::<_, _, i32>(&redis_key, 1).await {
                    if count == 1 {
                        let _: Result<(), _> = con.expire(&redis_key, 10).await;
                    }
                    if count > 3 {
                        let _ = msg.channel_id.say(&ctx.http, "⚠️ **Slow down!** You are sending commands too quickly.").await;
                        return;
                    }
                }
            }
        }

        if content.starts_with("!status") {
            let raw_data = match get_raw_status(&self.state).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting status: {}", e);
                    "Sorry, I encountered an error fetching the status.".to_string()
                }
            };
            let response = humanize_response(&self.state, &raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        } else if content.starts_with("!room") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            let room_query = if parts.len() > 1 { parts[1] } else { "" };
            
            let raw_data = match get_raw_room_status(&self.state, room_query).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting room status: {}", e);
                    "Sorry, I encountered an error fetching the room status.".to_string()
                }
            };
            let response = humanize_response(&self.state, &raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        } else if content.starts_with("!usage") {
            let raw_data = match get_raw_usage(&self.state).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting usage: {}", e);
                    "Sorry, I encountered an error fetching the usage stats.".to_string()
                }
            };
            let response = humanize_response(&self.state, &raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        } else if content.starts_with("!export") {
            let response = match handle_export(&self.state).await {
                Ok(url) => format!("Here is your exported power history report (valid for 15 minutes):\n{}", url),
                Err(e) => {
                    error!("Error generating export: {}", e);
                    "Sorry, I encountered an error generating your report.".to_string()
                }
            };
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Discord Bot {} is connected!", ready.user.name);

        if let Some(channel_id) = self.alert_channel_id {
            let state = self.state.clone();
            let http = ctx.http.clone();
            let sent_alerts = self.sent_alerts.clone();

            tokio::spawn(async move {
                info!("Starting proactive Discord alert task for channel {}...", channel_id);
                loop {
                    sleep(Duration::from_secs(60)).await;
                    if let Err(e) = check_and_send_alerts(&state, ChannelId::new(channel_id), &http, &sent_alerts).await {
                        error!("Error checking alerts: {}", e);
                    }
                }
            });
        }
    }
}


fn format_room_device_counts(fans: i32, lights: i32) -> String {
    if fans == 0 && lights == 0 {
        return "all off".to_string();
    }
    let mut parts = Vec::new();
    if fans > 0 {
        parts.push(format!("{} fan{}", fans, if fans == 1 { " ON" } else { "s ON" }));
    }
    if lights > 0 {
        parts.push(format!("{} light{}", lights, if lights == 1 { " ON" } else { "s ON" }));
    }
    parts.join(", ")
}

async fn get_raw_status(state: &AppState) -> Result<String, sqlx::Error> {
    let pool = &state.pool;
    let devices_result = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await;

    let devices = match devices_result {
        Ok(devs) => devs,
        Err(e) => {
            error!("Database offline, bot using in-memory cache for status: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values().cloned().collect()
        }
    };

    let mut drawing_fans = 0;
    let mut drawing_lights = 0;
    let mut work1_fans = 0;
    let mut work1_lights = 0;
    let mut work2_fans = 0;
    let mut work2_lights = 0;

    for d in devices {
        if d.status {
            match d.room.as_str() {
                "drawing_room" => {
                    if d.device_type == "fan" { drawing_fans += 1; } else { drawing_lights += 1; }
                }
                "work_room_1" => {
                    if d.device_type == "fan" { work1_fans += 1; } else { work1_lights += 1; }
                }
                "work_room_2" => {
                    if d.device_type == "fan" { work2_fans += 1; } else { work2_lights += 1; }
                }
                _ => {}
            }
        }
    }

    let drawing_status = format_room_device_counts(drawing_fans, drawing_lights);
    let work1_status = format_room_device_counts(work1_fans, work1_lights);
    let work2_status = format_room_device_counts(work2_fans, work2_lights);

    Ok(format!(
        "Drawing Room: {}. Work Room 1: {}. Work Room 2: {}.",
        drawing_status, work1_status, work2_status
    ))
}

async fn get_raw_room_status(state: &AppState, query: &str) -> Result<String, sqlx::Error> {
    let room_id = match query.to_lowercase().replace(" ", "").replace("_", "").as_str() {
        "drawing" | "drawingroom" => "drawing_room",
        "work1" | "workroom1" | "wr1" => "work_room_1",
        "work2" | "workroom2" | "wr2" => "work_room_2",
        _ => {
            return Ok("Invalid room name. Please specify drawing, work1, or work2.".to_string());
        }
    };

    let pool = &state.pool;
    let devices_result = sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE room = $1")
        .bind(room_id)
        .fetch_all(pool)
        .await;

    let devices = match devices_result {
        Ok(devs) => devs,
        Err(e) => {
            error!("Database offline, bot using in-memory cache for room status: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values()
                .filter(|d| d.room == room_id)
                .cloned()
                .collect()
        }
    };

    let room_name = match room_id {
        "drawing_room" => "Drawing Room",
        "work_room_1" => "Work Room 1",
        "work_room_2" => "Work Room 2",
        _ => "Unknown",
    };

    let mut response = format!("Status for {}:\n", room_name);
    for d in devices {
        response.push_str(&format!("- {}: {}\n", d.name, if d.status { "ON" } else { "OFF" }));
    }

    Ok(response)
}

async fn get_raw_usage(state: &AppState) -> Result<String, sqlx::Error> {
    let pool = &state.pool;
    let devices_result = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await;

    let devices = match devices_result {
        Ok(devs) => devs,
        Err(e) => {
            error!("Database offline, bot using in-memory cache for usage: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values().cloned().collect()
        }
    };

    let mut total_current_watts = 0;
    for d in &devices {
        if d.status {
            total_current_watts += d.power_consumption;
        }
    }

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
                    crate::features::usage::calculate_today_kwh_in_memory(&devices, &history).await
                }
            };
            if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
                let _: Result<(), _> = con.set_ex("today_kwh", kwh.to_string(), 3).await;
            }
            kwh
        }
    };

    Ok(format!(
        "Total power right now: {}W. Today's estimated usage: {:.2} kWh.",
        total_current_watts, today_kwh
    ))
}

async fn check_and_send_alerts(
    state: &AppState,
    channel_id: ChannelId,
    http: &Arc<serenity::http::Http>,
    sent_alerts: &Arc<Mutex<HashSet<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = &state.pool;
    let now = Utc::now();
    let dhaka_offset = chrono::FixedOffset::east_opt(6 * 3600).unwrap();
    let now_dhaka = now.with_timezone(&dhaka_offset);

    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await?;

    let mut active_alert_keys = HashSet::new();

    let hour = now_dhaka.hour();
    let is_after_hours = hour < 9 || hour >= 17;

    if is_after_hours {
        let active_devices: Vec<String> = devices
            .iter()
            .filter(|d| d.status)
            .map(|d| format!("{} in {}", d.name, d.room.replace("_", " ")))
            .collect();

        if !active_devices.is_empty() {
            let alert_key = "after_hours_alert".to_string();
            active_alert_keys.insert(alert_key.clone());

            let mut guard = sent_alerts.lock().await;
            if !guard.contains(&alert_key) {
                let list = active_devices.join(", ");
                let alert_msg = format!(
                    "⚠️ **After-Hours Power Alert!**\nThe following devices are still active at {:02}:{:02} Dhaka time:\n{}\nDid someone forget to turn them off?",
                    hour, now_dhaka.minute(), list
                );
                
                let humanized = humanize_response(state, &alert_msg).await;
                channel_id.say(http, humanized).await?;
                guard.insert(alert_key);
            }
        }
    }

    let rooms = vec!["drawing_room", "work_room_1", "work_room_2"];
    for room in rooms {
        let room_devices: Vec<&Device> = devices.iter().filter(|d| d.room == room).collect();
        let all_on = !room_devices.is_empty() && room_devices.iter().all(|d| d.status);

        if all_on {
            if let Some(min_last_changed) = room_devices.iter().map(|d| d.last_changed).min() {
                let duration = now.signed_duration_since(min_last_changed);
                if duration.num_hours() >= 2 {
                    let alert_key = format!("all_on_2h_{}", room);
                    active_alert_keys.insert(alert_key.clone());

                    let mut guard = sent_alerts.lock().await;
                    if !guard.contains(&alert_key) {
                        let min_dhaka = min_last_changed.with_timezone(&dhaka_offset);
                        let alert_msg = format!(
                            "⚠️ **Efficiency Alert!**\nAll devices in {} have been running continuously for over 2 hours (since {}).",
                            room.replace("_", " "), min_dhaka.format("%H:%M Dhaka time")
                        );
                        
                        let humanized = humanize_response(state, &alert_msg).await;
                        channel_id.say(http, humanized).await?;
                        guard.insert(alert_key);
                    }
                }
            }
        }
    }

    let mut guard = sent_alerts.lock().await;
    guard.retain(|key| active_alert_keys.contains(key));

    Ok(())
}

async fn humanize_response(state: &AppState, raw_data: &str) -> String {
    let redis_key = format!("llm_cache:{}", raw_data);

    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = con.get::<_, String>(&redis_key).await {
            return cached;
        }
    }

    let response = if env::var("GEMINI_API_KEY").is_ok() {
        if let Ok(client) = rig::providers::gemini::Client::from_env() {
            let agent = client
                .agent("gemini-1.5-flash")
                .preamble("You are a friendly office assistant. Translate the raw office device status/usage data into a warm, natural, and friendly message for the boss. Keep it concise, friendly, and structured. Avoid robotic data dumps.")
                .build();
            if let Ok(resp) = agent.prompt(raw_data).await {
                resp
            } else {
                format!("Here is the status summary:\n\n{}", raw_data)
            }
        } else {
            format!("Here is the status summary:\n\n{}", raw_data)
        }
    } else if env::var("OPENAI_API_KEY").is_ok() {
        if let Ok(client) = rig::providers::openai::Client::from_env() {
            let agent = client
                .agent("gpt-4o-mini")
                .preamble("You are a friendly office assistant. Translate the raw office device status/usage data into a warm, natural, and friendly message for the boss. Keep it concise, friendly, and structured. Avoid robotic data dumps.")
                .build();
            if let Ok(resp) = agent.prompt(raw_data).await {
                resp
            } else {
                format!("Here is the status summary:\n\n{}", raw_data)
            }
        } else {
            format!("Here is the status summary:\n\n{}", raw_data)
        }
    } else {
        format!("Here is the status summary:\n\n{}", raw_data)
    };

    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = con.set_ex(&redis_key, &response, 3600).await;
    }

    response
}

#[derive(Debug, sqlx::FromRow)]
struct ExportHistoryRow {
    timestamp: DateTime<Utc>,
    device_id: String,
    name: String,
    room: String,
    status: bool,
}

async fn handle_export(state: &AppState) -> Result<String, Box<dyn std::error::Error>> {
    let mut cached_url: Option<String> = None;
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(val) = con.get::<_, String>("cached_export_url").await {
            cached_url = Some(val);
        }
    }

    if let Some(url) = cached_url {
        return Ok(url);
    }

    let rows = sqlx::query_as::<_, ExportHistoryRow>(
        r#"
        SELECT h.timestamp, h.device_id, d.name, d.room, h.status 
        FROM device_history h
        JOIN devices d ON h.device_id = d.id
        ORDER BY h.timestamp DESC
        LIMIT 1000
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

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
        .body(aws_sdk_s3::primitives::ByteStream::from(csv_content.into_bytes()))
        .content_type("text/csv")
        .send()
        .await?;

    let presigned = state.s3_client
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .presigned(aws_sdk_s3::presigning::PresigningConfig::expires_in(Duration::from_secs(900))?)
        .await?;

    let download_url = presigned.uri().to_string();
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = con.set_ex("cached_export_url", &download_url, 300).await;
    }

    Ok(download_url)
}

pub fn start_bot(token: String, state: AppState) {
    tokio::spawn(async move {
        info!("Starting Discord Bot gateway listener...");

        let alert_channel_id = env::var("DISCORD_ALERT_CHANNEL_ID")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        let sent_alerts = Arc::new(Mutex::new(HashSet::new()));

        let handler = Handler {
            state: state.clone(),
            alert_channel_id,
            sent_alerts: sent_alerts.clone(),
        };

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = match Client::builder(&token, intents)
            .event_handler(handler)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                error!("Error creating serenity client: {}", e);
                return;
            }
        };

        let http = client.http.clone();
        let loop_state = state.clone();
        
        tokio::spawn(async move {
            if let Some(channel_id) = alert_channel_id {
                let channel = ChannelId::new(channel_id);
                loop {
                    sleep(Duration::from_secs(60)).await;
                    let alerts = crate::features::alerts::check_alerts(&loop_state).await;
                    let mut sent = sent_alerts.lock().await;
                    
                    let mut current_ids = HashSet::new();
                    for alert in alerts {
                        current_ids.insert(alert.id.clone());
                        if !sent.contains(&alert.id) {
                            let _ = channel.say(&http, format!("⚠️ **ALERT**: {}", alert.message)).await;
                            sent.insert(alert.id);
                        }
                    }
                    sent.retain(|id| current_ids.contains(id));
                }
            }
        });

        if let Err(e) = client.start().await {
            error!("Serenity client error: {}", e);
        }
    });
}
