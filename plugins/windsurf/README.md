# Chetna Memory Integration

## For Windsurf

Windsurf supports MCP servers. To integrate Chetna:

### Option 1: Direct REST API (Recommended)

Add to your project `.windsurf/mcp.json` or use in terminal:

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

### Option 2: MCP via stdio

Create a script `chetna-mcp.js`:

```javascript
#!/usr/bin/env node
const { spawn } = require('child_process');
const readline = require('readline');

const CHETNA_URL = process.env.CHETNA_URL || 'http://localhost:1987';

async function callChetna(method, params = {}) {
  const response = await fetch(`${CHETNA_URL}/api/mcp`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now()
    })
  });
  return response.json();
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

rl.on('line', async (line) => {
  try {
    const msg = JSON.parse(line);
    if (msg.method === 'tools/list') {
      console.log(JSON.stringify({
        jsonrpc: '2.0',
        id: msg.id,
        result: {
          tools: [
            { name: 'memory_create', description: 'Store important information in memory' },
            { name: 'memory_search', description: 'Search memories semantically' },
            { name: 'memory_context', description: 'Get relevant context for queries' },
            { name: 'memory_stats', description: 'View memory statistics' }
          ]
        }
      }));
    } else if (msg.method === 'tools/call') {
      const { name, arguments: args } = msg.params;
      let result;
      if (name === 'memory_create') {
        result = await callChetna('memory/create', args);
      } else if (name === 'memory_search') {
        result = await callChetna('memory/search', args);
      } else if (name === 'memory_context') {
        result = await callChetna('memory/context', args);
      } else if (name === 'memory_stats') {
        result = await callChetna('stats');
      }
      console.log(JSON.stringify({ jsonrpc: '2.0', id: msg.id, result }));
    }
  } catch (e) {
    console.log(JSON.stringify({ jsonrpc: '2.0', error: { message: e.message } }));
  }
});
```

Then configure in Windsurf settings:

```json
{
  "mcpServers": {
    "chetna-memory": {
      "command": "node",
      "args": ["/path/to/chetna-mcp.js"]
    }
  }
}
```
