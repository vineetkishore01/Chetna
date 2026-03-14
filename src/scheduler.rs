//! Background scheduler for memory consolidation and maintenance

use crate::db::brain::Brain;
use crate::config_file::UserConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct Scheduler {
    brain: Arc<Brain>,
    running: Arc<Mutex<bool>>,
    user_config: Arc<tokio::sync::RwLock<UserConfig>>,
}

impl Scheduler {
    pub fn new(brain: Arc<Brain>, user_config: Arc<tokio::sync::RwLock<UserConfig>>) -> Self {
        Self {
            brain,
            running: Arc::new(Mutex::new(false)),
            user_config,
        }
    }

    pub async fn start(&self) {
        let mut is_running = self.running.lock().await;
        if *is_running {
            warn!("Scheduler already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        let brain = self.brain.clone();
        let running = self.running.clone();
        let user_config = self.user_config.clone();

        tokio::spawn(async move {
            // Initial config load
            let user_cfg = user_config.read().await;
            let mut consolidation_interval_hours = user_cfg.consolidation_interval_hours.unwrap_or(6);
            drop(user_cfg);

            info!("📅 Scheduler started with interval: {} hours", consolidation_interval_hours);

            loop {
                let interval_duration = if consolidation_interval_hours > 0 {
                    Duration::from_secs(consolidation_interval_hours as u64 * 3600)
                } else {
                    // If disabled, use a long duration - we'll check and skip
                    Duration::from_secs(86400) // 24 hours
                };

                let mut ticker = interval(interval_duration);
                ticker.tick().await;

                // Check if we should stop
                {
                    let is_running = running.lock().await;
                    if !*is_running {
                        break;
                    }
                }

                // Re-read config for this iteration
                let user_cfg = user_config.read().await;
                consolidation_interval_hours = user_cfg.consolidation_interval_hours.unwrap_or(6);
                let decay_enabled = user_cfg.auto_decay_enabled.unwrap_or(true);
                let flush_enabled = user_cfg.auto_flush_enabled.unwrap_or(true);
                let threshold = user_cfg.min_importance_threshold.unwrap_or(0.1);
                let llm_configured = user_cfg.llm_model.is_some();
                drop(user_cfg);

                // Skip if consolidation is disabled
                if consolidation_interval_hours <= 0 {
                    info!("📅 Scheduler: consolidation disabled, skipping");
                    continue;
                }

                info!("🔄 Running scheduled consolidation...");

                if llm_configured {
                    match brain.consolidate_memories_llm(100).await {
                        Ok((processed, updated)) => {
                            info!("   LLM consolidation: processed={}, updated={}", processed, updated);
                        }
                        Err(e) => {
                            warn!("   LLM consolidation failed: {}", e);
                        }
                    }
                }

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
            }

            info!("📅 Scheduler stopped");
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.running.lock().await;
        *is_running = false;
    }
}
