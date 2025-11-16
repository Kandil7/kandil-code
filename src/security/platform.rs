use crate::core::hardware::{HardwareProfile, PlatformKind};
#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
use tracing::{info, warn};

/// Applies lightweight platform-specific hardening checks.
pub struct PlatformHardener<'a> {
    profile: &'a HardwareProfile,
}

impl<'a> PlatformHardener<'a> {
    pub fn new(profile: &'a HardwareProfile) -> Self {
        Self { profile }
    }

    pub fn apply(&self) -> Result<()> {
        match self.profile.platform {
            PlatformKind::Windows | PlatformKind::WindowsWsl => self.harden_windows()?,
            PlatformKind::Linux => self.harden_linux()?,
            PlatformKind::MacOs => self.harden_macos()?,
            PlatformKind::Unknown => {
                info!("⚠️  Unable to apply platform hardening for unknown OS.")
            }
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn harden_windows(&self) -> Result<()> {
        use std::process::Command;

        let status = Command::new("powershell")
            .arg("-Command")
            .arg("Get-BitLockerVolume -MountPoint C: | Select-Object -ExpandProperty VolumeStatus")
            .output();

        match status {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("FullyEncrypted") {
                    warn!("BitLocker not enabled on system drive. Enable BitLocker to protect model files.");
                }
            }
            Err(err) => {
                warn!("Unable to determine BitLocker status: {err}");
            }
        }

        if self.profile.free_disk_gb < 5 {
            warn!(
                "Less than 5GB free disk space detected ({}GB). Model downloads may fail.",
                self.profile.free_disk_gb
            );
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn harden_windows(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn harden_linux(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;

        if Path::new("/var/run/ollama/ollama.sock").exists() {
            let metadata = std::fs::metadata("/var/run/ollama/ollama.sock")
                .context("Failed to read Ollama socket metadata")?;
            let mode = metadata.permissions().mode() & 0o777;
            if mode != 0o660 {
                warn!(
                    "Ollama socket permissions are {:o}. Run `sudo chmod 660 /var/run/ollama/ollama.sock` to harden.",
                    mode
                );
            }
        }

        if self.profile.free_disk_gb < 5 {
            warn!(
                "Less than 5GB free disk space detected ({}GB). Model downloads may fail.",
                self.profile.free_disk_gb
            );
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn harden_linux(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn harden_macos(&self) -> Result<()> {
        use std::process::Command;

        let status = Command::new("fdesetup")
            .arg("isactive")
            .output()
            .context("Failed to query FileVault status")?;

        let stdout = String::from_utf8_lossy(&status.stdout);
        if !stdout.trim().eq_ignore_ascii_case("true") {
            warn!("FileVault is not enabled. Enable FileVault to encrypt local model storage.");
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    fn harden_macos(&self) -> Result<()> {
        Ok(())
    }
}
