//! Creates, lists and restores whole server backups.
//!
//! Reads the same environment as the server. Restore replaces the database and every stored
//! file, so stop the server first.
//!
//! ```sh
//! backup create
//! backup list
//! backup restore <file|latest> --yes
//! ```

use std::fmt::Display;
use std::process;

use clap::{Parser, Subcommand};
use db::MySqlPool;
use server::backups::{self, BackupKind, RestoreOptions};
use server::config::Config;

#[derive(Parser)]
#[command(about = "Create, list and restore server backups")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Writes a new backup to the backup directory.
    Create,
    /// Lists the backups in the backup directory, oldest first.
    List,
    /// Replaces the database and stored files with a backup. Stop the server first.
    Restore {
        /// Backup file name, path, or `latest`.
        backup: String,
        /// Confirms that the current data is overwritten.
        #[arg(long)]
        yes: bool,
        /// Skips the backup of the current state taken before restoring.
        #[arg(long)]
        no_safety_backup: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config =
        Config::from_env().unwrap_or_else(|error| fail(format!("invalid configuration: {error}")));

    match cli.command {
        Command::Create => {
            let pool = connect(&config).await;
            let path = backups::create(&config, &pool, BackupKind::Manual)
                .await
                .unwrap_or_else(|error| fail(error));
            println!("created: {}", path.display());
        }
        Command::List => list(&config).await,
        Command::Restore {
            backup,
            yes,
            no_safety_backup,
        } => restore(&config, &backup, yes, !no_safety_backup).await,
    }
}

async fn list(config: &Config) {
    let entries = backups::list(config)
        .await
        .unwrap_or_else(|error| fail(error));
    if entries.is_empty() {
        println!("no backups in {}", config.backup_path);
        return;
    }

    for entry in entries {
        let name = entry
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let size = format!("{:.1} MB", entry.size as f64 / 1_000_000.0);
        match entry.manifest {
            Some(manifest) => println!(
                "{name}  {}  commit {}  {} files  {size}",
                manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                manifest.commit,
                manifest.file_count
            ),
            None => println!("{name}  unreadable manifest  {size}"),
        }
    }
}

async fn restore(config: &Config, backup: &str, confirmed: bool, safety_backup: bool) {
    let path = backups::resolve(config, backup)
        .await
        .unwrap_or_else(|error| fail(error));
    let manifest = backups::inspect(&path)
        .await
        .unwrap_or_else(|error| fail(error));

    println!("backup: {}", path.display());
    println!(
        "taken {} by commit {}, {} files",
        manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        manifest.commit,
        manifest.file_count
    );
    if manifest.commit != config.commit {
        println!("warning: this build is commit {}", config.commit);
    }
    if !confirmed {
        fail("restore replaces the database and every stored file, re-run with --yes to continue");
    }

    let pool = connect(config).await;
    let report = backups::restore(config, &pool, &path, &RestoreOptions { safety_backup })
        .await
        .unwrap_or_else(|error| fail(error));

    if let Some(safety_backup) = report.safety_backup {
        println!("safety backup: {}", safety_backup.display());
    }
    if report.rebased {
        println!("assets moved to {}", config.storage_base_url);
    }
    for missing in &report.missing_files {
        println!("warning: missing file {missing}");
    }
    println!("done: start the server to apply any newer migrations");
}

async fn connect(config: &Config) -> MySqlPool {
    db::connect_existing(&config.database_url)
        .await
        .unwrap_or_else(|error| fail(format!("failed to connect to database: {error}")))
}

fn fail(message: impl Display) -> ! {
    eprintln!("error: {message}");
    process::exit(1);
}
