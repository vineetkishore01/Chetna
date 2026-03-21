"""
Chetna SDK - Python library for AI agents to use Chetna memory system.

Usage:
    from chetna import Chetna
    
    # Connect to Chetna
    memory = Chetna("http://localhost:1987")
    
    # Store memory
    memory.remember("User prefers dark mode", importance=0.9)
    
    # Recall later
    context = memory.recall("What does user prefer?")
    
    # Build context for AI
    ctx = memory.context("What do you know about the user?")
"""

import requests
import os
import json
import time
from typing import Optional, List, Dict, Any
from dataclasses import dataclass, field
from datetime import datetime

__version__ = "0.2.0"

# ==================== DATA CLASSES ====================

@dataclass
class Memory:
    id: str
    content: str
    importance: float
    memory_type: str
    category: str
    tags: List[str] = field(default_factory=list)
    created_at: str = ""
    updated_at: str = ""
    is_pinned: bool = False

@dataclass
class Session:
    id: str
    project: str
    directory: str = ""
    started_at: str = ""
    ended_at: str = ""

@dataclass
class Skill:
    id: str
    name: str
    description: str
    code: str
    language: str
    enabled: bool = True

@dataclass
class Stats:
    total_memories: int = 0
    total_sessions: int = 0
    avg_importance: float = 0.0
    memories_by_type: Dict[str, int] = field(default_factory=dict)
    memories_by_category: Dict[str, int] = field(default_factory=dict)

@dataclass  
class Capabilities:
    semantic_search: bool
    context_building: bool
    mcp_tools: bool
    batch_operations: bool
    pinning: bool
    categories: bool
    emotional_memory: bool
    embedding_model: str = ""

# ==================== CHETNA CLIENT ====================

