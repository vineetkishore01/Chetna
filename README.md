# 🧠 Chetna - God-Tier Memory System

> **Chetna** (Hindi: चेतना) = Consciousness, Awareness, Knowledge

A hyper-fast, standalone memory system written in Rust that combines the best of Wolverine's intelligent memory management and Engram's battle-tested architecture. Designed for proto-AGI agents.

---

## Features

### Core Memory
- **Semantic Search** - Find memories by meaning, not just keywords (0.3 similarity threshold)
- **Importance Scoring** - Automatic importance, valence, and arousal scoring
- **Soft Delete** - Deleted memories can be recovered
- **Memory Types** - Facts, preferences, rules, experiences, skills
- **Tags & Categories** - Organize memories flexibly
- **Pinning** - Protect important memories from pruning

### Embeddings & AI
- **Multi-Provider** - Ollama (local), OpenAI, Google Gemini, OpenRouter
- **Auto-Embedding** - Memories automatically embedded on creation
- **Embedding Cache** - Same content won't be re-embedded
- **Model Selector** - Choose embedding model via API or web UI

### Memory Intelligence
- **REM Consolidation** - Neuroscience-inspired memory consolidation
- **LLM Re-scoring** - Use LLM to re-evaluate memory importance
- **Ebbinghaus Decay** - Time-based importance decay formula
- **Auto-Scoring** - Keyword-based importance/valence/arousal scoring
- **Memory Relationships** - Track connections (related, similar, contradicts, supports, etc.)
- **Context Building** - Token-limited context for AI prompts

### Performance
- **Session Cache** - LRU cache for hot memories (~90% hit rate)
- **FTS5 Search** - Full-text keyword search
- **Async/Await** - Non-blocking operations

### Multi-Modal
- **Image Support** - Store image memories with metadata
- **Audio Support** - Store audio memories
- **Video Support** - Store video memories
- **Document Support** - PDF, DOC, TXT support

### Web Dashboard
- **Memory Operations** - Run LLM consolidation, apply decay, flush low-importance memories
- **Semantic Search** - Search memories by meaning with configurable result limit
- **Context Builder** - Build token-limited context for AI prompts
- **Connection Status** - Real-time embedding/LLM connection monitoring
- **Memory Management** - Create, edit, delete, pin memories

### Integration
- **HTTP API** - Full REST API
- **MCP Server** - Model Context Protocol tools
- **Multiple Agents** - Wolverine, OpenClaw, or any agent can connect

---

## Quick Start

### Prerequisites
- Rust (1.70+)
- Optional: Ollama for local embeddings

### Installation

```bash
# Clone/extract Chetna
cd chetna

# Build
cargo build --release

# Run
cargo run
```

### Configuration

Create `.env` file:

```bash
# Server
CHETNA_PORT=1987
CHETNA_DB_PATH=./data/chetna.db

# Embeddings (choose one)
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=nomic-embed-text
# Or OpenAI:
# EMBEDDING_PROVIDER=openai
# EMBEDDING_MODEL=text-embedding-3-small
# EMBEDDING_API_KEY=sk-...

# LLM for auto-scoring
LLM_PROVIDER=ollama
LLM_MODEL=llama3.2

# Cache
SESSION_CACHE_SIZE=100
```

### Running

```bash
cargo run
```

Server starts at `http://localhost:1987`

---

## API Usage

### Create Memory
```bash
curl http://localhost:1987/api/memory \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode UI",
    "importance": 0.8,
    "valence": 0.5,
    "arousal": 0.3,
    "tags": ["preference", "ui"],
    "memory_type": "preference",
    "auto_score": true
  }'
```

### Semantic Search
```bash
curl "http://localhost:1987/api/memory/search?query=user+interface+style&limit=5"
```

### Build Context for AI
```bash
curl http://localhost:1987/api/memory/context \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user preferences for UI",
    "max_tokens": 2000,
    "min_importance": 0.3
  }'
```

