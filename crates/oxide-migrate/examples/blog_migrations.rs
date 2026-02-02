//! Example: Blog Application Migrations
//!
//! This example demonstrates how to use oxide-migrate to manage database
//! schema changes for a blog application with users, posts, and comments.
//!
//! Run with: cargo run --example blog_migrations -p oxide-migrate

use oxide_migrate::OxideMigration;
use oxide_migrate::prelude::*;

// =============================================================================
// Migration Definitions
// =============================================================================

/// Initial migration: Create users table
struct Migration0001;

impl OxideMigration for Migration0001 {
    const APP: &'static str = "blog";
    const NAME: &'static str = "0001_create_users";
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[];

    fn operations() -> Vec<MigrationOperation> {
        vec![MigrationOperation::CreateTable {
            name: "users".to_string(),
            columns: vec![
                ColumnSchema::new("id", SqlType::BigInt)
                    .primary_key()
                    .auto_increment(),
                ColumnSchema::new("username", SqlType::Varchar(100))
                    .not_null()
                    .unique(),
                ColumnSchema::new("email", SqlType::Varchar(255)).not_null(),
                ColumnSchema::new("password_hash", SqlType::Varchar(255)).not_null(),
                ColumnSchema::new("is_active", SqlType::Boolean)
                    .not_null()
                    .default(DefaultValue::Bool(true)),
                ColumnSchema::new("created_at", SqlType::Timestamp)
                    .not_null()
                    .default(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string())),
            ],
            primary_key: vec!["id".to_string()],
            if_not_exists: false,
        }]
    }
}

/// Second migration: Create posts table
struct Migration0002;

impl OxideMigration for Migration0002 {
    const APP: &'static str = "blog";
    const NAME: &'static str = "0002_create_posts";
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[("blog", "0001_create_users")];

    fn operations() -> Vec<MigrationOperation> {
        vec![
            MigrationOperation::CreateTable {
                name: "posts".to_string(),
                columns: vec![
                    ColumnSchema::new("id", SqlType::BigInt)
                        .primary_key()
                        .auto_increment(),
                    ColumnSchema::new("author_id", SqlType::BigInt).not_null(),
                    ColumnSchema::new("title", SqlType::Varchar(200)).not_null(),
                    ColumnSchema::new("slug", SqlType::Varchar(200))
                        .not_null()
                        .unique(),
                    ColumnSchema::new("content", SqlType::Text).not_null(),
                    ColumnSchema::new("is_published", SqlType::Boolean)
                        .not_null()
                        .default(DefaultValue::Bool(false)),
                    ColumnSchema::new("published_at", SqlType::Timestamp),
                    ColumnSchema::new("created_at", SqlType::Timestamp)
                        .not_null()
                        .default(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string())),
                    ColumnSchema::new("updated_at", SqlType::Timestamp)
                        .not_null()
                        .default(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string())),
                ],
                primary_key: vec!["id".to_string()],
                if_not_exists: false,
            },
            // Create index on author_id for faster lookups
            MigrationOperation::CreateIndex {
                name: "idx_posts_author".to_string(),
                table: "posts".to_string(),
                columns: vec!["author_id".to_string()],
                unique: false,
                condition: None,
                if_not_exists: false,
            },
            // Partial index for published posts
            MigrationOperation::CreateIndex {
                name: "idx_posts_published".to_string(),
                table: "posts".to_string(),
                columns: vec!["published_at".to_string()],
                unique: false,
                condition: Some("is_published = 1".to_string()),
                if_not_exists: false,
            },
        ]
    }
}

/// Third migration: Create comments table
struct Migration0003;

impl OxideMigration for Migration0003 {
    const APP: &'static str = "blog";
    const NAME: &'static str = "0003_create_comments";
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[("blog", "0002_create_posts")];

