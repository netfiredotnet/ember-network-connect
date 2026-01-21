use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{error, info};

use crate::network::NetworkCommand;

/// Global timer for countdown
static TIMER: AtomicU64 = AtomicU64::new(0);

/// Shared state passed to handlers
#[derive(Clone)]
pub struct AppState {
    gateway: Ipv4Addr,
    network_tx: mpsc::Sender<NetworkCommand>,
}

/// Start the HTTP server
pub async fn start_server(
    gateway: Ipv4Addr,
    listening_port: u16,
    network_tx: mpsc::Sender<NetworkCommand>,
    ui_directory: PathBuf,
) -> Result<(), std::io::Error> {
    let state = AppState {
        gateway,
        network_tx,
    };

    // Static file serving for UI
    let serve_dir = ServeDir::new(&ui_directory)
        .append_index_html_on_directories(true);

    // Build the router
    let app = Router::new()
        .route("/get_timer", get(get_timer))
        .route("/reset_dhcp", post(reset_dhcp))
        .nest_service("/static", ServeDir::new(ui_directory.join("static")))
        .nest_service("/css", ServeDir::new(ui_directory.join("css")))
        .nest_service("/img", ServeDir::new(ui_directory.join("img")))
        .nest_service("/js", ServeDir::new(ui_directory.join("js")))
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            captive_portal_redirect,
        ))
        .with_state(state);

    let addr = SocketAddr::new(gateway.into(), listening_port);
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}

/// Middleware to redirect captive portal requests to the gateway
async fn captive_portal_redirect(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    // Check if the Host header matches our gateway
    if let Some(host) = req.headers().get(header::HOST) {
        if let Ok(host_str) = host.to_str() {
            let gateway_str = state.gateway.to_string();
            // If host doesn't match gateway (captive portal detection), redirect
            if !host_str.starts_with(&gateway_str) {
                return Redirect::temporary(&format!("http://{}/", gateway_str)).into_response();
            }
        }
    }

    next.run(req).await
}

/// Start the countdown timer
pub fn start_timer(secs: u64, network_tx: mpsc::Sender<NetworkCommand>) {
    TIMER.store(secs, Ordering::Relaxed);

    tokio::spawn(async move {
        while TIMER.load(Ordering::Relaxed) > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let current = TIMER.load(Ordering::Relaxed);
            if current > 0 {
                TIMER.store(current - 1, Ordering::Relaxed);
            }
        }

        if let Err(e) = network_tx.send(NetworkCommand::OverallTimeout).await {
            error!("Sending NetworkCommand::OverallTimeout failed: {}", e);
        }
    });
}

/// GET /get_timer - Return the current countdown value
async fn get_timer(State(state): State<AppState>) -> Result<String, StatusCode> {
    // Signal that user is active
    if let Err(e) = state.network_tx.send(NetworkCommand::Activate).await {
        error!("Sending NetworkCommand::Activate failed: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let time = TIMER.load(Ordering::Relaxed);
    Ok(time.to_string())
}

/// POST /reset_dhcp - Trigger DHCP reset
async fn reset_dhcp(State(state): State<AppState>) -> StatusCode {
    info!("Requested DHCP reset");

    if let Err(e) = state.network_tx.send(NetworkCommand::Reset).await {
        error!("Sending NetworkCommand::Reset failed: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
