use crate::{enhanced_ui::context::ProjectContext, pwa};
use anyhow::Result;
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse,
    },
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[derive(Clone)]
struct WebState {
    tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
struct HealthPayload {
    status: &'static str,
}

pub async fn start(address: &str) -> Result<()> {
    let addr: SocketAddr = address
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:7878".parse().expect("valid default addr"));
    let (tx, _) = broadcast::channel(128);
    let state = WebState { tx: tx.clone() };

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let context = ProjectContext::detect();
            if let Ok(payload) = serde_json::to_string(&context) {
                let _ = tx.send(payload);
            }
        }
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/pwa", get(pwa_shell))
        .route("/sw.js", get(service_worker))
        .route("/manifest.webmanifest", get(manifest))
        .route("/health", get(health))
        .route("/context", get(context_endpoint))
        .route("/events", get(events_handler))
        .with_state(state);

    let listener = TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;
    println!("🌐 Web Companion available at http://{}", bound_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Kandil Companion</title>
    <style>
        body { font-family: system-ui, sans-serif; margin: 2rem; color: #111; }
        pre { background: #111; color: #0f0; padding: 1rem; min-height: 200px; }
        header { display:flex; justify-content:space-between; align-items:center; }
    </style>
</head>
<body>
    <header>
        <h1>Kandil Companion</h1>
        <span id="status">connecting…</span>
    </header>
    <pre id="stream">{ }</pre>
    <script>
        const status = document.getElementById("status");
        const stream = document.getElementById("stream");
        const source = new EventSource("/events");
        source.onopen = () => status.textContent = "live";
        source.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                stream.textContent = JSON.stringify(data, null, 2);
            } catch (err) {
                console.error(err);
            }
        };
        source.onerror = () => status.textContent = "reconnecting…";
    </script>
</body>
</html>"#,
    )
}

async fn pwa_shell() -> Html<&'static str> {
    Html(pwa::INDEX_HTML)
}

async fn manifest() -> impl IntoResponse {
    (
        [("content-type", "application/manifest+json")],
        pwa::MANIFEST,
    )
}

async fn service_worker() -> impl IntoResponse {
    (
        [("content-type", "application/javascript")],
        pwa::SERVICE_WORKER,
    )
}

async fn health() -> Json<HealthPayload> {
    Json(HealthPayload { status: "ok" })
}

async fn context_endpoint() -> Json<ProjectContext> {
    Json(ProjectContext::detect())
}

async fn events_handler(
    State(state): State<WebState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.tx.subscribe()).filter_map(|msg| match msg {
        Ok(payload) => Some(Ok(Event::default().event("context").data(payload))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("ping"),
    )
}
