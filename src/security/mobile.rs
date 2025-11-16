use anyhow::{Context, Result};
use ring::aead::{self, Aad, LessSafeKey, UnboundKey, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use std::fs;
use std::path::Path;

pub fn enforce_ios_bundle_security(bundle: &Path) -> Result<()> {
    ensure_security_files(
        bundle,
        "iOS",
        "Import this bundle via Files.app or iCloud Drive. Kandil for iOS decrypts models at install time and stores them inside the app sandbox.",
    )
}

pub fn enforce_android_bundle_security(bundle: &Path) -> Result<()> {
    ensure_security_files(
        bundle,
        "Android",
        "Copy this directory to /sdcard/kandil/models. Use Termux or Kandil Mobile to register the bundle; models remain encrypted at rest and keys stay in Android Keystore.",
    )
}

pub fn enforce_edge_bundle_security(bundle: &Path) -> Result<()> {
    ensure_security_files(
        bundle,
        "Edge",
        "Transfer this snapshot to your edge device (Raspberry Pi / Jetson). Keep the encryption key separate and provision it as an environment variable before loading ONNX runtimes.",
    )
}

fn ensure_security_files(bundle: &Path, target: &str, instructions: &str) -> Result<()> {
    fs::create_dir_all(bundle).with_context(|| {
        format!(
            "Unable to prepare secure bundle directory {}",
            bundle.display()
        )
    })?;

    let key_path = bundle.join("encryption.key");
    if !key_path.exists() {
        let key = generate_key()?;
        fs::write(&key_path, &key)
            .with_context(|| format!("Failed to write {}", key_path.display()))?;
        seal_models(bundle, &key)?;
    } else {
        let key = fs::read(&key_path)
            .with_context(|| format!("Failed to read {}", key_path.display()))?;
        seal_models(bundle, &key)?;
    }

    let readme_path = bundle.join("SECURITY.md");
    let readme = format!(
        "# {target} Secure Bundle\n\n\
* Encryption key stored in `encryption.key` (keep it private!).\n\
* {instructions}\n\
* Delete the exported key once the device confirms successful import.\n\
* All `.gguf` files are encrypted and require the key to decrypt on device.\n"
    );
    fs::write(&readme_path, readme)
        .with_context(|| format!("Failed to write {}", readme_path.display()))?;

    Ok(())
}

fn generate_key() -> Result<Vec<u8>> {
    let mut key = vec![0u8; 32];
    SystemRandom::new()
        .fill(&mut key)
        .map_err(|err| anyhow::anyhow!("Failed to generate encryption key: {err}"))?;
    Ok(key)
}

fn seal_models(bundle: &Path, key: &[u8]) -> Result<()> {
    let unbound =
        UnboundKey::new(&aead::AES_256_GCM, key).map_err(|_| anyhow::anyhow!("Invalid key"))?;
    let sealing_key = LessSafeKey::new(unbound);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|err| anyhow::anyhow!("Failed to generate nonce: {err}"))?;
    let nonce_path = bundle.join("encryption.nonce");
    fs::write(&nonce_path, &nonce_bytes)?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut counter: u64 = 0;
    for entry in fs::read_dir(bundle)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "gguf" {
                let per_file_nonce = increment_nonce(&nonce, counter);
                encrypt_file(&path, &sealing_key, per_file_nonce)?;
                counter = counter.saturating_add(1);
            }
        }
    }
    Ok(())
}

fn increment_nonce(base: &aead::Nonce, offset: u64) -> aead::Nonce {
    let mut bytes = base.as_ref().to_vec();
    let mut carry = offset;
    for byte in bytes.iter_mut().rev() {
        let sum = *byte as u64 + carry;
        *byte = sum as u8;
        carry = sum >> 8;
        if carry == 0 {
            break;
        }
    }
    let mut array = [0u8; NONCE_LEN];
    array.copy_from_slice(&bytes);
    aead::Nonce::assume_unique_for_key(array)
}

fn encrypt_file(path: &Path, key: &LessSafeKey, nonce: aead::Nonce) -> Result<()> {
    let mut data = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut data)
        .map_err(|_| anyhow::anyhow!("Failed to encrypt {}", path.display()))?;

    fs::write(path, data).with_context(|| format!("Failed to write {}", path.display()))
}
