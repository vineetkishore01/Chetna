//! Database module - SQLite schema and migrations

pub mod brain;
pub mod embedding;
pub mod relationships;

use rusqlite::{Connection, Result as SqliteResult};
use tracing::info;

/// Initialize the database with schema
pub fn init_db(conn: &Connection) -> SqliteResult<()> {
    info!("📦 Initializing database schema...");
    
    // Enable FTS5
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    
    // Create memories table with embeddings
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            namespace TEXT NOT NULL DEFAULT 'default',
            
            -- Content
            category TEXT NOT NULL DEFAULT 'fact',
            content TEXT NOT NULL,
            entities TEXT NOT NULL DEFAULT '',
            
            -- Intelligence (Wolverine's best)
            importance REAL NOT NULL DEFAULT 0.5,
            emotional_tone REAL NOT NULL DEFAULT 0.0,
            arousal REAL NOT NULL DEFAULT 0.0,
            
            -- Embeddings for semantic search
            embedding BLOB,
            embedding_model TEXT,
            embedding_created_at TEXT,
            
            -- Tags for filtering
            tags TEXT NOT NULL DEFAULT '[]',
            memory_type TEXT NOT NULL DEFAULT 'fact',
            
            -- Access tracking
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            
            -- Metadata
            source TEXT NOT NULL DEFAULT 'agent',
            scope TEXT NOT NULL DEFAULT 'global',
            
            -- Pinning and categorization
            is_pinned INTEGER NOT NULL DEFAULT 0,
            memory_category TEXT NOT NULL DEFAULT 'general',
            
            -- Ranking
            last_ranked TEXT,
            rank_source TEXT,
            
            deleted_at TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_memories_session ON memories(session_id);
        CREATE INDEX IF NOT EXISTS idx_memories_namespace ON memories(namespace);
        CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);
        CREATE INDEX IF NOT EXISTS idx_memories_memory_category ON memories(memory_category);
        CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_updated ON memories(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_content ON memories(content);
        "#,
    )?;

    // Create FTS5 virtual table for keyword search
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            content,
            category,
            tags,
            entities,
            namespace,
            memory_id UNINDEXED
        );

        -- Populate FTS table if empty but memories exist
        INSERT INTO memories_fts(content, category, tags, entities, namespace, memory_id)
        SELECT content, category, tags, entities, namespace, id FROM memories 
        WHERE id NOT IN (SELECT memory_id FROM memories_fts);
        "#,
    )?;

    // Create triggers to keep FTS in sync
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
            INSERT INTO memories_fts(content, category, tags, entities, namespace, memory_id)
            VALUES (NEW.content, NEW.category, NEW.tags, NEW.entities, NEW.namespace, NEW.id);
        END;

        CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
            DELETE FROM memories_fts WHERE memory_id = OLD.id;
        END;

        CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
            DELETE FROM memories_fts WHERE memory_id = OLD.id;
            INSERT INTO memories_fts(content, category, tags, entities, namespace, memory_id)
            VALUES (NEW.content, NEW.category, NEW.tags, NEW.entities, NEW.namespace, NEW.id);
        END;
        "#,
    )?;

    // Create sessions table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            namespace TEXT NOT NULL DEFAULT 'default',
            name TEXT,
            agent_id TEXT,
            project TEXT,
            directory TEXT,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            summary TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_sessions_namespace ON sessions(namespace);
        "#,
    )?;
    
    // Create embedding cache table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS embedding_cache (
            content_hash TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            embedding BLOB NOT NULL,
            model TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_embedding_cache_hash ON embedding_cache(content_hash);
        "#,
    )?;
    
    // Create memory relationships table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_relationships (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relationship_type TEXT NOT NULL,
            strength REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (source_id) REFERENCES memories(id),
            FOREIGN KEY (target_id) REFERENCES memories(id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_memrel_source ON memory_relationships(source_id);
        CREATE INDEX IF NOT EXISTS idx_memrel_target ON memory_relationships(target_id);
        CREATE INDEX IF NOT EXISTS idx_memrel_type ON memory_relationships(relationship_type);
        "#,
    )?;
    
    // Create multimodal memories table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS multimodal_memories (
            id TEXT PRIMARY KEY,
            memory_id TEXT,
            namespace TEXT NOT NULL DEFAULT 'default',
            content_type TEXT NOT NULL,
            content TEXT NOT NULL,
            mime_type TEXT,
            file_size INTEGER,
            embedding BLOB,
            embedding_model TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (memory_id) REFERENCES memories(id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_multimodal_memory ON multimodal_memories(memory_id);
        CREATE INDEX IF NOT EXISTS idx_multimodal_type ON multimodal_memories(content_type);
        "#,
    )?;

    // Create history events table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS history_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            namespace TEXT NOT NULL DEFAULT 'default',
            session_id TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_history_events_timestamp ON history_events(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_history_events_type ON history_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_history_events_namespace ON history_events(namespace);
        "#,
    )?;

    // Create query results table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS query_results (
            id TEXT PRIMARY KEY,
            event_id TEXT NOT NULL,
            memory_id TEXT NOT NULL,
            rank INTEGER NOT NULL,
            similarity_score REAL,
            recall_score REAL,
            FOREIGN KEY (event_id) REFERENCES history_events(id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_query_results_event ON query_results(event_id);
        CREATE INDEX IF NOT EXISTS idx_query_results_memory ON query_results(memory_id);
        "#,
    )?;
    
    info!("✅ Database schema initialized");
    Ok(())
}

