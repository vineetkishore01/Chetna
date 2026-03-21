# Chetna - Claude Code Memory Plugin

> Give Claude Code permanent memory that persists across sessions

## The Problem

Claude Code forgets everything between sessions:
- User preferences you mentioned once
- Project context from last week
- Important facts about your workflow

## The Solution

Chetna plugin gives Claude permanent memory via semantic search + importance-weighted recall.

## Installation

### Prerequisites

1. Install Chetna server:
```bash
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
cargo build --release
./target/release/chetna
```

Server runs on `http://localhost:1987`

2. Install the plugin:
```bash
# Clone this repo (if not already)
git clone https://github.com/vineetkishore01/Chetna.git

# Copy plugin to Claude plugins directory
cp -r Chetna/claude-code-plugin ~/.claude/plugins/chetna
```

Or use Claude Code's plugin install command:
```bash
claude plugin install ~/.claude/plugins/chetna
```

## Usage

### Commands

| Command | Description |
|---------|-------------|
| `/chetna:remember [content]` | Store important info |
| `/chetna:recall [query]` | Search memories |
| `/chetna:search [query]` | Advanced search |
| `/chetna:stats` | View statistics |

### Proactive Memory

The skill teaches Claude to automatically remember:
- User preferences ("I prefer dark mode")
- Project context ("We use Next.js")
- Important facts ("John handles API")

Claude will proactively recall relevant memories when you ask questions like:
- "What IDE do I use?"
- "What's this project's setup?"
- "Do you remember my deployment process?"

## Example Session

```
You: I prefer using pytest for testing, not unittest
Claude: I'll remember that you prefer pytest. Stored: "User prefers pytest over unittest" ✓

[Session ends]

You: What testing framework do I use?
Claude: You prefer pytest over unittest. I remembered from our previous session! ✓
```

## Features

- **Semantic Search**: Find memories by meaning, not just keywords
- **Importance Weighting**: Critical memories (0.9+) persist; trivial ones fade
- **Emotional Memory**: Tracks valence for better recall
- **Auto-Decay**: Ebbinghaus forgetting curve simulation
- **Sessions**: Group memories by context/project
- **Fully Local**: No cloud, no external APIs except Ollama

## Configuration

Default server URL: `http://localhost:1987`

To customize, set environment variable:
```bash
export CHETNA_URL=http://your-server:1987
```

## License

MIT + Trademark protections. See LICENSE file for details.

## Author

Vineet Kishore - @vineetkishore01

## Repository

https://github.com/vineetkishore01/Chetna
