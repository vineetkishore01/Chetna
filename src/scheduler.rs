//! Background scheduler for memory consolidation and maintenance

use crate::config::Config;
use crate::db::brain::Brain;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct Scheduler {
    brain: Arc<Brain>,
    config: Config,
    running: Arc<Mutex<bool>>,
}

impl Scheduler {
    pub fn new(brain: Arc<Brain>, config: Config) -> Self {
        Self {
            brain,
            config,
            running: Arc::new(Mutex::new(false)),
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
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let consolidation_interval = if config.consolidation_interval_hours > 0 {
                Duration::from_secs(config.consolidation_interval_hours as u64 * 3600)
            } else {
                Duration::from_secs(3600)
            };

            let mut ticker = interval(consolidation_interval);
            info!("📅 Scheduler started with interval: {} hours", config.consolidation_interval_hours);

            loop {
                ticker.tick().await;

                let is_running = running.lock().await;
                if !*is_running {
                    break;
                }
                drop(is_running);

                if config.consolidation_interval_hours > 0 {
                    info!("🔄 Running scheduled consolidation...");

                    if config.has_llm() {
                        match brain.consolidate_memories_llm(100).await {
                            Ok((processed, updated)) => {
                                info!("   LLM consolidation: processed={}, updated={}", processed, updated);
                            }
                            Err(e) => {
                                warn!("   LLM consolidation failed: {}", e);
                            }
                        }
                    }

                    if config.auto_decay_enabled {
                        match brain.apply_decay_formula().await {
                            Ok(count) => {
                                info!("   Decay applied to {} memories", count);
                            }
                            Err(e) => {
                                warn!("   Decay failed: {}", e);
                            }
                        }
                    }

                    if config.auto_flush_enabled {
                        match brain.flush_low_importance(config.min_importance_threshold).await {
                            Ok(count) => {
                                info!("   Flushed {} low-importance memories", count);
                            }
                            Err(e) => {
                                warn!("   Flush failed: {}", e);
                            }
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