pub fn migrate_db(conn: &Connection) -> SqliteResult<()> {
    info!("🔄 Running database migrations...");
    
    let mut has_is_pinned = false;
    let mut has_memory_category = false;
    let mut has_last_ranked = false;
    let mut has_rank_source = false;
    let mut has_entities = false;
    let mut has_consolidated_into_id = false;
    let mut has_namespace_memories = false;
    let mut has_namespace_sessions = false;

    {
        let mut stmt = conn.prepare("PRAGMA table_info(memories)")?;
        let rows = stmt.query_map([], |row| {
            row.get::<_, String>(1)
        })?;
        for name in rows.flatten() {
            match name.as_str() {
                "is_pinned" => has_is_pinned = true,
                "memory_category" => has_memory_category = true,
                "last_ranked" => has_last_ranked = true,
                "rank_source" => has_rank_source = true,
                "entities" => has_entities = true,
                "consolidated_into_id" => has_consolidated_into_id = true,
                "namespace" => has_namespace_memories = true,
                _ => {}
            }
        }
    }

    {
        let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
        let rows = stmt.query_map([], |row| {
            row.get::<_, String>(1)
        })?;
        for name in rows.flatten() {
            if name == "namespace" {
                has_namespace_sessions = true;
                break;
            }
        }
    }
    
    if !has_namespace_memories {
        conn.execute("ALTER TABLE memories ADD COLUMN namespace TEXT NOT NULL DEFAULT 'default'", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_namespace ON memories(namespace)", [])?;
        info!("   Added namespace column to memories");
    }
    if !has_namespace_sessions {
        conn.execute("ALTER TABLE sessions ADD COLUMN namespace TEXT NOT NULL DEFAULT 'default'", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_sessions_namespace ON sessions(namespace)", [])?;
        info!("   Added namespace column to sessions");
    }
    if !has_consolidated_into_id {
        conn.execute("ALTER TABLE memories ADD COLUMN consolidated_into_id TEXT", [])?;
        info!("   Added consolidated_into_id column");
    }
    if !has_entities {
        conn.execute("ALTER TABLE memories ADD COLUMN entities TEXT NOT NULL DEFAULT ''", [])?;
        info!("   Added entities column");
    }
    if !has_is_pinned {
        conn.execute("ALTER TABLE memories ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0", [])?;
        info!("   Added is_pinned column");
    }
    if !has_memory_category {
        conn.execute("ALTER TABLE memories ADD COLUMN memory_category TEXT NOT NULL DEFAULT 'general'", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_memory_category ON memories(memory_category)", [])?;
        info!("   Added memory_category column and index");
    }
    if !has_last_ranked {
        conn.execute("ALTER TABLE memories ADD COLUMN last_ranked TEXT", [])?;
        info!("   Added last_ranked column");
    }
    if !has_rank_source {
        conn.execute("ALTER TABLE memories ADD COLUMN rank_source TEXT", [])?;
        info!("   Added rank_source column");
    }
    
    info!("✅ Migrations complete");
    Ok(())
}