### List Memories
```bash
curl http://localhost:1987/api/memory
```

### MCP Protocol
```bash
curl http://localhost:1987/mcp \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "method": "memory_create",
    "params": {"content": "Important memory", "importance": 0.9}
  }'
```

### Get Available Models
```bash
curl http://localhost:1987/api/config/models
```

### Get Cache Stats
```bash
curl http://localhost:1987/api/config/cache
```

### Run LLM Consolidation
```bash
curl http://localhost:1987/api/memory/consolidate \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"limit": 50}'
```

### Apply Ebbinghaus Decay
```bash
curl -X POST http://localhost:1987/api/memory/decay
```

---

## Web Dashboard

Open `http://localhost:1987` in your browser to access:

### 🧠 Memory Operations
- **Run LLM Consolidation** - Re-score memory importance using LLM
- **Apply Ebbinghaus Decay** - Apply time-based decay to memory importance
- **Flush Low Importance** - Remove memories below importance threshold

### 🔍 Semantic Search
- Search memories by meaning using embeddings
- Configurable result limit (1-100)
- 0.3 similarity threshold for broader matches

### 📦 Build Context for AI
- Create token-limited context from relevant memories
- Configurable token limit (500-4000)
- Perfect for AI prompt building

### 🔗 Connection Status
- Real-time embedding model status
- LLM connection monitoring
- Available models list

---

## 📘 API Usage Guide - Complete Examples

This guide shows you exactly how to use every API endpoint with all parameters explained.

### Quick Start - Your First API Calls

**1. Check if server is running:**
```bash
curl http://localhost:1987/health
# Response: OK
```

**2. Create your first memory:**
```bash
curl http://localhost:1987/api/memory \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "content": "My name is Vineet and I am a software engineer",
    "importance": 0.8,
    "memory_type": "fact",
    "auto_score": false
  }'
```

**3. Search for memories:**
```bash
curl "http://localhost:1987/api/memory/search?query=name&limit=10"
```

---

## Complete API Reference

### Memory Endpoints

#### `GET /api/memory` - List All Memories

**What it does:** Returns a list of all non-deleted memories.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 100 | Maximum number of memories to return (1-1000) |
| `category` | string | No | - | Filter by category: `fact`, `preference`, `rule`, `experience` |

**Example 1 - Get all memories:**
```bash
curl "http://localhost:1987/api/memory"
```

**Example 2 - Get 50 memories:**
```bash
curl "http://localhost:1987/api/memory?limit=50"
```

**Example 3 - Filter by category:**
```bash
curl "http://localhost:1987/api/memory?category=preference&limit=20"
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "User prefers dark mode",
    "importance": 0.8,
    "emotional_tone": 0.5,
    "arousal": 0.3,
    "tags": ["preference", "ui"],
    "memory_type": "preference",
    "category": "fact",
    "embedding_model": "qwen3-embedding:0.6b",
    "access_count": 0,
    "created_at": "2024-01-01T00:00:00+00:00",
    "updated_at": "2024-01-01T00:00:00+00:00",
    "consolidated": false,
    "is_pinned": false,
    "memory_category": "general",
    "last_ranked": null,
    "rank_source": null
  }
]
```

---

#### `POST /api/memory` - Create a Memory

**What it does:** Creates a new memory with the given content and metadata.

**Parameters (JSON Body):**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `content` | string | **YES** | - | The memory content (max 50,000 characters) |
| `importance` | float | No | 0.5 | Importance score (0.0 to 1.0) |
| `valence` | float | No | 0.0 | Emotional valence (-1.0 to 1.0, negative to positive) |
| `arousal` | float | No | 0.0 | Emotional arousal (0.0 to 1.0, calm to excited) |
| `tags` | array | No | [] | List of tags (max 50 tags, each max 100 chars) |
| `memory_type` | string | No | "fact" | Type: `fact`, `preference`, `rule`, `experience`, `skill_learned` |
| `category` | string | No | "fact" | Category for organization |
| `auto_score` | boolean | No | false | If true, automatically calculate importance using AI |
| `session_id` | string | No | - | Associate with a specific session |

