//! Background task that takes backups on a schedule and deletes expired ones.

use chrono::Utc;
use db::MySqlPool;
use tokio::time;

use crate::backups::{self, BackupKind};
use crate::config::{BackupSchedule, Config};

/// Takes a backup at every occurrence of the schedule, then deletes expired scheduled backups.
///
/// Expired backups are only deleted after a successful backup. Intended to be spawned with
/// [`tokio::spawn`] at startup, runs until the process exits.
pub async fn run(config: Config, pool: MySqlPool, schedule: BackupSchedule) {
    loop {
        let now = Utc::now();
        let next = match schedule.cron.find_next_occurrence(&now, false) {
            Ok(next) => next,
            Err(error) => {
                tracing::error!(error = %error, "backup schedule has no next run");
                return;
            }
        };
        tracing::info!(next = %next, "next scheduled backup");
        time::sleep((next - now).to_std().unwrap_or_default()).await;

        match backups::create(&config, &pool, BackupKind::Scheduled).await {
            Ok(path) => tracing::info!(path = %path.display(), "scheduled backup created"),
            Err(error) => {
                tracing::error!(error = %error, "scheduled backup failed");
                continue;
            }
        }
        let Some(retention) = schedule.retention else {
            continue;
        };
        match backups::prune(&config, Utc::now() - retention).await {
            Ok(deleted) => {
                for path in deleted {
                    tracing::info!(path = %path.display(), "expired backup deleted");
                }
            }
            Err(error) => tracing::error!(error = %error, "backup pruning failed"),
        }
    }
}
