---
name: chetna-recall
description: Retrieve relevant memories from permanent memory
---

# /chetna:recall

Retrieve memories from Chetna's permanent memory using semantic search.

## Usage

```
/chetna:recall [what to search for]
```

## Examples

```
/chetna:recall user preferences
/chetna:recall project setup
/chetna:recall coding standards
/chetna:recall John's preferences
/chetna:recall testing framework
```

## How It Works

Calls the context endpoint which combines semantic search with importance weighting:

```bash
curl -X POST http://localhost:1987/api/memory/context \
  -H "Content-Type: application/json" \
  -d '{"query": "user preferences", "max_tokens": 500}'
```

## Response Format

Returns memories sorted by recall score (similarity × importance × recency):

```
[fact] User prefers VS Code (importance: 0.95)
[preference] Dark mode enabled (importance: 0.85)
[rule] Run tests before commit (importance: 0.80)
```

## Tips

- Use natural language: "What does the user prefer?" works
- Be descriptive: "coding style preferences" finds more than "prefs"
- Works even with paraphrasing: "IDE preference" finds "prefers VS Code"
