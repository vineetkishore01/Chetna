# Chetna Technical Specification

Chetna is an advanced Long-Term Memory System (LTMS) that implements a **Cognitive Relational Architecture**.

## Architecture Overview

### 1. Hybrid Search Engine
Chetna implements **Reciprocal Rank Fusion (RRF)** to combine two distinct search methodologies:
- **Lexical Search (BM25):** Utilizes SQLite FTS5 to match technical strings (UUIDs, Git hashes, Paths) with 100% accuracy.
- **Semantic Search (Vector):** Utilizes cosine similarity on text embeddings to find memories with similar conceptual meaning.

**RRF Formula:**
The final rank is calculated by merging ranks from both sources:
`Score = Σ (1 / (k + rank_i))` where `k` is a constant (default 60).

### 2. Biological Decay System (Ebbinghaus 2.0)
Chetna mimics human memory retention through a mathematical model:
- **Base Formula:** `Importance_new = Importance_old * exp(-t / S)`
- **Active Recall:** Every time a memory is accessed, its **Stability (S)** increases logarithmically.
- **Retention:** Memories that are frequently "remembered" become more stable and decay slower than transient noise.

### 3. Knowledge Graph Integration
Instead of independent vectors, memories are linked via a **Directed Graph**:
- **Chunking:** Large documents are split into overlapping chunks, each linked to a parent via a `PartOf` relationship.
- **Logical Edges:** Supports `Contradicts`, `Supports`, `Reinforces`, and `PreRequisite` edges to allow agents to perform logical context traversal.

### 4. Technical Entity Extraction
An internal regex-based pipeline automatically identifies and indexes the following entities during ingestion:
- **IPv4 Addresses**
- **Git Hashes (Full & Short)**
- **File Paths**
- **UUIDs**

### 5. Multi-Tenancy (Namespacing)
To support multiple agents on a single server, all data is logically partitioned by a `namespace` string.
- Default namespace: `default`
- Isolation level: Database queries are filtered by `namespace` at the core engine level.

## Data Model

### Memory Struct
| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Primary Key |
| `namespace` | String | Application partition |
| `content` | String | Raw text |
| `entities` | String | Space-separated indexed entities |
| `importance` | Float | Current recall weight (0-1) |
| `embedding` | Blob | Vector data |
| `tags` | JSON | Metadata tags |
| `is_pinned` | Bool | Prevents auto-decay |

### Relationship Struct
| Field | Type | Description |
|-------|------|-------------|
| `source_id` | UUID | From memory |
| `target_id` | UUID | To memory |
| `relationship_type` | Enum | e.g., `part_of`, `contradicts` |
| `strength` | Float | Connection weight |