class Chetna:
    """AI Agent Memory System - Full feature client"""
    
    def __init__(self, url: str = "http://localhost:1987", api_key: Optional[str] = None):
        self.url = url.rstrip('/')
        self.api_key = api_key or os.environ.get("CHETNA_API_KEY", "")
        self._capabilities: Optional[Capabilities] = None
    
    def _headers(self) -> Dict[str, str]:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers
    
    def _handle_response(self, resp: requests.Response) -> Dict:
        resp.raise_for_status()
        return resp.json()
    
    # ==================== CONNECTION ====================
    
    @classmethod
    def auto_connect(cls, url: str = "http://localhost:1987", api_key: Optional[str] = None, 
                    timeout: int = 30) -> "Chetna":
        """Auto-connect to Chetna. Waits for server to be ready."""
        instance = cls(url, api_key)
        for i in range(timeout):
            if instance.is_ready():
                return instance
            time.sleep(1)
        raise ConnectionError(f"Could not connect to Chetna at {url}")
    
    def is_ready(self) -> bool:
        """Check if Chetna is running and ready"""
        try:
            resp = requests.get(f"{self.url}/health", timeout=2)
            return resp.text == "OK"
        except:
            return False
    
    def capabilities(self) -> Capabilities:
        """Get what Chetna can do"""
        if self._capabilities:
            return self._capabilities
        
        try:
            resp = requests.get(f"{self.url}/api/capabilities", 
                              headers=self._headers(), timeout=5)
            data = resp.json()
            
            features = data.get("features", {})
            models = data.get("models", {})
            
            self._capabilities = Capabilities(
                semantic_search=features.get("semantic_search", False),
                context_building=features.get("context_building", False),
                mcp_tools=features.get("mcp_tools", True),
                batch_operations=features.get("batch_operations", True),
                pinning=features.get("pinning", True),
                categories=features.get("categories", True),
                emotional_memory=features.get("emotional_memory", True),
                embedding_model=models.get("embedding", {}).get("model", "")
            )
        except:
            self._capabilities = Capabilities(
                semantic_search=False,
                context_building=False,
                mcp_tools=False,
                batch_operations=False,
                pinning=False,
                categories=False,
                emotional_memory=False
            )
        
        return self._capabilities
    
    # ==================== CORE MEMORY OPERATIONS ====================
    
    def remember(self, content: str, 
                importance: float = 0.5,
                memory_type: str = "fact",
                category: str = "fact",
                tags: Optional[List[str]] = None,
                valence: float = 0.0,
                arousal: float = 0.0,
                auto_score: bool = False,
                session_id: Optional[str] = None,
                source: str = "agent") -> Memory:
        """
        Store a memory.
        
        Usage:
            memory.remember("User prefers dark mode", importance=0.9)
            memory.remember("User is allergic to peanuts", importance=1.0, memory_type="rule")
            memory.remember("Fixed auth bug", memory_type="experience", auto_score=True)
        """
        data = {
            "content": content,
            "importance": importance,
            "memory_type": memory_type,
            "category": category,
            "tags": tags or [],
            "valence": valence,
            "arousal": arousal,
            "auto_score": auto_score,
            "source": source
        }
        if session_id:
            data["session_id"] = session_id
        
        result = self._handle_response(requests.post(
            f"{self.url}/api/memory",
            json=data,
            headers=self._headers(),
            timeout=30
        ))
        
        return Memory(
            id=result["id"],
            content=result["content"],
            importance=result["importance"],
            memory_type=result["memory_type"],
            category=result.get("category", category),
            tags=result.get("tags", []),
            created_at=result.get("created_at", ""),
            is_pinned=result.get("is_pinned", False)
        )
    
    def remember_batch(self, memories: List[Dict]) -> List[Memory]:
        """
        Store multiple memories at once.
        
        Usage:
            results = memory.remember_batch([
                {"content": "Fact 1", "importance": 0.8},
                {"content": "Fact 2", "importance": 0.6},
                {"content": "Fact 3", "memory_type": "rule"}
            ])
        """
        result = self._handle_response(requests.post(
            f"{self.url}/api/memory/batch",
            json={"memories": memories},
            headers=self._headers(),
            timeout=60
        ))
        
        created = []
        for m in result.get("created", []):
            created.append(Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            ))
        return created
    
    def recall(self, query: str, limit: int = 10) -> List[Memory]:
        """
        Semantic search - find memories by meaning.
        
        Usage:
            memories = memory.recall("What does user like?")
            for m in memories:
                print(f"- {m.content} (importance: {m.importance})")
        """
        results = self._handle_response(requests.get(
            f"{self.url}/api/memory/search",
            params={"query": query, "limit": limit},
            headers=self._headers(),
            timeout=30
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    def search(self, query: str, limit: int = 20) -> List[Memory]:
        """Alias for recall() - keyword search"""
        return self.recall(query, limit)
    
    def semantic_search(self, query: str, limit: int = 20) -> List[Memory]:
        """Semantic search - find by meaning"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/memory/search/semantic",
            params={"query": query, "limit": limit},
            headers=self._headers(),
            timeout=30
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    def context(self, query: str, max_tokens: int = 1000) -> str:
        """
        Build context for AI prompt.
        
        Usage:
            context = memory.context("What are user preferences?", max_tokens=500)
        """
        result = self._handle_response(requests.post(
            f"{self.url}/api/memory/context",
            json={"query": query, "max_tokens": max_tokens},
            headers=self._headers(),
            timeout=60
        ))
        
        return result.get("context", "")
    
    def get_all(self, limit: int = 100, category: Optional[str] = None) -> List[Memory]:
        """Get all memories"""
        params = {"limit": limit}
        if category:
            params["category"] = category
        
        results = self._handle_response(requests.get(
            f"{self.url}/api/memory",
            params=params,
            headers=self._headers(),
            timeout=10
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    def get(self, memory_id: str) -> Memory:
        """Get a specific memory by ID"""
        result = self._handle_response(requests.get(
            f"{self.url}/api/memory/{memory_id}",
            headers=self._headers(),
            timeout=10
        ))
        
        return Memory(
            id=result["id"],
            content=result["content"],
            importance=result["importance"],
            memory_type=result.get("memory_type", "fact"),
            category=result.get("category", "fact"),
            tags=result.get("tags", []),
            created_at=result.get("created_at", ""),
            updated_at=result.get("updated_at", ""),
            is_pinned=result.get("is_pinned", False)
        )
    
    def related(self, memory_id: str, limit: int = 10) -> List[Memory]:
        """Find memories related to a specific memory"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/memory/related/{memory_id}",
            params={"limit": limit},
            headers=self._headers(),
            timeout=30
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    # ==================== MEMORY MANAGEMENT ====================
    
    def forget(self, memory_id: str, permanent: bool = False) -> bool:
        """
        Delete a memory.
        
        Usage:
            memory.forget("memory-id")  # Soft delete
        """
        resp = requests.delete(
            f"{self.url}/api/memory/{memory_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def update(self, memory_id: str, 
               content: Optional[str] = None,
               importance: Optional[float] = None,
               memory_type: Optional[str] = None,
               category: Optional[str] = None,
               tags: Optional[List[str]] = None) -> Memory:
        """Update a memory"""
        data = {}
        if content is not None:
            data["content"] = content
        if importance is not None:
            data["importance"] = importance
        if memory_type is not None:
            data["memory_type"] = memory_type
        if category is not None:
            data["category"] = category
        if tags is not None:
            data["tags"] = tags
        
        result = self._handle_response(requests.patch(
            f"{self.url}/api/memory/{memory_id}",
            json=data,
            headers=self._headers(),
            timeout=10
        ))
        
        return Memory(
            id=result["id"],
            content=result["content"],
            importance=result["importance"],
            memory_type=result.get("memory_type", "fact"),
            category=result.get("category", "fact"),
            tags=result.get("tags", []),
            created_at=result.get("created_at", ""),
            updated_at=result.get("updated_at", ""),
            is_pinned=result.get("is_pinned", False)
        )
    
    def pin(self, memory_id: str) -> bool:
        """Pin a memory (won't be auto-deleted)"""
        resp = requests.post(
            f"{self.url}/api/memory/pin/{memory_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def unpin(self, memory_id: str) -> bool:
        """Unpin a memory"""
        resp = requests.delete(
            f"{self.url}/api/memory/pin/{memory_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def set_category(self, memory_id: str, category: str) -> bool:
        """Set memory category"""
        resp = requests.post(
            f"{self.url}/api/memory/category/{memory_id}",
            json={"category": category},
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def restore(self, memory_id: str) -> bool:
        """Restore a soft-deleted memory"""
        resp = requests.post(
            f"{self.url}/api/memory/restore/{memory_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def deleted(self, limit: int = 100) -> List[Memory]:
        """List soft-deleted memories"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/memory/deleted",
            params={"limit": limit},
            headers=self._headers(),
            timeout=10
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    # ==================== SESSIONS ====================
    
    def create_session(self, project: str, directory: str = "") -> Session:
        """Create a new session"""
        result = self._handle_response(requests.post(
            f"{self.url}/api/session",
            json={"project": project, "directory": directory},
            headers=self._headers(),
            timeout=10
        ))
        
        return Session(
            id=result["id"],
            project=result["project"],
            directory=result.get("directory", ""),
            started_at=result.get("started_at", ""),
            ended_at=result.get("ended_at", "")
        )
    
    def list_sessions(self, limit: int = 50) -> List[Session]:
        """List all sessions"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/session",
            params={"limit": limit},
            headers=self._headers(),
            timeout=10
        ))
        
        return [
            Session(
                id=s["id"],
                project=s["project"],
                directory=s.get("directory", ""),
                started_at=s.get("started_at", ""),
                ended_at=s.get("ended_at", "")
            )
            for s in results
        ]
    
    def get_session(self, session_id: str) -> Session:
        """Get a specific session"""
        result = self._handle_response(requests.get(
            f"{self.url}/api/session/{session_id}",
            headers=self._headers(),
            timeout=10
        ))
        
        return Session(
            id=result["id"],
            project=result["project"],
            directory=result.get("directory", ""),
            started_at=result.get("started_at", ""),
            ended_at=result.get("ended_at", "")
        )
    
    def end_session(self, session_id: str) -> bool:
        """End a session"""
        resp = requests.delete(
            f"{self.url}/api/session/{session_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    def session_memories(self, session_id: str, limit: int = 100) -> List[Memory]:
        """Get all memories from a session"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/session/{session_id}/memories",
            params={"limit": limit},
            headers=self._headers(),
            timeout=10
        ))
        
        return [
            Memory(
                id=m["id"],
                content=m["content"],
                importance=m["importance"],
                memory_type=m.get("memory_type", "fact"),
                category=m.get("category", "fact"),
                tags=m.get("tags", []),
                created_at=m.get("created_at", ""),
                is_pinned=m.get("is_pinned", False)
            )
            for m in results
        ]
    
    # ==================== SKILLS ====================
    
    def list_skills(self) -> List[Skill]:
        """List all skills"""
        results = self._handle_response(requests.get(
            f"{self.url}/api/skill",
            headers=self._headers(),
            timeout=10
        ))
        
        return [
            Skill(
                id=s["id"],
                name=s["name"],
                description=s.get("description", ""),
                code=s.get("code", ""),
                language=s.get("language", "text"),
                enabled=s.get("enabled", True)
            )
            for s in results
        ]
    
    def create_skill(self, name: str, code: str, 
                     description: str = "", 
                     language: str = "text") -> Skill:
        """Create a new skill"""
        result = self._handle_response(requests.post(
            f"{self.url}/api/skill",
            json={
                "name": name,
                "description": description,
                "code": code,
                "language": language
            },
            headers=self._headers(),
            timeout=10
        ))
        
        return Skill(
            id=result["id"],
            name=result["name"],
            description=result.get("description", ""),
            code=result.get("code", ""),
            language=result.get("language", "text"),
            enabled=result.get("enabled", True)
        )
    
    def delete_skill(self, skill_id: str) -> bool:
        """Delete a skill"""
        resp = requests.delete(
            f"{self.url}/api/skill/{skill_id}",
            headers=self._headers(),
            timeout=10
        )
        return resp.status_code == 200
    
    # ==================== ADVANCED OPERATIONS ====================
    
    def decay(self) -> Dict[str, Any]:
        """Apply Ebbinghaus decay formula to memories"""
        return self._handle_response(requests.post(
            f"{self.url}/api/memory/decay",
            headers=self._headers(),
            timeout=60
        ))
    
    def flush(self, threshold: float = 0.1) -> Dict[str, Any]:
        """Flush low-importance memories"""
        return self._handle_response(requests.post(
            f"{self.url}/api/memory/flush",
            json={"threshold": threshold},
            headers=self._headers(),
            timeout=60
        ))
    
    def prune(self, days: int = 30, importance: float = 0.1) -> int:
        """Prune old, low-importance memories"""
        result = self._handle_response(requests.post(
            f"{self.url}/api/memory/prune",
            json={"days": days, "min_importance": importance},
            headers=self._headers(),
            timeout=60
        ))
        return result.get("pruned", 0)
    
    def embed_batch(self) -> int:
        """Embed all unembedded memories"""
        result = self._handle_response(requests.post(
            f"{self.url}/api/memory/embed-batch",
            headers=self._headers(),
            timeout=300
        ))
        return result.get("embedded", 0)
    
    def stats(self) -> Stats:
        """Get memory statistics"""
        result = self._handle_response(requests.get(
            f"{self.url}/api/stats",
            headers=self._headers(),
            timeout=10
        ))
        
        return Stats(
            total_memories=result.get("total_memories", 0),
            total_sessions=result.get("total_sessions", 0),
            avg_importance=result.get("avg_importance", 0.0),
            memories_by_type=result.get("by_type", {}),
            memories_by_category=result.get("by_category", {})
        )
    
    # ==================== CONFIGURATION ====================
    
    def get_config(self) -> Dict[str, Any]:
        """Get current configuration"""
        return self._handle_response(requests.get(
            f"{self.url}/api/config/user",
            headers=self._headers(),
            timeout=10
        ))
    
    def set_config(self, **kwargs) -> Dict[str, Any]:
        """Update configuration"""
        return self._handle_response(requests.post(
            f"{self.url}/api/config/user",
            json=kwargs,
            headers=self._headers(),
            timeout=10
        ))
    
    def connection_status(self) -> Dict[str, Any]:
        """Get connection status for embedding"""
        return self._handle_response(requests.get(
            f"{self.url}/api/status/connections",
            headers=self._headers(),
            timeout=10
        ))
    
    # ==================== MCP PROTOCOL ====================
    
    def mcp(self, method: str, params: Optional[Dict] = None) -> Any:
        """
        Call MCP tool directly.
        
        Usage:
            result = memory.mcp("memory_create", {"content": "test", "importance": 0.5})
            result = memory.mcp("memory_search", {"query": "preferences", "limit": 5})
        """
        resp = requests.post(
            f"{self.url}/mcp",
            json={"method": method, "params": params or {}},
            headers=self._headers(),
            timeout=30
        )
        resp.raise_for_status()
        result = resp.json()
        
        if result.get("error"):
            raise RuntimeError(result["error"])
        
        return result.get("result")
    
    def list_mcp_tools(self) -> List[Dict]:
        """List available MCP tools"""
        result = self._handle_response(requests.get(
            f"{self.url}/mcp",
            headers=self._headers(),
            timeout=10
        ))
        return result.get("tools", [])
    
    # ==================== SHORTCUTS ====================
    
    def __getitem__(self, query: str) -> List[Memory]:
        """Shortcut: memory["what do you like?"]"""
        return self.recall(query)
    
    def __setitem__(self, content: str, importance: Any):
        """Shortcut: memory["preference"] = 0.9"""
        return self.remember(content, importance=float(importance))


# ==================== CONVENIENCE FUNCTIONS ====================

def connect(url: str = "http://localhost:1987", api_key: Optional[str] = None) -> Chetna:
    """Connect to Chetna instance"""
    return Chetna(url, api_key)

def auto_connect(url: str = "http://localhost:1987", api_key: Optional[str] = None) -> Chetna:
    """Auto-connect with discovery"""
    return Chetna.auto_connect(url, api_key)
