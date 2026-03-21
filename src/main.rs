//! Chetna - God-Tier Memory System
//!
//! Entry point for the CLI and HTTP server

use anyhow::Result;
use chetna::{config::Config, init_logging, AppState, create_router, scheduler::Scheduler, START_TIME};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize uptime tracking
    let _ = START_TIME.set(std::time::Instant::now());

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
    let scheduler = if config.consolidation_interval_hours > 0 {
        let scheduler = Scheduler::new(state.brain.clone(), state.user_config.clone());
        scheduler.start().await;
        info!("   Scheduler: enabled (every {} hours)", config.consolidation_interval_hours);
        Some(scheduler)
    } else {
        info!("   Scheduler: disabled");
        None
    };

    // Build router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;

    info!("🌐 Server running at http://{}", addr);
    info!("📖 API docs: http://{}/docs", addr);
    info!("🎨 Dashboard: http://{}/", addr);
    info!("📡 MCP endpoint: http://{}/mcp", addr);
    info!("Press Ctrl+C to shutdown gracefully");

    // Start server in background task
    let _server_handle = tokio::spawn(async {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    // Graceful shutdown handling
    let shutdown = async {
        let _ = signal::ctrl_c().await;
        info!("👋 Shutdown signal received");
    };

    shutdown.await;
    info!("🛑 Stopping server...");

    // Stop scheduler if running
    if let Some(scheduler) = scheduler {
        info!("🛑 Stopping scheduler...");
        scheduler.stop().await;
    }

    info!("✅ Chetna shutdown complete");
    Ok(())
}
