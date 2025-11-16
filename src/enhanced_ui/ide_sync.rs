use crate::enhanced_ui::adaptive::AdaptiveUI;
use anyhow::Result;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{Server, ServerBuilder};
use lsp_types::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct LspSession {
    pub workspace_root: PathBuf,
    pub documents: HashMap<Url, TextDocumentItem>,
    pub capabilities: ServerCapabilities,
}

impl Default for LspSession {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::new(),
            documents: HashMap::new(),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "/".to_string(), ":".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        }
    }
}

pub struct IdeSyncBridge {
    root: PathBuf,
    lsp_session: Arc<Mutex<LspSession>>,
    lsp_server: Option<Server>,
}

impl IdeSyncBridge {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let mut session = LspSession::default();
        session.workspace_root = root.clone();

        Self {
            root,
            lsp_session: Arc::new(Mutex::new(session)),
            lsp_server: None,
        }
    }

    pub fn launch(&self, adaptive_ui: AdaptiveUI) -> Result<()> {
        // Start LSP server
        self.start_lsp_server()?;

        // Start file watcher
        let watch_root = self.root.clone();
        if !watch_root.exists() {
            return Err(anyhow::anyhow!(
                "Watch root does not exist: {}",
                watch_root.display()
            ));
        }

        let session = Arc::clone(&self.lsp_session);
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        let watcher_thread = spawn_watcher(&watch_root, tx)?;
        tokio::spawn(async move {
            adaptive_ui.announce("ide-sync", "🔄 IDE sync bridge active");
            while let Some(event) = rx.recv().await {
                if let Some(path) = event.paths.first() {
                    if ignored(path) {
                        continue;
                    }

                    // Handle file changes in the LSP session
                    let url = Url::from_file_path(path).ok();
                    if let Some(url) = url {
                        let event_type = match event.kind {
                            notify::EventKind::Modify(_) => "didChange",
                            notify::EventKind::Create(_) => "didOpen",
                            notify::EventKind::Remove(_) => "didClose",
                            _ => "fileEvent",
                        };

                        // Update LSP session with the change
                        if let Ok(mut session) = session.lock() {
                            if matches!(event.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_)) {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    let text_doc = TextDocumentItem {
                                        uri: url.clone(),
                                        language_id: get_language_id(path).to_string(),
                                        version: 1,
                                        text: content,
                                    };
                                    session.documents.insert(url.clone(), text_doc);
                                }
                            } else if matches!(event.kind, notify::EventKind::Remove(_)) {
                                session.documents.remove(&url);
                            }
                        }

                        let message = match event.kind {
                            notify::EventKind::Modify(_) => "File updated",
                            notify::EventKind::Create(_) => "File created",
                            notify::EventKind::Remove(_) => "File removed",
                            _ => "File event",
                        };
                        adaptive_ui.announce(
                            "ide-sync",
                            &format!("{} → {}", message, display_relative(path, &watch_root)),
                        );
                    }
                }
            }
            drop(watcher_thread);
        });
        Ok(())
    }

    fn start_lsp_server(&mut self) -> Result<()> {
        let session = Arc::clone(&self.lsp_session);

        let mut io = jsonrpc_core::IoHandler::default();

        // Implement LSP methods
        io.add_method("initialize", move |params: Params| {
            let params: InitializeParams = params.parse().unwrap();
            let mut session = session.lock().unwrap();
            session.workspace_root = params
                .root_uri
                .as_ref()
                .and_then(|uri| uri.to_file_path().ok())
                .unwrap_or_else(|| std::env::current_dir().unwrap());

            Ok(InitializeResult {
                capabilities: session.capabilities.clone(),
                server_info: None,
            })
        });

        io.add_method("textDocument/didOpen", move |params: Params| {
            let params: DidOpenTextDocumentParams = params.parse().unwrap();
            let mut session = session.lock().unwrap();
            session.documents.insert(
                params.text_document.uri.clone(),
                TextDocumentItem {
                    uri: params.text_document.uri,
                    language_id: params.text_document.language_id,
                    version: params.text_document.version,
                    text: params.text_document.text,
                }
            );
            Ok(Value::Null)
        });

        io.add_method("textDocument/didChange", move |params: Params| {
            let params: DidChangeTextDocumentParams = params.parse().unwrap();
            let mut session = session.lock().unwrap();

            if let Some(text_doc) = session.documents.get_mut(&params.text_document.uri) {
                text_doc.version = params.text_document.version;

                // Apply content changes incrementally
                for change in params.content_changes {
                    if let Some(range) = change.range {
                        text_doc.text = apply_text_change(&text_doc.text, &change);
                    } else {
                        text_doc.text = change.text; // Full document update
                    }
                }
            }
            Ok(Value::Null)
        });

        io.add_method("textDocument/completion", move |params: Params| {
            let params: CompletionParams = params.parse().unwrap();
            let session = session.lock().unwrap();

            // Generate completions based on context
            let completions = generate_completions(&session, &params.text_document_position);
            Ok(json!(completions))
        });

        io.add_method("textDocument/hover", move |params: Params| {
            let params: TextDocumentPositionParams = params.parse().unwrap();
            let session = session.lock().unwrap();

            // Generate hover information
            let hover = generate_hover_info(&session, &params);
            Ok(json!(hover))
        });

        // Start the HTTP server for LSP communication
        let server = ServerBuilder::new(io)
            .start_http(&"127.0.0.1:9277".parse().unwrap())
            .expect("Failed to start LSP server");

        self.lsp_server = Some(server);
        Ok(())
    }
}

