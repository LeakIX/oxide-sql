//! Session management for user authentication.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use std::collections::HashMap;

use crate::error::{AuthError, Result};
use crate::user::User;

/// Session data stored as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    /// User ID if authenticated.
    pub user_id: Option<i64>,
    /// Additional session data.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A session for tracking user authentication state.
#[derive(Debug, Clone, FromRow)]
pub struct Session {
    /// Unique session key (64 character hex string).
    pub session_key: String,
    /// JSON-encoded session data.
    pub session_data: String,
    /// Session expiration timestamp.
    pub expire_date: DateTime<Utc>,
    /// Associated user ID (if authenticated).
    pub user_id: Option<i64>,
}

impl Session {
    /// Default session expiration time (2 weeks).
    pub const DEFAULT_EXPIRY_DAYS: i64 = 14;

    /// Creates a new anonymous session.
    pub fn new() -> Self {
        Self {
            session_key: generate_session_key(),
            session_data: serde_json::to_string(&SessionData::default()).unwrap(),
            expire_date: Utc::now() + Duration::days(Self::DEFAULT_EXPIRY_DAYS),
            user_id: None,
        }
    }

    /// Creates a new session for a user.
    pub fn for_user(user: &User) -> Self {
        let data = SessionData {
            user_id: Some(user.id),
            extra: HashMap::new(),
        };

        Self {
            session_key: generate_session_key(),
            session_data: serde_json::to_string(&data).unwrap(),
            expire_date: Utc::now() + Duration::days(Self::DEFAULT_EXPIRY_DAYS),
            user_id: Some(user.id),
        }
    }

    /// Returns whether this session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expire_date
    }

    /// Returns the decoded session data.
    pub fn get_data(&self) -> SessionData {
        serde_json::from_str(&self.session_data).unwrap_or_default()
    }

    /// Sets the session data.
    pub fn set_data(&mut self, data: SessionData) {
        self.session_data = serde_json::to_string(&data).unwrap();
        self.user_id = data.user_id;
    }

    /// Gets a value from the session data.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let data = self.get_data();
        data.extra
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Sets a value in the session data.
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        let mut data = self.get_data();
        data.extra
            .insert(key.to_string(), serde_json::to_value(value).unwrap());
        self.set_data(data);
    }

    /// Removes a value from the session data.
    pub fn remove(&mut self, key: &str) {
        let mut data = self.get_data();
        data.extra.remove(key);
        self.set_data(data);
    }

    /// Extends the session expiration.
    pub fn extend(&mut self, days: i64) {
        self.expire_date = Utc::now() + Duration::days(days);
    }

    /// Saves the session to the database.
    pub async fn save(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO auth_session (session_key, session_data, expire_date, user_id)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(session_key) DO UPDATE SET
                session_data = excluded.session_data,
                expire_date = excluded.expire_date,
                user_id = excluded.user_id
            "#,
        )
        .bind(&self.session_key)
        .bind(&self.session_data)
        .bind(self.expire_date)
        .bind(self.user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deletes the session from the database.
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM auth_session WHERE session_key = ?")
            .bind(&self.session_key)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Finds a session by its key.
    pub async fn get_by_key(pool: &SqlitePool, session_key: &str) -> Result<Self> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM auth_session WHERE session_key = ? AND expire_date > ?",
        )
        .bind(session_key)
        .bind(Utc::now())
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::SessionNotFound)?;

        Ok(session)
    }

    /// Finds all sessions for a user.
    pub async fn get_by_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<Self>> {
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT * FROM auth_session WHERE user_id = ? AND expire_date > ?",
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    /// Deletes all sessions for a user.
    pub async fn delete_for_user(pool: &SqlitePool, user_id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM auth_session WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Deletes all expired sessions.
    pub async fn clear_expired(pool: &SqlitePool) -> Result<u64> {
        let result = sqlx::query("DELETE FROM auth_session WHERE expire_date < ?")
            .bind(Utc::now())
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Returns the count of active sessions.
    pub async fn count(pool: &SqlitePool) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM auth_session WHERE expire_date > ?")
            .bind(Utc::now())
            .fetch_one(pool)
            .await?;
        Ok(row.get(0))
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a cryptographically secure session key.
fn generate_session_key() -> String {
    use rand::RngExt;
    let mut rng = rand::rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    hex::encode(&bytes)
}

/// Helper module for hex encoding (avoiding external dependency).
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// SQL to create the auth_session table.
pub const CREATE_SESSION_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS auth_session (
    session_key VARCHAR(64) PRIMARY KEY,
    session_data TEXT NOT NULL,
    expire_date TIMESTAMP NOT NULL,
    user_id INTEGER REFERENCES auth_user(id) ON DELETE CASCADE
)
"#;

/// Creates the auth_session table if it doesn't exist.
pub async fn create_session_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(CREATE_SESSION_TABLE_SQL).execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_generation() {
        let key1 = generate_session_key();
        let key2 = generate_session_key();

        assert_eq!(key1.len(), 64);
        assert_eq!(key2.len(), 64);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_session_data() {
        let mut session = Session::new();

        session.set("test_key", "test_value");
        let value: Option<String> = session.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));

        session.remove("test_key");
        let value: Option<String> = session.get("test_key");
        assert_eq!(value, None);
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new();
        assert!(!session.is_expired());

        // Force expiration
        session.expire_date = Utc::now() - Duration::days(1);
        assert!(session.is_expired());
    }
}
