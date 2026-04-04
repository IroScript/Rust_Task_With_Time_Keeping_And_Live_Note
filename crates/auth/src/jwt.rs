//! JWT authentication and token validation
//!
//! This module provides JWT token validation for authenticating
//! WebSocket connections and protected HTTP routes.

use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// JWT configuration settings
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for validating JWT signatures
    pub secret: String,
    /// Expected JWT issuer
    pub issuer: String,
    /// Expected audience (optional)
    pub audience: Option<String>,
    /// Token expiration tolerance in seconds (default: 60)
    pub expiration_tolerance: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key-change-in-production".to_string(),
            issuer: "sync-system".to_string(),
            audience: None,
            expiration_tolerance: 60,
        }
    }
}

/// JWT claims extracted from the token
///
/// Contains the standard JWT claims plus custom claims
/// for the sync system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User's email address
    pub email: Option<String>,
    /// Token issue timestamp
    pub iat: DateTime<Utc>,
    /// Token expiration timestamp
    pub exp: DateTime<Utc>,
    /// Issuer
    pub iss: String,
    /// Audience (optional)
    pub aud: Option<String>,
    /// Token type (e.g., "access")
    #[serde(default)]
    pub token_type: Option<String>,
}

/// Errors that can occur during JWT validation
#[derive(Error, Debug)]
pub enum JwtError {
    /// Token is missing or malformed
    #[error("Invalid token format: {0}")]
    InvalidFormat(String),

    /// Token signature verification failed
    #[error("Invalid token signature")]
    InvalidSignature,

    /// Token has expired
    #[error("Token expired at {0}")]
    Expired(DateTime<Utc>),

    /// Token is not yet valid (iat in future)
    #[error("Token not yet valid")]
    NotYetValid,

    /// Token issuer does not match expected issuer
    #[error("Invalid issuer: expected {expected}, got {actual}")]
    InvalidIssuer { expected: String, actual: String },

    /// Token audience does not match expected audience
    #[error("Invalid audience: expected {expected}, got {actual}")]
    InvalidAudience { expected: String, actual: String },

    /// Token validation error
    #[error("Token validation error: {0}")]
    ValidationError(String),

    /// Decoding error
    #[error("Failed to decode token: {0}")]
    DecodeError(String),
}

/// Result type for JWT operations
pub type JwtResult<T> = Result<T, JwtError>;

/// Validates a JWT token and returns the claims
///
/// # Arguments
/// * `token` - The JWT token string to validate
/// * `config` - JWT configuration with secret and issuer
///
/// # Returns
/// Ok(Claims) if the token is valid, Err(JwtError) otherwise
pub fn validate_token(token: &str, config: &JwtConfig) -> JwtResult<Claims> {
    // Decode the token header to get the algorithm
    let header = decode_header(token).map_err(|e| JwtError::DecodeError(e.to_string()))?;

    // Validate algorithm is not none
    if header.alg != Algorithm::HS256 {
        return Err(JwtError::InvalidFormat(
            "Only HS256 algorithm is supported".to_string(),
        ));
    }

    // Create validation with custom settings
    let mut validation = Validation::new(Algorithm::HS256);

    // Set expected issuer
    validation.set_issuer(&[&config.issuer]);

    // Set expected audience if configured
    if let Some(audience) = &config.audience {
        validation.set_audience(&[audience]);
    }

    // Set expiration tolerance
    validation.validate_exp = true;
    validation.validate_nbf = true;

    // Decode and validate the token
    let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                JwtError::Expired(Utc::now())
            }
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                JwtError::InvalidIssuer {
                    expected: config.issuer.clone(),
                    actual: e.to_string(),
                }
            }
            jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                JwtError::InvalidAudience {
                    expected: config.audience.clone().unwrap_or_default(),
                    actual: e.to_string(),
                }
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtError::InvalidSignature,
            jsonwebtoken::errors::ErrorKind::ImmatureSignature => JwtError::NotYetValid,
            _ => JwtError::ValidationError(e.to_string()),
        })?;

    Ok(token_data.claims)
}

/// Extracts the user ID from a valid token
///
/// # Arguments
/// * `token` - The JWT token string
/// * `config` - JWT configuration
///
/// # Returns
/// Ok(user_id) if the token is valid, Err otherwise
pub fn extract_user_id(token: &str, config: &JwtConfig) -> JwtResult<Uuid> {
    let claims = validate_token(token, config)?;
    Ok(claims.sub)
}

/// Checks if a token is expired without full validation
///
/// Useful for checking if a refresh is needed
pub fn is_token_expired(token: &str, config: &JwtConfig) -> bool {
    validate_token(token, config).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn create_test_token(
        sub: Uuid,
        email: Option<&str>,
        secret: &str,
        issuer: &str,
        expires_in: chrono::Duration,
    ) -> String {
        let now = Utc::now();
        let claims = Claims {
            sub,
            email: email.map(|s| s.to_string()),
            iat: now,
            exp: now + expires_in,
            iss: issuer.to_string(),
            aud: None,
            token_type: Some("access".to_string()),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_valid_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, Some("test@example.com"), "test-secret", "test-issuer", chrono::Duration::hours(1));

        let claims = validate_token(&token, &config).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.iss, "test-issuer");
    }

    #[test]
    fn test_expired_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "test-secret", "test-issuer", -chrono::Duration::hours(1));

        let result = validate_token(&token, &config);
        assert!(matches!(result, Err(JwtError::Expired(_))));
    }

    #[test]
    fn test_invalid_signature() {
        let config = JwtConfig {
            secret: "correct-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "wrong-secret", "test-issuer", chrono::Duration::hours(1));

        let result = validate_token(&token, &config);
        assert!(matches!(result, Err(JwtError::InvalidSignature)));
    }

    #[test]
    fn test_invalid_issuer() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "expected-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "test-secret", "wrong-issuer", chrono::Duration::hours(1));

        let result = validate_token(&token, &config);
        assert!(matches!(result, Err(JwtError::InvalidIssuer { .. })));
    }

    #[test]
    fn test_extract_user_id() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "test-secret", "test-issuer", chrono::Duration::hours(1));

        let extracted = extract_user_id(&token, &config).unwrap();
        assert_eq!(extracted, user_id);
    }

    #[test]
    fn test_is_token_expired() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        // Valid token
        let user_id = Uuid::new_v4();
        let valid_token = create_test_token(user_id, None, "test-secret", "test-issuer", chrono::Duration::hours(1));
        assert!(!is_token_expired(&valid_token, &config));

        // Expired token
        let expired_token = create_test_token(user_id, None, "test-secret", "test-issuer", -chrono::Duration::hours(1));
        assert!(is_token_expired(&expired_token, &config));
    }
}