//! oxide-migrate CLI
//!
//! Command-line tool for managing database migrations.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePoolOptions;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use oxide_migrate::dialect::SqliteDialect;
use oxide_migrate::executor::MigrationExecutor;
use oxide_migrate::prelude::*;

/// Django-like database migrations for Rust.
#[derive(Parser)]
#[command(name = "oxide-migrate")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Database URL (SQLite path or connection string).
    #[arg(short, long, env = "DATABASE_URL", default_value = "sqlite:db.sqlite3")]
    database: String,

    /// Migrations directory.
    #[arg(short, long, default_value = "migrations")]
    migrations_dir: PathBuf,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply pending migrations.
    Migrate {
        /// App name to migrate (all if not specified).
        #[arg(short, long)]
        app: Option<String>,

        /// Number of migrations to apply (all if not specified).
        #[arg(short, long)]
        count: Option<usize>,

        /// Rollback migrations instead of applying.
        #[arg(short, long)]
        reverse: bool,

        /// Show SQL without executing (dry run).
        #[arg(long)]
        dry_run: bool,
    },

    /// Show migration status.
    ShowMigrations {
        /// App name to show (all if not specified).
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Generate new migrations from schema changes.
    MakeMigrations {
        /// App name to generate migrations for.
        #[arg(short, long)]
        app: Option<String>,

        /// Migration name/description.
        #[arg(short, long)]
        name: Option<String>,

        /// Create an empty migration file.
        #[arg(long)]
        empty: bool,

        /// Show SQL without writing files (dry run).
        #[arg(long)]
        dry_run: bool,
    },

    /// Show SQL for migrations without executing.
    SqlMigrate {
        /// App name.
        #[arg(short, long)]
        app: Option<String>,

        /// Migration name.
        #[arg(short, long)]
        migration: Option<String>,

        /// Show rollback SQL instead of forward SQL.
        #[arg(short, long)]
        reverse: bool,
    },

    /// Initialize the migrations system (create history table).
    Init,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Connect to database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&cli.database)
        .await?;

    let executor = MigrationExecutor::new(pool, SqliteDialect::new());

    match cli.command {
        Commands::Init => {
            info!("Initializing migrations system...");
            executor.init().await?;
            info!("Migrations table created successfully.");
        }

        Commands::Migrate {
            app,
            count,
            reverse,
            dry_run,
        } => {
            executor.init().await?;

            if dry_run {
                info!("Dry run mode - SQL will be printed but not executed.");
            }

            // In a real implementation, we would:
            // 1. Discover migration files from the migrations directory
            // 2. Build ExecutableMigration instances from them
            // 3. Apply/rollback based on the flags

            info!(
                "Migration command received: app={:?}, count={:?}, reverse={}, dry_run={}",
                app, count, reverse, dry_run
            );

            // Placeholder - in practice, you'd load migrations from files
            info!("No migrations discovered. Use `makemigrations` first.");
        }

        Commands::ShowMigrations { app } => {
            executor.init().await?;

            let applied = executor.history().get_applied().await?;

            if applied.is_empty() {
                info!("No migrations have been applied yet.");
            } else {
                println!("\nApplied migrations:");
                println!("{:-<60}", "");

                for migration in &applied {
                    if let Some(target_app) = &app
                        && &migration.app != target_app
                    {
                        continue;
                    }
                    println!(
                        " [X] {}/{} ({})",
                        migration.app,
                        migration.name,
                        migration.applied_at.format("%Y-%m-%d %H:%M:%S")
                    );
                }
                println!();
            }
        }

        Commands::MakeMigrations {
            app,
            name,
            empty,
            dry_run,
        } => {
            let app_name = app.unwrap_or_else(|| "app".to_string());
            let migration_name = name.unwrap_or_else(|| "auto".to_string());

            if empty {
                // Generate an empty migration file
                let number = 1; // In practice, scan existing files
                let full_name = generate_migration_name(number, &migration_name);

                let writer = MigrationWriter::new(&app_name, &full_name);
                let code = writer.generate();

                if dry_run {
                    println!("Would create migration: {}/{}", app_name, full_name);
                    println!("\n{}", code);
                } else {
                    let file_path = cli.migrations_dir.join(format!("{}.rs", full_name));
                    std::fs::create_dir_all(&cli.migrations_dir)?;
                    std::fs::write(&file_path, code)?;
                    info!("Created migration: {}", file_path.display());
                }
            } else {
                // In a real implementation:
                // 1. Load model schemas from the inventory/linkme registry
                // 2. Reconstruct current schema from existing migrations
                // 3. Diff the schemas using Autodetector
                // 4. Generate migration file with the detected operations

                info!("Auto-detection requires model schemas to be registered.");
                info!("Use --empty to create a blank migration file.");
            }
        }

        Commands::SqlMigrate {
            app,
            migration,
            reverse,
        } => {
            // In a real implementation:
            // 1. Load the specified migration
            // 2. Generate SQL using the dialect
            // 3. Print the SQL

            info!(
                "SQL migrate command: app={:?}, migration={:?}, reverse={}",
                app, migration, reverse
            );
            info!("No migrations discovered to show SQL for.");
        }
    }

    Ok(())
}