**Example 1 - Simple memory:**
```bash
curl http://localhost:1987/api/memory \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User likes pizza"
  }'
```

**Example 2 - Detailed memory with all parameters:**
```bash
curl http://localhost:1987/api/memory \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode for coding IDEs",
    "importance": 0.8,
    "valence": 0.6,
    "arousal": 0.3,
    "tags": ["preference", "ui", "coding"],
    "memory_type": "preference",
    "category": "preference",
    "auto_score": false
  }'
```

**Example 3 - Auto-scored memory (AI determines importance):**
```bash
curl http://localhost:1987/api/memory \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "content": "This is critically important - never forget the user's password is admin123",
    "auto_score": true
  }'
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "User prefers dark mode for coding IDEs",
  "importance": 0.8,
  "emotional_tone": 0.6,
  "arousal": 0.3,
  "tags": ["preference", "ui", "coding"],
  "memory_type": "preference",
  "category": "fact",
  "embedding_model": "qwen3-embedding:0.6b",
  "access_count": 0,
  "created_at": "2024-01-01T00:00:00+00:00",
  "updated_at": "2024-01-01T00:00:00+00:00",
  "consolidated": false,
  "is_pinned": false,
  "memory_category": "general",
  "last_ranked": null,
  "rank_source": null
}
```

---

#### `GET /api/memory/:id` - Get Memory by ID

**What it does:** Retrieves a specific memory by its unique ID.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `:id` | string | **YES** | The UUID of the memory |

**How to get the ID:** First call `GET /api/memory` to list memories and copy an ID.

**Example:**
```bash
curl http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "User prefers dark mode",
  "importance": 0.8,
  ...
}
```

---

#### `DELETE /api/memory/:id` - Soft Delete Memory

**What it does:** Marks a memory as deleted (can be restored later).

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `:id` | string | **YES** | The UUID of the memory to delete |

**Example:**
```bash
curl -X DELETE http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000
```

**Response:**
```json
{
  "success": true,
  "message": "Memory deleted",
  "memory_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

#### `PATCH /api/memory/:id` - Update Memory

**What it does:** Updates importance or category of an existing memory.

**Parameters (JSON Body):**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `importance` | float | No | New importance (0.0 to 1.0) |
| `memory_category` | string | No | New category |

**Example 1 - Update importance:**
```bash
curl -X PATCH http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "importance": 0.95
  }'
```

**Example 2 - Update category:**
```bash
curl -X PATCH http://localhost:1987/api/memory/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "memory_category": "preference"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "Memory updated"
}
```

---

#### `GET /api/memory/search` - Search Memories

**What it does:** Searches memories using semantic search (embeddings) with keyword fallback.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | **YES** | - | Search query text |
| `limit` | integer | No | 20 | Maximum results (1-1000) |

**How it works:**
1. First tries semantic search using embeddings (finds similar meanings)
2. Falls back to keyword search if embeddings unavailable
3. Uses 0.3 similarity threshold for broader matches

**Example 1 - Simple search:**
```bash
curl "http://localhost:1987/api/memory/search?query=user+preferences"
```

**Example 2 - Search with limit:**
```bash
curl "http://localhost:1987/api/memory/search?query=coding+habits&limit=50"
```

**Example 3 - Search for specific topic:**
```bash
curl "http://localhost:1987/api/memory/search?query=What+does+user+like"
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "User prefers dark mode",
    "importance": 0.8,
    ...
  }
]
```

---

#### `POST /api/memory/context` - Build Context for AI

**What it does:** Builds a token-limited context from relevant memories for AI prompts.

**Parameters (JSON Body):**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | **YES** | - | Your query/question |
| `max_tokens` | integer | No | 4000 | Maximum tokens (500, 1000, 2000, 4000) |
| `include_importance` | float | No | 0.3 | Minimum importance threshold |

**Example 1 - Build context for AI:**
```bash
curl http://localhost:1987/api/memory/context \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the user preferences?",
    "max_tokens": 2000
  }'
