use base64::{engine::general_purpose::STANDARD, Engine};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

/// Encriptación AES-256-GCM para API keys
pub struct Encryption;

impl Encryption {
    /// Genera una clave derivada del hardware (machine-specific)
    fn derive_key() -> [u8; 32] {
        // Usa el hostname + un salt fijo como semilla para la clave
        // Esto hace que las API keys solo sean desencriptables en la misma máquina
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "nexus-lite-default".to_string());

        let salt = format!("NEXUS-LITE-2026-{}", hostname);
        let mut key = [0u8; 32];
        // ring HKDF para derivar la clave
        let salt_bytes = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, salt.as_bytes());
        let tag = ring::hmac::sign(&salt_bytes, b"nexus-lite-encryption-key");
        key.copy_from_slice(&tag.as_ref()[..32]);
        key
    }

    /// Encripta un texto plano usando AES-256-GCM
    pub fn encrypt(plaintext: &str) -> Result<String, String> {
        let key_bytes = Self::derive_key();
        let unbound_key =
            UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|e| format!("Key error: {}", e))?;
        let key = LessSafeKey::new(unbound_key);

        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|e| format!("RNG error: {}", e))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = plaintext.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Format: base64(nonce || ciphertext || tag)
        let mut output = Vec::with_capacity(12 + in_out.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&in_out);

        Ok(STANDARD.encode(&output))
    }

    /// Desencripta un texto encriptado con AES-256-GCM
    pub fn decrypt(ciphertext: &str) -> Result<String, String> {
        let key_bytes = Self::derive_key();
        let unbound_key =
            UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|e| format!("Key error: {}", e))?;
        let key = LessSafeKey::new(unbound_key);

        let decoded = STANDARD
            .decode(ciphertext)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        if decoded.len() < 12 {
            return Err("Ciphertext too short".to_string());
        }

        let (nonce_bytes, encrypted) = decoded.split_at(12);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes
                .try_into()
                .map_err(|_| "Invalid nonce length".to_string())?,
        );

        let mut in_out = encrypted.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| format!("Decryption error: {}", e))?;

        String::from_utf8(plaintext.to_vec()).map_err(|e| format!("UTF-8 error: {}", e))
    }
}
