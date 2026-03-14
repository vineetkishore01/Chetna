# AI Agent Integration Guide

A comprehensive guide for AI agents on how to use Chetna for memory management.

---

## What is Chetna?

Chetna is a **long-term memory system** for AI agents. It allows your AI to:

1. **Remember** - Store important information about users, tasks, and context
2. **Recall** - Find relevant memories when needed using semantic search
3. **Reason** - Build context from memories for AI prompts
4. **Forget** - Automatically decay irrelevant memories over time

---

## Quick Start for Agents

### Step 1: Connect to Chetna

```python
import requests

CHETNA_URL = "http://localhost:1987"  # Or your server URL

def chetna_request(method, params=None):
    """Make MCP request to Chetna"""
    response = requests.post(
        f"{CHETNA_URL}/mcp",
        json={"method": method, "params": params or {}}
    )
    result = response.json()
    if result.get("error"):
        raise Exception(f"Chetna error: {result['error']}")
    return result.get("result", {})
```

### Step 2: Store Important Information

When the user tells you something important:

```python
# User says: "My name is Vineet and I'm a Python developer"
def store_user_fact(content, importance=0.8):
    chetna_request("memory_create", {
        "content": content,
        "importance": importance,
        "memory_type": "fact",
        "tags": ["user", "personal"]
    })

# Store it
store_user_fact("User's name is Vineet and he's a Python developer", importance=0.9)
```

### Step 3: Recall When Needed

When you need to answer a question about the user:

```python
def get_user_context(query, max_tokens=500):
    """Build context from relevant memories"""
    result = chetna_request("memory_context", {
        "query": query,
        "max_tokens": max_tokens
    })
    return result.get("context", "")

# When user asks: "What do you know about me?"
context = get_user_context("What is the user's name and profession?")
# Returns: "[fact] User's name is Vineet and he's a Python developer (importance: 0.90)"
```

---

## Memory Types

Choose the right memory type for different information:

| Type | Use Case | Example |
|------|----------|---------|
| `fact` | Factual knowledge | "User's name is Vineet" |
| `preference` | User preferences | "User prefers dark mode" |
| `rule` | Rules and constraints | "Always backup before updates" |
| `experience` | Past experiences | "User had a great meeting" |
| `skill_learned` | Learned skills | "User knows Python" |

---

## Importance Guide

Set importance based on how critical the memory is:

| Importance | When to Use |
|------------|-------------|
| **0.9 - 1.0** | Critical information that should NEVER be forgotten (name, identity, critical rules) |
| **0.7 - 0.9** | Very important information (preferences, allergies, important context) |
| **0.5 - 0.7** | Moderately important (general facts, non-critical context) |
| **0.3 - 0.5** | Low importance (temporary context, minor details) |
| **0.1 - 0.3** | Very low importance (will decay/flush eventually) |
| **< 0.1** | Nearly useless - will be auto-deleted |

---

## Agent Patterns

### Pattern 1: Remember User Preferences

```python
def remember_preference(preference_type, value, importance=0.8):
    """Store a user preference"""
    chetna_request("memory_create", {
        "content": f"User prefers {preference_type}: {value}",
        "importance": importance,
        "memory_type": "preference",
        "tags": ["preference", preference_type]
    })

# Usage
remember_preference("dark mode", "enabled", importance=0.85)
remember_preference("programming language", "Python", importance=0.9)
```

### Pattern 2: Remember Important Rules

```python
def remember_rule(rule, importance=0.9):
    """Store an important rule or constraint"""
    chetna_request("memory_create", {
        "content": f"IMPORTANT RULE: {rule}",
        "importance": importance,
        "memory_type": "rule",
        "tags": ["rule", "important"]
    })

# Usage
remember_rule("Always backup database before running migrations")
remember_rule("User is allergic to nuts", importance=1.0)
```

### Pattern 3: Answer Questions with Memory

```python
def answer_with_memory(question):
    """Answer a question using memory context"""
    # Get relevant context
    context = get_user_context(question, max_tokens=500)
    
    if not context:
        return "I don't have any information about that."
    
    # Build prompt for your AI
    prompt = f"""Based on my memory of the user:

{context}

Question: {question}

Answer:"""
    
    # Send to your AI (e.g., GPT-4, Claude, etc.)
    answer = your_llm.generate(prompt)
    return answer

# Usage
answer = answer_with_memory("What does the user prefer for UI?")
# AI gets context: "[preference] User prefers dark mode..."
# AI answers: "You prefer dark mode for UI."
```

### Pattern 4: Session-Based Memory

```python
def start_session(agent_id, session_name):
    """Start a new session"""
    result = chetna_request("session_create", {
        "name": session_name,
        "agent_id": agent_id
    })
    return result["id"]

def end_session(session_id):
    """End the current session"""
    chetna_request("session_end", {"id": session_id})

# Usage
session_id = start_session("assistant", "User Conversation - 2024-01-15")
# ... conversation happens ...
end_session(session_id)
```

### Pattern 5: Find Related Memories

