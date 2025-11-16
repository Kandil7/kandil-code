use anyhow::{bail, Context, Result};
use blake3::Hasher;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::time::{self, Duration};
use tracing::{debug, warn};

/// Provides integrity validation for downloaded model artifacts.
pub struct ModelSecurityValidator;

impl ModelSecurityValidator {
    /// Computes a BLAKE3 checksum for the given file path.
    pub fn compute_digest(path: &Path) -> Result<String> {
        let file =
            File::open(path).with_context(|| format!("Unable to open {}", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        let mut buffer = [0u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Writes the computed digest alongside the model artifact.
    pub fn persist_digest(path: &Path, digest: &str) -> Result<()> {
        std::fs::write(Self::digest_path(path), digest)?;
        Ok(())
    }

    /// Loads a previously stored digest if it exists.
    pub fn load_stored_digest(path: &Path) -> Result<Option<String>> {
        let digest_path = Self::digest_path(path);
        if digest_path.exists() {
            let value = std::fs::read_to_string(digest_path)?;
            return Ok(Some(value.trim().to_string()));
        }
        Ok(None)
    }

    /// Runs a best-effort sandbox smoke test. If the sandbox binary is missing,
    /// the check is skipped with a debug log.
    pub async fn sandbox_smoke_test(path: &Path) -> Result<()> {
        let command = "kandil-sandbox";
        let child = match Command::new(command)
            .arg("load-test")
            .arg(path)
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => child,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                debug!(
                    "Sandbox binary '{}' not found. Skipping sandbox test.",
                    command
                );
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };

        let output = time::timeout(Duration::from_secs(30), child.wait_with_output()).await;

        match output {
            Ok(Ok(result)) if result.status.success() => Ok(()),
            Ok(Ok(result)) => {
                bail!(
                    "Sandbox test failed with status {}: {}",
                    result.status,
                    String::from_utf8_lossy(&result.stderr)
                )
            }
            Ok(Err(err)) => Err(err.into()),
            Err(_) => Err(anyhow::anyhow!(
                "Sandbox test timed out. The downloaded model may be corrupted."
            )),
        }
    }

    pub async fn verify_artifact(path: &Path, expected_digest: Option<&str>) -> Result<String> {
        let digest = Self::compute_digest(path)?;

        if let Some(expected) = expected_digest {
            if !expected.eq_ignore_ascii_case(&digest) {
                bail!(
                    "Checksum mismatch for {}.\nExpected: {}\nActual:   {}",
                    path.display(),
                    expected,
                    digest
                );
            }
        }

        Self::sandbox_smoke_test(path).await?;
        Ok(digest)
    }

    fn digest_path(path: &Path) -> PathBuf {
        let mut output = path.to_path_buf();
        output.set_extension("blake3");
        output
    }
}
