//! REM Consolidation - Memory consolidation during sleep cycles
//!
//! Implements the neuroscience-inspired memory consolidation process:
//! - NREM Phase 1: Light sleep - consolidate high-importance emotional memories
//! - NREM Phase 2: Medium sleep - consolidate medium-importance memories
//! - NREM Phase 3: Deep sleep - consolidate factual memories
//! - REM Phase: Dreaming - strengthen connections between related memories

use crate::db::brain::Brain;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStats {
    pub memories_processed: i64,
    pub memories_consolidated: i64,
    pub memories_strengthened: i64,
    pub memories_pruned: i64,
    pub cycle_type: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RemCycle {
    Awake,
    Nrem1,  // Light sleep - emotional memories
    Nrem2,  // Medium sleep - procedural
    Nrem3,  // Deep sleep - factual
    REM,    // Dreaming - integration
}

impl RemCycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            RemCycle::Awake => "Awake",
            RemCycle::Nrem1 => "NREM 1 (Light)",
            RemCycle::Nrem2 => "NREM 2 (Medium)",
            RemCycle::Nrem3 => "NREM 3 (Deep)",
            RemCycle::REM => "REM (Dreaming)",
        }
    }

    pub fn importance_threshold(&self) -> f32 {
        match self {
            RemCycle::Nrem1 => 0.8,  // High importance
            RemCycle::Nrem2 => 0.5,  // Medium importance
            RemCycle::Nrem3 => 0.3,  // Low importance
            RemCycle::REM => 0.0,    // All
            _ => 0.0,
        }
    }

    pub fn arousal_boost(&self) -> f32 {
        match self {
            RemCycle::Nrem1 => 0.2,  // More emotional
            RemCycle::Nrem2 => 0.1,
            RemCycle::Nrem3 => 0.0,
            RemCycle::REM => 0.15,   // Dreams can be emotional
            _ => 0.0,
        }
    }
}

pub struct Consolidator {
    brain: Brain,
}

impl Consolidator {
    pub fn new(brain: Brain) -> Self {
        Self { brain }
    }

    pub async fn run_full_cycle(&self) -> Result<ConsolidationStats> {
        let start = std::time::Instant::now();
        info!("🌀 Starting full REM consolidation cycle");
        
        let mut total_processed = 0;
        let mut total_consolidated = 0;
        let mut total_strengthened = 0;
        let mut total_pruned = 0;
        
        // Run through each sleep phase
        for cycle in &[RemCycle::Nrem1, RemCycle::Nrem2, RemCycle::Nrem3, RemCycle::REM] {
            let result = self.process_phase(*cycle).await?;
            total_processed += result.memories_processed;
            total_consolidated += result.memories_consolidated;
            total_strengthened += result.memories_strengthened;
            total_pruned += result.memories_pruned;
            
            info!("   {}: processed={}, consolidated={}", 
                cycle.as_str(), result.memories_processed, result.memories_consolidated);
        }
        
        // Prune very old low-importance memories
        let pruned = self.prune_old_memories(90, 0.05).await?;
        total_pruned += pruned;
        
        let duration = start.elapsed().as_millis() as u64;
        
        info!("✅ Consolidation complete: {}ms, processed={}, consolidated={}, pruned={}", 
            duration, total_processed, total_consolidated, total_pruned);
        
        Ok(ConsolidationStats {
            memories_processed: total_processed,
            memories_consolidated: total_consolidated,
            memories_strengthened: total_strengthened,
            memories_pruned: total_pruned,
            cycle_type: "Full Cycle".to_string(),
            duration_ms: duration,
        })
    }

