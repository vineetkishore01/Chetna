# MCP Protocol Reference

Chetna supports the Model Context Protocol (MCP) for AI agent integration. This allows AI agents to interact with Chetna's memory system using a standardized protocol.

---

## Endpoint

```
POST http://localhost:1987/mcp
```

---

## Protocol Format

### Request

```json
{
  "method": "tool_name",
  "params": {
    "param1": "value1",
    "param2": "value2"
  },
  "id": "optional-request-id"
}
```

### Response

```json
{
  "result": { ... },
  "error": null,
  "id": "optional-request-id"
}
```

---

## Available Tools

### Memory Tools

#### memory_create

Create a new memory.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | Memory content |
| `importance` | float | No | 0.0-1.0 (default: 0.5) |
| `valence` | float | No | -1.0 to 1.0 |
| `arousal` | float | No | 0.0-1.0 |
| `memory_type` | string | No | fact, preference, rule, experience, skill_learned |
| `category` | string | No | Category name |
| `tags` | array | No | List of tags |

**Example:**
```json
{
  "method": "memory_create",
  "params": {
    "content": "User prefers dark mode in all applications",
    "importance": 0.85,
    "memory_type": "preference",
    "tags": ["ui", "preference"]
  }
}
```

**Response:**
```json
{
  "result": {
    "id": "uuid",
    "content": "User prefers dark mode...",
    "importance": 0.85,
    ...
  },
  "error": null
}
```

---

#### memory_search

Search memories by keyword or semantic query.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `limit` | number | No | Max results (default: 20) |
| `semantic` | boolean | No | Use semantic search (default: false) |

**Example:**
```json
{
  "method": "memory_search",
  "params": {
    "query": "user preferences",
    "limit": 5,
    "semantic": true
  }
}
```

**Response:**
```json
{
  "result": {
    "memories": [
      {
        "id": "uuid",
        "content": "User prefers dark mode...",
        "importance": 0.85,
        "category": "fact"
      }
    ]
  }
}
```

---

#### memory_list

List all memories.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `limit` | number | No | Max results (default: 100) |
| `category` | string | No | Filter by category |

**Example:**
```json
{
  "method": "memory_list",
  "params": {
    "limit": 10
  }
}
```

---

#### memory_get

Get a specific memory by ID.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Memory ID |

**Example:**
```json
{
  "method": "memory_get",
  "params": {
    "id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

#### memory_delete

Soft delete a memory.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Memory ID |

**Example:**
```json
{
  "method": "memory_delete",
  "params": {
    "id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

#### memory_related

Find memories related to a specific memory.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Source memory ID |
| `limit` | number | No | Max results (default: 10) |

**Example:**
```json
{
  "method": "memory_related",
  "params": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "limit": 5
  }
}
```

---

#### memory_context

Build context from relevant memories for AI prompts.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `query` | string | Yes | Context query |
| `max_tokens` | number | No | Token limit (default: 4000) |
| `min_importance` | number | No | Min importance (default: 0.3) |

**Example:**
```json
{
  "method": "memory_context",
  "params": {
    "query": "What do you know about the user?",
    "max_tokens": 500,
    "min_importance": 0.3
  }
}
```

**Response:**
```json
{
  "result": {
    "context": "[fact] User prefers dark mode (importance: 0.85)\n\n[fact] User lives in Mumbai...",
    "memories": [...],
    "total_tokens": 450
  }
}
```

---

### Session Tools

#### session_create

Create a new session.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Session name |
| `agent_id` | string | No | Agent identifier |

**Example:**
```json
{
  "method": "session_create",
  "params": {
    "name": "Coding Session",
    "agent_id": "wolverine"
  }
}
```

---

#### session_list

List all sessions.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `limit` | number | No | Max results (default: 50) |

---

#### session_end

End a session.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Session ID |

---

### Skill Tools

#### skill_list

List all skills.

**Parameters:** None

**Example:**
```json
{
  "method": "skill_list",
  "params": {}
}
```

---

#### skill_create

Create a new skill.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Skill name |
| `description` | string | No | Description |
| `code` | string | Yes | Skill code |
| `language` | string | No | Language (default: text) |

**Example:**
```json
{
  "method": "skill_create",
  "params": {
    "name": "greet_user",
    "description": "Greets the user by name",
    "code": "def greet(name): return f'Hello, {name}!'",
    "language": "python"
  }
}
```

---

#### skill_execute

Execute a skill by name.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Skill name |
| `parameters` | object | No | Skill parameters |

**Example:**
```json
{
  "method": "skill_execute",
  "params": {
    "name": "greet_user",
    "parameters": {
      "name": "Vineet"
    }
  }
}
```

---

### Procedure Tools

#### procedure_list

List all procedures.

**Parameters:** None

---

### System Tools

#### stats_get

Get memory system statistics.

**Parameters:** None

**Example:**
```json
{
  "method": "stats_get",
  "params": {}
}
```

**Response:**
```json
{
  "result": {
    "total_memories": 100,
    "total_sessions": 5,
    "total_skills": 3,
    "total_procedures": 2,
    "avg_importance": 0.72
  }
}
```

---

#### consolidate_run

Run memory consolidation.

**Parameters:** None

---

#### prune_run

Prune old low-importance memories.

**Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `days` | number | No | Delete memories older than N days |
| `min_importance` | number | No | Delete memories below importance threshold |

**Example:**
```json
{
  "method": "prune_run",
  "params": {
    "days": 30,
    "min_importance": 0.1
  }
}
```

---

## Example Integration

### Python Example

```python
import requests
import json

class ChetnaMCP:
    def __init__(self, base_url="http://localhost:1987"):
        self.base_url = base_url
        self.mcp_endpoint = f"{base_url}/mcp"
    
    def call(self, method, params=None):
        payload = {
            "method": method,
            "params": params or {}
        }
        response = requests.post(self.mcp_endpoint, json=payload)
        result = response.json()
        
        if result.get("error"):
            raise Exception(f"MCP Error: {result['error']}")
        
        return result.get("result")

# Usage
chetna = ChetnaMCP("http://localhost:1987")

# Create memory
chetna.call("memory_create", {
    "content": "User's name is Vineet",
    "importance": 0.9,
    "memory_type": "fact"
})

# Search memories
results = chetna.call("memory_search", {
    "query": "user name",
    "semantic": True,
    "limit": 5
})

# Build context for AI
context = chetna.call("memory_context", {
    "query": "Tell me about the user",
    "max_tokens": 500
})

print(context["context"])
```

### JavaScript/Node.js Example

```javascript
const fetch = require('fetch');

class ChetnaMCP {
  constructor(baseUrl = 'http://localhost:1987') {
    this.endpoint = `${baseUrl}/mcp`;
  }

  async call(method, params = {}) {
    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ method, params })
    });
    
    const result = await response.json();
    if (result.error) throw new Error(result.error);
    return result.result;
  }
}

// Usage
const chetna = new ChetnaMCP();

// Create memory
await chetna.call('memory_create', {
  content: "User's name is Vineet",
  importance: 0.9,
  memory_type: 'fact'
});

// Search
const memories = await chetna.call('memory_search', {
  query: 'user name',
  semantic: true
});
```

---

## Error Handling

All errors are returned in the response:

```json
{
  "result": null,
  "error": "Error message here",
  "id": "request-id"
}
```

Common errors:
- `Invalid method`: Unknown tool name
- `Missing required parameter`: Required parameter not provided
- `Memory not found`: Requested memory doesn't exist
- `Connection failed`: Cannot reach Ollama
