//! User Session Service
//!
//! Manages user sessions across all modules with cross-cutting concerns

use crate::domain::entities::{UserSession, DeviceInfo, DeviceType};
use crate::domain::value_objects::Email;
use crate::shared::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Domain service for managing user sessions
pub struct UserSessionService {
    max_sessions_per_user: u32,
    default_session_duration: Duration,
}

impl UserSessionService {
    pub fn new() -> Self {
        Self {
            max_sessions_per_user: 5,
            default_session_duration: Duration::hours(24),
        }
    }

    pub fn with_config(max_sessions: u32, duration_hours: i64) -> Self {
        Self {
            max_sessions_per_user: max_sessions,
            default_session_duration: Duration::hours(duration_hours),
        }
    }

    /// Create a new user session
    pub fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        device_info: Option<DeviceInfo>,
        permissions: Vec<String>,
    ) -> AppResult<UserSession> {
        let expires_at = Utc::now() + self.default_session_duration;
        let mut session = UserSession::new(user_id, token_hash, expires_at);

        session.device_info = device_info;
        session.permissions = permissions;

        Ok(session)
    }

    /// Validate and refresh an existing session
    pub fn validate_session(&self, session: &mut UserSession) -> AppResult<()> {
        if !session.is_active {
            return Err(AppError::unauthorized("Session is inactive"));
        }

        if session.is_expired() {
            return Err(AppError::unauthorized("Session has expired"));
        }

        // Update last accessed time
        session.last_accessed_at = Utc::now();

        Ok(())
    }

    /// Extend session expiration
    pub fn extend_session(&self, session: &mut UserSession, hours: i64) -> AppResult<()> {
        if !session.is_active || session.is_expired() {
            return Err(AppError::unauthorized("Cannot extend inactive or expired session"));
        }

        let new_expires_at = Utc::now() + Duration::hours(hours);
        session.refresh_expiration(new_expires_at);

        Ok(())
    }

    /// Revoke a session (logout)
    pub fn revoke_session(&self, session: &mut UserSession) -> AppResult<()> {
        session.revoke();
        Ok(())
    }

    /// Check if user can create a new session (limit enforcement)
    pub fn can_create_session(&self, active_sessions: &[UserSession]) -> AppResult<()> {
        let active_count = active_sessions
            .iter()
            .filter(|s| s.is_active && !s.is_expired())
            .count();

        if active_count >= self.max_sessions_per_user as usize {
            return Err(AppError::conflict(format!(
                "Maximum session limit of {} reached",
                self.max_sessions_per_user
            )));
        }

        Ok(())
    }

    /// Get oldest session to remove when limit is reached
    pub fn get_oldest_session(&self, sessions: &[UserSession]) -> Option<&UserSession> {
        sessions
            .iter()
            .filter(|s| s.is_active && !s.is_expired())
            .min_by_key(|s| s.last_accessed_at)
    }

    /// Check if session is from a trusted device
    pub fn is_trusted_device(&self, session: &UserSession) -> bool {
        match &session.device_info {
            Some(device_info) => match device_info.device_type {
                DeviceType::Desktop => true,
                DeviceType::Api => true,
                DeviceType::Mobile | DeviceType::Tablet => {
                    // Additional logic for mobile/tablet trust
                    session.last_accessed_at.signed_duration_since(Utc::now()).num_days() < 7
                }
                DeviceType::Unknown => false,
            },
            None => false,
        }
    }

    /// Get sessions by device type
    pub fn get_sessions_by_device_type(
        &self,
        sessions: &[UserSession],
        device_type: DeviceType,
    ) -> Vec<&UserSession> {
        sessions
            .iter()
            .filter(|s| {
                s.device_info
                    .as_ref()
                    .map_or(false, |d| d.device_type == device_type)
            })
            .collect()
    }

    /// Get sessions that are expiring soon
    pub fn get_sessions_expiring_soon(&self, sessions: &[UserSession], hours: i64) -> Vec<&UserSession> {
        let threshold = Utc::now() + Duration::hours(hours);
        sessions
            .iter()
            .filter(|s| s.is_active && s.expires_at <= threshold && s.expires_at > Utc::now())
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self, sessions: &mut Vec<UserSession>) -> usize {
        let initial_count = sessions.len();
        sessions.retain(|s| !s.is_expired());
        initial_count - sessions.len()
    }

    /// Validate session permissions
    pub fn session_has_permission(&self, session: &UserSession, permission: &str) -> bool {
        session.permissions.contains(&permission.to_string())
    }

    /// Add permission to session
    pub fn add_permission_to_session(&self, session: &mut UserSession, permission: String) {
        if !session.permissions.contains(&permission) {
            session.permissions.push(permission);
        }
    }

    /// Remove permission from session
    pub fn remove_permission_from_session(&self, session: &mut UserSession, permission: &str) {
        session.permissions.retain(|p| p != permission);
    }

    /// Get session security score (0-100)
    pub fn get_security_score(&self, session: &UserSession) -> u8 {
        let mut score = 50u8;

        // Device type scoring
        if let Some(device) = &session.device_info {
            match device.device_type {
                DeviceType::Desktop => score += 20,
                DeviceType::Api => score += 15,
                DeviceType::Tablet => score += 10,
                DeviceType::Mobile => score += 5,
                DeviceType::Unknown => score -= 10,
            }

            // Browser presence adds security
            if device.browser.is_some() {
                score += 5;
            }
        } else {
            score -= 10;
        }

        // Session age scoring (newer is better)
        let age_hours = Utc::now().signed_duration_since(session.created_at).num_hours();
        if age_hours < 1 {
            score += 10;
        } else if age_hours > 24 {
            score -= 10;
        }

        // Activity scoring
        let last_activity_hours = Utc::now().signed_duration_since(session.last_accessed_at).num_hours();
        if last_activity_hours < 1 {
            score += 10;
        } else if last_activity_hours > 8 {
            score -= 5;
        }

        // Permissions scoring (more permissions = lower security)
        let permission_penalty = (session.permissions.len() as u8).saturating_sub(5) * 2;
        score = score.saturating_sub(permission_penalty);

        score.min(100).max(0)
    }

    /// Detect suspicious session activity
    pub fn is_suspicious_activity(&self, session: &UserSession) -> bool {
        // Check for very old session
        let age_days = Utc::now().signed_duration_since(session.created_at).num_days();
        if age_days > 30 {
            return true;
        }

        // Check for session from unknown device type
        if let Some(device) = &session.device_info {
            if device.device_type == DeviceType::Unknown {
                return true;
            }

            // Check for missing user agent (bot-like behavior)
            if device.user_agent.is_none() && device.device_type == DeviceType::Desktop {
                return true;
            }
        }

        false
    }
}