    async fn process_phase(&self, phase: RemCycle) -> Result<ConsolidationStats> {
        let threshold = phase.importance_threshold();
        
        // Get memories above importance threshold
        let memories = self.brain.list_memories(1000).await?;
        
        let mut processed = 0;
        let mut consolidated = 0;
        let mut strengthened = 0;
        
        for memory in memories {
            if memory.importance < threshold as f64 {
                continue;
            }
            
            processed += 1;
            
            // Phase-specific processing
            match phase {
                RemCycle::Nrem1 => {
                    // Emotional memories - boost emotional_tone and arousal
                    if memory.emotional_tone.abs() > 0.3 {
                        self.strengthen_memory(&memory.id, 0.1, phase.arousal_boost()).await?;
                        strengthened += 1;
                    }
                    // Mark as consolidated
                    self.mark_consolidated(&memory.id).await?;
                    consolidated += 1;
                }
                RemCycle::Nrem2 => {
                    // Procedural and skill memories
                    if memory.memory_type == "skill_learned" || memory.category == "skill" {
                        self.strengthen_memory(&memory.id, 0.15, 0.0).await?;
                        strengthened += 1;
                    }
                    self.mark_consolidated(&memory.id).await?;
                    consolidated += 1;
                }
                RemCycle::Nrem3 => {
                    // Factual memories - just mark consolidated
                    self.mark_consolidated(&memory.id).await?;
                    consolidated += 1;
                }
                RemCycle::REM => {
                    // Find related memories and strengthen connections
                    let related = self.brain.find_related_memories(&memory.id, 5).await?;
                    if !related.is_empty() {
                        self.strengthen_memory(&memory.id, 0.05, 0.0).await?;
                        strengthened += 1;
                    }
                }
                _ => {}
            }
        }
        
        Ok(ConsolidationStats {
            memories_processed: processed,
            memories_consolidated: consolidated,
            memories_strengthened: strengthened,
            memories_pruned: 0,
            cycle_type: phase.as_str().to_string(),
            duration_ms: 0,
        })
    }

    async fn strengthen_memory(&self, id: &str, importance_boost: f32, arousal_boost: f32) -> Result<()> {
        // Get current memory
        let memory = self.brain.get_memory(id).await?;

        // Calculate new values
        let new_importance = (memory.importance as f32 + importance_boost).min(1.0);
        let new_arousal = (memory.arousal as f32 + arousal_boost).min(1.0);

        // Note: Direct database update would go here, but Connection is not Clone
        // For now, this is a placeholder that logs the update
        tracing::debug!("Would strengthen memory {}: importance={:.2}, arousal={:.2}", id, new_importance, new_arousal);

        Ok(())
    }

    async fn mark_consolidated(&self, id: &str) -> Result<()> {
        // Note: Direct database update would go here, but Connection is not Clone
        // For now, this is a placeholder that logs the consolidation
        tracing::debug!("Would mark memory {} as consolidated", id);
        Ok(())
    }

    pub async fn consolidate_high_importance(&self) -> Result<i64> {
        info!("⭐ Consolidating high-importance memories (importance > 0.8)");
        
        let memories = self.brain.list_memories(1000).await?;
        let mut count = 0;
        
        for memory in memories {
            if memory.importance > 0.8 && !memory.consolidated {
                self.mark_consolidated(&memory.id).await?;
                count += 1;
            }
        }
        
        info!("   Consolidated {} high-importance memories", count);
        Ok(count)
    }

    pub async fn prune_old_memories(&self, days_threshold: i64, min_importance: f32) -> Result<i64> {
        info!("🗑️ Pruning memories older than {} days with importance < {}", days_threshold, min_importance);
        
        let count = self.brain.prune_memories(days_threshold, min_importance).await?;
        
        info!("   Pruned {} memories", count);
        Ok(count)
    }

    pub async fn strengthen_frequently_accessed(&self) -> Result<i64> {
        info!("💪 Strengthening frequently accessed memories");
        
        let memories = self.brain.list_memories(500).await?;
        let mut count = 0;
        
        for memory in memories {
            if memory.access_count > 5 {
                self.strengthen_memory(&memory.id, 0.05, 0.0).await?;
                count += 1;
            }
        }
        
        info!("   Strengthened {} frequently accessed memories", count);
        Ok(count)
    }

    /// Get consolidation recommendations
    pub async fn get_recommendations(&self) -> Result<serde_json::Value> {
        let memories = self.brain.list_memories(100).await?;
        
        let mut unconsolidated = 0;
        let mut high_importance = 0;
        let mut low_importance = 0;
        let mut high_access = 0;
        
        for m in &memories {
            if !m.consolidated {
                unconsolidated += 1;
            }
            if m.importance > 0.7 {
                high_importance += 1;
            }
            if m.importance < 0.3 {
                low_importance += 1;
            }
            if m.access_count > 10 {
                high_access += 1;
            }
        }
        
        Ok(serde_json::json!({
            "unconsolidated_memories": unconsolidated,
            "high_importance_memories": high_importance,
            "low_importance_memories": low_importance,
            "frequently_accessed": high_access,
            "recommendations": {
                "run_consolidation": unconsolidated > 50,
                "prune_low_importance": low_importance > 20,
                "strengthen_accessed": high_access > 10
            }
        }))
    }
}