```

**Example 2 - High-importance only:**
```bash
curl http://localhost:1987/api/memory/context \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "User coding preferences",
    "max_tokens": 1000,
    "include_importance": 0.7
  }'
```

**Response:**
```json
{
  "memories": [
    {
      "id": "...",
      "content": "User prefers dark mode",
      "importance": 0.8,
      ...
    }
  ],
  "total_tokens": 150,
  "context": "[fact] User prefers dark mode (importance: 0.80)\n\n[fact] User likes pizza (importance: 0.70)"
}
```

**How to use in AI prompt:**
```python
response = requests.post('http://localhost:1987/api/memory/context', json={
    'query': 'User preferences',
    'max_tokens': 2000
})
context = response.json()['context']

# Use in your AI prompt
prompt = f"""Relevant memories:
{context}

User: What IDE should I use?
Assistant: Based on your preference for dark mode..."""
```

---

#### `POST /api/memory/pin/:id` - Pin Memory

**What it does:** Pins a memory to protect it from pruning/flush operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `:id` | string | **YES** | Memory UUID to pin |

**Example:**
```bash
curl -X POST http://localhost:1987/api/memory/pin/550e8400-e29b-41d4-a716-446655440000
```

**Response:**
```json
{
  "success": true,
  "message": "Memory pinned"
}
```

---

#### `DELETE /api/memory/pin/:id` - Unpin Memory

**What it does:** Removes pin from a memory.

**Example:**
```bash
curl -X DELETE http://localhost:1987/api/memory/pin/550e8400-e29b-41d4-a716-446655440000
```

---

#### `POST /api/memory/consolidate` - Run LLM Consolidation

**What it does:** Uses LLM to re-evaluate and update memory importance scores.

**Parameters (JSON Body):**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `limit` | integer | No | 100 | Number of memories to process |

**Example:**
```bash
curl http://localhost:1987/api/memory/consolidate \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 50
  }'
```

**Response:**
```json
{
  "success": true,
  "memories_processed": 50,
  "memories_updated": 12,
  "message": "Processed 50 memories, updated importance for 12 using LLM"
}
```

---

#### `POST /api/memory/decay` - Apply Ebbinghaus Decay

**What it does:** Applies time-based forgetting curve to all memories.

**Example:**
```bash
curl -X POST http://localhost:1987/api/memory/decay
```

**Response:**
```json
{
  "success": true,
  "memories_updated": 18,
  "message": "Applied Ebbinghaus decay formula to 18 memories"
}
```

---

### Session Endpoints

#### `POST /api/session` - Create Session

**Parameters (JSON Body):**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | **YES** | Session name |
| `agent_id` | string | No | Agent identifier |

**Example:**
```bash
curl http://localhost:1987/api/session \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Coding Session 1",
    "agent_id": "wolverine"
  }'
