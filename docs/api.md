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
