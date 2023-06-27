use crate::error::Result;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_id_hash(id: u64) -> Result<u64> {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let salt = current_time.as_nanos().to_string();
    let salted_id = format!("{}_{}", id, salt);
    let hash = generate_hash64(salted_id.as_bytes())?;

    Ok(hash)
}

pub fn generate_hash64(input: &[u8]) -> Result<u64> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash = hasher.finalize();
    let hash_bytes = &hash[..8];
    let hash_array: [u8; 8] = hash_bytes.try_into()?;
    let output = u64::from_be_bytes(hash_array);

    Ok(output)
}

#[test]
fn test_generate_hash64() -> Result<()> {
    let key = "key".to_owned();
    let hash = generate_hash64(key.as_bytes())?;
    let hash_retry = generate_hash64(key.as_bytes())?;

    assert_eq!(hash, hash_retry);
    Ok(())
}

#[test]
fn test_generate_id_hash() -> Result<()> {
    let id: u64 = 69;
    let hash = generate_id_hash(id)?;
    let hash_retry = generate_id_hash(id)?;

    assert_ne!(hash, hash_retry);
    Ok(())
}