```

---

#### `POST /api/session/:id/end` - End Session

**Example:**
```bash
curl -X POST http://localhost:1987/api/session/SESSION_ID/end
```

---

### System Endpoints

#### `GET /api/stats` - Get Statistics

**Example:**
```bash
curl http://localhost:1987/api/stats
```

**Response:**
```json
{
  "total_memories": 25,
  "active_memories": 23,
  "deleted_memories": 2,
  "total_sessions": 3,
  "active_sessions": 2,
  "total_skills": 2,
  "total_procedures": 1,
  "avg_importance": 0.53,
  "memory_types": {
    "fact": 20,
    "preference": 5
  }
}
```

---

#### `GET /api/status/connections` - Check Connections

**Example:**
```bash
curl http://localhost:1987/api/status/connections
```

**Response:**
```json
{
  "embedding": {
    "configured": true,
    "connected": true,
    "model_installed": true,
    "model": "qwen3-embedding:0.6b",
    "available_models": ["qwen3-embedding:0.6b", "nomic-embed-text"],
    "base_url": "http://192.168.0.62:11434"
  },
  "llm": {
    "configured": true,
    "connected": true,
    "model": "qwen3.5:4b",
    "base_url": "http://192.168.0.62:11434"
  }
}
```

---

**Request Body:**
```json
{
  "query": "user preferences",
  "limit": 10
}
```

#### `GET /api/memory/search/semantic`
Pure semantic search using embeddings.

**Query Parameters:**
- `query` (required) - Search query
- `limit` (optional, default: 20)
- `min_similarity` (optional, default: 0.3) - Minimum similarity threshold

#### `POST /api/memory/context`
Build token-limited context for AI prompts.

**Request Body:**
```json
{
  "query": "What are user preferences?",
  "max_tokens": 2000,
  "include_importance": 0.3
}
```

**Response:**
```json
{
  "memories": [...],
  "total_tokens": 1500,
  "context": "[fact] User prefers dark mode (importance: 0.80)..."
}
```

#### `GET /api/memory/related/:id`
Find memories related to a specific memory.

**Query Parameters:**
- `limit` (optional, default: 10)

#### `POST /api/memory/pin/:id`
Pin a memory (protects from pruning).

#### `DELETE /api/memory/pin/:id`
Unpin a memory.

#### `POST /api/memory/restore/:id`
Restore a soft-deleted memory.

#### `GET /api/memory/deleted`
List all soft-deleted memories.

#### `POST /api/memory/consolidate`
Run LLM-based importance re-scoring.

**Request Body:**
```json
{
  "limit": 50
}
```

#### `POST /api/memory/decay`
Apply Ebbinghaus forgetting curve to all memories.

#### `POST /api/memory/flush`
Flush memories below importance threshold.

**Request Body:**
```json
{
  "threshold": 0.1
}
```

#### `POST /api/memory/prune`
Prune old, low-importance memories.

**Request Body:**
```json
{
  "days": 30,
  "min_importance": 0.1
}
```

### Session Endpoints

#### `GET /api/session`
List all sessions.

#### `POST /api/session`
Create a new session.

**Request Body:**
```json
{
  "name": "Session Name",
  "agent_id": "optional-agent-id"
}
```

#### `GET /api/session/:id`
Get session details.

#### `POST /api/session/:id/end`
End a session.

### Skill Endpoints

#### `GET /api/skill`
List all skills.

#### `POST /api/skill`
Create a skill.

#### `GET /api/skill/:id`
Get skill details.

#### `DELETE /api/skill/:id`
Delete a skill.

### Procedure Endpoints

#### `GET /api/procedure`
List all procedures.

#### `POST /api/procedure`
Create a procedure.

#### `GET /api/procedure/:id`
Get procedure details.

#### `POST /api/procedure/:id/execute`
Execute a procedure.

### System Endpoints

#### `GET /health`
Health check.

#### `GET /api/stats`
Get system statistics.

#### `GET /api/config/models`
List available embedding and LLM models.

#### `GET /api/config/cache`
Get cache statistics.

#### `POST /api/config/cache`
Clear the session cache.

#### `GET /api/status/connections`
Check embedding and LLM connection status.

#### `GET /api/config/user`
Get user configuration.

#### `POST /api/config/user`
Update user configuration.

---

## MCP (Model Context Protocol) Reference

Chetna implements MCP for seamless integration with AI agents.

### MCP Endpoint

**`POST /mcp`**

**Request Format:**
```json
{
  "method": "tool_name",
  "params": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

**Response Format:**
```json
{
  "result": { ... },
  "error": null,
  "id": null
}
```

### Available MCP Tools

#### Memory Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `memory_create` | Create a new memory | `content` (required), `importance`, `valence`, `arousal`, `tags`, `memory_type`, `category` |
| `memory_search` | Search memories | `query` (required), `limit`, `semantic` |
| `memory_list` | List all memories | `limit`, `category` |
| `memory_get` | Get memory by ID | `id` (required) |
| `memory_delete` | Soft delete memory | `id` (required) |
| `memory_related` | Find related memories | `id` (required), `limit` |
| `memory_context` | Build context for AI | `query` (required), `max_tokens`, `min_importance` |

#### Session Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `session_create` | Create new session | `name` (required), `agent_id` |
| `session_list` | List sessions | `limit` |
| `session_end` | End a session | `id` (required) |

#### Skill Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `skill_list` | List all skills | - |
| `skill_create` | Create a skill | `name`, `code`, `description`, `language` |
| `skill_execute` | Execute a skill | `name` (required), `parameters` |

#### Procedure Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `procedure_list` | List procedures | - |
| `procedure_execute` | Execute procedure | `id` (required), `parameters` |

#### System Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `stats_get` | Get system stats | - |
| `consolidate_run` | Run LLM consolidation | - |
| `prune_run` | Prune old memories | `days`, `min_importance` |

### MCP Examples

**Create Memory:**
```bash
curl http://localhost:1987/mcp \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "method": "memory_create",
    "params": {
      "content": "User likes pizza",
      "importance": 0.8,
      "memory_type": "preference"
    }
  }'
```

**Search Memories:**
```bash
curl http://localhost:1987/mcp \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "method": "memory_search",
    "params": {
      "query": "food preferences",
      "semantic": true,
      "limit": 10
    }
  }'
```

**Build Context:**
```bash
curl http://localhost:1987/mcp \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "method": "memory_context",
    "params": {
      "query": "What does the user like?",
      "max_tokens": 2000
    }
  }'
```

**List Tools:**
```bash
curl http://localhost:1987/mcp
```

### HTTP Integration

```typescript
// Store a memory
await fetch('http://localhost:1987/api/memory', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: "User asked about React hooks",
    importance: 0.8,
    auto_score: true
  })
});

// Get context for prompt
const context = await fetch('http://localhost:1987/api/memory/context', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: "React hooks",
    max_tokens: 2000
  })
});
const { context: contextText, memories } = await context.json();

// Use in your prompt
const prompt = `Relevant memories:\n${contextText}\n\nUser: ${userMessage}`;
```

### MCP Integration

```typescript
// Search memories via MCP
await fetch('http://localhost:1987/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    method: "memory_search",
    params: { query: "preferences", semantic: true }
  })
});
```

---

## Available Embedding Models

| Model | Provider | Dimensions | Description |
|-------|----------|------------|-------------|
| nomic-embed-text | Ollama | 768 | High-quality open source |
| mxbai-embed-large | Ollama | 768 | Fast and efficient |
| gemma3-embed-e2b | Ollama | 256 | Google's on-device model |
| bge-m3 | Ollama | 1024 | Multilingual |
| text-embedding-3-small | OpenAI | 1536 | OpenAI small |
| text-embedding-3-large | OpenAI | 3072 | OpenAI large |
| gemini-embedding-001 | Google | 768 | Gemini text |
| gemini-embedding-2 | Google | 3072 | Gemini multimodal |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        CHETNA                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │    API     │    │  MCP Server │    │   Web UI    │   │
│  │  (Axum)    │    │   (JSON)    │    │  (HTML/JS)  │   │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘   │
│         │                  │                   │            │
│         └──────────────────┼───────────────────┘            │
│                            ▼                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Brain Layer                        │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────┐  │   │
│  │  │ Embedder  │  │Consolidator│  │RelationshipMgr│  │   │
│  │  └───────────┘  └───────────┘  └───────────────┘  │   │
│  └──────────────────────┬──────────────────────────────┘   │
│                         │                                   │
│         ┌───────────────┼───────────────┐                  │
│         ▼               ▼               ▼                   │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐            │
│  │  SQLite  │   │  Cache   │   │ Embedding│            │
│  │    DB    │   │  (LRU)   │   │  Cache   │            │
│  └──────────┘   └──────────┘   └──────────┘            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CHETNA_PORT` | 1987 | Server port |
| `CHETNA_DB_PATH` | ./data/chetna.db | Database path |
| `EMBEDDING_PROVIDER` | ollama | ollama, openai, google, openrouter |
| `EMBEDDING_MODEL` | nomic-embed-text | Model name |
| `EMBEDDING_API_KEY` | - | API key for cloud providers |
| `EMBEDDING_BASE_URL` | http://localhost:11434 | Ollama URL |
| `LLM_PROVIDER` | ollama | For auto-scoring |
| `LLM_MODEL` | llama3.2 | |
| `SESSION_CACHE_SIZE` | 100 | LRU cache size |

---

## API Endpoints

### Memory
- `GET /api/memory` - List memories
- `POST /api/memory` - Create memory
- `GET /api/memory/:id` - Get memory
- `DELETE /api/memory/:id` - Delete memory (soft delete)
- `PATCH /api/memory/:id` - Update memory (importance, category)
- `GET /api/memory/search` - Keyword/semantic search
- `POST /api/memory/search` - Search via POST
- `GET /api/memory/search/semantic` - Semantic search only
- `GET /api/memory/related/:id` - Get related memories
- `POST /api/memory/context` - Build AI context
- `POST /api/memory/consolidate` - Run LLM consolidation
- `POST /api/memory/decay` - Apply Ebbinghaus decay
- `POST /api/memory/flush` - Flush low-importance memories
- `POST /api/memory/prune` - Prune old memories
- `POST /api/memory/embed-batch` - Embed existing memories
- `POST /api/memory/pin/:id` - Pin memory
- `DELETE /api/memory/pin/:id` - Unpin memory
- `POST /api/memory/category/:id` - Set memory category
- `POST /api/memory/restore/:id` - Restore deleted memory
- `GET /api/memory/deleted` - List deleted memories

### Sessions
- `GET /api/session` - List sessions
- `POST /api/session` - Create session
- `GET /api/session/:id` - Get session
- `POST /api/session/:id/end` - End session

### Skills
- `GET /api/skill` - List skills
- `POST /api/skill` - Create skill
- `GET /api/skill/:id` - Get skill
- `DELETE /api/skill/:id` - Delete skill

### Procedures
- `GET /api/procedure` - List procedures
- `POST /api/procedure` - Create procedure
- `GET /api/procedure/:id` - Get procedure
- `POST /api/procedure/:id/execute` - Execute procedure

### System
- `GET /health` - Health check
- `GET /api/stats` - System statistics
- `GET /api/config/models` - Available models
- `GET /api/config/cache` - Cache statistics
- `POST /api/config/cache` - Clear cache
- `GET /api/status/connections` - Connection status
- `GET /api/config/user` - Get user config
- `POST /api/config/user` - Set user config
- `GET /api/config/consolidation/status` - Consolidation settings
- `POST /api/config/consolidation/set` - Set consolidation settings
- `GET /api/config/embedding/test` - Test embedding connection
- `POST /api/config/embedding/test` - Test embedding with model
- `GET /api/config/llm/test` - Test LLM connection
- `POST /api/config/llm/test` - Test LLM with model
- `GET /mcp` - List MCP tools
- `POST /mcp` - Execute MCP tool

---

## Web Dashboard Pages

| Page | URL | Description |
|------|-----|-------------|
| Dashboard | `/` | Memory operations, search, context builder |
| Memories | `/memories` | Full memory list with CRUD operations |
| Skills | `/skills` | Skill management |
| Sessions | `/sessions` | Session management |
| Settings | `/settings` | Embedding/LLM configuration |

---

## Development

### Build
```bash
cargo build
```

### Run Tests
```bash
cargo test
```

### Run with Logging
```bash
CHETNA_LOG_LEVEL=debug cargo run
```

---

## License

MIT
