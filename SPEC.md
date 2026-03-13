# 🧠 Chetna - Technical Specification

> "Chetna" (Hindi: चेतना) = Consciousness, Awareness, Knowledge

## Project Overview

A hyper-fast, standalone memory system written in Rust designed for proto-AGI agents. Combines:
- Wolverine's intelligent memory management (importance, emotional tones, REM consolidation, skills, procedures)
- Engram's battle-tested architecture (sessions, timeline, MCP server)

## Technology Stack

- **Runtime**: Rust with Tokio async
- **Web Server**: Axum
- **Database**: SQLite with FTS5
- **Embeddings**: Ollama, OpenAI, Google Gemini, OpenRouter
- **Protocol**: HTTP REST + MCP (Model Context Protocol)

## Project Structure

```
chetna/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── config.rs         # Configuration + model registry
│   ├── cache/
│   │   └── mod.rs       # Session LRU cache
│   ├── db/
│   │   ├── mod.rs       # Database schema
│   │   ├── brain.rs     # Core memory operations
│   │   ├── embedding.rs # Multi-provider embeddings
│   │   ├── search.rs    # FTS5 search
│   │   └── relationships.rs # Memory relationships
│   ├── api/
│   │   ├── mod.rs       # Router
│   │   ├── memory.rs    # Memory endpoints
│   │   ├── session.rs   # Session endpoints
│   │   ├── skill.rs     # Skill endpoints
│   │   ├── procedure.rs # Procedure endpoints
│   │   ├── stats.rs     # Statistics
│   │   └── config_api.rs # Model selection
│   ├── consolidate/
│   │   └── mod.rs       # REM consolidation
│   ├── mcp/
│   │   └── mod.rs       # MCP server
│   ├── multimodal/
│   │   └── mod.rs       # Multi-modal support
│   └── web/
│       └── mod.rs       # Web dashboard
├── data/                  # Database storage
└── .env                   # Configuration
```

## Database Schema

### Memories Table
```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    category TEXT,           -- fact, preference, rule, experience, skill_learned
    key TEXT,
    content TEXT,
    importance REAL,         -- 0-1 scale
    emotional_tone REAL,    -- -1 to 1 (valence)
    arousal REAL,           -- 0-1
    embedding BLOB,         -- Vector storage
    embedding_model TEXT,
    tags TEXT,              -- JSON array
    memory_type TEXT,
    access_count INTEGER,
    last_accessed TEXT,
    created_at TEXT,
    updated_at TEXT,
    consolidated INTEGER,   -- REM consolidated flag
    last_consolidated TEXT,
    source TEXT,            -- agent, user, system
    source_tool TEXT,
    scope TEXT,             -- global, session, project
    deleted_at TEXT         -- Soft delete
);
```

### Additional Tables
- `sessions` - Session tracking
- `skills` - Stored skills
- `procedures` - Stored procedures
- `embedding_cache` - Cached embeddings
- `memory_relationships` - Memory connections
- `multimodal_memories` - Images, audio, video, documents
- `session_cache` - Session-level cache

## Memory Types

| Type | Description |
|------|-------------|
| `fact` | Factual information |
| `preference` | User preferences |
| `rule` | Rules/guidelines |
| `experience` | Experiences/events |
| `skill_learned` | Learned skills |

## Importance System

- **0.0-0.3**: Low importance, candidates for pruning
- **0.3-0.7**: Medium importance
- **0.7-1.0**: High importance, protected from pruning

## REM Consolidation

The consolidation process mimics human sleep cycles:

1. **NREM 1 (Light)**: Consolidate high-importance emotional memories (importance > 0.8)
2. **NREM 2 (Medium)**: Consolidate procedural/skill memories
3. **NREM 3 (Deep)**: Consolidate factual memories
4. **REM (Dreaming)**: Strengthen connections between related memories

## Embedding Providers

### Supported
- **Ollama** (local) - nomic-embed-text, mxbai-embed-large, gemma3-embed-e2b, bge-m3
- **OpenAI** - text-embedding-3-small, text-embedding-3-large
- **Google** - gemini-embedding-001, gemini-embedding-2 (multimodal)
- **OpenRouter** - Compatible with OpenAI API

### Configuration
```rust
// Each provider has different requirements
Ollama: base_url (e.g., http://localhost:11434)
OpenAI: api_key required
Google: api_key required  
OpenRouter: api_key required
```

## Memory Relationships

| Type | Description |
|------|-------------|
| `related` | General relationship |
| `similar` | High semantic similarity (>0.9) |
| `contradicts` | Opposing information |
| `supports` | Supports another memory |
| `extends` | Extends another memory |
| `cause` | Causal relationship |
| `effect` | Result of another event |
| `part_of` | Part of larger context |
| `before`/`after` | Temporal relationship |

## API Design

### Response Format
```json
{
  "id": "uuid",
  "content": "memory content",
  "importance": 0.8,
  "emotional_tone": 0.5,
  "arousal": 0.3,
  "tags": ["tag1", "tag2"],
  "memory_type": "fact",
  "category": "fact",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Context Building
Returns token-limited relevant memories for AI prompts:
```json
{
  "memories": [...],
  "total_tokens": 1500,
  "context": "[fact] ... (truncated)"
}
```

## MCP Protocol

Tools available via MCP:
- memory_create, memory_search, memory_list, memory_get
- memory_delete, memory_related, memory_context
- session_create, session_list, session_end
- skill_list, skill_create, skill_execute
- procedure_list, procedure_execute
- stats_get, consolidate_run, prune_run

## Performance Targets

- **Memory creation**: <10ms (with embedding)
- **Semantic search**: <100ms (for 10K memories)
- **Context building**: <200ms
- **Cache hit rate**: >90% typical
- **Database size**: <1GB for 100K memories

## Security Considerations

- No encryption (local use only)
- No authentication (local use only)
- Soft delete preserves data until pruning

## Future Enhancements

- [ ] LanceDB for vector storage at scale
- [ ] Time-decay for importance (Ebbinghaus implemented)
- [ ] Encryption for sensitive memories
- [ ] Multi-instance sync
- [ ] Enhanced graph visualization in web UI

## Credits

- Wolverine (memory architecture)
- Engram (sessions, MCP)
- Google (EmbeddingGemma research)
