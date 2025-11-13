# 📄 PHASE_5_PROJECTS_CLOUD.md

```markdown
# Phase 5: Projects Manager & Cloud Sync

## Objectives
Implement a robust project management system with local SQLite memory storage, Supabase cloud synchronization, and session management. Enable `kandil projects list/switch/sync` and release v0.1 MVP with full cloud integration.

## Prerequisites
- Phase 4 complete (multi-model AI, refactor, test generation)
- Supabase account (free tier) with project created
- API keys stored in keyring: `supabase_url`, `supabase_anon_key`
- SQLite dev libraries installed (`libsqlite3-dev` on Linux)

## Detailed Sub-Tasks

### Day 1-2: SQLite Local Storage

1. **Add Dependencies**
```bash
cargo add rusqlite --features bundled # Static linking for easy deployment
cargo add rusqlite_migration # Schema management
cargo add serde_json
cargo add chrono # For timestamps
cargo add dirs # Cross-platform home directory
```

2. **Database Schema & Migration**
```rust
// src/core/db.rs
use rusqlite::{Connection, Result as SqlResult};
use rusqlite_migration::{Migrations, M};

const MIGRATIONS: Migrations = Migrations::new(vec![
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
        "#
    ),
    M::down("DROP TABLE projects; DROP TABLE memory; DROP TABLE sync_queue;"),
]);

pub fn get_connection() -> SqlResult<Connection> {
    let db_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".kandil")
        .join("kandil.db");
    
    let conn = Connection::open(db_path)?;
    MIGRATIONS.to_latest(&conn)?;
    Ok(conn)
}
```

3. **Project Manager**
```rust
// src/core/project_manager.rs
use super::db::get_connection;
use super::workspace::Workspace;
use rusqlite::{params, Connection};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub ai_provider: String,
    pub ai_model: String,
    pub last_opened: Option<i64>,
    pub memory_enabled: bool,
}

pub struct ProjectManager {
    conn: Connection,
}

impl ProjectManager {
    pub fn new() -> Result<Self> {
        let conn = get_connection()?;
        Ok(Self { conn })
    }
    
    pub fn get_or_create(&self, workspace: &Workspace) -> Result<Project> {
        let id = Self::generate_id(&workspace.root);
        
        // Try to find existing
        if let Some(project) = self.get_by_id(&id)? {
            // Update last_opened
            self.update_last_opened(&id)?;
            return Ok(project);
        }
        
        // Create new project
        let config = crate::utils::config::Config::load()?;
        let project = Project {
            id: id.clone(),
            name: workspace.root.split('/').last().unwrap_or("project").to_string(),
            root_path: workspace.root.clone(),
            ai_provider: config.ai.provider,
            ai_model: config.ai.model,
            last_opened: Some(Utc::now().timestamp()),
            memory_enabled: true,
        };
        
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, ai_provider, ai_model, last_opened)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET last_opened = ?6",
            params![
                project.id,
                project.name,
                project.root_path,
                project.ai_provider,
                project.ai_model,
                project.last_opened,
            ],
        )?;
        
        Ok(project)
    }
    
    pub fn list_all(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled
             FROM projects ORDER BY last_opened DESC"
        )?;
        
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                ai_provider: row.get(3)?,
                ai_model: row.get(4)?,
                last_opened: row.get(5)?,
                memory_enabled: row.get(6)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        
        Ok(projects)
    }
    
    pub fn switch_project(&self, project_id: &str) -> Result<()> {
        let project = self.get_by_id(project_id)?
            .ok_or_else(|| anyhow::anyhow!("Project not found: {}", project_id))?;
        
        // Update config to match project
        let mut config = crate::utils::config::Config::load()?;
        config.ai.provider = project.ai_provider;
        config.ai.model = project.ai_model;
        config.save()?;
        
        // Change directory
        std::env::set_current_dir(&project.root_path)?;
        
        Ok(())
    }
    
    pub fn delete_project(&self, project_id: &str) -> Result<()> {
        // Soft delete: just disable memory
        self.conn.execute(
            "UPDATE projects SET memory_enabled = 0 WHERE id = ?1",
            params![project_id],
        )?;
        Ok(())
    }
    
    fn get_by_id(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, ai_provider, ai_model, last_opened, memory_enabled
             FROM projects WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                ai_provider: row.get(3)?,
                ai_model: row.get(4)?,
                last_opened: row.get(5)?,
                memory_enabled: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    fn update_last_opened(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET last_opened = ?1 WHERE id = ?2",
            params![Utc::now().timestamp(), id],
        )?;
        Ok(())
    }
    
    fn generate_id(path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_project_lifecycle() {
        let manager = ProjectManager::new().unwrap();
        let temp = TempDir::new().unwrap();
        
        let ws = Workspace {
            root: temp.path().to_string_lossy().to_string(),
            project_type: "rust".to_string(),
            config_path: temp.path().join("kandil.toml").to_string_lossy().to_string(),
        };
        
        // Create
        let project = manager.get_or_create(&ws).unwrap();
        assert_eq!(project.root_path, ws.root);
        
        // List
        let projects = manager.list_all().unwrap();
        assert!(projects.iter().any(|p| p.id == project.id));
        
        // Switch
        manager.switch_project(&project.id).unwrap();
        assert_eq!(std::env::current_dir().unwrap(), temp.path());
    }
}
```

### Day 3-4: Memory Management

1. **Session Memory**
```rust
// src/core/memory.rs
use super::db::get_connection;
use rusqlite::params;
use anyhow::Result;

