---
name: chetna-stats
description: Show memory statistics and health
---

# /chetna:stats

Display statistics about your permanent memory.

## Usage

```
/chetna:stats
```

## Shows

- Total memories stored
- Memories by category (facts, preferences, rules, experiences)
- Average importance score
- Active sessions

## Example Output

```
📊 Memory Statistics
====================
Total Memories: 47
━━━━━━━━━━━━━━━━━━━━
Facts:        15 (32%)
Preferences:  18 (38%)
Rules:         8 (17%)
Experiences:   6 (13%)
━━━━━━━━━━━━━━━━━━━━
Avg Importance: 0.72
Sessions: 3 active
━━━━━━━━━━━━━━━━━━━━
```

## How It Works

```bash
curl http://localhost:1987/api/stats
```
