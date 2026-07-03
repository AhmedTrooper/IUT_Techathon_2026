use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod models;
mod simulator;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();

    // Setup logging/tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting IUT Techathon 2026 backend API...");

    // Connect to PostgreSQL database
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

    // Start background device simulator
    simulator::start_simulator(pool.clone());

    // Build Axum router with state and permissive CORS layer
    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "API is healthy" }))
        .route("/api/devices", axum::routing::get(handlers::get_devices))
        .route("/api/devices/:id/toggle", axum::routing::post(handlers::toggle_device))
        .route("/api/usage", axum::routing::get(handlers::get_usage))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(pool.clone());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}