    fn operations() -> Vec<MigrationOperation> {
        vec![
            MigrationOperation::CreateTable {
                name: "comments".to_string(),
                columns: vec![
                    ColumnSchema::new("id", SqlType::BigInt)
                        .primary_key()
                        .auto_increment(),
                    ColumnSchema::new("post_id", SqlType::BigInt).not_null(),
                    ColumnSchema::new("author_id", SqlType::BigInt).not_null(),
                    ColumnSchema::new("parent_id", SqlType::BigInt), // For nested comments
                    ColumnSchema::new("content", SqlType::Text).not_null(),
                    ColumnSchema::new("is_approved", SqlType::Boolean)
                        .not_null()
                        .default(DefaultValue::Bool(false)),
                    ColumnSchema::new("created_at", SqlType::Timestamp)
                        .not_null()
                        .default(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string())),
                ],
                primary_key: vec!["id".to_string()],
                if_not_exists: false,
            },
            MigrationOperation::CreateIndex {
                name: "idx_comments_post".to_string(),
                table: "comments".to_string(),
                columns: vec!["post_id".to_string()],
                unique: false,
                condition: None,
                if_not_exists: false,
            },
            MigrationOperation::CreateIndex {
                name: "idx_comments_author".to_string(),
                table: "comments".to_string(),
                columns: vec!["author_id".to_string()],
                unique: false,
                condition: None,
                if_not_exists: false,
            },
        ]
    }
}

/// Fourth migration: Add user profile fields
struct Migration0004;

impl OxideMigration for Migration0004 {
    const APP: &'static str = "blog";
    const NAME: &'static str = "0004_add_user_profile";
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[("blog", "0001_create_users")];

    fn operations() -> Vec<MigrationOperation> {
        vec![
            MigrationOperation::AddColumn {
                table: "users".to_string(),
                column: ColumnSchema::new("display_name", SqlType::Varchar(100)),
            },
            MigrationOperation::AddColumn {
                table: "users".to_string(),
                column: ColumnSchema::new("bio", SqlType::Text),
            },
            MigrationOperation::AddColumn {
                table: "users".to_string(),
                column: ColumnSchema::new("avatar_url", SqlType::Varchar(500)),
            },
        ]
    }
}

/// Fifth migration: Add tags system
struct Migration0005;

impl OxideMigration for Migration0005 {
    const APP: &'static str = "blog";
    const NAME: &'static str = "0005_add_tags";
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[("blog", "0002_create_posts")];

    fn operations() -> Vec<MigrationOperation> {
        vec![
            // Tags table
            MigrationOperation::CreateTable {
                name: "tags".to_string(),
                columns: vec![
                    ColumnSchema::new("id", SqlType::BigInt)
                        .primary_key()
                        .auto_increment(),
                    ColumnSchema::new("name", SqlType::Varchar(50))
                        .not_null()
                        .unique(),
                    ColumnSchema::new("slug", SqlType::Varchar(50))
                        .not_null()
                        .unique(),
                ],
                primary_key: vec!["id".to_string()],
                if_not_exists: false,
            },
            // Many-to-many junction table
            MigrationOperation::CreateTable {
                name: "post_tags".to_string(),
                columns: vec![
                    ColumnSchema::new("post_id", SqlType::BigInt).not_null(),
                    ColumnSchema::new("tag_id", SqlType::BigInt).not_null(),
                ],
                primary_key: vec!["post_id".to_string(), "tag_id".to_string()],
                if_not_exists: false,
            },
            MigrationOperation::CreateIndex {
                name: "idx_post_tags_tag".to_string(),
                table: "post_tags".to_string(),
                columns: vec!["tag_id".to_string()],
                unique: false,
                condition: None,
                if_not_exists: false,
            },
        ]
    }
}

