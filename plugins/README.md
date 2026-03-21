# Chetna Plugins for AI Code Editors

Give your AI coding assistant permanent memory that persists across sessions.

## The Problem

AI assistants forget everything between sessions:
- User preferences mentioned once
- Project context from last week
- Important facts about your workflow

## The Solution

Chetna provides persistent memory via semantic search + importance-weighted recall.

---

## Supported Editors

| Editor | Folder | Status |
|--------|--------|--------|
| Claude Code | `claude-code/` | ✅ Ready |
| Cursor | `cursor/` | ✅ Ready |
| Windsurf | `windsurf/` | ✅ Ready |
| VS Code | `vscode/` | 🔜 Planned |
| Zed | `zed/` | 🔜 Planned |

---

## Quick Start

### 1. Start Chetna Server

```bash
cd Chetna
cargo build --release
./target/release/chetna
```

Server runs on `http://localhost:1987`

### 2. Install Plugin

```bash
# Claude Code
cp -r plugins/claude-code ~/.claude/plugins/chetna

# Cursor
cp -r plugins/cursor ~/.cursor/plugins/chetna

# Windsurf
cp plugins/windsurf/mcp_config.json ~/.codeium/windsurf/
```

### 3. Use Commands

```
/remember User prefers dark mode
/recall user preferences
/stats
/search testing framework
```

---

## Commands

| Command | Description |
|---------|-------------|
| `/remember [content]` | Store important info |
| `/recall [query]` | Search memories |
| `/stats` | View statistics |
| `/search [query]` | Advanced search |

---

## How It Works

1. **Remember**: Stores content with semantic embedding + importance score
2. **Recall**: Searches by meaning (not just keywords), weighted by importance
3. **Persist**: Memories survive session restarts, machine changes, time

## Example Session

```
You: I prefer using pytest for testing
Claude: Stored: "User prefers pytest" ✓

[Session ends, days pass...]

You: What testing framework do I use?
Claude: You prefer pytest over unittest. I remembered! ✓
```

---

## API Reference

### REST Endpoints

```bash
# Create memory
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers dark mode", "importance": 0.9}'

# Search
curl "http://localhost:1987/api/memory/search?query=user+preferences"

# Context (semantic + importance weighted)
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{"query": "user preferences"}'

# Stats
curl http://localhost:1987/api/stats
```

---

## Chetna Features

- **Semantic Search**: Find memories by meaning
- **Importance Weighting**: Critical memories persist; trivial fade
- **Emotional Memory**: Tracks valence for recall
- **Auto-Decay**: Ebbinghaus forgetting curve
- **Sessions**: Group by project/context
- **Fully Local**: No cloud, runs with Ollama

---

## License

MIT + Trademark protections. See LICENSE file.

## Author

Vineet Kishore - @vineetkishore01
