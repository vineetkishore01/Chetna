# API Reference

Complete REST API documentation for Chetna.

---

## Base URL

```
http://localhost:1987
```

## Authentication

If `CHETNA_API_KEY` is configured, include it in requests:

```bash
# Via header (recommended)
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:1987/api/memory

# Via query parameter
curl "http://localhost:1987/api/memory?api_key=YOUR_API_KEY"
```

---

## Endpoints

### Health Check

#### GET /health

Check if the server is running.

**Response:**
```
OK
```

---

## Memories

### GET /api/memory

List all memories.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 100 | Maximum memories to return |
| `category` | string | all | Filter by category |
| `memory_type` | string | all | Filter by memory type |

**Example:**
```bash
curl "http://localhost:1987/api/memory?limit=10&memory_type=fact"
```

**Response:**
```json
[
  {
    "id": "uuid",
    "content": "Memory content",
    "importance": 0.85,
    "emotional_tone": 0.5,
    "arousal": 0.3,
    "tags": ["tag1", "tag2"],
    "memory_type": "fact",
    "category": "fact",
    "embedding_model": "qwen3-embedding:4b",
    "access_count": 5,
    "created_at": "2026-03-14T20:00:00Z",
    "updated_at": "2026-03-14T20:00:00Z",
    "is_pinned": false
  }
]
```

---

### POST /api/memory

Create a new memory.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | The memory content |
| `importance` | float | No | 0.0-1.0 (default: 0.5) |
| `valence` | float | No | -1.0 to 1.0 (emotional tone) |
| `arousal` | float | No | 0.0-1.0 (emotional intensity) |
| `memory_type` | string | No | fact, preference, rule, experience, skill_learned |
| `category` | string | No | Category name |
| `tags` | array | No | List of tags |
| `session_id` | string | No | Associated session |
| `auto_score` | boolean | No | Auto-calculate importance using LLM |

**Example:**
```bash
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode",
    "importance": 0.85,
    "memory_type": "preference",
    "tags": ["ui", "preference"]
  }'
```

**Response:**
```json
{
  "id": "uuid",
  "content": "User prefers dark mode",
  "importance": 0.85,
  "memory_type": "preference",
  ...
}
```

---

### GET /api/memory/:id

Get a specific memory by ID.

**Example:**
```bash
curl http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000
```

---

### PATCH /api/memory/:id

Update a memory.

**Request Body:**
```json
{
  "content": "Updated content",
  "importance": 0.9,
  "memory_type": "preference"
}
```

---

### DELETE /api/memory/:id

Soft delete a memory (can be restored).

**Example:**
```bash
curl -X DELETE http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000
```

**Response:**
```json
{
  "success": true,
  "message": "Memory deleted",
  "memory_id": "uuid"
}
```

---

### POST /api/memory/restore/:id

Restore a deleted memory.

---

### GET /api/memory/deleted

List deleted (soft-deleted) memories.

---

### POST /api/memory/batch

Create multiple memories at once.

**Request Body:**
```json
{
  "memories": [
    {"content": "Memory 1", "importance": 0.5},
    {"content": "Memory 2", "importance": 0.7},
    {"content": "Memory 3", "importance": 0.9}
  ]
}
```

**Response:**
```json
{
  "created": [...],
  "failed": [...],
  "total": 3,
  "success_count": 3,
  "failure_count": 0
}
```

---

## Search

### GET /api/memory/search

Search memories (keyword + semantic fallback).

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | string | Search query |
| `limit` | integer | Max results (default: 20) |

**Example:**
```bash
curl "http://localhost:1987/api/memory/search?query=user+preferences&limit=10"
```

---

### GET /api/memory/search/semantic

Semantic search using embeddings.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `query` | string | - | Search query |
| `limit` | integer | 20 | Max results |
| `min_similarity` | float | 0.3 | Minimum similarity (0.0-1.0) |

**Example:**
```bash
curl "http://127.0.0.1:1987/api/memory/search/semantic?query=who+owns+me&limit=3&min_similarity=0.3"
```

---

### GET /api/memory/related/:id

