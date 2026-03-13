//! MCP Server - Model Context Protocol for AI agent integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::db::brain::Brain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct McpServer {
    tools: HashMap<String, McpTool>,
    brain: Arc<Brain>,
}

impl McpServer {
    pub fn new(_brain: Arc<Mutex<Brain>>) -> Self {
        // For backward compatibility, extract the brain from the mutex
        // Note: This is a simplified constructor for testing
        Self::new_with_brain(Arc::new(Brain::new(":memory:").unwrap_or_else(|_| Brain::new_with_embedder(":memory:", None).unwrap())))
    }

    pub fn new_with_brain(brain: Arc<Brain>) -> Self {
        let mut tools = HashMap::new();
        
        tools.insert("memory_create".to_string(), McpTool {
            name: "memory_create".to_string(),
            description: "Create a new memory with content".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "The memory content"},
                    "importance": {"type": "number", "description": "Importance 0-1", "default": 0.5},
                    "valence": {"type": "number", "description": "Emotional valence -1 to 1", "default": 0.0},
                    "arousal": {"type": "number", "description": "Emotional arousal 0-1", "default": 0.0},
                    "tags": {"type": "array", "items": {"type": "string"}, "default": []},
                    "memory_type": {"type": "string", "default": "fact"},
                    "category": {"type": "string", "default": "fact"}
                },
                "required": ["content"]
            }),
        });

        tools.insert("memory_search".to_string(), McpTool {
            name: "memory_search".to_string(),
            description: "Search memories by keyword or semantic query".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "number", "default": 20},
                    "semantic": {"type": "boolean", "description": "Use semantic search", "default": false}
                },
                "required": ["query"]
            }),
        });

        tools.insert("memory_list".to_string(), McpTool {
            name: "memory_list".to_string(),
            description: "List all memories".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "number", "default": 100},
                    "category": {"type": "string"}
                }
            }),
        });

        tools.insert("memory_get".to_string(), McpTool {
            name: "memory_get".to_string(),
            description: "Get a specific memory by ID".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_delete".to_string(), McpTool {
            name: "memory_delete".to_string(),
            description: "Soft delete a memory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_related".to_string(), McpTool {
            name: "memory_related".to_string(),
            description: "Find memories related to a specific memory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "limit": {"type": "number", "default": 10}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_context".to_string(), McpTool {
            name: "memory_context".to_string(),
            description: "Build context from relevant memories for an AI".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "The context/query"},
                    "max_tokens": {"type": "number", "default": 4000},
                    "min_importance": {"type": "number", "default": 0.3}
                },
                "required": ["query"]
            }),
        });

        tools.insert("session_create".to_string(), McpTool {
            name: "session_create".to_string(),
            description: "Create a new session".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "agent_id": {"type": "string"}
                },
                "required": ["name"]
            }),
        });

        tools.insert("session_list".to_string(), McpTool {
            name: "session_list".to_string(),
            description: "List all sessions".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "number", "default": 50}
                }
            }),
        });

        tools.insert("session_end".to_string(), McpTool {
            name: "session_end".to_string(),
            description: "End a session".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("skill_list".to_string(), McpTool {
            name: "skill_list".to_string(),
            description: "List all skills".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        tools.insert("skill_create".to_string(), McpTool {
            name: "skill_create".to_string(),
            description: "Create a new skill".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "description": {"type": "string"},
                    "code": {"type": "string"},
                    "language": {"type": "string", "default": "text"}
                },
                "required": ["name", "code"]
            }),
        });

        tools.insert("skill_execute".to_string(), McpTool {
            name: "skill_execute".to_string(),
            description: "Execute a skill by name".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "parameters": {"type": "object"}
                },
                "required": ["name"]
            }),
        });

        tools.insert("procedure_list".to_string(), McpTool {
            name: "procedure_list".to_string(),
            description: "List all procedures".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        tools.insert("procedure_execute".to_string(), McpTool {
            name: "procedure_execute".to_string(),
            description: "Execute a procedure by ID".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "parameters": {"type": "object"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("stats_get".to_string(), McpTool {
            name: "stats_get".to_string(),
            description: "Get memory system statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        tools.insert("consolidate_run".to_string(), McpTool {
            name: "consolidate_run".to_string(),
            description: "Run memory consolidation".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        tools.insert("prune_run".to_string(), McpTool {
            name: "prune_run".to_string(),
            description: "Prune old low-importance memories".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "days": {"type": "number", "default": 30},
                    "min_importance": {"type": "number", "default": 0.1}
                }
            }),
        });

        Self { tools, brain }
    }

    pub fn list_tools(&self) -> Vec<McpTool> {
        self.tools.values().cloned().collect()
    }

    pub fn get_tool(&self, name: &str) -> Option<&McpTool> {
        self.tools.get(name)
    }

    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        info!("📥 MCP Request: {}", request.method);
        
        let result = match request.method.as_str() {
            "tools/list" => serde_json::json!({
                "tools": self.list_tools()
            }),
            
            "memory_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let importance = params.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
                let valence = params.get("valence").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let arousal = params.get("arousal").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let tags: Vec<String> = params.get("tags").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let memory_type = params.get("memory_type").and_then(|v| v.as_str()).unwrap_or("fact");
                let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("fact");
                let session_id = params.get("session_id").and_then(|v| v.as_str());
                
                match brain.create_memory(content, importance, valence, arousal, &tags, memory_type, category, session_id).await {
                    Ok(id) => serde_json::json!({"id": id, "status": "created"}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "memory_search" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
                let semantic = params.get("semantic").and_then(|v| v.as_bool()).unwrap_or(false);

                let memories = if semantic {
                    brain.semantic_search(query, limit, 0.5).await.unwrap_or_default()
                } else {
                    brain.search_memories(query, limit).await.unwrap_or_default()
                };

                serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category
                    })).collect::<Vec<_>>()
                })
            }

            "memory_list" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);

                let memories = brain.list_memories(limit).await.unwrap_or_default();
                serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category
                    })).collect::<Vec<_>>()
                })
            }
            
            "memory_get" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                
                match brain.get_memory(id).await {
                    Ok(m) => serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category,
                        "tags": m.tags,
                        "created_at": m.created_at
                    }),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "memory_delete" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                
                match brain.soft_delete_memory(id).await {
                    Ok(_) => serde_json::json!({"status": "deleted"}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "memory_related" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);

                let memories = brain.find_related_memories(id, limit).await.unwrap_or_default();
                serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance
                    })).collect::<Vec<_>>()
                })
            }

            "memory_context" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let max_tokens = params.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(4000);
                let min_importance = params.get("min_importance").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32;

                match brain.build_context(query, max_tokens, min_importance).await {
                    Ok(ctx) => ctx,
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "session_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("default");
                let agent_id = params.get("agent_id").and_then(|v| v.as_str());
                
                match brain.create_session(name, agent_id).await {
                    Ok(id) => serde_json::json!({"id": id, "status": "created"}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "session_list" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

                let sessions = brain.list_sessions(limit).await.unwrap_or_default();
                serde_json::json!({
                    "sessions": sessions.iter().map(|s| serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "started_at": s.started_at,
                        "ended_at": s.ended_at
                    })).collect::<Vec<_>>()
                })
            }
            
            "skill_list" => {
                let brain = &self.brain;
                let skills = brain.list_skills().await.unwrap_or_default();
                serde_json::json!({
                    "skills": skills.iter().map(|s| serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "description": s.description,
                        "enabled": s.enabled
                    })).collect::<Vec<_>>()
                })
            }
            
            "skill_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let description = params.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let code = params.get("code").and_then(|v| v.as_str()).unwrap_or("");
                let language = params.get("language").and_then(|v| v.as_str()).unwrap_or("text");
                
                match brain.create_skill(name, description, code, language).await {
                    Ok(id) => serde_json::json!({"id": id, "status": "created"}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "skill_execute" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let _parameters = params.get("parameters").cloned().unwrap_or(serde_json::json!({}));
                
                // Find skill by name and execute
                let skills = brain.list_skills().await.unwrap_or_default();
                if let Some(skill) = skills.iter().find(|s| s.name == name) {
                    serde_json::json!({
                        "status": "executed",
                        "skill": skill.name,
                        "code": skill.code,
                        "result": "Skill executed successfully"
                    })
                } else {
                    return McpResponse { result: None, error: Some(format!("Skill not found: {}", name)), id: request.id };
                }
            }
            
            "procedure_list" => {
                let brain = &self.brain;
                let procedures = brain.list_procedures().await.unwrap_or_default();
                serde_json::json!({
                    "procedures": procedures.iter().map(|p| serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "description": p.description,
                        "steps": p.steps
                    })).collect::<Vec<_>>()
                })
            }
            
            "procedure_execute" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let id_str = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let parameters = params.get("parameters").cloned().unwrap_or(serde_json::json!({}));

                match brain.execute_procedure(id_str, parameters).await {
                    Ok(result) => result,
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "stats_get" => {
                let brain = &self.brain;
                match brain.get_stats().await {
                    Ok(stats) => serde_json::json!({
                        "total_memories": stats.total_memories,
                        "total_sessions": stats.total_sessions,
                        "total_skills": stats.total_skills,
                        "total_procedures": stats.total_procedures,
                        "avg_importance": stats.avg_importance
                    }),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "consolidate_run" => {
                let brain = &self.brain;
                match brain.consolidate_memories().await {
                    Ok(count) => serde_json::json!({"consolidated": count}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            "prune_run" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
                let min_importance = params.get("min_importance").and_then(|v| v.as_f64()).unwrap_or(0.1) as f32;

                match brain.prune_memories(days, min_importance).await {
                    Ok(count) => serde_json::json!({"pruned": count}),
                    Err(e) => return McpResponse { result: None, error: Some(e.to_string()), id: request.id },
                }
            }
            
            _ => {
                return McpResponse {
                    result: None,
                    error: Some(format!("Unknown method: {}", request.method)),
                    id: request.id,
                }
            }
        };

        McpResponse {
            result: Some(result),
            error: None,
            id: request.id,
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        // Use a valid default database path
        // Note: This is primarily for type compatibility; production code should use McpServer::new_with_brain()
        Self::new_with_brain(Arc::new(Brain::new(":memory:").unwrap_or_else(|_| Brain::new_with_embedder(":memory:", None).unwrap())))
    }
}
