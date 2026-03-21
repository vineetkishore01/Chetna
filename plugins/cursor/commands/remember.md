---
name: chetna-remember
description: Store important information in permanent memory
---

# /chetna:remember

Store important information in Chetna's permanent memory system.

## Usage

```
/chetna:remember [content to remember]
```

## Examples

```
/chetna:remember User prefers VS Code over other editors
/chetna:remember This project uses PostgreSQL
/chetna:remember Team has standups on Mondays at 10am
/chetna:remember John's birthday is December 15th
```

## How It Works

Executes a curl command to the local Chetna server:

```bash
curl -X POST http://localhost:1987/api/memory \
  -H "Content-Type: application/json" \
  -d '{"content": "User prefers VS Code", "importance": 0.9, "memory_type": "preference"}'
```

## Importance Levels

- **0.9-1.0**: Critical (allergies, security rules)
- **0.7-0.9**: High (preferences, project structure)
- **0.4-0.7**: Medium (experiences, context)
- **0.1-0.4**: Low (minor details)

## Tips

- Be specific: "User prefers dark mode in VS Code" is better than "prefers dark mode"
- Include context: "Uses pytest for backend tests"
- Claude will automatically remember important-sounding content
