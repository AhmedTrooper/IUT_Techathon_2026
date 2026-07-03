use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod features;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_bucket: String,
    pub redis_client: redis::Client,
}

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

    let s3_endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "admin".to_string());
    let s3_secret_key = env::var("S3_SECRET_KEY").unwrap_or_else(|_| "supersecretpassword".to_string());
    let s3_bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "iut-techathon-2026-bucket".to_string());

    let credentials = aws_sdk_s3::config::Credentials::new(
        s3_access_key,
        s3_secret_key,
        None,
        None,
        "static",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(s3_endpoint)
        .credentials_provider(aws_sdk_s3::config::SharedCredentialsProvider::new(credentials))
        .region(aws_config::Region::new("us-east-1"))
        .force_path_style(true)
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    let state = AppState {
        pool: pool.clone(),
        s3_client,
        s3_bucket,
        redis_client,
    };

    features::simulator::start_simulator(pool.clone());

    if let Ok(token) = env::var("DISCORD_TOKEN") {
        features::bot::start_bot(token, state.clone());
    } else {
        info!("DISCORD_TOKEN environment variable not set, skipping Discord Bot startup.");
    }

    let api_routes = axum::Router::new()
        .nest("/", features::devices::router())
        .nest("/", features::usage::router())
        .nest("/", features::reports::router())
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware));

    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "API is healthy" }))
        .nest("/api", api_routes)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    use redis::AsyncCommands;

    let ip = req.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "127.0.0.1".to_string())
        });

    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        let redis_key = format!("ratelimit:{}", ip);
        if let Ok(count) = con.incr::<_, _, i32>(&redis_key, 1).await {
            if count == 1 {
                let _: Result<(), _> = con.expire(&redis_key, 60).await;
            }
            if count > 60 {
                return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
            }
        }
    }

    Ok(next.run(req).await)
}
