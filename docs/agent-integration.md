# AI Agent Integration Guide

**For AI developers building agents that connect to Chetna**

---

## Overview

Chetna provides **persistent memory** for AI agents. Your agent can:

1. **Store** important user information
2. **Retrieve** relevant memories when needed
3. **Build context** automatically for AI prompts

---

## Quick Setup for AI Agents

**Fully autonomous setup:**
```bash
git clone https://github.com/vineetkishore01/Chetna.git
cd Chetna
./install.sh --auto
```

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

---

## Auto-Discovery for AI Agents

**Can an AI agent auto-setup Chetna?**

**Partially.** Here's what agents can and cannot do:

### ✅ What Agents CAN Do Automatically

| Task | How |
|------|-----|
| Check if Chetna is running | `curl http://localhost:1987/health` |
| Detect available models | `GET /api/status/connections` |
| Create memories | `POST /api/memory` |
| Search memories | `GET /api/memory/search` |
| Build context | `POST /api/memory/context` |

### ⚠️ What Requires Human Help

| Task | Why | Agent Prompt |
|------|-----|--------------|
| Install Rust | System-level install | "Please install Rust from https://rustup.rs" |
| Install Ollama | System-level install | "Please install Ollama from https://ollama.ai" |
| Choose embedding model | Preference/RAM constraints | "Which embedding model? (1) qwen3-embedding:4b (best), (2) nomic-embed-text (balanced), (3) gemma3-embed-e2b (lightweight)" |
| Pull models | Takes time/bandwidth | "I need to download the embedding model (~2GB). Continue?" |

### 🤖 Agent Setup Flow

```python
async def setup_chetna_if_needed():
    """AI agent checks and prompts for Chetna setup"""
    
    # Step 1: Check if Chetna is running
    try:
        response = await http.get("http://localhost:1987/health")
        if response.text == "OK":
            print("✅ Chetna is running")
            return True
    except:
        pass
    
    # Step 2: Chetna not running - guide user
    print("❌ Chetna is not running")
    print("")
    print("I need Chetna for long-term memory. Setup options:")
    print("")
    print("Option 1: Quick Setup (recommended)")
    print("  Run: cd /path/to/Chetna && ./setup.sh --auto")
    print("")
    print("Option 2: Docker")
    print("  Run: docker-compose up -d")
    print("")
    print("Option 3: Manual")
    print("  1. Install Rust: https://rustup.rs")
    print("  2. Install Ollama: https://ollama.ai")
    print("  3. Run: ollama pull qwen3-embedding:4b")
    print("  4. Run: cargo build --release && ./target/release/chetna")
    print("")
    
    # Wait for user to complete setup
    while True:
        try:
            response = await http.get("http://localhost:1987/health")
            if response.text == "OK":
                print("✅ Chetna is now running!")
                return True
        except:
            pass
        await asyncio.sleep(2)
```

---

## Memory Operations

### Create Memory

**When:** User shares important information

```python
# MCP
result = mcp_call("memory_create", {
    "content": "User's name is Vineet",
    "importance": 0.95,  # High for identity
    "memory_type": "fact",
    "tags": ["identity", "personal"]
})

# REST
response = requests.post("http://localhost:1987/api/memory", json={
    "content": "User's name is Vineet",
    "importance": 0.95,
    "memory_type": "fact",
    "tags": ["identity", "personal"]
})
```

**Importance Guide:**

| Importance | When to Use | Examples |
|------------|-------------|----------|
| **0.9-1.0** | Critical identity | Name, allergies, core rules |
| **0.7-0.9** | Important preferences | IDE preference, coding style |
| **0.5-0.7** | General facts | Works at Company X |
| **0.3-0.5** | Temporary context | Current project |
| **<0.3** | Trivial details | Will decay automatically |

### Search Memories

**When:** You need relevant context

```python
# MCP
result = mcp_call("memory_search", {
    "query": "What does user do for work?",
    "limit": 5,
    "semantic": True  # Use semantic search
})

# REST
response = requests.get(
    "http://localhost:1987/api/memory/search",
    params={"query": "user profession", "limit": 5}
)
```

