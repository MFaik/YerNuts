use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::{ws::WebSocket, ConnectInfo, WebSocketUpgrade},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new("backend".into(), std::io::stdout);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber.");

    let assets_dir = PathBuf::from("./static");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/", get(index_get))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    tracing::info!("listening on 127.0.0.1:3000");
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    tracing::info!("Request done on endpoint /ws");
    ws.on_upgrade(move |socket| handle_connection(socket))
}

async fn handle_connection(mut socket: WebSocket) {
    tracing::info!("New websocket connection: {:?}", socket);
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            tracing::info!("Received message: {:?}", msg);
            socket.send(msg).await.unwrap();
        } else {
            // client disconnected
            return;
        };
    }
}

async fn index_get() -> impl IntoResponse {
    tracing::info!("Request done on endpoint /");
    Html(include_str!("../../game/src/index.html").to_string())
}
