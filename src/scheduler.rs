//! Background scheduler for memory maintenance (decay and flush)

use crate::db::brain::Brain;
use crate::config_file::UserConfig;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, watch};
use tokio::time::Duration;
use tracing::{info, warn};

pub struct Scheduler {
    brain: Arc<Brain>,
    running: Arc<AtomicBool>,
    user_config: Arc<tokio::sync::RwLock<UserConfig>>,
    shutdown_tx: Arc<Mutex<Option<watch::Sender<()>>>>,
}

impl Scheduler {
    pub fn new(brain: Arc<Brain>, user_config: Arc<tokio::sync::RwLock<UserConfig>>) -> Self {
        Self {
            brain,
            running: Arc::new(AtomicBool::new(false)),
            user_config,
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("Scheduler already running");
            return;
        }

        let brain = self.brain.clone();
        let running = self.running.clone();
        let user_config = self.user_config.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        let (tx, mut rx) = watch::channel(());
        {
            let mut tx_guard = shutdown_tx.lock().await;
            *tx_guard = Some(tx);
        }

        tokio::spawn(async move {
            let user_cfg = user_config.read().await;
            let mut maintenance_interval_hours: i64 = user_cfg.maintenance_interval_hours.unwrap_or(6) as i64;
            drop(user_cfg);

            info!("📅 Scheduler started with interval: {} hours", maintenance_interval_hours);

            loop {
                let interval_duration = if maintenance_interval_hours > 0 {
                    Duration::from_secs(maintenance_interval_hours as u64 * 3600)
                } else {
                    Duration::from_secs(86400)
                };

                tokio::select! {
                    _ = tokio::time::sleep(interval_duration) => {}
                    result = rx.changed() => {
                        if result.is_ok() {
                            let _ = rx.borrow();
                        }
                        info!("📅 Scheduler received shutdown signal");
                        break;
                    }
                }

                {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                }

                let user_cfg = user_config.read().await;
                maintenance_interval_hours = user_cfg.maintenance_interval_hours.unwrap_or(6) as i64;
                let decay_enabled = user_cfg.auto_decay_enabled.unwrap_or(true);
                let flush_enabled = user_cfg.auto_flush_enabled.unwrap_or(true);
                let threshold = user_cfg.min_importance_threshold.unwrap_or(0.1);
                drop(user_cfg);

                if maintenance_interval_hours <= 0 {
                    info!("📅 Scheduler: maintenance disabled, skipping");
                    continue;
                }

                info!("🔄 Running scheduled maintenance...");

                if decay_enabled {
                    match brain.apply_decay_formula().await {
                        Ok(count) => {
                            info!("   Decay applied to {} memories", count);
                        }
                        Err(e) => {
                            warn!("   Decay failed: {}", e);
                        }
                    }
                }

                if flush_enabled {
                    match brain.flush_low_importance(threshold).await {
                        Ok(count) => {
                            info!("   Flushed {} low-importance memories", count);
                        }
                        Err(e) => {
                            warn!("   Flush failed: {}", e);
                        }
                    }
                }

                // Cleanup old history events (30-day retention)
                match brain.cleanup_old_history(30).await {
                    Ok(count) => {
                        info!("   Cleaned up {} old history events", count);
                    }
                    Err(e) => {
                        warn!("   History cleanup failed: {}", e);
                    }
                }
            }

            // Reset running flag on exit so scheduler can be restarted
            running.store(false, Ordering::SeqCst);

            info!("📅 Scheduler stopped");
        });
    }

    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);

        let mut tx_guard = self.shutdown_tx.lock().await;
        if let Some(tx) = tx_guard.take() {
            let _ = tx.send(());
        }
    }
}
