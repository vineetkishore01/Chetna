# Chetna Integrations for AI Code Editors

This directory contains integrations for various AI code editors to use Chetna memory system.

## Overview

| Editor | Integration | Status |
|--------|-------------|--------|
| Claude Code | Plugin | ✅ Ready |
| Cursor | Plugin | ✅ Ready |
| Windsurf | MCP Config | ✅ Ready |
| Zed | Extension | 🔜 Planned |
| VS Code | Extension | 🔜 Planned |

---

## Claude Code

**Location**: `claude-code-plugin/`

### Installation

```bash
# Copy to Claude plugins directory
cp -r claude-code-plugin ~/.claude/plugins/chetna
```

### Commands

| Command | Description |
|---------|-------------|
| `/chetna:remember [content]` | Store important info |
| `/chetna:recall [query]` | Search memories |
| `/chetna:stats` | View statistics |

### Features

- Semantic search via `/chetna:recall`
- Automatic importance weighting
- Sessions for project isolation
- Proactive memory via skills

---

## Cursor

**Location**: `cursor-plugin/`

Cursor shares the same plugin architecture as Claude Code.

### Installation

```bash
# Copy to Cursor plugins directory
cp -r cursor-plugin ~/.cursor/plugins/chetna

# Or use Claude Code config (Cursor reads from ~/.claude/)
cp -r claude-code-plugin ~/.claude/plugins/chetna
```

### Commands

Same as Claude Code since they share configuration.

---

## Windsurf

**Location**: `windsurf-mcp/`

### Installation

```bash
# Copy MCP config
cp windsurf-mcp/mcp_config.json ~/.codeium/windsurf/mcp_config.json

# Restart Windsurf
```

### Usage

After installation, these tools are available in Cascade:

- `memory_create` - Store information
- `memory_search` - Semantic search
- `memory_context` - Get relevant context
- `memory_stats` - View statistics

### Alternative: Use REST API directly

```bash
# In Windsurf terminal
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers dark mode", "importance": 0.9}'
```

---

## Zed

**Status**: Planned

Zed extensions require Rust/WebAssembly compilation. This is a future enhancement.

For now, use the REST API:

```bash
# In Zed terminal
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "Remember this"}'
```

---

## VS Code

**Status**: Planned

A proper VS Code extension with:
- Memory panel
- Context commands
- Inline suggestions

For now, use the REST API via terminal or create a custom task.

---

## Quick Start (All Editors)

1. **Start Chetna server**:
```bash
cd Chetna
cargo build --release
./target/release/chetna
```

2. **Remember something**:
```bash
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode in VS Code",
    "importance": 0.9,
    "memory_type": "preference"
  }'
```

3. **Recall later**:
```bash
curl "http://localhost:1987/api/memory/search?query=IDE+preferences"
```

---

## Chetna Server

The server must be running for any integration to work:

```bash
# Default URL: http://localhost:1987
# To customize: set CHETNA_URL environment variable
```

---

## License

MIT + Trademark protections. See LICENSE file.
