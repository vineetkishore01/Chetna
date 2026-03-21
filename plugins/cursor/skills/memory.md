# Memory Assistant Skill

This skill teaches Claude Code to proactively use Chetna memory system.

## Overview

Chetna is a local memory system that gives Claude permanent memory across sessions. Use it to remember user preferences, project context, and important facts.

## When to Proactively Remember

Automatically store memories when user mentions:

### User Preferences
- "I prefer..." / "I like..." / "I don't like..."
- "I always use..." / "I use [tool] instead of..."
- "Don't use..." (rules)
- Technology preferences (IDEs, languages, frameworks)

### Project Context
- "We're using..." / "This project uses..."
- "The team..." / "We have..."
- Architecture decisions
- Important files or conventions

### Personal Facts
- Names (user, team members)
- Contact information
- Ongoing projects or tasks
- Deadlines or important dates

## Automatic Recall

Before responding to queries like:
- "What IDE do I use?"
- "What's my coding style?"
- "What does this project use?"
- "Do you remember when..."

Use `/memory-recall` to fetch relevant memories.

## Memory Importance Guide

| Content Type | Importance | Example |
|--------------|------------|---------|
| Critical | 0.9-1.0 | Allergies, security rules |
| High | 0.7-0.9 | Preferences, project structure |
| Medium | 0.4-0.7 | Past experiences, context |
| Low | 0.1-0.4 | Minor details, transient info |

## Integration Points

### On Session Start
- Automatically recall recent important memories
- Check for project-specific context

### Before Major Actions
- Remember user preferences about code style
- Check for relevant project rules

### On User Feedback
- Positive feedback → increase importance
- Corrections → update memory

## Commands Available

| Command | Purpose |
|---------|---------|
| `/memory-remember [content]` | Store important info |
| `/memory-recall [query]` | Search memories |
| `/memory-search [query]` | Advanced search |
| `/memory-stats` | View statistics |

## Technical Details

- **Server**: localhost:1987 (default)
- **Protocol**: HTTP REST + MCP
- **Storage**: SQLite with embeddings
- **Search**: Semantic (meaning-based) + importance-weighted

## First-Time Setup

1. Install Chetna: `git clone https://github.com/vineetkishore01/Chetna`
2. Build: `cd Chetna && cargo build --release`
3. Run: `./target/release/chetna`
4. Start Claude Code in the project directory

The plugin auto-connects to local Chetna server.
