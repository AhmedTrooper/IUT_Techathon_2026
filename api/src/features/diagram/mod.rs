use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::{error, info};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/diagram", get(get_diagram))
}

async fn get_diagram() -> Result<impl IntoResponse, StatusCode> {
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
    "#;

    info!("Compiling LaTeX diagram using Tectonic...");
    
    let pdf_data = match tokio::task::spawn_blocking(move || {
        tectonic::latex_to_pdf(tex_content)
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

    Ok((
        [(header::CONTENT_TYPE, "application/pdf")],
        pdf_data,
    ))
}
