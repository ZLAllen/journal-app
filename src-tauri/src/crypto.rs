use crate::models::{AppError, Result};
use age::secrecy::SecretString;
use argon2::Argon2;
use rand::RngCore;
use std::fs;
use std::path::PathBuf;

/// Derive an encryption key from a passphrase using Argon2id
/// Returns (key_bytes, salt_string)
pub fn derive_key_from_passphrase(
    passphrase: &str,
    salt: Option<&str>,
) -> Result<(Vec<u8>, String)> {
    let salt_string = if let Some(s) = salt {
        s.to_string()
    } else {
        let mut salt_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        hex::encode(salt_bytes)
    };

    let mut key = vec![0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt_string.as_bytes(), &mut key)
        .map_err(|e| AppError::Encryption(format!("Key derivation failed: {}", e)))?;

    Ok((key, salt_string))
}

/// Encrypt data using age encryption
pub fn encrypt_data(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let mut encrypted = Vec::new();
    let encryptor = age::Encryptor::with_user_passphrase(SecretString::new(passphrase.to_owned()));

    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;

    std::io::Write::write_all(&mut writer, plaintext)
        .map_err(|e| AppError::Encryption(format!("Write failed: {}", e)))?;

    writer
        .finish()
        .map_err(|e| AppError::Encryption(format!("Finalization failed: {}", e)))?;

    Ok(encrypted)
}

/// Decrypt data using age decryption
pub fn decrypt_data(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let decryptor = age::Decryptor::new(ciphertext)
        .map_err(|e| AppError::Decryption(format!("Decryptor creation failed: {}", e)))?;

    let mut decrypted = Vec::new();
    let mut reader = match decryptor {
        age::Decryptor::Passphrase(d) => d
            .decrypt(&SecretString::new(passphrase.to_owned()), None)
            .map_err(|e| AppError::Decryption(format!("Decryption failed: {}", e)))?,
        _ => {
            return Err(AppError::Decryption(
                "Unexpected encrypted payload type".to_string(),
            ));
        }
    };

    std::io::Read::read_to_end(&mut reader, &mut decrypted)
        .map_err(|e| AppError::Decryption(format!("Read failed: {}", e)))?;

    Ok(decrypted)
}

/// Get the config directory for storing salt
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        )))?;

    let journal_config = config_dir.join("journal");
    fs::create_dir_all(&journal_config)?;
    Ok(journal_config)
}

/// Save salt to config file
pub fn save_salt(salt: &str) -> Result<()> {
    let config_dir = get_config_dir()?;
    let salt_file = config_dir.join("salt.txt");
    fs::write(salt_file, salt)?;
    Ok(())
}

/// Load salt from config file
pub fn load_salt() -> Result<Option<String>> {
    let config_dir = get_config_dir()?;
    let salt_file = config_dir.join("salt.txt");

    if salt_file.exists() {
        let salt = fs::read_to_string(salt_file)?;
        Ok(Some(salt.trim().to_string()))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_consistency() {
        let passphrase = "test_passphrase_123";
        let salt = "some_test_salt_value";

        let (key1, _) = derive_key_from_passphrase(passphrase, Some(salt))
            .expect("First derivation failed");
        let (key2, _) = derive_key_from_passphrase(passphrase, Some(salt))
            .expect("Second derivation failed");

        assert_eq!(key1, key2, "Keys should be identical with same passphrase and salt");
    }

    #[test]
    fn test_different_passphrases_produce_different_keys() {
        let salt = "same_salt";
        let (key1, _) = derive_key_from_passphrase("passphrase1", Some(salt))
            .expect("First derivation failed");
        let (key2, _) = derive_key_from_passphrase("passphrase2", Some(salt))
            .expect("Second derivation failed");

        assert_ne!(key1, key2, "Different passphrases should produce different keys");
    }

    #[test]
    fn test_random_salt_generation() {
        let (_, salt1) = derive_key_from_passphrase("password", None)
            .expect("First salt generation failed");
        let (_, salt2) = derive_key_from_passphrase("password", None)
            .expect("Second salt generation failed");

        assert_ne!(salt1, salt2, "Random salts should be different");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"journal secret payload";
        let passphrase = "super-secret-passphrase";

        let encrypted = encrypt_data(plaintext, passphrase).expect("encryption failed");
        let decrypted = decrypt_data(&encrypted, passphrase).expect("decryption failed");

        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
