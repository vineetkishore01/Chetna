//! Database module - SQLite schema and migrations

pub mod brain;
pub mod search;
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
            
            -- Content
            category TEXT NOT NULL DEFAULT 'fact',
            key TEXT,
            content TEXT NOT NULL,
            
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
            
            -- Consolidation
            consolidated INTEGER NOT NULL DEFAULT 0,
            last_consolidated TEXT,
            
            -- Metadata
            source TEXT NOT NULL DEFAULT 'agent',
            source_tool TEXT,
            scope TEXT NOT NULL DEFAULT 'global',
            deleted_at TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_memories_session ON memories(session_id);
        CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);
        CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_memories_content ON memories(content);
        "#,
    )?;

    // Create FTS5 virtual table for keyword search (standalone, not content-associated)
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS memories_fts;
        DROP TRIGGER IF EXISTS memories_ai;
        DROP TRIGGER IF EXISTS memories_ad;
        DROP TRIGGER IF EXISTS memories_au;

        CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
            content,
            category,
            key,
            tags,
            memory_id
        );
        "#,
    )?;

    // Create triggers to keep FTS in sync
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
            INSERT INTO memories_fts(content, category, key, tags, memory_id)
            VALUES (NEW.content, NEW.category, COALESCE(NEW.key, ''), NEW.tags, NEW.id);
        END;

        CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
            DELETE FROM memories_fts WHERE memory_id = OLD.id;
        END;

        CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
            DELETE FROM memories_fts WHERE memory_id = OLD.id;
            INSERT INTO memories_fts(content, category, key, tags, memory_id)
            VALUES (NEW.content, NEW.category, COALESCE(NEW.key, ''), NEW.tags, NEW.id);
        END;
        "#,
    )?;

    // Create sessions table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT,
            agent_id TEXT,
            project TEXT,
            directory TEXT,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            summary TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC);
        "#,
    )?;
    
    // Create skills table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            code TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'text',
            trigger_keywords TEXT NOT NULL DEFAULT '[]',
            enabled INTEGER NOT NULL DEFAULT 1,
            eligible INTEGER NOT NULL DEFAULT 1,
            eligible_reason TEXT,
            success_count INTEGER NOT NULL DEFAULT 0,
            fail_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_skills_enabled ON skills(enabled);
        "#,
    )?;
    
    // Create procedures table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS procedures (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            steps TEXT NOT NULL DEFAULT '[]',
            trigger_keywords TEXT NOT NULL DEFAULT '[]',
            success_count INTEGER NOT NULL DEFAULT 0,
            fail_count INTEGER NOT NULL DEFAULT 0,
            last_used TEXT,
            created_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_procedures_name ON procedures(name);
        "#,
    )?;
    
    // Create embedding cache table (to avoid re-embedding same content)
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
    
    // Create sessions table with cache
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS session_cache (
            session_id TEXT PRIMARY KEY,
            cached_memories TEXT NOT NULL,
            last_accessed TEXT NOT NULL,
            ttl_seconds INTEGER NOT NULL DEFAULT 3600
        );
        
        CREATE INDEX IF NOT EXISTS idx_session_cache_access ON session_cache(last_accessed);
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
                _ => {}
            }
        }
    }
    
    if !has_is_pinned {
        conn.execute("ALTER TABLE memories ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0", [])?;
        info!("   Added is_pinned column");
    }
    if !has_memory_category {
        conn.execute("ALTER TABLE memories ADD COLUMN memory_category TEXT NOT NULL DEFAULT 'general'", [])?;
        info!("   Added memory_category column");
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
