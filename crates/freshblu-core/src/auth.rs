use rand::distributions::Alphanumeric;
use rand::Rng;
use sha2::{Digest, Sha256};

const TOKEN_LENGTH: usize = 40;
const BCRYPT_COST: u32 = 10;

/// Generate a new random plaintext token (40 hex chars)
pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

/// Hash a plaintext token using bcrypt
pub fn hash_token(token: &str) -> crate::error::Result<String> {
    bcrypt::hash(token, BCRYPT_COST)
        .map_err(|e| crate::error::FreshBluError::Internal(e.to_string()))
}

/// Verify a plaintext token against a bcrypt hash
pub fn verify_token(token: &str, hash: &str) -> bool {
    bcrypt::verify(token, hash).unwrap_or(false)
}

/// Compute a device hash (SHA256 of the canonical device JSON)
pub fn compute_device_hash(device_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(device_json.as_bytes());
    let result = hasher.finalize();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
}

/// Generate a session token (shorter expiry, used for temporary access)
pub fn generate_session_token() -> String {
    generate_token()
}

/// Basic auth parsing from "uuid:token"
pub fn parse_basic_auth(header: &str) -> Option<(String, String)> {
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        header.trim_start_matches("Basic "),
    )
    .ok()?;
    let s = String::from_utf8(decoded).ok()?;
    let mut parts = s.splitn(2, ':');
    let uuid = parts.next()?.to_string();
    let token = parts.next()?.to_string();
    Some((uuid, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_roundtrip() {
        let token = generate_token();
        assert_eq!(token.len(), TOKEN_LENGTH);
        let hash = hash_token(&token).unwrap();
        assert!(verify_token(&token, &hash));
        assert!(!verify_token("wrongtoken", &hash));
    }

    #[test]
    fn test_basic_auth_parse() {
        use base64::Engine;
        let creds = base64::engine::general_purpose::STANDARD.encode("my-uuid:my-token");
        let (u, t) = parse_basic_auth(&format!("Basic {}", creds)).unwrap();
        assert_eq!(u, "my-uuid");
        assert_eq!(t, "my-token");
    }
}