**Semantic Search Examples:**

| User Says | Query to Chetna | Finds |
|-----------|-----------------|-------|
| "What's my name?" | "user identity" | "Name is Vineet" |
| "What do I code in?" | "programming language" | "Prefers Python" |
| "Any UI preferences?" | "interface style" | "Dark mode preference" |

### Build Context

**When:** Generating AI response

```python
# MCP
context_result = mcp_call("memory_context", {
    "query": "user work and skills",
    "max_tokens": 500
})

context = context_result["context"]
# Returns: "[fact] User is a software engineer (importance: 0.85)\n[fact] Knows Python (importance: 0.75)"

# Use in your AI prompt
prompt = f"""You are a helpful assistant.

Relevant memories about user:
{context}

User: What programming languages do I know?
Assistant:"""
```

---

## Agent Patterns

### Pattern 1: Remember Before Responding

```python
async def respond_to_user(user_message: str):
    # Step 1: Check if user shared new info
    new_info = extract_facts(user_message)
    
    for fact in new_info:
        # Store before responding
        await chetna.create_memory(
            content=fact,
            importance=estimate_importance(fact),
            memory_type="fact"
        )
    
    # Step 2: Get relevant context
    context = await chetna.build_context(
        query=user_message,
        max_tokens=500
    )
    
    # Step 3: Generate response with context
    response = await llm.generate(f"""
    Context from memory:
    {context}
    
    User: {user_message}
    Assistant:""")
    
    return response
```

### Pattern 2: Session-Based Memory

```python
# Start session when conversation begins
session = mcp_call("session_create", {
    "name": f"Conversation - {datetime.now().isoformat()}",
    "agent_id": "my-agent"
})

# During conversation, memories are tagged with session_id

# End session when done
mcp_call("session_end", {"id": session["id"]})
```

### Pattern 3: Preference Learning

```python
async def learn_preference(user_statement: str):
    """Detect and store user preferences"""
    
    # Detect preference patterns
    if "I prefer" in user_statement or "I like" in user_statement:
        mcp_call("memory_create", {
            "content": user_statement,
            "importance": 0.8,
            "memory_type": "preference",
            "tags": ["preference"]
        })
        
    # Detect rule patterns
    elif "always" in user_statement or "never" in user_statement:
        mcp_call("memory_create", {
            "content": user_statement,
            "importance": 0.95,
            "memory_type": "rule",
            "tags": ["rule", "important"]
        })
```

### Pattern 4: Contradiction Handling

```python
async def handle_potential_contradiction(new_statement: str):
    """Check if new info contradicts existing memories"""
    
    # Search for similar memories
    similar = mcp_call("memory_search", {
        "query": new_statement,
        "limit": 5,
        "semantic": True
    })
    
    for memory in similar.get("memories", []):
        # Check for contradiction signals
        if detect_contradiction(new_statement, memory["content"]):
            # Ask user for clarification
            print(f"I remember you said: '{memory['content']}'")
            print(f"Now you're saying: '{new_statement}'")
            print("Which is correct?")
            
            # User confirms - update old memory
            user_confirmed = get_user_confirmation()
            if user_confirmed:
                mcp_call("memory_delete", {"id": memory["id"]})
                mcp_call("memory_create", {
                    "content": new_statement,
                    "importance": memory["importance"],
                    "memory_type": "fact"
                })
```

---

## Complete Agent Example