```python
def find_related_memories(memory_id, limit=5):
    """Find memories related to a specific memory"""
    result = chetna_request("memory_related", {
        "id": memory_id,
        "limit": limit
    })
    return result.get("memories", [])

# Usage
# User mentioned something, find related context
memories = find_related_memories("memory-uuid-123")
for mem in memories:
    print(f"- {mem['content']}")
```

---

## Semantic Search Examples

Chetna understands meaning, not just keywords:

### Example 1: Direct Match

```python
# Store: "User's name is Vineet"
# Query: "What is the user's name?"

chetna_request("memory_create", {
    "content": "User's name is Vineet",
    "importance": 0.95
})

# Later query
result = chetna_request("memory_search", {
    "query": "What is the user's name?",
    "semantic": True
})
# Returns: "User's name is Vineet" ✓
```

### Example 2: Synonyms

```python
# Store: "User likes coding in Python"
# Query: "What programming language does user prefer?"

chetna_request("memory_create", {
    "content": "User likes coding in Python",
    "importance": 0.8
})

result = chetna_request("memory_search", {
    "query": "What programming language does user prefer?",
    "semantic": True
})
# Returns: "User likes coding in Python" ✓
```

### Example 3: Related Concepts

```python
# Store: "User prefers dark mode UI"
# Query: "What is the UI theme?"

chetna_request("memory_create", {
    "content": "User prefers dark mode in all applications",
    "importance": 0.85
})

result = chetna_request("memory_search", {
    "query": "What is the UI theme?",
    "semantic": True
})
# Returns: "User prefers dark mode..." ✓
```

---

## Best Practices

### Do's

✅ **Store important information immediately** when user shares it
✅ **Set appropriate importance** - higher for critical info
✅ **Use semantic search** - it understands meaning
✅ **Pin critical memories** - they'll never decay
✅ **Clean up old sessions** - use session management

### Don'ts

❌ Don't store everything - focus on important context
❌ Don't set everything as importance 1.0 - it defeats the purpose
❌ Don't ignore memory decay - it's there for a reason
❌ Don't forget to query memory before answering questions

---

## Troubleshooting

### "Search returns no results"

1. Check if memories have embeddings:
```python
memories = chetna_request("memory_list", {"limit": 1})
print(memories[0].get("embedding_model"))  # Should not be null
```

2. Lower the similarity threshold:
```python
result = chetna_request("memory_search", {
    "query": "your query",
    "semantic": True
    # Use keyword search as fallback
})
if not result.get("memories"):
    result = chetna_request("memory_search", {
        "query": "your query",
        "semantic": False
    })
```

### "Context is too long"

Reduce max_tokens:
```python
context = chetna_request("memory_context", {
    "query": "your query",
    "max_tokens": 200  # Smaller limit
})
```

### "Ollama connection failed"

1. Check Ollama is running
2. Verify EMBEDDING_BASE_URL in config
3. Check model is installed: `ollama list`

---

## Complete Agent Example

```python
import requests

class ChetnaAgent:
    def __init__(self, base_url="http://localhost:1987"):
        self.base_url = base_url
        self.session_id = None
    
    def _call(self, method, params=None):
        resp = requests.post(
            f"{self.base_url}/mcp",
            json={"method": method, "params": params or {}}
        )
        result = resp.json()
        if result.get("error"):
            raise RuntimeError(result["error"])
        return result.get("result", {})
    
    # Memory operations
    def remember(self, content, importance=0.7, memory_type="fact", tags=None):
        return self._call("memory_create", {
            "content": content,
            "importance": importance,
            "memory_type": memory_type,
            "tags": tags or []
        })
    
    def recall(self, query, semantic=True, limit=5):
        return self._call("memory_search", {
            "query": query,
            "semantic": semantic,
            "limit": limit
        })
    
    def get_context(self, query, max_tokens=500):
        result = self._call("memory_context", {
            "query": query,
            "max_tokens": max_tokens
        })
        return result.get("context", "")
    
    def forget(self, memory_id):
        return self._call("memory_delete", {"id": memory_id})
    
    # Session management
    def start_session(self, name, agent_id="agent"):
        result = self._call("session_create", {
            "name": name,
            "agent_id": agent_id
        })
        self.session_id = result["id"]
        return self.session_id
    
    def end_session(self):
        if self.session_id:
            self._call("session_end", {"id": self.session_id})
            self.session_id = None
    
    # Utility
    def stats(self):
        return self._call("stats_get", {})


# Usage example
agent = ChetnaAgent("http://localhost:1987")

# Remember user info
agent.remember("User's name is Vineet", importance=0.95)
agent.remember("User is a Python developer", importance=0.85)
agent.remember("User prefers dark mode UI", importance=0.8, memory_type="preference")

# Later: Answer question
context = agent.get_context("What do you know about the user?")
print(f"Context: {context}")

# Use context in your AI prompt
# answer = llm.generate(f"Context: {context}\n\nQuestion: What's my name?")
```

---

## Next Steps

1. **Read the API Reference** - See [API Reference](./api.md)
2. **Read the MCP Protocol** - See [MCP Reference](./mcp.md)
3. **Deploy with Docker** - See [Manual](./manual.md#docker-deployment)