Find related memories to a given memory.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 10 | Max results |

**Example:**
```bash
curl "http://localhost:1987/api/memory/related/550e8400-e29b-41d4-a716-446655440000?limit=5"
```

---

## Context

### POST /api/memory/context

Build AI context from relevant memories.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `query` | string | Yes | The context query |
| `max_tokens` | integer | No | Token limit (default: 4000) |
| `min_importance` | float | No | Minimum importance to include |

**Example:**
```bash
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What do you know about the user?",
    "max_tokens": 500
  }'
```

**Response:**
```json
{
  "context": "[fact] My name is Wolverine...\n\n[preference] User prefers dark mode...",
  "memories": [...],
  "total_tokens": 450,
  "query": "What do you know about the user?"
}
```

---

## Memory Operations

### POST /api/memory/pin/:id

Pin a memory (never decays).

### DELETE /api/memory/pin/:id

Unpin a memory.

### POST /api/memory/consolidate

Run LLM consolidation.

**Request Body:**
```json
{
  "limit": 50
}
```

### POST /api/memory/decay

Apply Ebbinghaus decay formula.

### POST /api/memory/flush

Flush low-importance memories.

**Request Body:**
```json
{
  "threshold": 0.1
}
```

### POST /api/memory/embed-batch

Embed all memories without embeddings.

---

## Sessions

### GET /api/session

List all sessions.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Max results |

### POST /api/session

Create a new session.

**Request Body:**
```json
{
  "name": "Coding Session",
  "agent_id": "wolverine"
}
```

### POST /api/session/:id/end

End a session.

### DELETE /api/session/:id

Delete a session.

---

## Skills

### GET /api/skill

List all skills.

### POST /api/skill

Create a new skill.

**Request Body:**
```json
{
  "name": "greet_user",
  "description": "Greets the user",
  "code": "def greet(name): return f'Hello, {name}!'",
  "language": "python"
}
```

### DELETE /api/skill/:id

Delete a skill.

---

## Procedures

### GET /api/procedure

List all procedures.

### POST /api/procedure

Create a new procedure.

**Request Body:**
```json
{
  "name": "deploy_app",
  "description": "Deploy the application",
  "steps": ["git pull", "docker build", "docker push"]
}
```

### POST /api/procedure/:id/execute

Execute a procedure.

---

## System

### GET /api/stats

Get system statistics.

**Response:**
```json
{
  "total_memories": 100,
  "active_memories": 95,
  "deleted_memories": 5,
  "total_sessions": 10,
  "active_sessions": 2,
  "total_skills": 5,
  "total_procedures": 3,
  "avg_importance": 0.65,
  "memory_types": {
    "fact": 50,
    "preference": 30,
    "rule": 15,
    "experience": 3,
    "skill_learned": 2
  }
}
```

---

### GET /api/status/connections

Check Ollama connection status.

**Response:**
```json
{
  "embedding": {
    "base_url": "http://localhost:11434",
    "model": "qwen3-embedding:4b",
    "configured": true,
    "connected": true,
    "model_installed": true,
    "available_models": ["qwen3-embedding:4b", "nomic-embed-text"]
  },
  "llm": {
    "base_url": "http://localhost:11434",
    "model": "qwen3.5:4b",
    "configured": true,
    "connected": true
  }
}
```

---

### GET /api/config/models

List available Ollama models.

---

### GET /api/config/user

Get user configuration.

### POST /api/config/user

Update user configuration.

**Request Body:**
```json
{
  "embedding_model": "qwen3-embedding:4b",
  "llm_model": "qwen3.5:4b",
  "consolidation_interval_hours": 6,
  "auto_decay_enabled": true,
  "auto_flush_enabled": true,
  "min_importance_threshold": 0.1
}
```

---

### GET /api/config/cache

Get cache statistics.

### POST /api/config/cache

Clear the session cache.

---

## Error Responses

All endpoints may return error responses:

### 400 Bad Request
```json
{
  "error": "Invalid request parameters"
}
```

### 401 Unauthorized
```json
{
  "error": "Invalid or missing API key"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```