impl Default for UserSessionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let service = UserSessionService::new();
        let user_id = Uuid::new_v4();
        let token_hash = "test_hash".to_string();
        let permissions = vec!["read".to_string(), "write".to_string()];

        let session = service
            .create_session(user_id, token_hash.clone(), None, permissions.clone())
            .unwrap();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.token_hash, token_hash);
        assert_eq!(session.permissions, permissions);
        assert!(session.is_active);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_validation() {
        let service = UserSessionService::new();
        let user_id = Uuid::new_v4();
        let token_hash = "test_hash".to_string();

        let mut session = service
            .create_session(user_id, token_hash, None, vec![])
            .unwrap();

        // Valid session should pass
        assert!(service.validate_session(&mut session).is_ok());

        // Expired session should fail
        session.expires_at = Utc::now() - Duration::minutes(1);
        assert!(service.validate_session(&mut session).is_err());
    }

    #[test]
    fn test_session_limits() {
        let service = UserSessionService::with_config(2, 24);
        let user_id = Uuid::new_v4();

        let mut sessions = Vec::new();

        // First session should succeed
        let session1 = service
            .create_session(user_id, "hash1".to_string(), None, vec![])
            .unwrap();
        sessions.push(session1);
        assert!(service.can_create_session(&sessions).is_ok());

        // Second session should succeed
        let session2 = service
            .create_session(user_id, "hash2".to_string(), None, vec![])
            .unwrap();
        sessions.push(session2);
        assert!(service.can_create_session(&sessions).is_ok());

        // Third session should fail (limit reached)
        assert!(service.can_create_session(&sessions).is_err());
    }

    #[test]
    fn test_device_trust() {
        let service = UserSessionService::new();
        let user_id = Uuid::new_v4();

        let desktop_device = DeviceInfo {
            device_type: DeviceType::Desktop,
            device_name: Some("MacBook Pro".to_string()),
            operating_system: Some("macOS".to_string()),
            browser: Some("Chrome".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0...".to_string()),
        };

        let unknown_device = DeviceInfo {
            device_type: DeviceType::Unknown,
            device_name: None,
            operating_system: None,
            browser: None,
            ip_address: None,
            user_agent: None,
        };

        let desktop_session = service
            .create_session(user_id, "hash1".to_string(), Some(desktop_device), vec![])
            .unwrap();
        let unknown_session = service
            .create_session(user_id, "hash2".to_string(), Some(unknown_device), vec![])
            .unwrap();

        assert!(service.is_trusted_device(&desktop_session));
        assert!(!service.is_trusted_device(&unknown_session));
    }

    #[test]
    fn test_security_scoring() {
        let service = UserSessionService::new();
        let user_id = Uuid::new_v4();

        let desktop_device = DeviceInfo {
            device_type: DeviceType::Desktop,
            device_name: Some("MacBook Pro".to_string()),
            operating_system: Some("macOS".to_string()),
            browser: Some("Chrome".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0...".to_string()),
        };

        let session = service
            .create_session(
                user_id,
                "hash".to_string(),
                Some(desktop_device),
                vec!["read".to_string()],
            )
            .unwrap();

        let score = service.get_security_score(&session);
        assert!(score > 50);
        assert!(score <= 100);
    }

    #[test]
    fn test_suspicious_activity_detection() {
        let service = UserSessionService::new();
        let user_id = Uuid::new_v4();

        let unknown_device = DeviceInfo {
            device_type: DeviceType::Unknown,
            device_name: None,
            operating_system: None,
            browser: None,
            ip_address: None,
            user_agent: None,
        };

        let mut session = service
            .create_session(user_id, "hash".to_string(), Some(unknown_device), vec![])
            .unwrap();

        // Make session very old
        session.created_at = Utc::now() - Duration::days(35);

        assert!(service.is_suspicious_activity(&session));
    }
}