```python
import requests
from typing import Optional

class ChetnaAgent:
    """Base class for AI agents using Chetna"""
    
    def __init__(self, chetna_url: str = "http://localhost:1987"):
        self.chetna_url = chetna_url
        self.session_id: Optional[str] = None
    
    def _mcp_call(self, method: str, params: dict) -> dict:
        """Make MCP request"""
        response = requests.post(
            f"{self.chetna_url}/mcp",
            json={"method": method, "params": params}
        )
        result = response.json()
        if result.get("error"):
            raise Exception(result["error"])
        return result.get("result", {})
    
    def remember(self, content: str, importance: float = 0.7, 
                 memory_type: str = "fact", tags: list = None):
        """Store a memory"""
        return self._mcp_call("memory_create", {
            "content": content,
            "importance": importance,
            "memory_type": memory_type,
            "tags": tags or []
        })
    
    def recall(self, query: str, limit: int = 5) -> list:
        """Search memories"""
        result = self._mcp_call("memory_search", {
            "query": query,
            "limit": limit,
            "semantic": True
        })
        return result.get("memories", [])
    
    def get_context(self, query: str, max_tokens: int = 500) -> str:
        """Build context for AI"""
        result = self._mcp_call("memory_context", {
            "query": query,
            "max_tokens": max_tokens
        })
        return result.get("context", "")
    
    def start_session(self, name: str):
        """Start conversation session"""
        result = self._mcp_call("session_create", {
            "name": name,
            "agent_id": "chetna-agent"
        })
        self.session_id = result.get("id")
        return self.session_id
    
    def end_session(self):
        """End conversation session"""
        if self.session_id:
            self._mcp_call("session_end", {"id": self.session_id})
            self.session_id = None


# Usage in your agent
class MyAssistant(ChetnaAgent):
    async def chat(self, user_message: str) -> str:
        # Remember new info
        facts = self.extract_facts(user_message)
        for fact in facts:
            self.remember(fact, importance=0.8)
        
        # Get context
        context = self.get_context(user_message)
        
        # Generate response
        response = await self.llm.generate(f"""
        Memory context:
        {context}
        
        User: {user_message}
        Assistant:""")
        
        return response
    
    def extract_facts(self, text: str) -> list:
        """Extract factual statements from text"""
        # Implement your fact extraction logic
        # This is a simple example
        facts = []
        if "my name is" in text.lower():
            facts.append(f"User's name: {text.split('my name is')[1].strip()}")
        if "i work" in text.lower():
            facts.append(f"User works: {text.split('i work')[1].strip()}")
        return facts
```

---

## Error Handling

```python
def robust_chetna_call(method: str, params: dict, retries: int = 3):
    """MCP call with retry logic"""
    
    for attempt in range(retries):
        try:
            response = requests.post(
                "http://localhost:1987/mcp",
                json={"method": method, "params": params},
                timeout=30
            )
            
            if response.status_code == 401:
                raise Exception("Authentication failed - check API key")
            
            result = response.json()
            
            if result.get("error"):
                # Handle specific errors
                if "embedding" in result["error"].lower():
                    # Embedding issue - fall back to keyword search
                    if method == "memory_search":
                        params["semantic"] = False
                        continue
                
                raise Exception(result["error"])
            
            return result.get("result", {})
            
        except requests.exceptions.ConnectionError:
            if attempt == retries - 1:
                raise Exception("Chetna is not running")
            time.sleep(2 ** attempt)  # Exponential backoff
```

---

## Best Practices

### DO ✅

- Store identity info with high importance (0.9+)
- Use semantic search (it understands meaning)
- Pin critical memories (never forget)
- Build context before responding
- End sessions when done

### DON'T ❌

- Don't store everything (focus on important)
- Don't set all importance to 1.0 (defeats the purpose)
- Don't ignore memory decay (it's useful)
- Don't forget to query memory before answering

---

## Troubleshooting for Agents

| Problem | Solution |
|---------|----------|
| "Connection refused" | Chetna not running - guide user to start it |
| "No memories found" | Try semantic search with broader query |
| "Embedding failed" | Ollama not running - `ollama serve` |
| "Context too long" | Reduce `max_tokens` parameter |
| "401 Unauthorized" | API key required - check `CHETNA_API_KEY` |

---

## API Reference

See [API Reference](api.md) for complete endpoint documentation.

---

**Questions?** Open an issue: https://github.com/vineetkishore01/Chetna/issues
