//! HashiCorp Vault integration for ephemeral database credentials.
//!
//! This module provides secure credential management through HashiCorp Vault,
//! ensuring zero-credential-exposure by using short-lived ephemeral credentials
//! with automatic rotation.
//!
//! # Key Features
//! - Request ephemeral database credentials from Vault
//! - Cache credentials with TTL tracking
//! - Automatic credential rotation before expiration
//! - Handle Vault unavailability using cached credentials
//!
//! # Requirements
//! - Credentials have TTL ≤ 3600 seconds (1 hour)
//! - No fallback to static credentials on Vault unavailability
//! - No static passwords in configuration

use chrono::{DateTime, Duration, Utc};
use shared::{DatabaseCredentials, VaultError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use vaultrs::client::VaultClient;

/// Maximum credential TTL in seconds (1 hour)
const MAX_CREDENTIAL_TTL: u64 = 3600;

/// Buffer time before expiration to trigger rotation (5 minutes)
const ROTATION_BUFFER_SECONDS: i64 = 300;

/// Cached credentials with metadata for TTL tracking.
#[derive(Debug, Clone)]
struct CachedCredentials {
    /// The actual credentials
    credentials: DatabaseCredentials,
    /// When the credentials were cached
    cached_at: DateTime<Utc>,
    /// When rotation should occur (before expiration)
    rotate_at: DateTime<Utc>,
}

impl CachedCredentials {
    /// Check if credentials are still valid (not expired)
    fn is_valid(&self) -> bool {
        Utc::now() < self.rotate_at
    }

    /// Time remaining until rotation is needed
    fn time_until_rotation(&self) -> i64 {
        let now = Utc::now();
        if now >= self.rotate_at {
            0
        } else {
            self.rotate_at
                .signed_duration_since(now)
                .num_seconds()
        }
    }
}

/// Vault credential manager with caching and automatic rotation.
///
/// This manager handles the lifecycle of ephemeral database credentials:
/// 1. Requests credentials from Vault
/// 2. Caches them with TTL tracking
/// 3. Automatically rotates before expiration
/// 4. Falls back to cached credentials if Vault is temporarily unavailable
#[derive(Clone)]
pub struct VaultCredentialManager {
    /// The underlying Vault client
    vault_client: Arc<VaultClient>,
    /// Cached credentials with TTL tracking
    cached_credentials: Arc<RwLock<Option<CachedCredentials>>>,
    /// Database role name in Vault
    role_name: String,
    /// Mount path for database secrets engine
    mount_path: String,
}

impl VaultCredentialManager {
    /// Create a new Vault credential manager.
    ///
    /// # Arguments
    /// * `vault_client` - Authenticated Vault client
    /// * `role_name` - Database role name configured in Vault
    /// * `mount_path` - Mount path for database secrets engine (e.g., "database")
    ///
    /// # Returns
    /// Configured credential manager ready to use
    pub fn new(vault_client: Arc<VaultClient>, role_name: String, mount_path: String) -> Self {
        Self {
            vault_client,
            cached_credentials: Arc::new(RwLock::new(None)),
            role_name,
            mount_path,
        }
    }

    /// Get ephemeral database credentials, using cache if available and valid.
    ///
    /// This function implements the core credential retrieval logic:
    /// 1. Check if cached credentials are still valid
    /// 2. If valid and not near expiration, return cached credentials
    /// 3. If near expiration or invalid, request new credentials from Vault
    /// 4. If Vault is unavailable, use cached credentials if not expired
    ///
    /// # Returns
    /// Ephemeral database credentials with TTL ≤ 3600 seconds
    ///
    /// # Errors
    /// * `VaultError::NoCachedCredentials` - No cached credentials and Vault unavailable
    /// * `VaultError::ConnectionFailed` - Vault connection failed
    /// * `VaultError::RequestFailed` - Vault request failed
    /// * `VaultError::InvalidCredentials` - Invalid credentials received
    pub async fn get_credentials(&self) -> Result<DatabaseCredentials, VaultError> {
        // Try to get cached credentials first
        if let Some(cached) = self.get_cached_credentials().await {
            // Check if cache is still valid
            if cached.is_valid() {
                debug!(
                    "Using cached credentials, rotation needed in {} seconds",
                    cached.time_until_rotation()
                );
                return Ok(cached.credentials.clone());
            }

            // Cache is expired or near expiration, try to rotate
            debug!("Cached credentials expired or near expiration, requesting new credentials");
        }

        // Request new credentials from Vault
        match self.fetch_credentials_from_vault().await {
            Ok(credentials) => {
                self.cache_credentials(&credentials).await;
                Ok(credentials)
            }
            Err(vault_err) => {
                // Vault unavailable - try to use cached credentials if available
                if let Some(cached) = self.get_cached_credentials().await {
                    // Check if cached credentials are still usable (not past actual expiry)
                    let actual_expiry = cached.credentials.issued_at
                        + Duration::seconds(cached.credentials.ttl as i64);

                    if Utc::now() < actual_expiry {
                        warn!(
                            "Vault unavailable, using cached credentials (expire at {})",
                            actual_expiry
                        );
                        return Ok(cached.credentials.clone());
                    }

                    // Cached credentials are also expired
                    error!("Vault unavailable and cached credentials expired");
                    return Err(VaultError::ConnectionFailed(format!(
                        "Vault unavailable and cached credentials expired: {}",
                        vault_err
                    )));
                }

                // No cached credentials available
                error!("Vault unavailable and no cached credentials available");
                Err(vault_err)
            }
        }
    }

    /// Fetch fresh credentials directly from Vault.
    ///
    /// # Returns
    /// New ephemeral credentials from Vault
    ///
    /// # Errors
    /// * `VaultError::ConnectionFailed` - Vault connection failed
    /// * `VaultError::RequestFailed` - Vault request failed
    /// * `VaultError::InvalidCredentials` - Invalid credentials received
    async fn fetch_credentials_from_vault(&self) -> Result<DatabaseCredentials, VaultError> {
        debug!("Requesting ephemeral credentials from Vault for role: {}", self.role_name);

        // Request credentials from Vault database secrets engine
        let vault_creds = match vaultrs::database::static_role::creds(
            self.vault_client.as_ref(),
            &self.mount_path,
            &self.role_name,
        ).await {
            Ok(creds) => creds,
            Err(e) => {
                error!("Failed to fetch credentials from Vault: {}", e);
                return Err(VaultError::RequestFailed(e.to_string()));
            }
        };

        // Validate credentials
        if vault_creds.username.is_empty() || vault_creds.password.is_empty() {
            error!("Received empty credentials from Vault");
            return Err(VaultError::InvalidCredentials);
        }

        // Enforce maximum TTL of 1 hour
        let ttl = vault_creds.ttl.min(MAX_CREDENTIAL_TTL);

        let credentials = DatabaseCredentials {
            username: vault_creds.username,
            password: vault_creds.password,
            ttl,
            issued_at: Utc::now(),
        };

        info!(
            "Successfully obtained ephemeral credentials from Vault, TTL: {} seconds",
            ttl
        );

        Ok(credentials)
    }

    /// Cache credentials with TTL tracking.
    async fn cache_credentials(&self, credentials: &DatabaseCredentials) {
        let now = Utc::now();
        let rotate_at = now + Duration::seconds(credentials.ttl as i64 - ROTATION_BUFFER_SECONDS);

        let cached = CachedCredentials {
            credentials: credentials.clone(),
            cached_at: now,
            rotate_at,
        };

        let mut write_guard = self.cached_credentials.write().await;
        *write_guard = Some(cached);

        debug!(
            "Cached credentials, will rotate at {}",
            rotate_at
        );
    }

    /// Get cached credentials without triggering a refresh.
    async fn get_cached_credentials(&self) -> Option<CachedCredentials> {
        let read_guard = self.cached_credentials.read().await;
        read_guard.clone()
    }

    /// Force a credential refresh, bypassing the cache.
    ///
    /// This is useful for:
    /// - Manual rotation triggers
    /// - Health check verification
    /// - After Vault connectivity is restored
    ///
    /// # Returns
    /// Fresh credentials from Vault
    pub async fn force_refresh(&self) -> Result<DatabaseCredentials, VaultError> {
        debug!("Force refreshing credentials from Vault");
        let credentials = self.fetch_credentials_from_vault().await?;
        self.cache_credentials(&credentials).await;
        Ok(credentials)
    }

    /// Check if credentials need rotation.
    ///
    /// # Returns
    /// True if credentials need to be rotated soon
    pub async fn needs_rotation(&self) -> bool {
        if let Some(cached) = self.get_cached_credentials().await {
            !cached.is_valid() || cached.time_until_rotation() <= 0
        } else {
            true
        }
    }

    /// Get the current cached credentials (for inspection).
    ///
    /// # Returns
    /// Clone of cached credentials if available
    pub async fn get_cached_info(&self) -> Option<DatabaseCredentials> {
        self.get_cached_credentials()
            .await
            .map(|c| c.credentials.clone())
    }
}

/// Start the automatic credential rotation background task.
///
/// This task runs periodically and refreshes credentials before they expire.
/// It ensures the system always has valid credentials available.
///
/// # Arguments
/// * `manager` - The credential manager to use
/// * `check_interval` - How often to check if rotation is needed
///
/// # Notes
/// - The task runs until the spawned task is cancelled
/// - Uses a conservative rotation buffer to ensure credentials are always valid
pub async fn start_credential_rotation_task(
    manager: VaultCredentialManager,
    check_interval: std::time::Duration,
) {
    info!(
        "Starting credential rotation task with check interval {:?}",
        check_interval
    );

    let mut interval = tokio::time::interval(check_interval);

    loop {
        interval.tick().await;

        if let Err(e) = try_rotate_credentials(&manager).await {
            error!("Credential rotation failed: {}", e);
            // Continue trying on next interval
        }
    }
}

/// Attempt to rotate credentials if needed.
///
/// # Arguments
/// * `manager` - The credential manager to use
///
/// # Returns
/// Ok if rotation was successful or not needed, Err on failure
async fn try_rotate_credentials(manager: &VaultCredentialManager) -> Result<(), VaultError> {
    if !manager.needs_rotation().await {
        debug!("Credentials do not need rotation yet");
        return Ok(());
    }

    debug!("Credentials need rotation, fetching new credentials");

    match manager.force_refresh().await {
        Ok(credentials) => {
            info!(
                "Successfully rotated credentials, new TTL: {} seconds",
                credentials.ttl
            );
            Ok(())
        }
        Err(e) => {
            // Check if we can still use cached credentials
            if let Some(cached) = manager.get_cached_info().await {
                let actual_expiry = cached.issued_at + Duration::seconds(cached.ttl as i64);
                if Utc::now() < actual_expiry {
                    warn!(
                        "Rotation failed but cached credentials still valid until {}",
                        actual_expiry
                    );
                    return Ok(());
                }
            }

            Err(e)
        }
    }
}

/// Create a Vault client with the given configuration.
///
/// # Arguments
/// * `vault_addr` - Vault server address
/// * `vault_token` - Vault authentication token
///
/// # Returns
/// Configured and authenticated Vault client
///
/// # Errors
/// * `VaultError::ConnectionFailed` - Cannot connect to Vault
/// * `VaultError::AuthenticationFailed` - Invalid Vault token
pub async fn create_vault_client(
    _vault_addr: &str,
    _vault_token: &str,
) -> Result<Arc<VaultClient>, VaultError> {
    // Comment out: VaultClientSettings not properly configured in current vaultrs version
    // Requires proper initialization - placeholder for now
    Err(VaultError::ConnectionFailed(
        "VaultClientSettings initialization not configured - update vaultrs dependency".to_string()
    ))
    
    // let settings = VaultClientSettings {
    //     address: vault_addr.to_string(),
    //     token: vault_token.to_string(),
    //     ..Default::default()
    // };
    //
    // let client = match VaultClient::new(settings) {
    //     Ok(c) => c,
    //     Err(e) => {
    //         error!("Failed to create Vault client: {}", e);
    //         return Err(VaultError::ConnectionFailed(e.to_string()));
    //     }
    // };
    //
    // info!("Successfully connected to Vault");
    //
    // Ok(Arc::new(client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cached_credentials_validity() {
        let now = Utc::now();
        let credentials = DatabaseCredentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            ttl: 3600,
            issued_at: now,
        };

        let cached = CachedCredentials {
            credentials,
            cached_at: now,
            rotate_at: now + Duration::seconds(3600 - 300), // 55 minutes from now
        };

        assert!(cached.is_valid());
        assert!(cached.time_until_rotation() > 0);
    }

    #[tokio::test]
    async fn test_cached_credentials_expired() {
        let now = Utc::now();
        let credentials = DatabaseCredentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            ttl: 3600,
            issued_at: now - Duration::hours(2), // 2 hours ago
        };

        let cached = CachedCredentials {
            credentials,
            cached_at: now - Duration::hours(2),
            rotate_at: now - Duration::hours(1), // 1 hour ago
        };

        assert!(!cached.is_valid());
        assert_eq!(cached.time_until_rotation(), 0);
    }

    #[tokio::test]
    async fn test_rotation_buffer() {
        let now = Utc::now();
        let credentials = DatabaseCredentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            ttl: 3600,
            issued_at: now,
        };

        let rotate_at = now + Duration::seconds(credentials.ttl as i64 - ROTATION_BUFFER_SECONDS);
        let expected_rotate = now + Duration::seconds(3300); // 55 minutes

        assert_eq!(rotate_at, expected_rotate);
    }
}