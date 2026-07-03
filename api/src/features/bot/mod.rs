use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use chrono::{Utc, Timelike};
use std::collections::HashSet;
use tokio::sync::Mutex;
use rig::client::ProviderClient;
use rig::client::CompletionClient;
use rig::completion::Prompt;

use crate::features::devices::Device;
use crate::features::usage::calculate_today_kwh;

struct Handler {
    pool: PgPool,
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

        if content.starts_with("!status") {
            let raw_data = match get_raw_status(&self.pool).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting status: {}", e);
                    "Sorry, I encountered an error fetching the status.".to_string()
                }
            };
            let response = humanize_response(&raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        } else if content.starts_with("!room") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            let room_query = if parts.len() > 1 { parts[1] } else { "" };
            
            let raw_data = match get_raw_room_status(&self.pool, room_query).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting room status: {}", e);
                    "Sorry, I encountered an error fetching the room status.".to_string()
                }
            };
            let response = humanize_response(&raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        } else if content.starts_with("!usage") {
            let raw_data = match get_raw_usage(&self.pool).await {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting usage: {}", e);
                    "Sorry, I encountered an error fetching the usage stats.".to_string()
                }
            };
            let response = humanize_response(&raw_data).await;
            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                error!("Error sending message: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Discord Bot {} is connected!", ready.user.name);

        if let Some(channel_id) = self.alert_channel_id {
            let pool = self.pool.clone();
            let http = ctx.http.clone();
            let sent_alerts = self.sent_alerts.clone();

            tokio::spawn(async move {
                info!("Starting proactive Discord alert task for channel {}...", channel_id);
                loop {
                    sleep(Duration::from_secs(60)).await;
                    if let Err(e) = check_and_send_alerts(&pool, ChannelId::new(channel_id), &http, &sent_alerts).await {
                        error!("Error checking alerts: {}", e);
                    }
                }
            });
        }
    }
}


async fn get_raw_status(pool: &PgPool) -> Result<String, sqlx::Error> {
    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await?;

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

    let drawing_status = if drawing_fans == 0 && drawing_lights == 0 {
        "all off".to_string()
    } else {
        format!("{} fan ON, {} light ON", drawing_fans, drawing_lights)
    };

    let work1_status = if work1_fans == 0 && work1_lights == 0 {
        "all off".to_string()
    } else {
        format!("{} fan ON, {} light ON", work1_fans, work1_lights)
    };

    let work2_status = if work2_fans == 0 && work2_lights == 0 {
        "all off".to_string()
    } else {
        format!("{} fan ON, {} light ON", work2_fans, work2_lights)
    };

    Ok(format!(
        "Drawing Room: {}.\nWork Room 1: {}.\nWork Room 2: {}.",
        drawing_status, work1_status, work2_status
    ))
}

async fn get_raw_room_status(pool: &PgPool, query: &str) -> Result<String, sqlx::Error> {
    let room_id = match query.to_lowercase().replace(" ", "").replace("_", "").as_str() {
        "drawing" | "drawingroom" => "drawing_room",
        "work1" | "workroom1" | "wr1" => "work_room_1",
        "work2" | "workroom2" | "wr2" => "work_room_2",
        _ => {
            return Ok("Invalid room name. Please specify drawing, work1, or work2.".to_string());
        }
    };

    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE room = $1")
        .bind(room_id)
        .fetch_all(pool)
        .await?;

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

async fn get_raw_usage(pool: &PgPool) -> Result<String, sqlx::Error> {
    let devices = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await?;

    let mut total_current_watts = 0;
    for d in &devices {
        if d.status {
            total_current_watts += d.power_consumption;
        }
    }

    let today_kwh = calculate_today_kwh(pool, &devices).await?;

    Ok(format!(
        "Total power right now: {}W. Today's estimated usage: {:.2} kWh.",
        total_current_watts, today_kwh
    ))
}

async fn check_and_send_alerts(
    pool: &PgPool,
    channel_id: ChannelId,
    http: &Arc<serenity::http::Http>,
    sent_alerts: &Arc<Mutex<HashSet<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
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
                
                let humanized = humanize_response(&alert_msg).await;
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
                        
                        let humanized = humanize_response(&alert_msg).await;
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

async fn humanize_response(raw_data: &str) -> String {
    if env::var("GEMINI_API_KEY").is_ok() {
        if let Ok(client) = rig::providers::gemini::Client::from_env() {
            let agent = client
                .agent("gemini-1.5-flash")
                .preamble("You are a friendly office assistant. Translate the raw office device status/usage data into a warm, natural, and friendly message for the boss. Keep it concise, friendly, and structured. Avoid robotic data dumps.")
                .build();
            if let Ok(resp) = agent.prompt(raw_data).await {
                return resp;
            }
        }
    } else if env::var("OPENAI_API_KEY").is_ok() {
        if let Ok(client) = rig::providers::openai::Client::from_env() {
            let agent = client
                .agent("gpt-4o-mini")
                .preamble("You are a friendly office assistant. Translate the raw office device status/usage data into a warm, natural, and friendly message for the boss. Keep it concise, friendly, and structured. Avoid robotic data dumps.")
                .build();
            if let Ok(resp) = agent.prompt(raw_data).await {
                return resp;
            }
        }
    }
    
    format!("Here is the status summary:\n\n{}", raw_data)
}

pub fn start_bot(token: String, pool: PgPool) {
    tokio::spawn(async move {
        info!("Starting Discord Bot gateway listener...");

        let alert_channel_id = env::var("DISCORD_ALERT_CHANNEL_ID")
            .ok()
            .and_then(|val| val.parse::<u64>().ok());

        let handler = Handler {
            pool,
            alert_channel_id,
            sent_alerts: Arc::new(Mutex::new(HashSet::new())),
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

        if let Err(e) = client.start().await {
            error!("Serenity client error: {}", e);
        }
    });
}
