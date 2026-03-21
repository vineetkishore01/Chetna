# Chetna for Zed

Zed extensions require Rust/WebAssembly compilation. This is a future enhancement.

## Current Status

🔜 Planned - requires Rust/WASM development

## For Now - Use REST API

In Zed terminal:

```bash
# Remember something
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers dark mode", "importance": 0.9}'

# Search memories
curl "http://localhost:1987/api/memory/search?query=user+preferences"

# Get context
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{"query": "user preferences"}'
```

## Want to Contribute?

Zed extensions are written in Rust. If you'd like to help build the Chetna Zed extension:

1. Fork the repo
2. Create a Rust extension following Zed's extension docs
3. Submit a PR

The extension would provide:
- Slash commands for /remember, /recall
- Memory panel
- Auto-context injection

## Resources

- [Zed Extension Docs](https://zed.dev/docs/extensions/developing-extensions)
- [Zed MCP Support](https://zed.dev/docs/extensions/mcp)
