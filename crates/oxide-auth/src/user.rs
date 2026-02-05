//! User model and related types.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row, SqlitePool};

use crate::error::{AuthError, Result};
use crate::password::{hash_password, validate_password, verify_password};

/// A user account for authentication.
///
/// This is the default user model similar to Django's auth.User.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    /// Primary key.
    pub id: i64,
    /// Unique username.
    pub username: String,
    /// Email address.
    pub email: String,
    /// Argon2 password hash.
    #[sqlx(rename = "password_hash")]
    password_hash: String,
    /// Whether the user can log in.
    pub is_active: bool,
    /// Whether the user has staff privileges.
    pub is_staff: bool,
    /// Whether the user has all permissions.
    pub is_superuser: bool,
    /// Last login timestamp.
    pub last_login: Option<DateTime<Utc>>,
    /// Account creation timestamp.
    pub date_joined: DateTime<Utc>,
}

impl User {
    /// Creates a new user with the given credentials.
    ///
    /// The password will be hashed automatically.
    pub fn create(username: &str, email: &str, password: &str) -> Result<Self> {
        validate_password(password)?;
        let password_hash = hash_password(password)?;

        Ok(Self {
            id: 0, // Will be set by database
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            is_active: true,
            is_staff: false,
            is_superuser: false,
            last_login: None,
            date_joined: Utc::now(),
        })
    }

    /// Creates a new superuser with the given credentials.
    pub fn create_superuser(username: &str, email: &str, password: &str) -> Result<Self> {
        let mut user = Self::create(username, email, password)?;
        user.is_staff = true;
        user.is_superuser = true;
        Ok(user)
    }

    /// Checks if the given password matches this user's password.
    pub fn check_password(&self, password: &str) -> bool {
        verify_password(password, &self.password_hash)
    }

    /// Sets a new password for this user.
    ///
    /// The password will be hashed automatically.
    pub fn set_password(&mut self, password: &str) -> Result<()> {
        validate_password(password)?;
        self.password_hash = hash_password(password)?;
        Ok(())
    }

    /// Returns whether this user has the given permission.
    ///
    /// Superusers have all permissions.
    pub fn has_perm(&self, _perm: &str) -> bool {
        // TODO: Implement permission checking
        self.is_superuser
    }

    /// Returns whether this user has all the given permissions.
    pub fn has_perms(&self, perms: &[&str]) -> bool {
        perms.iter().all(|p| self.has_perm(p))
    }

    /// Returns the user's full name or username.
    pub fn get_full_name(&self) -> &str {
        &self.username
    }

    /// Returns the user's short name (username).
    pub fn get_short_name(&self) -> &str {
        &self.username
    }

    /// Saves the user to the database.
    pub async fn save(&mut self, pool: &SqlitePool) -> Result<()> {
        if self.id == 0 {
            // Insert new user
            let result = sqlx::query(
                r#"
                INSERT INTO auth_user (username, email, password_hash, is_active,
                    is_staff, is_superuser, last_login, date_joined)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&self.username)
            .bind(&self.email)
            .bind(&self.password_hash)
            .bind(self.is_active)
            .bind(self.is_staff)
            .bind(self.is_superuser)
            .bind(self.last_login)
            .bind(self.date_joined)
            .execute(pool)
            .await?;

            self.id = result.last_insert_rowid();
        } else {
            // Update existing user
            sqlx::query(
                r#"
                UPDATE auth_user
                SET username = ?, email = ?, password_hash = ?, is_active = ?,
                    is_staff = ?, is_superuser = ?, last_login = ?
                WHERE id = ?
                "#,
            )
            .bind(&self.username)
            .bind(&self.email)
            .bind(&self.password_hash)
            .bind(self.is_active)
            .bind(self.is_staff)
            .bind(self.is_superuser)
            .bind(self.last_login)
            .bind(self.id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Deletes the user from the database.
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM auth_user WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Finds a user by ID.
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Self> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM auth_user WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
    }

    /// Finds a user by username.
    pub async fn get_by_username(pool: &SqlitePool, username: &str) -> Result<Self> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM auth_user WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
    }

    /// Finds a user by email.
    pub async fn get_by_email(pool: &SqlitePool, email: &str) -> Result<Self> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM auth_user WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
    }

    /// Returns all users.
    pub async fn all(pool: &SqlitePool) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM auth_user")
            .fetch_all(pool)
            .await?;
        Ok(users)
    }

    /// Returns the count of all users.
    pub async fn count(pool: &SqlitePool) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM auth_user")
            .fetch_one(pool)
            .await?;
        Ok(row.get(0))
    }
}

/// SQL to create the auth_user table.
pub const CREATE_USER_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS auth_user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(150) NOT NULL UNIQUE,
    email VARCHAR(254) NOT NULL,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_staff BOOLEAN NOT NULL DEFAULT FALSE,
    is_superuser BOOLEAN NOT NULL DEFAULT FALSE,
    last_login TIMESTAMP,
    date_joined TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
"#;

/// Creates the auth_user table if it doesn't exist.
pub async fn create_user_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(CREATE_USER_TABLE_SQL).execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let user = User::create("testuser", "test@example.com", "password123").unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(!user.is_staff);
        assert!(!user.is_superuser);
    }

    #[test]
    fn test_create_superuser() {
        let user = User::create_superuser("admin", "admin@example.com", "password123").unwrap();
        assert!(user.is_staff);
        assert!(user.is_superuser);
    }

    #[test]
    fn test_password_check() {
        let user = User::create("testuser", "test@example.com", "password123").unwrap();
        assert!(user.check_password("password123"));
        assert!(!user.check_password("wrongpassword"));
    }

    #[test]
    fn test_set_password() {
        let mut user = User::create("testuser", "test@example.com", "password123").unwrap();
        user.set_password("newpassword123").unwrap();
        assert!(user.check_password("newpassword123"));
        assert!(!user.check_password("password123"));
    }
}
