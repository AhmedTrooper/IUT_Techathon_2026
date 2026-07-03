use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use tracing::{error, info};
use crate::AppState;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CompileRequest {
    pub latex_code: String,
}

#[derive(Serialize)]
pub struct DiagramResponse {
    pub download_url: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/diagram/compile", post(compile_diagram))
}

async fn compile_diagram(
    State(state): State<AppState>,
    Json(payload): Json<CompileRequest>,
) -> Result<Json<DiagramResponse>, StatusCode> {
    let tex_content = payload.latex_code;

    info!("Compiling LaTeX diagram from frontend using Tectonic on an 8MB stack thread...");
    
    let pdf_data = match tokio::task::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let res = tectonic::latex_to_pdf(tex_content);
                let _ = tx.send(res);
            })
            .unwrap()
            .join()
            .unwrap();
        rx.recv().unwrap()
    }).await {
        Ok(Ok(pdf)) => pdf,
        Ok(Err(e)) => {
            error!("Tectonic compilation error: {}", e);
            return Err(StatusCode::BAD_REQUEST); // User submitted bad LaTeX
        }
        Err(e) => {
            error!("Tokio blocking task error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let file_key = "diagrams/system_diagram.pdf".to_string();

    state.s3_client
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .body(ByteStream::from(pdf_data))
        .content_type("application/pdf")
        .send()
        .await
        .map_err(|e| {
            error!("Error uploading diagram to S3: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let presigned = state.s3_client
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .presigned(PresigningConfig::expires_in(Duration::from_secs(900)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .await
        .map_err(|e| {
            error!("Error generating presigned S3 url: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(DiagramResponse {
        download_url: presigned.uri().to_string(),
    }))
}
