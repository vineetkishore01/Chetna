---
name: chetna-search
description: Advanced semantic search through all memories
---

# /chetna:search

Advanced search through memories with filters.

## Usage

```
/chetna:search [query]
```

## Examples

```
/chetna:search pytest
/chetna:search API setup
/chetna:search dark mode
/chetna:search deployment process
```

## How It Works

```bash
curl "http://localhost:1987/api/memory/search?query=pytest&limit=10"
```

## Response

Returns JSON with matching memories:

```json
{
  "memories": [
    {
      "content": "User prefers pytest over unittest",
      "importance": 0.9,
      "memory_type": "preference"
    }
  ]
}
```

## Tips

- Combine with /chetna:remember for best results
- Search is semantic - "testing" finds "pytest"
- Use /chetna:recall for natural language queries
