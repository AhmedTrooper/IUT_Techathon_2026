use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod features;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting IUT Techathon 2026 backend API...");

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    info!("Connecting to PostgreSQL database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Database connection established. Initializing database schema...");
    if let Err(e) = db::init_db(&pool).await {
        error!("Failed to initialize database: {}", e);
        return Err(e.into());
    }

    info!("Database initialized successfully.");

    features::simulator::start_simulator(pool.clone());

    if let Ok(token) = env::var("DISCORD_TOKEN") {
        features::bot::start_bot(token, pool.clone());
    } else {
        info!("DISCORD_TOKEN environment variable not set, skipping Discord Bot startup.");
    }

    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "API is healthy" }))
        .nest("/api", features::devices::router())
        .nest("/api", features::usage::router())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(pool.clone());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}
