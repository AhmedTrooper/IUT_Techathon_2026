use crate::{AppState, get_default_devices, features};
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::postgres::PgPoolOptions;
use axum::extract::State;
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

pub fn build_test_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .nest("/api", axum::Router::new()
            .merge(features::devices::router())
            .merge(features::usage::router())
            .merge(features::alerts::router())
        )
        .with_state(state)
}

fn create_fake_state() -> AppState {
    let fake_pool = PgPoolOptions::new()
        .connect_lazy("postgres://fake:fake@127.0.0.1:5432/fake")
        .unwrap();
    
    let s3_config = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_config::Region::new("us-east-1"))
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    
    AppState {
        pool: fake_pool,
        s3_client,
        s3_bucket: "test-bucket".to_string(),
        redis_client,
        memory_devices: Arc::new(RwLock::new(get_default_devices())),
        memory_history: Arc::new(RwLock::new(Vec::new())),
    }
}

#[tokio::test]
async fn test_graceful_degradation_devices_endpoint() {
    let state = create_fake_state();
    let app = build_test_router(state);

    let response = app
        .oneshot(Request::builder().uri("/api/devices").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Devices endpoint should succeed via memory fallback despite broken DB");
}

#[tokio::test]
async fn test_graceful_degradation_usage_endpoint() {
    let state = create_fake_state();
    let app = build_test_router(state);

    let response = app
        .oneshot(Request::builder().uri("/api/usage").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Usage endpoint should successfully calculate power from memory fallback");
}

#[tokio::test]
async fn test_graceful_degradation_alerts_endpoint() {
    let state = create_fake_state();
    let app = build_test_router(state);

    let response = app
        .oneshot(Request::builder().uri("/api/alerts").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Alerts endpoint should successfully calculate anomalies from memory fallback");
}
