# Chetna API Reference

Chetna runs on port `1987` by default. All API endpoints are prefixed with `/api`.

## Authentication
If `CHETNA_API_KEY` is set in the environment, all requests must include:
`Authorization: Bearer your_key_here`

---

## Memory Operations

### `POST /api/memory`
Store a new piece of information.

**Request Body:**
```json
{
  "content": "User prefers dark mode for all dashboards.",
  "importance": 0.9,
  "category": "preference",
  "tags": ["ui", "config"]
}
```

### `GET /api/memory/search`
Find memories using Hybrid Search (combines meaning + exact keywords).

**Query Parameters:**
- `query`: Your search term (e.g., "ui preferences")
- `limit`: Number of results to return (default: 20)
- `namespace`: Filter by namespace (default: "default")

### `POST /api/memory/context`
The most powerful endpoint. Automatically builds a ranked context block that you can copy-paste directly into an AI prompt.

**Request Body:**
```json
{
  "query": "What are the user's UI preferences?",
  "max_tokens": 1000
}
```

**Response:**
```json
{
  "context": "[preference] User prefers dark mode for all dashboards. (importance: 0.90)",
  "total_tokens": 15
}
```

---

## Session Operations

### `POST /api/session`
Create a new active session for an agent. This helps track what the agent is currently working on.

**Request Body:**
```json
{
  "name": "Refactoring the API",
  "agent_id": "coding-assistant-v1"
}
```

---

## System Status

### `GET /health`
Returns the status of the server and its database connection.

### `GET /api/status/connections`
Returns detailed information about your embedding provider (Ollama, OpenAI, etc.).

---

## Advanced Mechanics

### 🧬 Hybrid Search (RRF)
Chetna uses **Reciprocal Rank Fusion**. It doesn't just find "similar" things; it finds exactly what you asked for. If you search for an exact Git hash, Chetna will find it even if it's "conceptually" different from other code.

### 📉 Biological Decay
Memories have an **Importance** score. Every 6 hours, Chetna runs a maintenance cycle:
1. Memories with low importance lose a small percentage of their score.
2. If a memory's importance falls below `0.1`, it is automatically deleted.
3. **Active Recall:** Searching for or accessing a memory resets its decay timer and increases its "Stability".

### 🧩 Automatic Chunking
If you post a very long memory (> 3000 characters), Chetna will:
1. Summarize the content.
2. Split it into smaller, overlapping "chunks".
3. Link them together so your agent can navigate the whole document logically.

### 📜 History & Analytics
Chetna now includes comprehensive history logging and analytics:

- **History Events**: Track all memory operations and queries
- **Query Results**: Log search results with similarity and recall scores
- **Analytics Dashboard**: View query patterns, most accessed memories, performance metrics
- **30-Day Retention**: Automatic cleanup of old history events

### 🚀 Performance Optimizations
Chetna includes several performance optimizations:

- **Query Caching**: Cache query embeddings to avoid repeated generation (100-500ms savings)
- **Connection Pooling**: Pool of 10 SQLite connections for concurrent queries
- **HNSW Index**: O(log n) search complexity with 1000-2000× speedup
- **Vector Pooling**: Pool of 100 vector buffers to reduce allocation overhead
- **Parallel Processing**: Parallel batch processing for large datasets
- **Async Logging**: Background task for event persistence with <1ms overhead

### 📊 Performance Metrics
| Metric | Value | Notes |
|--------|-------|-------|
| Query Cache Hit | <1ms | 100-500× faster |
| Query Cache Miss | <10ms | With optimizations |
| Memory Creation | <50ms | Average time |
| Context Building | <100ms | With optimizations |
| History Logging | <1ms | Async background |
| Total Query | 60-215ms | 10-12× faster |

---

## History Endpoints

### `GET /api/history`
List history events with filters.

**Query Parameters:**
- `event_type`: Filter by event type (memory_created, query_searched, context_built)
- `namespace`: Filter by namespace
- `session_id`: Filter by session ID
- `limit`: Maximum results (default: 50)
- `offset`: Pagination offset

### `GET /api/history/{id}`
Get detailed information about a specific event including query results.

### `GET /api/history/analytics`
Get analytics for a time range.

**Query Parameters:**
- `days`: Number of days to analyze (default: 30)

### `GET /api/history/cleanup`
Cleanup old history events.

**Query Parameters:**
- `days`: Delete events older than this many days (default: 30)

---

## MCP Protocol

Chetna supports the Model Context Protocol (MCP) for AI agent integration. Available tools include:

- `memory_create` - Create a memory
- `memory_search` - Search memories
- `memory_get` - Get memory by ID
- `memory_list` - List memories
- `memory_update` - Update a memory
- `memory_delete` - Delete a memory
- `memory_related` - Find related memories
- `memory_context` - Build context for AI
- `memory_batch_create` - Create multiple memories
- `memory_pin` - Pin a memory
- `memory_unpin` - Unpin a memory
- `memory_set_category` - Set memory category
- `session_create` - Create a session
- `session_list` - List sessions
- `session_end` - End a session
- `skill_list` - List skills
- `skill_create` - Create a skill
- `skill_execute` - Execute a skill
- `procedure_list` - List procedures
- `procedure_execute` - Execute a procedure
- `stats_get` - Get statistics
- `history_list` - List history events
- `history_analytics` - Get analytics
- `history_cleanup` - Cleanup old history

**Example:**
```bash
curl -X POST http://localhost:1987/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "id": 1
  }'
```

---

## Web Dashboard

Chetna includes a beautiful web dashboard at `http://localhost:1987` with:

- **Dashboard**: Overview with statistics and quick search
- **Memories**: Browse and manage all memories
- **Sessions**: View and manage sessions
- **History**: Timeline of all operations with analytics
- **Settings**: Configure embedding provider and model

---

## Error Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request |
| 401 | Unauthorized |
| 404 | Not Found |
| 500 | Internal Server Error |
| -32602 | Invalid params |
| -32603 | Internal error |

---

**Last Updated**: 2026-03-28
**Version**: 0.5.0