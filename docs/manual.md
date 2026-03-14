# Chetna - God-Tier Memory System

> **Chetna** (Hindi: चेतना) = Consciousness, Awareness, Knowledge

Chetna is a hyper-fast, standalone memory system for AI agents. It provides semantic search, importance scoring, memory consolidation, and more - designed to mimic how human brains work.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Running the Server](#running-the-server)
5. [Docker Deployment](#docker-deployment)
6. [Web Dashboard](#web-dashboard)
7. [REST API Reference](./api.md)
8. [MCP Protocol Reference](./mcp.md)
9. [AI Agent Integration Guide](./agent-integration.md)
10. [How Chetna Works (Design)](#how-chetna-works-design)
11. [Memory Management](#memory-management)
12. [Consolidation & Decay](#consolidation--decay)
13. [Security](#security)
14. [Troubleshooting](#troubleshooting)

---

## Quick Start

```bash
# 1. Clone and build
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
cargo build --release

# 2. Configure Ollama (for embeddings)
export EMBEDDING_BASE_URL=http://localhost:11434
export EMBEDDING_MODEL=qwen3-embedding:4b

# 3. Run
./target/release/chetna

# 4. Open dashboard
# Visit http://localhost:1987
```

---

## Installation

### Prerequisites

- **Rust** (1.70+) - [Install Rust](https://rustup.rs/)
- **Ollama** - For embeddings and LLM [Install Ollama](https://ollama.ai/)

### Recommended Models

```bash
# Pull recommended models for best experience
ollama pull qwen3-embedding:4b   # Semantic search (recommended)
ollama pull qwen3.5:4b          # For consolidation & auto-scoring
```

### Build from Source

```bash
# Clone or navigate to the Chetna directory
cd chetna

# Build the project
cargo build --release

# The binary will be at ./target/release/chetna
```

---

## Configuration

### Environment Variables

Create a `.env` file in the `ChetnaData` directory:

```bash
# Server Settings
CHETNA_PORT=1987
CHETNA_DB_PATH=./ChetnaData/chetna.db

# Embedding Configuration (for semantic search)
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=qwen3-embedding:4b
EMBEDDING_BASE_URL=http://localhost:11434

# LLM Configuration (for auto-scoring and consolidation)
LLM_PROVIDER=ollama
LLM_MODEL=qwen3.5:4b
LLM_BASE_URL=http://localhost:11434

# Cache Settings
SESSION_CACHE_SIZE=100

# Consolidation Settings
CONSOLIDATION_INTERVAL=6
AUTO_DECAY_ENABLED=true
AUTO_FLUSH_ENABLED=true
MIN_IMPORTANCE_THRESHOLD=0.1

# Security (optional)
CHETNA_API_KEY=your_secret_api_key_here
```

### Configuration Options

| Variable | Default | Description |
|----------|---------|-------------|
| `CHETNA_PORT` | 1987 | Server port |
| `CHETNA_DB_PATH` | `./ChetnaData/chetna.db` | Database file path |
| `EMBEDDING_PROVIDER` | ollama | Embedding provider: `ollama`, `openai`, `google`, `openrouter` |
| `EMBEDDING_MODEL` | nomic-embed-text | Embedding model name |
| `EMBEDDING_BASE_URL` | http://localhost:11434 | Ollama API URL |
| `LLM_PROVIDER` | ollama | LLM provider for auto-scoring |
| `LLM_MODEL` | llama3.2 | LLM model name |
| `SESSION_CACHE_SIZE` | 100 | LRU cache size for hot memories |
| `CONSOLIDATION_INTERVAL` | 6 | Hours between automatic consolidation (0 = disabled) |
| `AUTO_DECAY_ENABLED` | true | Enable Ebbinghaus forgetting curve |
| `AUTO_FLUSH_ENABLED` | true | Enable automatic low-importance memory flush |
| `MIN_IMPORTANCE_THRESHOLD` | 0.1 | Memories below this get auto-flushed |
| `CHETNA_API_KEY` | (none) | API key for authentication |

---

## Running the Server

### Basic Usage

```bash
# Run with default settings
cargo run

# Run with custom port
CHETNA_PORT=3000 cargo run

# Run in background
cargo run --release &
```

### Production Deployment

For production, consider:

1. **Using a reverse proxy** (nginx, Caddy)
2. **Enabling API authentication** (set `CHETNA_API_KEY`)
3. **Setting up proper logging**
4. **Configuring automatic startup**

---

## Docker Deployment

### Using Docker Compose (Recommended)

```bash
# Start Chetna with Docker
docker-compose up -d
```

The default `docker-compose.yml` includes:
- Port mapping: `1987:1987`
- Volume mapping for persistent data: `./ChetnaData:/app/ChetnaData`
- Environment variables for Ollama connection

### Manual Docker Run

```bash
# Build image
docker build -t chetna:latest .

# Run container
docker run -d \
  --name chetna \
  -p 1987:1987 \
  -v ./ChetnaData:/app/ChetnaData \
  -e EMBEDDING_BASE_URL=http://host.docker.internal:11434 \
  -e LLM_BASE_URL=http://host.docker.internal:11434 \
  chetna:latest
```

### Environment Variables in Docker

| Variable | Description | Docker Example |
|----------|-------------|----------------|
| `EMBEDDING_BASE_URL` | Ollama server for embeddings | `http://host.docker.internal:11434` |
| `LLM_BASE_URL` | Ollama server for LLM | `http://host.docker.internal:11434` |
| `EMBEDDING_MODEL` | Embedding model | `qwen3-embedding:4b` |
| `LLM_MODEL` | LLM model | `qwen3.5:4b` |

---

## Web Dashboard

Access the dashboard at `http://localhost:1987`

### Dashboard Features

- **Memory Operations**: Run consolidation, decay, flush
- **Semantic Search**: Search memories by meaning
- **Context Builder**: Build AI context from memories
- **Connection Status**: View embedding/LLM status

### Pages

| Page | URL | Description |
|------|-----|-------------|
| Dashboard | `/` | Main dashboard with operations |
| Memories | `/memories` | Browse and manage memories |
| Skills | `/skills` | Manage skills |
| Sessions | `/sessions` | Manage sessions |
| Settings | `/settings` | Configure embedding/LLM |

---

## REST API

Complete API reference available at [API Reference](./api.md)

### Quick Examples

```bash
# Create memory
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers dark mode", "importance": 0.8}'

# Semantic search
curl "http://localhost:1987/api/memory/search/semantic?query=user+preferences&limit=5"

# Build context for AI
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{"query": "What are user preferences?", "max_tokens": 500}'
```

---

## MCP Protocol

Complete MCP reference available at [MCP Reference](./mcp.md)

Chetna supports the Model Context Protocol for AI agent integration.

```bash
# Example MCP call
curl -X POST http://localhost:1987/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "method": "memory_search",
    "params": {"query": "user preferences", "semantic": true}
  }'
```

---

## AI Agent Integration

Complete integration guide available at [Agent Integration Guide](./agent-integration.md)

---

## How Chetna Works (Design)

This section explains the internal architecture and how Chetna mimics human memory.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CHETNA                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐       │
│  │   REST API  │    │     MCP     │    │   Web UI    │       │
│  │   (HTTP)    │    │   (JSON)    │    │   (HTML)    │       │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘       │
│         │                  │                  │                │
│         └──────────────────┼──────────────────┘                │
│                            ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    BRAIN (Memory Engine)                │    │
│  │                                                          │    │
│  │  ┌────────────────┐    ┌────────────────┐              │    │
│  │  │ Semantic Search │    │  Keyword Search │              │    │
│  │  │   (Embeddings) │    │   (Full-text)   │              │    │
│  │  └───────┬────────┘    └───────┬────────┘              │    │
│  │          │                     │                         │    │
│  │          ▼                     ▼                         │    │
│  │  ┌─────────────────────────────────────────────────┐    │    │
│  │  │      Human-Like Recall Scoring Engine          │    │    │
│  │  │                                                 │    │    │
│  │  │  Score = Similarity(40%) + Importance(25%)   │    │    │
│  │  │       + Recency(15%) + Access(10%) +         │    │    │
│  │  │       Emotional(10%)                          │    │    │
│  │  └─────────────────────────────────────────────────┘    │    │
│  │                         │                               │    │
│  └─────────────────────────┼───────────────────────────────┘    │
│                            ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              SQLITE DATABASE                           │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐  │    │
│  │  │Memories │ │ Sessions│ │ Skills  │ │ Embeddings │  │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              EXTERNAL SERVICES                         │    │
│  │  ┌─────────────────┐    ┌─────────────────┐          │    │
│  │  │   OLLAMA       │    │      LLM        │          │    │
│  │  │ (Embeddings)   │    │ (Consolidation) │          │    │
│  │  └─────────────────┘    └─────────────────┘          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Memory Storage

Each memory stores:

```rust
struct Memory {
    // Content
    content: String,           // The actual memory text
    
    // Intelligence (Wolverine's best)
    importance: f64,           // 0.0-1.0: How critical is this?
    emotional_tone: f64,      // -1.0 to 1.0: Negative to positive
    arousal: f64,             // 0.0-1.0: Calm to excited
    
    // Metadata
    memory_type: String,     // fact, preference, rule, experience, skill_learned
    tags: Vec<String>,        // User-defined tags
    
    // Access tracking
    access_count: i64,        // How many times accessed
    last_accessed: String,    // When last accessed
    
    // Forgetting curve
    created_at: String,      // When created
    is_pinned: bool,         // Never forget
    
    // Embedding (for semantic search)
    embedding: Vec<f32>,     // Vector representation
    embedding_model: String,  // Which model created it
}
```

### Semantic Search: How Meaning Works

Chetna uses **embeddings** to understand meaning, not just keywords.

```
┌─────────────────────────────────────────────────────────────┐
│                    SEMANTIC SEARCH                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Step 1: Convert Query to Vector                            │
│  ─────────────────────────────────                          │
│  Query: "who owns me"                                       │
│       │                                                     │
│       ▼                                                     │
│  [0.2, -0.1, 0.8, 0.5, ...] ──► Embedding Vector         │
│                                                              │
│  Step 2: Compare with Memory Vectors                        │
│  ────────────────────────────────────────                    │
│                                                              │
│  Memory: "My owner is Vineet"                               │
│  Vector: [0.2, -0.1, 0.9, 0.4, ...]                       │
│                    │                                         │
│                    │ Cosine Similarity = 0.92 ✓             │
│                    ▼                                         │
│  Memory: "I like pizza"                                     │
│  Vector: [-0.5, 0.3, 0.1, -0.2, ...]                      │
│                    │                                         │
│                    │ Cosine Similarity = 0.15 ✗             │
│                    ▼                                         │
│  Step 3: Return Ranked Results                              │
│  ────────────────────────────                               │
│  1. "My owner is Vineet" (92% similar)                     │
│  2. ...                                                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Human-Like Recall: The Scoring Algorithm

This is the key innovation. When you search, Chetna doesn't just return "most similar" - it combines multiple factors like human memory:

```rust
fn calculate_recall_score(memory, similarity, now) {
    
    // 1. SIMILARITY (40%) - How relevant to current query?
    let similarity_weight = 0.40;
    
    // 2. IMPORTANCE (25%) - Critical memories stick longer
    // Pinned memories get a 1.0 boost
    let importance_boost = if memory.is_pinned { 1.0 } else { memory.importance };
    let importance_weight = 0.25;
    
    // 3. RECENCY (15%) - Recent memories are easier to recall
    // Memory fades over 30 days (720 hours)
    let hours_since_creation = (now - memory.created_at).num_hours();
    let recency_score = exp(-hours_since_creation / 720.0);
    let recency_weight = 0.15;
    
    // 4. ACCESS FREQUENCY (10%) - Frequently accessed = important
    let access_score = sqrt(access_count / 10).min(1.0);
    let access_weight = 0.10;
    
    // 5. EMOTIONAL WEIGHT (10%) - Emotional memories stick better
    let emotional_intensity = abs(emotional_tone);
    let emotional_weight = 0.10;
    
    // Combined Score (like human brain!)
    return (similarity * 0.40) 
         + (importance_boost * 0.25)
         + (recency_score * 0.15)
         + (access_score * 0.10)
         + (emotional_intensity * 0.10);
}
```

### Why This Matters

| Factor | Human Memory | Chetna |
|--------|--------------|--------|
| **Similarity** | "This reminds me of..." | Semantic embedding match |
| **Importance** | "This is critical" | `importance` field + `is_pinned` |
| **Recency** | "I just learned this" | Decay over time |
| **Access** | "I think about this often" | `access_count` increments |
| **Emotion** | "This made me feel..." | `emotional_tone` value |

### Context Building for AI

When building context for an AI prompt:

```
┌─────────────────────────────────────────────────────────────┐
│                 BUILD CONTEXT                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Input: "What do you know about the user?"                 │
│         max_tokens: 500                                     │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 1. Semantic Search → Find relevant memories           │  │
│  │ 2. Score each with human-like algorithm              │  │
│  │ 3. Sort by combined score                           │  │
│  │ 4. Add memories until token limit                   │  │
│  └───────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  Output:                                                   │
│  ────────                                                   │
│  [fact] My name is Wolverine and my human is Vineet        │
│  (importance: 0.95)                                        │
│                                                              │
│  [fact] User prefers dark mode in all applications         │
│  (importance: 0.85)                                        │
│                                                              │
│  [preference] I enjoy coding in Python late at night       │
│  (importance: 0.75)                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Consolidation: The "Sleep" Phase

Like human brains consolidating memories during sleep, Chetna periodically reviews and updates memories:

1. **LLM Review**: An LLM re-reads memories and updates importance scores
2. **Similarity Merge**: Similar memories might be merged
3. **Access Boost**: Frequently accessed memories get importance boost

### Decay: The Forgetting Curve

Based on Ebbinghaus's forgetting curve research:

| Memory Type | Stability (hours) | Description |
|-------------|-------------------|-------------|
| `preference` | 720 (30 days) | User preferences last long |
| `skill_learned` | 336 (14 days) | Learned skills decay slower |
| `fact` | 168 (7 days) | General facts decay medium |
| `rule` | 240 (10 days) | Important rules last longer |
| `experience` | 24 (1 day) | Experiences decay faster |
| Pinned | ∞ | Never decay |

---

## Memory Management

### Memory Types

- **fact**: Factual knowledge ("Vineet lives in Mumbai")
- **preference**: User preferences ("User prefers dark mode")
- **rule**: Rules and constraints ("Always backup before updates")
- **experience**: Past experiences ("User had a great meeting today")
- **skill_learned**: Learned skills ("Wolverine knows Rust")

### Importance Scoring

Memories have three scoring dimensions:

| Score | Range | Description |
|-------|-------|-------------|
| `importance` | 0.0 - 1.0 | How important the memory is |
| `valence` | -1.0 - 1.0 | Emotional tone (negative to positive) |
| `arousal` | 0.0 - 1.0 | Emotional intensity (calm to excited) |

### Auto-Scoring

Set `auto_score: true` when creating a memory to automatically score using keywords

### Tags

Add up to 50 tags per memory (max 100 characters each)

---

## Consolidation & Decay

### LLM Consolidation

Uses an LLM to re-evaluate memory importance

```bash
curl -X POST http://localhost:1987/api/memory/consolidate \
  -H "Content-Type: application/json" \
  -d '{"limit": 50}'
```

### Ebbinghaus Decay

Applies the forgetting curve

```bash
curl -X POST http://localhost:1987/api/memory/decay
```

### Auto-Management

Configure automatic consolidation:

```bash
curl -X POST http://localhost:1987/api/config/user \
  -H "Content-Type: application/json" \
  -d '{
    "consolidation_interval_hours": 6,
    "auto_decay_enabled": true,
    "auto_flush_enabled": true,
    "min_importance_threshold": 0.1
  }'
```

---

## Security

### API Authentication

```bash
# Via header
curl -H "Authorization: Bearer YOUR_KEY" http://localhost:1987/api/memory

# Via query
curl "http://localhost:1987/api/memory?api_key=YOUR_KEY"
```

### Best Practices

1. Use HTTPS in production (reverse proxy)
2. Set a strong API key
3. Restrict network access
4. Monitor logs

---

## Troubleshooting

### Server Won't Start

```bash
# Check if port is in use
lsof -i :1987

# Check database permissions
ls -la ChetnaData/
```

### Embeddings Not Working

```bash
# Ensure Ollama is running
ollama serve

# Check models
ollama list

# Test connection
curl http://localhost:11434/api/tags
```

### Search Returns No Results

1. Check memories have embeddings
2. Lower similarity threshold
3. Use keyword search as fallback

---

## Getting Help

- **Issues**: https://github.com/vineetkishore01/Chetna/issues
- **Logs**: Enable debug with `CHETNA_LOG_LEVEL=debug`

---

## License

MIT License