// Helper functions for LSP functionality

fn get_language_id(path: &Path) -> &str {
    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("rs") => "rust",
            Some("js") => "javascript",
            Some("ts") => "typescript",
            Some("py") => "python",
            Some("json") => "json",
            Some("yaml") | Some("yml") => "yaml",
            Some("toml") => "toml",
            Some("md") => "markdown",
            _ => "plaintext",
        },
        None => "plaintext",
    }
}

fn apply_text_change(document_text: &str, change: &TextDocumentContentChangeEvent) -> String {
    if let Some(range) = change.range {
        // Apply incremental change
        let start_line = range.start.line as usize;
        let start_char = range.start.character as usize;
        let end_line = range.end.line as usize;
        let end_char = range.end.character as usize;

        let mut lines: Vec<&str> = document_text.lines().collect();
        if start_line < lines.len() && end_line < lines.len() {
            // This is a simplified implementation; a real implementation would handle ranges more carefully
            let mut result = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i == start_line && i == end_line {
                    // Same line change
                    if start_char < line.len() && end_char <= line.len() {
                        result.push_str(&line[..start_char]);
                        result.push_str(&change.text);
                        result.push_str(&line[end_char..]);
                    } else {
                        result.push_str(line);
                    }
                } else {
                    result.push_str(line);
                    if i < lines.len() - 1 {
                        result.push('\n');
                    }
                }
            }
            result
        } else {
            document_text.to_string()
        }
    } else {
        // Full document replacement
        change.text.clone()
    }
}

fn generate_completions(session: &LspSession, pos: &TextDocumentPositionParams) -> CompletionList {
    // Generate context-aware completions
    let mut items = Vec::new();

    // Generate some basic completions for demonstration
    items.push(CompletionItem {
        label: "kandil".to_string(),
        detail: Some("Kandil Code CLI tool".to_string()),
        documentation: Some(Documentation::String(
            "Intelligent development platform with AI assistance".to_string()
        )),
        ..CompletionItem::default()
    });

    items.push(CompletionItem {
        label: "chat".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("Start an AI chat session".to_string()),
        ..CompletionItem::default()
    });

    CompletionList {
        is_incomplete: false,
        items,
    }
}

fn generate_hover_info(session: &LspSession, params: &TextDocumentPositionParams) -> Hover {
    Hover {
        contents: HoverContents::Scalar(MarkedString::String(
            "Kandil Code - Intelligent development platform with AI assistance".to_string()
        )),
        range: None,
    }
}

fn spawn_watcher(root: &Path, tx: mpsc::UnboundedSender<Event>) -> Result<thread::JoinHandle<()>> {
    let root = root.to_path_buf();
    let handle = thread::spawn(move || {
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .expect("Failed to start watcher");

        watcher
            .watch(root.as_path(), RecursiveMode::Recursive)
            .expect("Failed to watch project root");

        loop {
            thread::sleep(Duration::from_secs(60));
        }
    });
    Ok(handle)
}

fn ignored(path: &Path) -> bool {
    path.components().any(|component| {
        if let std::path::Component::Normal(os) = component {
            os == "target" || os == ".git"
        } else {
            false
        }
    })
}

fn display_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
