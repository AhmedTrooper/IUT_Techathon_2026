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
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
pub struct DiagramResponse {
    pub download_url: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/diagram/compile", post(compile_diagram))
}

async fn compile_diagram(State(state): State<AppState>) -> Result<Json<DiagramResponse>, StatusCode> {
    let tex_content = r#"
\documentclass[tikz,border=10pt]{standalone}
\usepackage{tikz}
\usetikzlibrary{shapes.geometric, arrows, positioning, fit, backgrounds}
\begin{document}
\begin{tikzpicture}[
    node distance=2.5cm,
    box/.style={rectangle, draw=blue!50, fill=blue!10, very thick, minimum width=3cm, minimum height=1.5cm, text centered, font=\sffamily},
    database/.style={cylinder, shape border rotate=90, aspect=0.25, draw=orange!50, fill=orange!10, very thick, minimum width=2.5cm, minimum height=2.5cm, text centered, font=\sffamily},
    arrow/.style={thick,->,>=stealth},
    bidir/.style={thick,<->,>=stealth}
]

% Nodes
\node (sim) [box] {Simulated Devices};
\node (api) [box, right=of sim, xshift=2cm] {Backend API};
\node (db) [database, above=of api] {Database};
\node (web) [box, right=of api, yshift=2cm] {Web Dashboard};
\node (bot) [box, right=of api, yshift=-2cm] {Discord Bot};
\node (user) [box, fill=green!10, draw=green!50, right=of web] {Boss};

\begin{scope}[on background layer]
    \node[fill=gray!5, draw=gray!20, thick, fit=(api)(db), rounded corners, label=above:\textbf{Backend}] {};
\end{scope}

\draw [arrow] (sim) -- node[above] {Updates} (api);
\draw [bidir] (api) -- node[right] {Read/Write} (db);
\draw [bidir] (api) |- (web);
\draw [bidir] (api) |- (bot);
\draw [bidir] (web) -- (user);

\end{tikzpicture}
\end{document}
    "#.to_string();

    info!("Compiling LaTeX diagram using Tectonic on an 8MB stack thread...");
    
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
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(e) => {
            error!("Tokio blocking task error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let file_key = format!("diagrams/system_diagram_{}.pdf", Utc::now().timestamp());

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
