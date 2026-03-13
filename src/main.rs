//! Chetna - God-Tier Memory System
//!
//! Entry point for the CLI and HTTP server

use anyhow::Result;
use chetna::{config::Config, init_logging, AppState, create_router, scheduler::Scheduler};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Load configuration
    let config = Config::from_env()?;

    info!("🧠 Starting Chetna Memory System");
    info!("   Host: {}:{}", config.host, config.port);
    info!("   Database: {}", config.db_path);

    // Create application state
    let state = AppState::new(config.clone())?;

    // Start background scheduler if consolidation is enabled
    if config.consolidation_interval_hours > 0 {
        let scheduler = Scheduler::new(state.brain.clone(), config.clone());
        scheduler.start().await;
        info!("   Scheduler: enabled (every {} hours)", config.consolidation_interval_hours);
    } else {
        info!("   Scheduler: disabled");
    }

    // Build router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;

    info!("🌐 Server running at http://{}", addr);
    info!("📖 API docs: http://{}/docs", addr);
    info!("🎨 Dashboard: http://{}/", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
