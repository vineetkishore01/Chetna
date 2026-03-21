# Chetna Quickstart Guide

**Get Chetna running in 5 minutes**

---

## Prerequisites

You need **one** of these setups:

### Option A: Local Setup (Recommended for Development)
- **Rust 1.70+** - [Install](https://rustup.rs/)
- **Ollama** (optional, for semantic search) - [Install](https://ollama.ai/)

### Option B: Docker (Recommended for Production)
- **Docker & Docker Compose** - [Install](https://docker.com/)

---

## 5-Minute Setup

### Step 1: Clone

```bash
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
```

### Step 2: Run Auto-Setup

```bash
./install.sh --auto
```

This script will:
1. ✅ Check for Rust (installs if missing)
2. ✅ Check for Ollama (installs if missing)
3. ✅ Pull recommended embedding model (`qwen3-embedding:4b`)
4. ✅ Build Chetna
5. ✅ Start the server

**Wait time:** 5-10 minutes (building the project is the longest part)

### Step 3: Verify

```bash
curl http://localhost:1987/health
# Should return status: healthy
```

### Step 4: Open Dashboard

Visit **http://localhost:1987** in your browser.

---

## Your First Memory

### Via Web UI

1. Open http://localhost:1987
2. Click **"New Memory"**
3. Enter: *"The user's favorite language is Rust"*
4. Set importance to **0.9** (high)
5. Click **"Create"**

### Via API (CLI)

```bash
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{
    "content": "The user favorities Python for data scripts",
    "importance": 0.8,
    "memory_type": "fact",
    "tags": ["personal"]
  }'
```

---

## Test Semantic Search

Now search for that memory using different words:

```bash
curl "http://localhost:1987/api/memory/search?query=which+language+does+user+like"
```

**Magic:** Chetna finds your memories because it understands **meaning**, not just keywords.

---

## Build Context for AI

The killer feature of Chetna is the ability to automatically assemble context for your prompts:

```bash
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the users coding preferences?",
    "max_tokens": 500
  }'
```

**Response:**
```json
{
  "memories": [...],
  "total_tokens": 45,
  "context": "[fact] The user favorities Python for data scripts (importance: 0.80)"
}
```

Use this `context` directly in your AI's system prompt!

---

## Connect Your AI Agent

### MCP Protocol (Highly Recommended)

If your agent supports MCP (like OpenClaw, Windsurf, or Claude Desktop):

1. Point your agent to Chetna's MCP endpoint: `http://localhost:1987/mcp`
2. Your agent gains tools like `memory_create` and `memory_search`.
3. It can now manage its own long-term memory automatically.

---

## Common Tasks

### List All Memories
`curl http://localhost:1987/api/memory`

### Pin Important Memory (Never Delete)
`curl -X POST http://localhost:1987/api/memory/pin/MEMORY_ID`

### Delete Memory
`curl -X DELETE http://localhost:1987/api/memory/MEMORY_ID`

---

## Troubleshooting

### "Ollama error"
Ensure Ollama is running and you've pulled the model:
`ollama pull qwen3-embedding:4b`

### Port already in use
If another service uses port 1987, you can change it:
`CHETNA_PORT=3000 ./target/release/chetna`

### Build fails
Ensure Rust is up to date:
`rustup update`

---

## Next Steps

1. **Read the [API Reference](api.md)** - All endpoints
2. **Read [Agent Integration](agent-integration.md)** - Connect your AI
3. **Explore the Web UI** - http://localhost:1987

---
**Need Help?**
- Open an [Issue](https://github.com/vineetkishore01/Chetna/issues)
- Email: vineetkishore01@gmail.com
