# 🧠 Chetna: Long-Term Memory for AI Agents

**Chetna** (Sanskrit for *awareness*) is a high-performance, standalone **Long-Term Memory as a Service (LTMS)**. It gives your AI agents a persistent brain that remembers facts, preferences, and context across sessions—so they never have "context amnesia" again.

Designed for developers and researchers, Chetna turns a simple vector database into a smart relational engine that learns and evolves.

---

## 🚀 Why Chetna?

Most AI agents forget everything once you close the chat. They hallucinate details and waste your expensive context window on old, irrelevant information.

**Chetna solves this by:**
- 🧩 **Persistent Memory:** Store facts once, retrieve them forever.
- 🔍 **Hybrid Search:** Finds the *exact* code snippet or IP address you mentioned, not just "similar" concepts.
- 📉 **Biological Decay:** Automatically forgets unimportant noise while "Active Recall" keeps vital info fresh.
- 🤝 **Knowledge Graph:** Links related memories together, allowing agents to "walk" through complex logic.

---

## ✨ Features You'll Love

### 🖥️ Zero-Config Dashboard
Chetna comes with a beautiful, minimalist web dashboard. On your first launch, a guided setup wizard helps you connect to your embedding provider (like **Ollama** or **OpenAI**) and validates everything instantly. 

### 🧬 Semantic + Keyword Search (RRF)
We use **Reciprocal Rank Fusion** to combine the power of vector similarity with exact keyword matching. 
*Example: It knows what you mean when you ask "how do I fix the bug?", but also perfectly finds the exact error code `E0432`.*

### 🛡️ Multi-Agent Namespacing
Running a fleet of agents? Use **Namespaces** to keep their memories separate. Your "Python Dev Agent" won't get confused by memories from your "Marketing Research Agent."

### 🔗 Protocol Ready (MCP)
Chetna fully supports the **Model Context Protocol (MCP)**. If your agent supports MCP (like Claude Desktop or Windsurf), it can autonomously manage its own memory without you writing a single line of integration code.

---

## 🛠️ Quick Start (in 60 Seconds)

The easiest way to get started is with our automated installer.

```bash
# Clone the repository
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna

# Run the interactive installer
./install.sh
```

**What happens next?**
1. The script will check if you have **Rust** and **Ollama** installed.
2. It will build the Chetna binary for your system.
3. Once finished, visit **`http://localhost:1987`** to complete the setup.

---

## 🔌 Using Chetna with your Agent

Chetna is built as a **Memory-as-a-Service**. You can connect via a simple REST API.

### 1. Store a Memory
```python
import requests

requests.post("http://localhost:1987/api/memory", json={
    "content": "The user prefers Python over JavaScript for data processing.",
    "importance": 0.9,
    "category": "preference"
})
```

### 2. Search for Context
```python
# Ask Chetna for relevant context before sending a prompt to your AI
response = requests.get("http://localhost:1987/api/memory/search", params={
    "query": "Which language should I use for the data script?"
})
context = response.json()
# Now use 'context' to ground your AI's response!
```

---

## 🏗️ Technical Stack
- **Core:** Rust (for lightning-fast performance)
- **Database:** SQLite + FTS5 + Vector Extensions
- **UI:** OLED Minimalist (Zero bloat, pure JS/CSS)
- **Protocol:** REST API & MCP Support

## 📄 Documentation
- 📖 [Quickstart Guide](docs/QUICKSTART.md)
- 📡 [REST API Reference](docs/api.md)
- 🔗 [MCP Integration](docs/mcp.md)
- 📐 [Technical Specification](docs/SPEC.md)

---
*Built with ❤️ for the future of autonomous agents.*
