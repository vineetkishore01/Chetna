# Chetna Memory - VS Code Extension

> Give VS Code permanent memory using Chetna

## Overview

This extension provides commands to interact with Chetna memory system from VS Code.

## Installation

1. Install Chetna server:
```bash
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
cargo build --release
./target/release/chetna
```

2. This extension is coming soon!

## Features (Planned)

- **Remember**: Store important information (`Ctrl+Shift+M`)
- **Recall**: Search memories (`Ctrl+Shift+R`)
- **Context**: Get relevant context for current task
- **Stats**: View memory statistics

## Development Status

We're building a VS Code extension. For now, use the REST API directly:

```bash
# Remember something
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers VS Code", "importance": 0.9}'

# Search
curl "http://localhost:1987/api/memory/search?query=user+preferences"

# Get context
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{"query": "current task context"}'
```

## Contributing

VS Code extension development help welcome! Open an issue to contribute.
