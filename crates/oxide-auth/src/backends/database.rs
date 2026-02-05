//! Database authentication backend.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{AuthError, Result};
use crate::session::Session;
use crate::user::User;

/// Database-backed authentication backend.
///
/// This is the default authentication backend that authenticates users
/// against the database using username and password.
pub struct DatabaseBackend;

impl DatabaseBackend {
    /// Authenticates a user by username and password.
    ///
    /// Returns the user if authentication succeeds, None if credentials are invalid.
    pub async fn authenticate(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<Option<User>> {
        // Find user by username
        let user = match User::get_by_username(pool, username).await {
            Ok(u) => u,
            Err(AuthError::UserNotFound) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Verify password
        if !user.check_password(password) {
            return Ok(None);
        }

        Ok(Some(user))
    }

    /// Authenticates and logs in a user, creating a session.
    ///
    /// Returns the session if authentication succeeds.
    pub async fn login(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<(User, Session)> {
        // Authenticate
        let user = Self::authenticate(pool, username, password)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Update last login
        let mut user = user;
        user.last_login = Some(Utc::now());
        user.save(pool).await?;

        // Create session
        let session = Session::for_user(&user);
        session.save(pool).await?;

        Ok((user, session))
    }

    /// Logs out a user by deleting their session.
    pub async fn logout(pool: &SqlitePool, session_key: &str) -> Result<()> {
        let session = Session::get_by_key(pool, session_key).await?;
        session.delete(pool).await?;
        Ok(())
    }

    /// Logs out all sessions for a user.
    pub async fn logout_all(pool: &SqlitePool, user_id: i64) -> Result<u64> {
        Session::delete_for_user(pool, user_id).await
    }

    /// Gets the user for a session.
    pub async fn get_user(pool: &SqlitePool, session_key: &str) -> Result<Option<User>> {
        let session = match Session::get_by_key(pool, session_key).await {
            Ok(s) => s,
            Err(AuthError::SessionNotFound) => return Ok(None),
            Err(e) => return Err(e),
        };

        if session.is_expired() {
            session.delete(pool).await?;
            return Ok(None);
        }

        match session.user_id {
            Some(user_id) => {
                let user = User::get(pool, user_id).await?;
                if !user.is_active {
                    return Err(AuthError::UserInactive);
                }
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Validates a session and returns it if valid.
    pub async fn get_session(pool: &SqlitePool, session_key: &str) -> Result<Session> {
        let session = Session::get_by_key(pool, session_key).await?;

        if session.is_expired() {
            session.delete(pool).await?;
            return Err(AuthError::SessionNotFound);
        }

        Ok(session)
    }

    /// Changes a user's password.
    pub async fn change_password(
        pool: &SqlitePool,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let mut user = User::get(pool, user_id).await?;

        if !user.check_password(old_password) {
            return Err(AuthError::InvalidCredentials);
        }

        user.set_password(new_password)?;
        user.save(pool).await?;

        // Optionally: Invalidate all other sessions
        // Session::delete_for_user(pool, user_id).await?;

        Ok(())
    }

    /// Sets a user's password directly (for admin use).
    pub async fn set_password(pool: &SqlitePool, user_id: i64, new_password: &str) -> Result<()> {
        let mut user = User::get(pool, user_id).await?;
        user.set_password(new_password)?;
        user.save(pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here
}
