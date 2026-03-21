"""
Chetna SDK - Python library for AI agents to use Chetna memory system.

Quick Start:
    from chetna_sdk import Chetna
    
    # Connect
    memory = Chetna("http://localhost:1987")
    
    # Store
    memory.remember("User prefers dark mode", importance=0.9)
    
    # Recall
    context = memory.context("What does user prefer?")
"""

import requests
import os
import time
from typing import Optional, List, Dict, Any
from dataclasses import dataclass

__version__ = "0.1.0"

@dataclass
class Memory:
    id: str
    content: str
    importance: float
    memory_type: str
    category: str
    created_at: str

@dataclass
class Capabilities:
    semantic_search: bool
    context_building: bool
    mcp_enabled: bool
    embedding_model: str
class Chetna:
    """AI Agent Memory System"""
    
    def __init__(self, url: str = "http://localhost:1987", api_key: Optional[str] = None):
        self.url = url.rstrip('/')
        self.api_key = api_key or os.environ.get("CHETNA_API_KEY", "")
        self._capabilities: Optional[Capabilities] = None
    
    def _headers(self) -> Dict[str, str]:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers
    
    @classmethod
    def auto_connect(cls, url: str = "http://localhost:1987", api_key: Optional[str] = None, 
                    timeout: int = 30) -> "Chetna":
        """Auto-connect to Chetna"""
        instance = cls(url, api_key)
        for i in range(timeout):
            if instance.is_ready():
                return instance
            time.sleep(1)
        raise ConnectionError(f"Could not connect to Chetna at {url}")
    
    def is_ready(self) -> bool:
        """Check if Chetna is running"""
        try:
            resp = requests.get(f"{self.url}/health", timeout=2)
            return resp.text == "OK"
        except:
            return False
    
    def capabilities(self) -> Capabilities:
        """Get capabilities"""
        if self._capabilities:
            return self._capabilities
        try:
            resp = requests.get(f"{self.url}/api/capabilities", headers=self._headers(), timeout=5)
            data = resp.json()
            features = data.get("features", {})
            models = data.get("models", {})
            self._capabilities = Capabilities(
                semantic_search=features.get("semantic_search", False),
                context_building=features.get("context_building", False),
                mcp_enabled=features.get("mcp_tools", False),
                embedding_model=models.get("embedding", {}).get("model", "unknown")
            )
        except:
            self._capabilities = Capabilities(False, False, False, "unknown")
        return self._capabilities
    
    def remember(self, content: str, importance: float = 0.7, 
                memory_type: str = "fact", tags: Optional[List[str]] = None,
                category: str = "fact") -> Memory:
        """Store a memory"""
        resp = requests.post(f"{self.url}/api/memory", 
            json={"content": content, "importance": importance, 
                  "memory_type": memory_type, "category": category, "tags": tags or []},
            headers=self._headers(), timeout=30)
        resp.raise_for_status()
        r = resp.json()
        return Memory(r["id"], r["content"], r["importance"], r["memory_type"], r["category"], r["created_at"])
    
    def recall(self, query: str, limit: int = 10) -> List[Memory]:
        """Semantic search"""
        resp = requests.get(f"{self.url}/api/memory/search",
            params={"query": query, "limit": limit}, headers=self._headers(), timeout=30)
        resp.raise_for_status()
        return [Memory(m["id"], m["content"], m["importance"], m["memory_type"], m["category"], m["created_at"]) 
                for m in resp.json()]
    
    def context(self, query: str, max_tokens: int = 1000) -> str:
        """Build context for AI"""
        resp = requests.post(f"{self.url}/api/memory/context",
            json={"query": query, "max_tokens": max_tokens}, headers=self._headers(), timeout=30)
        resp.raise_for_status()
        return resp.json()["context"]
    
    def get_all(self, limit: int = 100) -> List[Memory]:
        """Get all memories"""
        resp = requests.get(f"{self.url}/api/memory", params={"limit": limit}, headers=self._headers(), timeout=10)
        resp.raise_for_status()
        return [Memory(m["id"], m["content"], m["importance"], m["memory_type"], m["category"], m["created_at"])
                for m in resp.json()]
    
    def forget(self, memory_id: str) -> bool:
        """Delete memory"""
        resp = requests.delete(f"{self.url}/api/memory/{memory_id}", headers=self._headers(), timeout=10)
        return resp.status_code == 200
    
    def pin(self, memory_id: str) -> bool:
        """Pin memory"""
        resp = requests.post(f"{self.url}/api/memory/pin/{memory_id}", headers=self._headers(), timeout=10)
        return resp.status_code == 200
    
    def unpin(self, memory_id: str) -> bool:
        """Unpin memory"""
        resp = requests.delete(f"{self.url}/api/memory/pin/{memory_id}", headers=self._headers(), timeout=10)
        return resp.status_code == 200
    
    def stats(self) -> Dict[str, Any]:
        """Get stats"""
        resp = requests.get(f"{self.url}/api/stats", headers=self._headers(), timeout=10)
        resp.raise_for_status()
        return resp.json()
    
    def mcp(self, method: str, params: Optional[Dict] = None) -> Any:
        """Call MCP tool"""
        resp = requests.post(f"{self.url}/mcp",
            json={"method": method, "params": params or {}}, headers=self._headers(), timeout=30)
        resp.raise_for_status()
        result = resp.json()
        if result.get("error"):
            raise RuntimeError(result["error"])
        return result.get("result")
    
    def __getitem__(self, query: str) -> List[Memory]:
        return self.recall(query)
    
    def __setitem__(self, content: str, importance: float):
        return self.remember(content, importance=importance)


def connect(url: str = "http://localhost:1987", api_key: Optional[str] = None) -> Chetna:
    """Connect to Chetna"""
    return Chetna(url, api_key)

def auto_connect(url: str = "http://localhost:1987", api_key: Optional[str] = None) -> Chetna:
    """Auto-connect"""
    return Chetna.auto_connect(url, api_key)
