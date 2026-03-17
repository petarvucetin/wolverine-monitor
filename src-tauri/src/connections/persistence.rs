use std::fs;
use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use sha2::{Digest, Sha256};
use tauri::Manager;

use crate::error::AppError;
use crate::models::connection::ConnectionConfig;

const STORE_FILENAME: &str = "connections.enc";
const NONCE_LEN: usize = 12;

/// Derive a 256-bit key from machine-specific identifiers so the encrypted
/// file is tied to this machine.
fn derive_key() -> [u8; 32] {
    let computer = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_default();
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    let seed = format!("wolverine-monitor::{computer}::{user}");
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.finalize().into()
}

fn store_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Config(format!("Cannot resolve app data dir: {e}")))?;
    fs::create_dir_all(&dir).map_err(|e| AppError::Config(e.to_string()))?;
    Ok(dir.join(STORE_FILENAME))
}

fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Config(format!("Cipher init failed: {e}")))?;

    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| AppError::Config(format!("Encryption failed: {e}")))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt(data: &[u8]) -> Result<Vec<u8>, AppError> {
    if data.len() < NONCE_LEN {
        return Err(AppError::Config("Encrypted data too short".into()));
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Config(format!("Cipher init failed: {e}")))?;
    let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
    let plaintext = cipher
        .decrypt(nonce, &data[NONCE_LEN..])
        .map_err(|e| AppError::Config(format!("Decryption failed: {e}")))?;
    Ok(plaintext)
}

/// Save connection configs to an encrypted file on disk.
pub fn save_connections(
    app: &tauri::AppHandle,
    configs: &[ConnectionConfig],
) -> Result<(), AppError> {
    let json = serde_json::to_vec(configs)?;
    let encrypted = encrypt(&json)?;
    let encoded = B64.encode(&encrypted);
    let path = store_path(app)?;
    fs::write(&path, encoded).map_err(|e| AppError::Config(e.to_string()))?;
    Ok(())
}

/// Load connection configs from the encrypted file on disk.
pub fn load_connections(app: &tauri::AppHandle) -> Result<Vec<ConnectionConfig>, AppError> {
    let path = store_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let encoded = fs::read_to_string(&path).map_err(|e| AppError::Config(e.to_string()))?;
    let encrypted = B64
        .decode(encoded.trim())
        .map_err(|e| AppError::Config(format!("Base64 decode failed: {e}")))?;
    let plaintext = decrypt(&encrypted)?;
    let configs: Vec<ConnectionConfig> = serde_json::from_slice(&plaintext)?;
    Ok(configs)
}
