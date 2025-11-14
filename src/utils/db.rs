//! Database management for Kandil Code
//! 
//! Contains SQLite schema, migrations, and data access layer

use anyhow::Result;
use rusqlite::{Connection, params};
use rusqlite_migration::{Migrations, M};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::path::Path;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub ai_provider: String,
    pub ai_model: String,
    pub last_opened: Option<DateTime<Utc>>,
    pub memory_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub project_id: String,
    pub session_id: String,
    pub role: String,  // 'user' or 'ai'
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SyncQueue {
    pub id: i64,
    pub operation: String,  // 'insert', 'update', 'delete'
    pub table_name: String,
    pub record_id: String,
    pub data: String,
    pub synced: bool,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let mut conn = Connection::open(db_path)?;
        
        // Run migrations
        let migrations = Migrations::new(vec![
            M::up(
                r#"
                CREATE TABLE projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    root_path TEXT NOT NULL UNIQUE,
                    ai_provider TEXT NOT NULL,
                    ai_model TEXT NOT NULL,
                    last_opened TIMESTAMP,
                    memory_enabled BOOLEAN DEFAULT 1,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE memory (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL, -- 'user' or 'ai'
                    content TEXT NOT NULL,
                    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    tokens_used INTEGER,
                    FOREIGN KEY (project_id) REFERENCES projects(id)
                );

                CREATE TABLE sync_queue (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    operation TEXT NOT NULL, -- 'insert', 'update', 'delete'
                    table_name TEXT NOT NULL,
                    record_id TEXT NOT NULL,
                    data TEXT NOT NULL,
                    synced BOOLEAN DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );

                CREATE INDEX idx_memory_project ON memory(project_id);
                CREATE INDEX idx_sync_unsynced ON sync_queue(synced);
                "#,
            ).down(
                r#"
                DROP TABLE IF EXISTS sync_queue;
                DROP TABLE IF EXISTS memory;
                DROP TABLE IF EXISTS projects;
                "#
            )
        ]);

        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn create_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                project.id,
                project.name,
                project.root_path,
                project.ai_provider,
                project.ai_model,
                project.last_opened.map(|t| t.to_rfc3339()),
                project.memory_enabled,
                project.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled, created_at 
             FROM projects WHERE id = ?1"
        )?;

        let project = stmt.query_row([id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                ai_provider: row.get(3)?,
                ai_model: row.get(4)?,
                last_opened: row.get::<_, Option<String>>(5)? 
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                memory_enabled: row.get(6)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?) 
                    .unwrap().with_timezone(&Utc),
            })
        }).optional()?;

        Ok(project)
    }

    pub fn get_project_by_path(&self, path: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled, created_at 
             FROM projects WHERE root_path = ?1"
        )?;

        let project = stmt.query_row([path], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                ai_provider: row.get(3)?,
                ai_model: row.get(4)?,
                last_opened: row.get::<_, Option<String>>(5)? 
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                memory_enabled: row.get(6)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?) 
                    .unwrap().with_timezone(&Utc),
            })
        }).optional()?;

        Ok(project)
    }

    pub fn update_project_last_opened(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET last_opened = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled, created_at 
             FROM projects ORDER BY last_opened DESC"
        )?;

        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                ai_provider: row.get(3)?,
                ai_model: row.get(4)?,
                last_opened: row.get::<_, Option<String>>(5)? 
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                memory_enabled: row.get(6)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?) 
                    .unwrap().with_timezone(&Utc),
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    pub fn save_memory(&self, memory: &Memory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO memory (project_id, session_id, role, content, timestamp, tokens_used) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                memory.project_id,
                memory.session_id,
                memory.role,
                memory.content,
                memory.timestamp.to_rfc3339(),
                memory.tokens_used
            ],
        )?;
        Ok(())
    }

    pub fn get_memory_for_project(&self, project_id: &str, limit: Option<i32>) -> Result<Vec<Memory>> {
        let query = match limit {
            Some(n) => format!("SELECT id, project_id, session_id, role, content, timestamp, tokens_used FROM memory WHERE project_id = ?1 ORDER BY timestamp DESC LIMIT {}", n),
            None => "SELECT id, project_id, session_id, role, content, timestamp, tokens_used FROM memory WHERE project_id = ?1 ORDER BY timestamp DESC".to_string(),
        };

        let mut stmt = self.conn.prepare(&query)?;

        let memories = stmt.query_map([project_id], |row| {
            Ok(Memory {
                id: row.get(0)?,
                project_id: row.get(1)?,
                session_id: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?) 
                    .unwrap().with_timezone(&Utc),
                tokens_used: row.get(6)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    pub fn add_to_sync_queue(&self, operation: &str, table_name: &str, record_id: &str, data: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sync_queue (operation, table_name, record_id, data, synced, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                operation,
                table_name,
                record_id,
                data,
                false,  // synced = false initially
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_unsynced_items(&self) -> Result<Vec<SyncQueue>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, operation, table_name, record_id, data, synced, created_at 
             FROM sync_queue WHERE synced = 0 ORDER BY created_at ASC"
        )?;

        let items = stmt.query_map([], |row| {
            Ok(SyncQueue {
                id: row.get(0)?,
                operation: row.get(1)?,
                table_name: row.get(2)?,
                record_id: row.get(3)?,
                data: row.get(4)?,
                synced: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?) 
                    .unwrap().with_timezone(&Utc),
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn mark_synced(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE sync_queue SET synced = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn clear_sync_queue(&self) -> Result<()> {
        self.conn.execute("DELETE FROM sync_queue WHERE synced = 1", [])?;
        Ok(())
    }
}