//! MCP Server - Model Context Protocol for AI agent integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::db::brain::Brain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: Option<String>,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
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
                    "category": {"type": "string", "default": "fact"},
                    "namespace": {"type": "string", "description": "Optional namespace", "default": "default"}
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
                    "semantic": {"type": "boolean", "description": "Use semantic search", "default": false},
                    "namespace": {"type": "string", "description": "Optional namespace", "default": "default"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional tags to filter by"},
                    "memory_type": {"type": "string", "description": "Optional memory_type to filter by"},
                    "min_importance": {"type": "number", "description": "Optional minimum importance"}
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
                    "category": {"type": "string"},
                    "namespace": {"type": "string", "description": "Optional namespace", "default": "default"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional tags to filter by"},
                    "memory_type": {"type": "string", "description": "Optional memory_type to filter by"},
                    "min_importance": {"type": "number", "description": "Optional minimum importance"}
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
                    "limit": {"type": "number", "default": 10},
                    "namespace": {"type": "string", "description": "Optional namespace", "default": "default"}
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
                    "min_importance": {"type": "number", "default": 0.3},
                    "min_similarity": {"type": "number", "default": 0.4},
                    "namespace": {"type": "string", "description": "Optional namespace", "default": "default"},
                    "session_id": {"type": "string", "description": "Optional session ID for working memory priming"}
                },
                "required": ["query"]
            }),
        });

        tools.insert("memory_batch_create".to_string(), McpTool {
            name: "memory_batch_create".to_string(),
            description: "Create multiple memories at once for efficiency".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "memories": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {"type": "string"},
                                "importance": {"type": "number", "default": 0.5},
                                "memory_type": {"type": "string", "default": "fact"},
                                "category": {"type": "string", "default": "fact"},
                                "tags": {"type": "array", "items": {"type": "string"}, "default": []},
                                "valence": {"type": "number", "default": 0.0},
                                "arousal": {"type": "number", "default": 0.0},
                                "namespace": {"type": "string", "default": "default"}
                            },
                            "required": ["content"]
                        }
                    }
                },
                "required": ["memories"]
            }),
        });

        tools.insert("memory_update".to_string(), McpTool {
            name: "memory_update".to_string(),
            description: "Update an existing memory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "content": {"type": "string"},
                    "importance": {"type": "number"},
                    "memory_type": {"type": "string"},
                    "category": {"type": "string"},
                    "tags": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_pin".to_string(), McpTool {
            name: "memory_pin".to_string(),
            description: "Pin a memory to prevent auto-deletion".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_unpin".to_string(), McpTool {
            name: "memory_unpin".to_string(),
            description: "Unpin a memory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                },
                "required": ["id"]
            }),
        });

        tools.insert("memory_set_category".to_string(), McpTool {
            name: "memory_set_category".to_string(),
            description: "Set the category of a memory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "category": {"type": "string"}
                },
                "required": ["id", "category"]
            }),
        });

        tools.insert("session_create".to_string(), McpTool {
            name: "session_create".to_string(),
            description: "Create a new session".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "agent_id": {"type": "string"},
                    "project": {"type": "string"},
                    "directory": {"type": "string"},
                    "namespace": {"type": "string", "default": "default"}
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
                    "limit": {"type": "number", "default": 50},
                    "namespace": {"type": "string", "default": "default"}
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

        tools.insert("stats_get".to_string(), McpTool {
            name: "stats_get".to_string(),
            description: "Get memory system statistics".to_string(),
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
            "tools/list" => {
                Ok(serde_json::json!({
                    "tools": self.list_tools()
                }))
            }
            
            "memory_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
                
                if content.is_empty() {
                    Err("Content cannot be empty".to_string())
                } else if content.len() > 50000 {
                    Err("Content too long (max 50000 chars)".to_string())
                } else {
                    let importance = params.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
                    let valence = params.get("valence").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let arousal = params.get("arousal").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let tags: Vec<String> = params.get("tags").and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    
                    if tags.len() > 50 {
                        Err("Too many tags (max 50)".to_string())
                    } else {
                        let memory_type = params.get("memory_type").and_then(|v| v.as_str()).unwrap_or("fact");
                        let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("fact");
                        
                        const CATEGORIES: &[&str] = &["fact", "preference", "rule", "experience"];
                        if !CATEGORIES.contains(&category) {
                            Err(format!("Invalid category '{}'. Valid options: {:?}", category, CATEGORIES))
                        } else {
                            let session_id = params.get("session_id").and_then(|v| v.as_str());
                            let namespace = params.get("namespace").and_then(|v| v.as_str());
                            
                            match brain.create_memory(content, importance, valence, arousal, &tags, memory_type, category, session_id, namespace).await {
                                Ok(memory) => Ok(serde_json::json!({"id": memory.id, "status": "created"})),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                    }
                }
            }
            
            "memory_search" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(20);
                let semantic = params.get("semantic").and_then(|v| v.as_bool()).unwrap_or(false);
                let namespace = params.get("namespace").and_then(|v| v.as_str());
                
                let memory_type_filter = params.get("memory_type").and_then(|v| v.as_str());
                let min_importance_filter = params.get("min_importance").and_then(|v| v.as_f64());
                let tags_filter: Option<Vec<String>> = params.get("tags").and_then(|v| {
                    v.as_array().map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
                });

                // Fetch a larger set to allow for in-memory filtering
                let fetch_limit = if memory_type_filter.is_some() || tags_filter.is_some() || min_importance_filter.is_some() { limit * 10 } else { limit };

                let mut memories = if semantic {
                    brain.semantic_search(query, fetch_limit, 0.1, namespace, None).await.unwrap_or_default()
                } else {
                    brain.search_memories(query, fetch_limit, 0.1, namespace, None).await.unwrap_or_default()
                };

                // Apply filters
                memories.retain(|m| {
                    if let Some(ref mt) = memory_type_filter {
                        if &m.memory_type != mt { return false; }
                    }
                    if let Some(ref min_imp) = min_importance_filter {
                        if m.importance < *min_imp { return false; }
                    }
                    if let Some(ref req_tags) = tags_filter {
                        if !req_tags.iter().all(|t| m.tags.contains(t)) { return false; }
                    }
                    true
                });

                memories.truncate(limit as usize);

                Ok(serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category,
                        "tags": m.tags,
                        "memory_type": m.memory_type
                    })).collect::<Vec<_>>()
                }))
            }

            "memory_list" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);
                let namespace = params.get("namespace").and_then(|v| v.as_str());

                let memory_type_filter = params.get("memory_type").and_then(|v| v.as_str());
                let min_importance_filter = params.get("min_importance").and_then(|v| v.as_f64());
                let tags_filter: Option<Vec<String>> = params.get("tags").and_then(|v| {
                    v.as_array().map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
                });

                let fetch_limit = if memory_type_filter.is_some() || tags_filter.is_some() || min_importance_filter.is_some() { limit * 10 } else { limit };

                let mut memories = brain.list_memories(fetch_limit, namespace).await.unwrap_or_default();
                
                memories.retain(|m| {
                    if let Some(ref mt) = memory_type_filter {
                        if &m.memory_type != mt { return false; }
                    }
                    if let Some(ref min_imp) = min_importance_filter {
                        if m.importance < *min_imp { return false; }
                    }
                    if let Some(ref req_tags) = tags_filter {
                        if !req_tags.iter().all(|t| m.tags.contains(t)) { return false; }
                    }
                    true
                });

                memories.truncate(limit as usize);

                Ok(serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category,
                        "tags": m.tags,
                        "memory_type": m.memory_type
                    })).collect::<Vec<_>>()
                }))
            }
            
            "memory_get" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                
                match brain.get_memory(id).await {
                    Ok(m) => Ok(serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance,
                        "category": m.category,
                        "tags": m.tags,
                        "created_at": m.created_at
                    })),
                    Err(e) => Err(e.to_string()),
                }
            }
            
            "memory_delete" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                
                match brain.soft_delete_memory(id).await {
                    Ok(_) => Ok(serde_json::json!({"status": "deleted"})),
                    Err(e) => Err(e.to_string()),
                }
            }
            
            "memory_related" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                let namespace = params.get("namespace").and_then(|v| v.as_str());

                let memories = brain.find_related_memories(id, limit, namespace).await.unwrap_or_default();
                Ok(serde_json::json!({
                    "memories": memories.iter().map(|m| serde_json::json!({
                        "id": m.id,
                        "content": m.content,
                        "importance": m.importance
                    })).collect::<Vec<_>>()
                }))
            }

            "memory_context" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let max_tokens = params.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(4000);
                let min_importance = params.get("min_importance").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32;
                let min_similarity = params.get("min_similarity").and_then(|v| v.as_f64()).unwrap_or(0.4) as f32;
                let namespace = params.get("namespace").and_then(|v| v.as_str());
                let session_id = params.get("session_id").and_then(|v| v.as_str());

                match brain.build_context(query, max_tokens, min_importance, min_similarity, namespace, session_id).await {
                    Ok(ctx) => Ok(ctx),
                    Err(e) => Err(e.to_string()),
                }
            }

            "memory_batch_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                match params.get("memories").and_then(|v| v.as_array()) {
                    None => Err("memories array required".to_string()),
                    Some(memories_vec) => {
                        let mut created = Vec::new();
                        let mut errors = Vec::new();

                        for (i, mem) in memories_vec.iter().enumerate() {
                            let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
                            if content.is_empty() {
                                errors.push(format!("Item {}: content required", i));
                                continue;
                            }

                            let importance = mem.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
                            let valence = mem.get("valence").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                            let arousal = mem.get("arousal").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                            let tags: Vec<String> = mem.get("tags").and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default();
                            let memory_type = mem.get("memory_type").and_then(|v| v.as_str()).unwrap_or("fact");
                            let category = mem.get("category").and_then(|v| v.as_str()).unwrap_or("fact");
                            let namespace = mem.get("namespace").and_then(|v| v.as_str());

                            match brain.create_memory(content, importance, valence, arousal, &tags, memory_type, category, None, namespace).await {
                                Ok(memory) => created.push(serde_json::json!({"id": memory.id, "index": i})),
                                Err(e) => errors.push(format!("Item {}: {}", i, e)),
                            }
                        }

                        Ok(serde_json::json!({
                            "created": created,
                            "errors": errors,
                            "total": memories_vec.len()
                        }))
                    }
                }
            }

            "memory_update" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let id = match params.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => return McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(serde_json::json!({"code": -32602, "message": "id required"})),
                        id: request.id,
                    },
                };

                let content = params.get("content").and_then(|v| v.as_str());
                let importance = params.get("importance").and_then(|v| v.as_f64()).map(|v| v as f32);
                let memory_type = params.get("memory_type").and_then(|v| v.as_str());
                let category = params.get("category").and_then(|v| v.as_str());
                let tags = params.get("tags").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

                match brain.update_memory(id, content, importance, memory_type, category, tags).await {
                    Ok(_) => Ok(serde_json::json!({"status": "updated", "id": id})),
                    Err(e) => Err(e.to_string()),
                }
            }

            "memory_pin" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");

                match brain.set_memory_pinned(id, true).await {
                    Ok(_) => Ok(serde_json::json!({"status": "pinned", "id": id})),
                    Err(e) => Err(e.to_string()),
                }
            }

            "memory_unpin" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");

                match brain.set_memory_pinned(id, false).await {
                    Ok(_) => Ok(serde_json::json!({"status": "unpinned", "id": id})),
                    Err(e) => Err(e.to_string()),
                }
            }

            "memory_set_category" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("");

                match brain.set_memory_category(id, category).await {
                    Ok(_) => Ok(serde_json::json!({"status": "updated", "id": id, "category": category})),
                    Err(e) => Err(e.to_string()),
                }
            }
            
            "session_create" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
                if name.is_empty() {
                    Err("Session name cannot be empty".to_string())
                } else {
                    let agent_id = params.get("agent_id").and_then(|v| v.as_str());
                    let project = params.get("project").and_then(|v| v.as_str());
                    let directory = params.get("directory").and_then(|v| v.as_str());
                    let namespace = params.get("namespace").and_then(|v| v.as_str());
                    
                    match brain.create_session(name, agent_id, project, directory, namespace).await {
                        Ok(id) => Ok(serde_json::json!({"id": id, "status": "created"})),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
            
            "session_list" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;
                let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
                let namespace = params.get("namespace").and_then(|v| v.as_str());

                let sessions = brain.list_sessions(limit, namespace).await.unwrap_or_default();
                Ok(serde_json::json!({
                    "sessions": sessions.iter().map(|s| serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "project": s.project,
                        "directory": s.directory,
                        "started_at": s.started_at,
                        "ended_at": s.ended_at
                    })).collect::<Vec<_>>()
                }))
            }
            
            "stats_get" => {
                let brain = &self.brain;
                match brain.get_stats().await {
                    Ok(stats) => Ok(serde_json::json!({
                        "total_memories": stats.total_memories,
                        "total_sessions": stats.total_sessions,
                        "avg_importance": stats.avg_importance
                    })),
                    Err(e) => Err(e.to_string()),
                }
            }
            
            "prune_run" => {
                let params = request.params.unwrap_or_default();
                let brain = &self.brain;

                let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
                let min_importance = params.get("min_importance").and_then(|v| v.as_f64()).unwrap_or(0.1) as f32;

                match brain.prune_memories(days, min_importance).await {
                    Ok(count) => Ok(serde_json::json!({"pruned": count})),
                    Err(e) => Err(e.to_string()),
                }
            }

            _ => {
                Err(format!("Unknown method: {}", request.method))
            }
        };

        match result {
            Ok(res) => McpResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(res),
                error: None,
                id: request.id,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32603,
                    "message": e
                })),
                id: request.id,
            }
        }
    }
}
