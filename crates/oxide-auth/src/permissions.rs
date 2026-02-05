//! Permission and group models for authorization.

use sqlx::{FromRow, SqlitePool};

use crate::error::{AuthError, Result};

/// A permission that can be assigned to users or groups.
#[derive(Debug, Clone, FromRow)]
pub struct Permission {
    /// Primary key.
    pub id: i64,
    /// Permission codename (e.g., "add_user", "change_post").
    pub codename: String,
    /// Human-readable name.
    pub name: String,
    /// Content type/model this permission applies to (optional).
    pub content_type: Option<String>,
}

impl Permission {
    /// Creates a new permission.
    pub fn new(codename: &str, name: &str) -> Self {
        Self {
            id: 0,
            codename: codename.to_string(),
            name: name.to_string(),
            content_type: None,
        }
    }

    /// Creates a new permission for a specific content type.
    pub fn for_model(codename: &str, name: &str, content_type: &str) -> Self {
        Self {
            id: 0,
            codename: codename.to_string(),
            name: name.to_string(),
            content_type: Some(content_type.to_string()),
        }
    }

    /// Saves the permission to the database.
    pub async fn save(&mut self, pool: &SqlitePool) -> Result<()> {
        if self.id == 0 {
            let result = sqlx::query(
                "INSERT INTO auth_permission (codename, name, content_type) VALUES (?, ?, ?)",
            )
            .bind(&self.codename)
            .bind(&self.name)
            .bind(&self.content_type)
            .execute(pool)
            .await?;

            self.id = result.last_insert_rowid();
        } else {
            sqlx::query(
                "UPDATE auth_permission SET codename = ?, name = ?, content_type = ? WHERE id = ?",
            )
            .bind(&self.codename)
            .bind(&self.name)
            .bind(&self.content_type)
            .bind(self.id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Deletes the permission from the database.
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM auth_permission WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Finds a permission by ID.
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Self> {
        let perm = sqlx::query_as::<_, Permission>("SELECT * FROM auth_permission WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::PermissionDenied)?;
        Ok(perm)
    }

    /// Finds a permission by codename.
    pub async fn get_by_codename(pool: &SqlitePool, codename: &str) -> Result<Self> {
        let perm =
            sqlx::query_as::<_, Permission>("SELECT * FROM auth_permission WHERE codename = ?")
                .bind(codename)
                .fetch_optional(pool)
                .await?
                .ok_or(AuthError::PermissionDenied)?;
        Ok(perm)
    }

    /// Returns all permissions.
    pub async fn all(pool: &SqlitePool) -> Result<Vec<Self>> {
        let perms = sqlx::query_as::<_, Permission>("SELECT * FROM auth_permission")
            .fetch_all(pool)
            .await?;
        Ok(perms)
    }
}

/// A group that can have permissions and users.
#[derive(Debug, Clone, FromRow)]
pub struct Group {
    /// Primary key.
    pub id: i64,
    /// Unique group name.
    pub name: String,
}

impl Group {
    /// Creates a new group.
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
        }
    }

    /// Saves the group to the database.
    pub async fn save(&mut self, pool: &SqlitePool) -> Result<()> {
        if self.id == 0 {
            let result = sqlx::query("INSERT INTO auth_group (name) VALUES (?)")
                .bind(&self.name)
                .execute(pool)
                .await?;

            self.id = result.last_insert_rowid();
        } else {
            sqlx::query("UPDATE auth_group SET name = ? WHERE id = ?")
                .bind(&self.name)
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes the group from the database.
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM auth_group WHERE id = ?")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Finds a group by ID.
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Self> {
        let group = sqlx::query_as::<_, Group>("SELECT * FROM auth_group WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::PermissionDenied)?;
        Ok(group)
    }

    /// Finds a group by name.
    pub async fn get_by_name(pool: &SqlitePool, name: &str) -> Result<Self> {
        let group = sqlx::query_as::<_, Group>("SELECT * FROM auth_group WHERE name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?
            .ok_or(AuthError::PermissionDenied)?;
        Ok(group)
    }

    /// Returns all groups.
    pub async fn all(pool: &SqlitePool) -> Result<Vec<Self>> {
        let groups = sqlx::query_as::<_, Group>("SELECT * FROM auth_group")
            .fetch_all(pool)
            .await?;
        Ok(groups)
    }

    /// Adds a permission to this group.
    pub async fn add_permission(&self, pool: &SqlitePool, permission_id: i64) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO auth_group_permissions (group_id, permission_id) VALUES (?, ?)",
        )
        .bind(self.id)
        .bind(permission_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Removes a permission from this group.
    pub async fn remove_permission(&self, pool: &SqlitePool, permission_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM auth_group_permissions WHERE group_id = ? AND permission_id = ?")
            .bind(self.id)
            .bind(permission_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Returns all permissions for this group.
    pub async fn get_permissions(&self, pool: &SqlitePool) -> Result<Vec<Permission>> {
        let perms = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM auth_permission p
            JOIN auth_group_permissions gp ON p.id = gp.permission_id
            WHERE gp.group_id = ?
            "#,
        )
        .bind(self.id)
        .fetch_all(pool)
        .await?;
        Ok(perms)
    }
}

/// Adds a user to a group.
pub async fn add_user_to_group(pool: &SqlitePool, user_id: i64, group_id: i64) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO auth_user_groups (user_id, group_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(group_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Removes a user from a group.
pub async fn remove_user_from_group(pool: &SqlitePool, user_id: i64, group_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM auth_user_groups WHERE user_id = ? AND group_id = ?")
        .bind(user_id)
        .bind(group_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Returns all groups for a user.
pub async fn get_user_groups(pool: &SqlitePool, user_id: i64) -> Result<Vec<Group>> {
    let groups = sqlx::query_as::<_, Group>(
        r#"
        SELECT g.* FROM auth_group g
        JOIN auth_user_groups ug ON g.id = ug.group_id
        WHERE ug.user_id = ?
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(groups)
}

/// Adds a permission directly to a user.
pub async fn add_user_permission(
    pool: &SqlitePool,
    user_id: i64,
    permission_id: i64,
) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO auth_user_permissions (user_id, permission_id) VALUES (?, ?)",
    )
    .bind(user_id)
    .bind(permission_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Removes a permission from a user.
pub async fn remove_user_permission(
    pool: &SqlitePool,
    user_id: i64,
    permission_id: i64,
) -> Result<()> {
    sqlx::query("DELETE FROM auth_user_permissions WHERE user_id = ? AND permission_id = ?")
        .bind(user_id)
        .bind(permission_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Returns all permissions for a user (direct and through groups).
pub async fn get_user_permissions(pool: &SqlitePool, user_id: i64) -> Result<Vec<Permission>> {
    let perms = sqlx::query_as::<_, Permission>(
        r#"
        SELECT DISTINCT p.* FROM auth_permission p
        LEFT JOIN auth_user_permissions up ON p.id = up.permission_id AND up.user_id = ?
        LEFT JOIN auth_group_permissions gp ON p.id = gp.permission_id
        LEFT JOIN auth_user_groups ug ON gp.group_id = ug.group_id AND ug.user_id = ?
        WHERE up.user_id IS NOT NULL OR ug.user_id IS NOT NULL
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(perms)
}

/// Checks if a user has a specific permission.
pub async fn user_has_permission(pool: &SqlitePool, user_id: i64, codename: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM auth_permission p
        LEFT JOIN auth_user_permissions up ON p.id = up.permission_id AND up.user_id = ?
        LEFT JOIN auth_group_permissions gp ON p.id = gp.permission_id
        LEFT JOIN auth_user_groups ug ON gp.group_id = ug.group_id AND ug.user_id = ?
        WHERE p.codename = ? AND (up.user_id IS NOT NULL OR ug.user_id IS NOT NULL)
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind(codename)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// SQL to create the permission tables.
pub const CREATE_PERMISSION_TABLES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS auth_permission (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    codename VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    content_type VARCHAR(100)
);

CREATE TABLE IF NOT EXISTS auth_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(150) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS auth_group_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES auth_group(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES auth_permission(id) ON DELETE CASCADE,
    UNIQUE(group_id, permission_id)
);

CREATE TABLE IF NOT EXISTS auth_user_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES auth_user(id) ON DELETE CASCADE,
    group_id INTEGER NOT NULL REFERENCES auth_group(id) ON DELETE CASCADE,
    UNIQUE(user_id, group_id)
);

CREATE TABLE IF NOT EXISTS auth_user_permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES auth_user(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES auth_permission(id) ON DELETE CASCADE,
    UNIQUE(user_id, permission_id)
);
"#;

/// Creates the permission tables if they don't exist.
pub async fn create_permission_tables(pool: &SqlitePool) -> Result<()> {
    // SQLite doesn't support multiple statements in one query, so split them
    for statement in CREATE_PERMISSION_TABLES_SQL.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("add_user", "Can add user");
        assert_eq!(perm.codename, "add_user");
        assert_eq!(perm.name, "Can add user");
        assert!(perm.content_type.is_none());
    }

    #[test]
    fn test_permission_for_model() {
        let perm = Permission::for_model("add_post", "Can add post", "blog.post");
        assert_eq!(perm.codename, "add_post");
        assert_eq!(perm.content_type, Some("blog.post".to_string()));
    }

    #[test]
    fn test_group_creation() {
        let group = Group::new("Editors");
        assert_eq!(group.name, "Editors");
    }
}