pub struct MemoryManager {
    project_id: String,
    session_id: String,
}

impl MemoryManager {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    pub fn add_interaction(&self, role: &str, content: &str, tokens: Option<u32>) -> Result<()> {
        let conn = get_connection()?;
        
        conn.execute(
            "INSERT INTO memory (project_id, session_id, role, content, tokens_used)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![self.project_id, self.session_id, role, content, tokens],
        )?;
        
        // Keep only last 1000 interactions per project
        self.prune_old_memory(&conn)?;
        
        Ok(())
    }
    
    pub fn get_recent_context(&self, limit: usize) -> Result<Vec<String>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT role, content FROM memory
             WHERE project_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;
        
        let context = stmt.query_map(params![self.project_id, limit], |row| {
            let role: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok(format!("{}: {}", role, content))
        })?.collect::<Vec<_>>().join("\n");
        
        Ok(vec![context])
    }
    
    pub fn get_project_memory(&self) -> Result<Vec<(String, String, i64)>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT role, content, timestamp FROM memory
             WHERE project_id = ?1
             ORDER BY timestamp DESC"
        )?;
        
        let memory = stmt.query_map(params![self.project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?.collect::<Vec<_>>()?;
        
        Ok(memory)
    }
    
    fn prune_old_memory(&self, conn: &rusqlite::Connection) -> Result<()> {
        conn.execute(
            "DELETE FROM memory
             WHERE id IN (
                 SELECT id FROM memory
                 WHERE project_id = ?1
                 ORDER BY timestamp DESC
                 LIMIT -1 OFFSET 1000
             )",
            params![self.project_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_persistence() {
        let manager = MemoryManager::new("test-project".to_string());
        manager.add_interaction("user", "Hello", Some(10)).unwrap();
        manager.add_interaction("ai", "Hi there", Some(20)).unwrap();
        
        let context = manager.get_recent_context(10).unwrap();
        assert!(context.len() == 1);
        assert!(context[0].contains("Hello"));
        assert!(context[0].contains("Hi there"));
    }
}
```

2. **AI Context from Memory**
```rust
// src/adapters/ai/contextual.rs
use crate::core::memory::MemoryManager;
use crate::utils::config::Config;

pub struct ContextualAI {
    ai_provider: Box<dyn AIProvider>,
    memory_manager: MemoryManager,
}

impl ContextualAI {
    pub async fn new(project_id: String) -> Result<Self> {
        let config = Config::load()?;
        let factory = AIProviderFactory::new(config.ai);
        let ai_provider = factory.create().await?;
        
        let memory_manager = MemoryManager::new(project_id);
        
        Ok(Self {
            ai_provider,
            memory_manager,
        })
    }
    
    pub async fn chat_with_memory(&self, message: &str) -> Result<String> {
        // Get recent context
        let context = self.memory_manager.get_recent_context(10)?;
        
        // Add current message to memory
        self.memory_manager.add_interaction("user", message, None)?;
        
        // Call AI with context
        let response = self.ai_provider.chat(message, Some(&context.join("\n"))).await?;
        
        // Add AI response to memory
        self.memory_manager.add_interaction("ai", &response, None)?;
        
        Ok(response)
    }
}
```

### Day 5-6: Supabase Sync

1. **Sync Engine (Offline-First)**
```rust
// src/cloud/sync.rs
use crate::core::db::get_connection;
use rusqlite::params;
use anyhow::Result;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SyncEngine {
    client: Client,
    supabase_url: String,
    api_key: String,
    queue: Arc<Mutex<Vec<SyncOperation>>>,
}

#[derive(Serialize, Deserialize)]
struct SyncOperation {
    operation: String,
    table: String,
    record_id: String,
    data: serde_json::Value,
}

impl SyncEngine {
    pub fn new() -> Result<Self> {
        let supabase_url = std::env::var("KANDIL_SUPABASE_URL")
            .or_else(|_| crate::utils::keys::SecureKey::load("supabase_url")
                .map(|k| k.expose().clone()))?;
        
        let api_key = std::env::var("KANDIL_SUPABASE_ANON_KEY")
            .or_else(|_| crate::utils::keys::SecureKey::load("supabase_anon_key")
                .map(|k| k.expose().clone()))?;
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "apikey",
            api_key.parse()?,
        );
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse()?,
        );
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            supabase_url,
            api_key,
            queue: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub async fn sync_project(&self, project: &crate::core::project_manager::Project) -> Result<()> {
        // Queue project data
        self.queue_operation(
            "upsert",
            "projects",
            &project.id,
            serde_json::to_value(project)?,
        ).await;
        
        // Queue memory data (summary only, not full history for privacy)
        let memory_manager = crate::core::memory::MemoryManager::new(project.id.clone());
        let memory_summary = memory_manager.get_project_memory()?;
        
        self.queue_operation(
            "upsert",
            "memory_summary",
            &project.id,
            serde_json::json!({
                "project_id": project.id,
                "interaction_count": memory_summary.len(),
                "last_updated": chrono::Utc::now().to_rfc3339(),
            }),
        ).await;
        
        // Process queue
        self.process_queue().await
    }
    
    pub async fn fetch_projects(&self) -> Result<Vec<crate::core::project_manager::Project>> {
        let url = format!("{}/rest/v1/projects", self.supabase_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let projects: Vec<crate::core::project_manager::Project> = response.json().await?;
            Ok(projects)
        } else {
            Err(anyhow::anyhow!("Failed to fetch projects: {}", response.text().await?))
        }
    }
    
    async fn queue_operation(
        &self,
        operation: &str,
        table: &str,
        record_id: &str,
        data: serde_json::Value,
    ) {
        let mut queue = self.queue.lock().await;
        queue.push(SyncOperation {
            operation: operation.to_string(),
            table: table.to_string(),
            record_id: record_id.to_string(),
            data,
        });
    }
    
    async fn process_queue(&self) -> Result<()> {
        let mut queue = self.queue.lock().await;
        
        if queue.is_empty() {
            return Ok(());
        }
        
        println!("🔄 Syncing {} changes...", queue.len());
        
        for op in queue.iter() {
            match op.operation.as_str() {
                "upsert" => self.sync_upsert(op).await?,
                "delete" => self.sync_delete(op).await?,
                _ => tracing::warn!("Unknown sync operation: {}", op.operation),
            }
        }
        
        // Clear queue
        queue.clear();
        
        println!("✅ Sync complete");
        Ok(())
    }
    
    async fn sync_upsert(&self, op: &SyncOperation) -> Result<()> {
        let url = format!("{}/rest/v1/{}?id=eq.{}", self.supabase_url, op.table, op.record_id);
        
        let response = self.client
            .post(&url)
            .json(&op.data)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::debug!("Synced {}: {}", op.table, op.record_id);
            Ok(())
        } else {
            tracing::error!("Sync failed {}: {}", op.table, response.text().await?);
            Err(anyhow::anyhow!("Sync failed"))
        }
    }
    
    async fn sync_delete(&self, op: &SyncOperation) -> Result<()> {
        let url = format!("{}/rest/v1/{}?id=eq.{}", self.supabase_url, op.table, op.record_id);
        
        let response = self.client
            .delete(&url)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Delete sync failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    
    #[tokio::test]
    async fn test_sync_with_mock() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/rest/v1/projects"))
            .and(header("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;
        
        // Test sync
        // ... would require injecting mock server URL
    }
}
```

### Day 7-8: CLI Integration & v0.1 Prep

1. **Projects Commands**
```rust
// src/cli/projects.rs
use crate::core::project_manager::ProjectManager;
use crate::core::memory::MemoryManager;
use crate::cloud::sync::SyncEngine;
use anyhow::Result;

pub async fn handle_projects(sub: ProjectSub) -> Result<()> {
    match sub {
        ProjectSub::List => list_projects().await,
        ProjectSub::Switch { id } => switch_project(&id).await,
        ProjectSub::Sync { cloud } => {
            if cloud {
                sync_to_cloud().await
            } else {
                sync_local().await
            }
        }
        ProjectSub::Memory { project_id } => show_memory(&project_id).await,
    }
}

async fn list_projects() -> Result<()> {
    let manager = ProjectManager::new()?;
    let projects = manager.list_all()?;
    
    if projects.is_empty() {
        println!("No projects found. Navigate to a project directory and run 'kandil init'");
        return Ok(());
    }
    
    println!("📦 Projects:");
    for (i, project) in projects.iter().enumerate() {
        let last_opened = if let Some(ts) = project.last_opened {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Never".to_string()
        };
        
        println!("  {}. {} ({})\n     Path: {}\n     AI: {} ({})\n     Last: {}",
            i + 1,
            project.name,
            project.project_type,
            project.root_path,
            project.ai_provider,
            project.ai_model,
            last_opened
        );
    }
    
    Ok(())
}

async fn switch_project(project_id: &str) -> Result<()> {
    let manager = ProjectManager::new()?;
    manager.switch_project(project_id)?;
    
    println!("✅ Switched to project {}", project_id);
    println!("Run 'kandil tui' to open the project");
    
    Ok(())
}

async fn sync_to_cloud() -> Result<()> {
    let manager = ProjectManager::new()?;
    let projects = manager.list_all()?;
    
    if projects.is_empty() {
        println!("No projects to sync");
        return Ok(());
    }
    
    let sync_engine = SyncEngine::new()?;
    
    for project in projects {
        if project.memory_enabled {
            sync_engine.sync_project(&project).await?;
        }
    }
    
    Ok(())
}

async fn sync_local() -> Result<()> {
    // Pull from cloud and merge
    let sync_engine = SyncEngine::new()?;
    let cloud_projects = sync_engine.fetch_projects().await?;
    
    println!("📥 Downloaded {} projects from cloud", cloud_projects.len());
    // Merge logic would go here
    
    Ok(())
}

async fn show_memory(project_id: &str) -> Result<()> {
    let memory = MemoryManager::new(project_id.to_string());
    let interactions = memory.get_project_memory()?;
    
    println!("💭 Memory for project {} ({} interactions):", project_id, interactions.len());
    
    for (role, content, ts) in interactions.iter().take(10) {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_default();
        
        println!("  [{}] {}: {}", time, role, content.chars().take(50).collect::<String>());
    }
    
    if interactions.len() > 10 {
        println!("  ... and {} more interactions", interactions.len() - 10);
    }
    
    Ok(())
}
```

2. **v0.1 Release Checklist**
```bash
# Create release checklist script
cat > scripts/release_v0.1.sh <<'EOF'
#!/bin/bash
set -e

echo "🔍 Running v0.1 release checklist..."

# 1. Tests
cargo test --all-features
cargo tarpaulin --fail-under 85

# 2. Security
cargo audit --deny warnings
cargo deny check licenses

# 3. Build (cross-platform)
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu

# 4. Manual smoke tests
./target/release/kandil init
./target/release/kandil create flutter test_app
cd test_app
./target/release/kandil chat "Explain this project"
./target/release/kandil tui # (test navigation, then quit)

# 5. Package
mkdir -p dist
cp target/release/kandil dist/kandil-v0.1.0-linux
cp target/x86_64-apple-darwin/release/kandil dist/kandil-v0.1.0-macos

echo "✅ All checks passed!"
EOF

chmod +x scripts/release_v0.1.sh
```

3. **Update README for v0.1**
```markdown
# Kandil Code v0.1 (MVP) 🎉

## Installation
```bash
# Download from releases
curl -L https://github.com/Kandil7/kandil_code/releases/download/v0.1.0/kandil-v0.1.0-linux -o kandil
chmod +x kandil
sudo mv kandil /usr/local/bin/

# Or install from source
cargo install --path .
```

## Quick Start
```bash
# Initialize a project
cd my-flutter-app
kandil init

# Chat with AI
kandil chat "How do I implement BLoC pattern?"

# Create new project
kandil create python my-api

# Open TUI studio
kandil tui

# Refactor code
kandil refactor lib/main.dart --goal="Use Clean Architecture"

# Generate tests
kandil test generate lib/main.dart --coverage=80

# Switch to cloud AI
kandil switch-model anthropic claude-3-sonnet-20240229
```

## v0.1 Features
- ✅ Secure CLI with OS keychain
- ✅ Local AI (Ollama) + Cloud AI (Claude, OpenAI, Qwen)
- ✅ Multi-language templates (Flutter, Python, JS, Rust)
- ✅ Interactive TUI studio
- ✅ Code analysis with Tree-sitter
- ✅ Refactoring with preview
- ✅ Test generation
- ✅ Project management with SQLite
- ✅ Cost tracking
- ✅ Cloud sync (Supabase)
```

## Tools & Dependencies
| Tool | Purpose |
|------|---------|
| Supabase | Cloud sync backend |
| rusqlite | Local database |
| rusqlite_migration | Schema management |
| chrono | Timestamps |
| dirs | Cross-platform paths |
| uuid | Session IDs |
| wiremock | Sync testing |

## Testing Strategy
- **Unit**: Database operations with in-memory SQLite (90% coverage)
- **Integration**: Full sync workflow with mock Supabase
- **Manual**: Create 3 projects, switch between them, verify memory persistence
- **Load**: 1000 sync operations, ensure queue processes correctly

## Deliverables
- ✅ `kandil projects list` shows all projects with metadata
- ✅ `kandil projects switch <id>` changes directory and AI config
- ✅ `kandil projects sync --cloud` uploads to Supabase
- ✅ Memory persists across sessions locally
- ✅ Memory synced to cloud (summary only for privacy)
- ✅ Session isolation (UUID per run)
- ✅ v0.1 binary for Linux, macOS, Windows
- ✅ GitHub Release with changelog
- ✅ Installation documentation

## Timeline Breakdown
- **Days 1-2**: SQLite schema + project manager
- **Days 3-4**: Memory manager + context integration
- **Days 5-6**: Supabase sync engine + queue system
- **Days 7-8**: CLI integration + sync commands
- **Days 9-10**: v0.1 polish + documentation
- **Days 11-14**: Release testing + multi-platform builds

## Success Criteria
- Project list loads in <100ms
- Switch project in <1s
- Memory survives app restart
- Sync completes in <5s for typical project (<100MB data)
- v0.1 binaries run on clean machines (no dev dependencies)
- GitHub Release has 50+ downloads in first week
- User can complete full workflow: init → create → chat → refactor → sync

## Potential Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| SQLite locked (concurrent access) | Use WAL mode + connection pooling |
| Supabase rate limits | Batch operations; local queue + retry |
| Memory grows too large | Prune to last 1000 interactions; compress old data |
| Sync conflicts (offline edits) | Implement CRDT or last-write-wins with timestamps |
| Cloud credentials leaked | Use Row Level Security in Supabase; never log tokens |
| Privacy concerns (sending code) | Sync only metadata; warn user before uploading code |

---

**Next**: Proceed to PHASE_6_REQUIREMENTS_DESIGN_AGENTS.md after v0.1 release and user feedback collection.