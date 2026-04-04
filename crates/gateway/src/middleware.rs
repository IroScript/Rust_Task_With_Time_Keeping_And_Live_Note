//! Authentication middleware for Axum
//!
//! Provides JWT authentication middleware for protecting HTTP routes
//! and WebSocket connections.

use auth::jwt::{validate_token, JwtConfig, JwtError};
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use headers::Authorization;
use http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use uuid::Uuid;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    /// JWT configuration for token validation
    pub jwt_config: Arc<JwtConfig>,
    // Add other state fields as needed (db_pool, s3_client, etc.)
}

/// Extension type for authenticated requests
///
/// This is stored in request extensions to provide access
/// to authenticated user information in handlers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthenticatedUser {
    /// User ID from JWT claims
    pub user_id: Uuid,
    /// User's email from JWT claims
    pub email: Option<String>,
}

impl AuthenticatedUser {
    /// Create a new authenticated user
    pub fn new(user_id: Uuid, email: Option<String>) -> Self {
        Self { user_id, email }
    }
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthFormat,

    #[error("Invalid or expired token: {0}")]
    InvalidToken(#[from] JwtError),

    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::MissingAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".to_string(),
            )
                .into_response(),
            AuthError::InvalidAuthFormat => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format. Expected: Bearer <token>".to_string(),
            )
                .into_response(),
            AuthError::InvalidToken(e) => (
                StatusCode::UNAUTHORIZED,
                format!("Invalid or expired token: {}", e),
            )
                .into_response(),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
                .into_response(),
        }
    }
}

/// Extract JWT token from Authorization header
fn extract_token_from_header(auth_header: &Authorization<headers::HeaderValue>) -> Result<&str, AuthError> {
    match auth_header.clone() {
        Authorization::bearer(token) => {
            token.to_str().map_err(|_| AuthError::InvalidAuthFormat)
        }
        _ => Err(AuthError::InvalidAuthFormat),
    }
}

/// Authentication middleware for HTTP routes
///
/// Validates JWT token from Authorization header and adds
/// AuthenticatedUser to request extensions.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Get authorization header
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(AuthError::MissingAuthHeader)?
        .parse::<Authorization<headers::HeaderValue>>()
        .map_err(|_| AuthError::InvalidAuthFormat)?;

    // Extract and validate token
    let token = extract_token_from_header(&auth_header)?;
    let claims = validate_token(token, &state.jwt_config)?;

    // Create authenticated user and add to extensions
    let user = AuthenticatedUser::new(claims.sub, claims.email);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Layer for requiring authentication on routes
///
/// This can be used with `.layer()` on individual routes or routers
/// to require authentication for all routes in that router.
pub fn require_auth() -> ValidateRequestHeaderLayer<AuthError, Body, Body> {
    ValidateRequestHeaderLayer::custom(|request: &Request<Body>| {
        let auth_header = request
            .headers()
            .get(http::header::AUTHORIZATION)
            .cloned();

        match auth_header {
            Some(header) => {
                let auth: Result<Authorization<headers::HeaderValue>, _> = header.parse();
                match auth {
                    Ok(Authorization::bearer(token)) => {
                        let token_str = token.to_str().ok();
                        if token_str.is_none() {
                            return Err(AuthError::InvalidAuthFormat);
                        }
                        // Note: Full validation happens in the middleware
                        // This layer just checks the header format exists
                        Ok(())
                    }
                    _ => Err(AuthError::InvalidAuthFormat),
                }
            }
            None => Err(AuthError::MissingAuthHeader),
        }
    })
}

/// Helper function to get authenticated user from request extensions
///
/// # Panics
/// Panics if the request extensions don't contain an AuthenticatedUser.
/// This should only be called after the auth middleware has run.
pub fn get_authenticated_user(req: &Request) -> AuthenticatedUser {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .expect("AuthenticatedUser not found in request extensions. Did you add the auth middleware?")
}

/// Optional authentication that doesn't fail if no token is present
///
/// Useful for routes that can work with or without authentication.
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = req.headers().get(http::header::AUTHORIZATION) {
        if let Ok(Authorization(bearer)) = auth_header.parse::<Authorization<headers::HeaderValue>>() {
            if let Ok(token_str) = bearer.to_str() {
                if let Ok(claims) = validate_token(token_str, &state.jwt_config) {
                    let user = AuthenticatedUser::new(claims.sub, claims.email);
                    req.extensions_mut().insert(user);
                }
            }
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use auth::jwt::{Claims, JwtConfig};
    use axum::{routing::get, Router};
    use chrono::{DateTime, Utc};
    use http::StatusCode;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use tower::ServiceExt;

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

    fn create_app(jwt_config: JwtConfig) -> Router {
        let state = Arc::new(jwt_config);
        Router::new()
            .route("/protected", get(|| async { "protected content" }))
            .route("/public", get(|| async { "public content" }))
            .with_state(AppState { jwt_config: state })
            .layer(middleware::from_fn(auth_middleware))
    }

    #[tokio::test]
    async fn test_valid_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, Some("test@example.com"), "test-secret", "test-issuer", chrono::Duration::hours(1));

        let app = create_app(config);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let app = create_app(config);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let app = create_app(config);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(http::header::AUTHORIZATION, "Bearer invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_token() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "test-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "test-secret", "test-issuer", -chrono::Duration::hours(1));

        let app = create_app(config);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_wrong_issuer() {
        let config = JwtConfig {
            secret: "test-secret".to_string(),
            issuer: "expected-issuer".to_string(),
            audience: None,
            expiration_tolerance: 60,
        };

        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, None, "test-secret", "wrong-issuer", chrono::Duration::hours(1));

        let app = create_app(config);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}