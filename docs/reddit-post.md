# Reddit Post (r/programming or r/LocalLLaMA)

---

## 🧠 I built a memory system for AI agents that actually thinks like a human brain

Hey everyone! I've been working on something I'm really excited about and want to share.

**Chetna** (Hindi for "Consciousness") - a memory system for AI agents that mimics human memory.

### The Problem

Most AI memory solutions are just fancy databases:
- Store memory → Retrieve memory
- Search by keywords
- Return "most similar" results

But that's not how human memory works!

When you ask me "What's my name?", my brain doesn't just search. It considers:
- Is this IMPORTANT? (your name = very important)
- How RECENT was it?
- How OFTEN do I think about it?
- Was it EMOTIONAL?

### My Solution

I built Chetna with **5-factor human-like recall**:

```python
Recall Score = Similarity(40%) + Importance(25%) + Recency(15%) + Access(10%) + Emotion(10%)
```

**Demo time:**

```
User: "My name is Wolverine and my human is Vineet"
[Stored with importance: 0.95]

Later, User asks: "Who owns me?"
[Traditional keyword search: ❌ NO MATCH]
[Chetna semantic search: ✅ "My human is Vineet" - FOUND IT!
```

The embedding model understands that "owns me" = "human is" = related!

### Features

- 🌐 REST API + MCP Protocol for any agent
- 🔍 Semantic search with qwen3-embedding:4b
- 📊 Importance scoring (0.0-1.0)
- 😢 Emotional tone tracking
- 🔄 Auto-consolidation (LLM reviews memories)
- 📉 Ebbinghaus forgetting curve
- 🐳 Docker support

### Code Snippet

```python
# Build context for your AI
import requests

response = requests.post("http://localhost:1987/api/memory/context", json={
    "query": "What do you know about the user?",
    "max_tokens": 500
})

print(response.json()["context"])
# Output:
# [fact] User's name is Vineet (importance: 0.95)
# [preference] User prefers dark mode (importance: 0.85)
```

### Try It

```bash
# Docker (easiest)
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
docker-compose up -d

# Or from source
cargo build --release
./target/release/chetna
```

Server runs on `http://localhost:1987`

### What's Next

Working on:
- Vector database backup/restore
- Memory encryption
- Multi-agent shared memory

Would love feedback! PRs welcome! ⭐

**Repo:** https://github.com/vineetkishore01/Chetna

---

### TL;DR
Built a memory system for AI agents that combines semantic search + importance + recency + access frequency + emotion = human-like recall. Try it, tear it apart, let me know what you think!
