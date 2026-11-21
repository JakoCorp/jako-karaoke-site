pub(crate) mod scheduled_backup;
pub(crate) mod session_cleanup;

use crate::config::Config;

/// Spawns all background tasks. Intended to be called once at startup.
pub fn spawn_background_tasks(pool: db::MySqlPool, config: &Config) {
    tokio::spawn(session_cleanup::run(pool.clone()));
    if let Some(schedule) = config.backup_schedule.clone() {
        tokio::spawn(scheduled_backup::run(config.clone(), pool, schedule));
    }
}