// =============================================================================
// Main: Demonstrate the Migration System
// =============================================================================

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!(" OXIDE-MIGRATE: Blog Application Example");
    println!("{}", "=".repeat(70));
    println!();

    // Create in-memory SQLite database
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await?;

    // Create executor with SQLite dialect
    let executor = MigrationExecutor::new(pool, SqliteDialect::new());

    // Initialize (create migrations history table)
    println!("[1] Initializing migration system...");
    executor.init().await?;
    println!("    Created oxide_migrations table\n");

    // Convert our migrations to executable format
    let migrations = vec![
        Migration0001::to_executable(),
        Migration0002::to_executable(),
        Migration0003::to_executable(),
        Migration0004::to_executable(),
        Migration0005::to_executable(),
    ];

    // Show pending migrations
    println!("[2] Checking pending migrations...");
    let pending = executor.pending(&migrations).await?;
    println!("    {} migrations pending:\n", pending.len());
    for m in &pending {
        println!("    - {}/{}", m.app, m.name);
    }
    println!();

    // Show SQL for each migration (dry run)
    println!("[3] Generated SQL for migrations:");
    println!("{}", "-".repeat(70));
    for migration in &migrations {
        println!("\n-- Migration: {}/{}", migration.app, migration.name);
        let sql_statements = executor.sql_for(migration);
        for sql in sql_statements {
            println!("{};", sql);
        }
    }
    println!();
    println!("{}", "-".repeat(70));
    println!();

    // Apply all migrations
    println!("[4] Applying migrations...\n");
    for migration in &migrations {
        print!("    Applying {}/{}...", migration.app, migration.name);
        executor.apply(migration).await?;
        println!(" OK");
    }
    println!();

    // Verify all applied
    println!("[5] Verifying applied migrations...\n");
    let applied = executor.history().get_applied().await?;
    println!("    {} migrations applied:", applied.len());
    for m in &applied {
        println!(
            "    [X] {}/{} ({})",
            m.app,
            m.name,
            m.applied_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
    println!();

    // Demonstrate autodetector
    println!("[6] Demonstrating schema diff (autodetector)...\n");

    // Simulate current schema vs desired schema
    let current = DatabaseSchema::new().table(
        TableSchema::new("users")
            .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
            .column(ColumnSchema::new("name", SqlType::Text)),
    );

    let desired = DatabaseSchema::new().table(
        TableSchema::new("users")
            .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
            .column(ColumnSchema::new("name", SqlType::Varchar(100)).not_null())
            .column(ColumnSchema::new("email", SqlType::Varchar(255))),
    );

    let autodetector = Autodetector::new();
    let diff_ops = autodetector.diff(&current, &desired);

    println!("    Schema changes detected:");
    for op in &diff_ops {
        println!("    - {}", op.description());
    }
    println!();

    // Demonstrate rollback
    println!("[7] Demonstrating rollback...\n");
    let last_migration = &migrations[4]; // 0005_add_tags
    print!(
        "    Rolling back {}/{}...",
        last_migration.app, last_migration.name
    );
    executor.rollback(last_migration).await?;
    println!(" OK\n");

    // Show final state
    println!("[8] Final migration state:\n");
    let final_applied = executor.history().get_applied().await?;
    println!("    {} migrations applied:", final_applied.len());
    for m in &final_applied {
        println!("    [X] {}/{}", m.app, m.name);
    }
    println!();

    // Re-apply the rolled back migration
    print!("    Re-applying {}...", last_migration.name);
    executor.apply(last_migration).await?;
    println!(" OK\n");

    // Demonstrate migration writer
    println!("[9] Generating migration file (writer)...\n");
    let writer = MigrationWriter::new("blog", "0006_add_likes")
        .depends_on("blog", "0002_create_posts")
        .operation(MigrationOperation::CreateTable {
            name: "likes".to_string(),
            columns: vec![
                ColumnSchema::new("id", SqlType::BigInt)
                    .primary_key()
                    .auto_increment(),
                ColumnSchema::new("user_id", SqlType::BigInt).not_null(),
                ColumnSchema::new("post_id", SqlType::BigInt).not_null(),
                ColumnSchema::new("created_at", SqlType::Timestamp)
                    .not_null()
                    .default(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string())),
            ],
            primary_key: vec!["id".to_string()],
            if_not_exists: false,
        })
        .operation(MigrationOperation::add_unique_constraint(
            "likes",
            "uq_likes_user_post",
            vec!["user_id".to_string(), "post_id".to_string()],
        ));

    println!("    Generated migration file content:");
    println!("{}", "-".repeat(70));
    println!("{}", writer.generate());
    println!("{}", "-".repeat(70));

    println!();
    println!("{}", "=".repeat(70));
    println!(" Example completed successfully!");
    println!("{}", "=".repeat(70));

    Ok(())
}
