use jsonrpc_core::futures::FutureExt;
use jsonrpc_core::{IoHandler, Params};
use lsp_types::{
    InitializeParams, InitializeResult, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Uri,
};
use serde_json::Value;
use std::path::PathBuf;
use tokio::sync::RwLock;
use anyhow::Result;

#[derive(Debug)]
pub struct IdeSync {
    sessions: RwLock<Vec<IdeSession>>,
}

#[derive(Debug, Clone)]
struct IdeSession {
    id: String,
    workspace_root: PathBuf,
    capabilities: ServerCapabilities,
}

impl IdeSync {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }

    pub async fn start_language_server(
        &self,
        workspace_path: PathBuf,
    ) -> Result<()> {
        let mut handler = IoHandler::new();

        // Initialize method - returns a future
        handler.add_method("initialize", |params: Params| {
            async move {
                let params: InitializeParams = params.parse().unwrap();

                // We can ignore root_path for now since we're not using it in the response
                // But we'll keep the logic for URI to file path conversion if needed later
                let _root_path = if let Some(workspace_folders) = &params.workspace_folders {
                    if let Some(first_folder) = workspace_folders.first() {
                        uri_to_file_path(&first_folder.uri)
                    } else {
                        None
                    }
                } else {
                    // Fallback to deprecated root_uri if workspace_folders is empty
                    if let Some(uri) = &params.root_uri {
                        uri_to_file_path(uri)
                    } else {
                        None
                    }
                };

                let init_result = InitializeResult {
                    capabilities: ServerCapabilities {
                        text_document_sync: Some(TextDocumentSyncCapability::Kind(
                            TextDocumentSyncKind::INCREMENTAL,
                        )),
                        ..Default::default()
                    },
                    server_info: None,
                };

                Ok(serde_json::to_value(init_result).unwrap())
            }
            .boxed()
        });

        // Initialized method
        handler.add_method("initialized", |_params: Params| {
            async move { Ok(Value::Null) }.boxed()
        });

        // Text document opened
        handler.add_method("textDocument/didOpen", |params: Params| {
            async move {
                // Process the didOpen notification
                println!("Document opened: {:?}", params);
                Ok(Value::Null)
            }
            .boxed()
        });

        // Text document changed
        handler.add_method("textDocument/didChange", |params: Params| {
            async move {
                // Process the didChange notification
                println!("Document changed: {:?}", params);
                Ok(Value::Null)
            }
            .boxed()
        });

        // Text document saved
        handler.add_method("textDocument/didSave", |params: Params| {
            async move {
                // Process the didSave notification
                println!("Document saved: {:?}", params);
                Ok(Value::Null)
            }
            .boxed()
        });

        // Text document closed
        handler.add_method("textDocument/didClose", |params: Params| {
            async move {
                // Process the didClose notification
                println!("Document closed: {:?}", params);
                Ok(Value::Null)
            }
            .boxed()
        });

        // Add the session
        let session = IdeSession {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_root: workspace_path,
            capabilities: ServerCapabilities::default(),
        };

        self.sessions.write().await.push(session);

        Ok(())
    }

    pub async fn send_did_change(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<()> {
        // In a real implementation, this would send the change to the language server
        println!("Sending didChange for {}: {}", file_path, content);
        Ok(())
    }

    pub async fn get_completions(
        &self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<String>> {
        // In a real implementation, this would request completions from the language server
        println!(
            "Requesting completions for {}:{}:{}",
            file_path, line, character
        );
        Ok(vec!["example_completion".to_string()])
    }
}

// Helper function to convert URI to file path
fn uri_to_file_path(uri: &Uri) -> Option<PathBuf> {
    let uri_str = uri.as_str();
    if uri_str.starts_with("file://") {
        // Remove the file:// prefix
        let path_str = &uri_str[7..];
        // Simple approach: replace %20 with spaces (not full URL decoding)
        let decoded_path = path_str.replace("%20", " ");
        Some(PathBuf::from(decoded_path))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ide_sync() {
        let ide_sync = IdeSync::new();
        let temp_dir = std::env::temp_dir();

        assert!(ide_sync.start_language_server(temp_dir).await.is_ok());
    }
}
