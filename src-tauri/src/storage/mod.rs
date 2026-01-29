pub mod error;
pub mod models;

use error::{StorageError, StorageResult};
use models::{CreateSessionInput, SessionRow};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row,
    SqlitePool,
};
use std::{path::PathBuf, str::FromStr};
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn init_pool(app: &AppHandle) -> StorageResult<SqlitePool> {
    let db_path = resolve_db_path(app)?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?;

    opts = opts
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    MIGRATOR.run(&pool).await?;

    info!("DB initialized at {}", db_path.display());
    Ok(pool)
}

fn resolve_db_path(app: &AppHandle) -> StorageResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| StorageError::NoAppDataDir)?;
    Ok(dir.join("waypace.db"))
}

pub async fn create_session(pool: &SqlitePool, input: CreateSessionInput) -> StorageResult<()> {
    let now = now_ms();
    sqlx::query(
        r#"
        INSERT INTO sessions
          (id, start_ts_ms, planned_duration_sec, capture_enabled, mode, task_title, created_at_ms, updated_at_ms)
        VALUES
          (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(input.id)
    .bind(input.start_ts_ms)
    .bind(input.planned_duration_sec)
    .bind(bool_to_i64(input.capture_enabled))
    .bind(input.mode)
    .bind(input.task_title)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn end_session(
    pool: &SqlitePool,
    id: &str,
    end_ts_ms: i64,
    reason: &str,
) -> StorageResult<()> {
    let now = now_ms();
    let res = sqlx::query(
        r#"
        UPDATE sessions
        SET end_ts_ms = ?, ended_reason = ?, updated_at_ms = ?
        WHERE id = ?
        "#,
    )
    .bind(end_ts_ms)
    .bind(reason)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        warn!("end_session: no session row found for id={}", id);
    }

    Ok(())
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> StorageResult<Option<SessionRow>> {
    let row = sqlx::query(
        r#"
        SELECT id, start_ts_ms, end_ts_ms, planned_duration_sec, ended_reason, capture_enabled, mode, task_title, created_at_ms, updated_at_ms
        FROM sessions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(SessionRow {
        id: row.try_get("id")?,
        start_ts_ms: row.try_get("start_ts_ms")?,
        end_ts_ms: row.try_get("end_ts_ms")?,
        planned_duration_sec: row.try_get("planned_duration_sec")?,
        ended_reason: row.try_get("ended_reason")?,
        capture_enabled: row.try_get::<i64, _>("capture_enabled")? != 0,
        mode: row.try_get("mode")?,
        task_title: row.try_get("task_title")?,
        created_at_ms: row.try_get("created_at_ms")?,
        updated_at_ms: row.try_get("updated_at_ms")?,
    }))
}

fn bool_to_i64(v: bool) -> i64 {
    if v { 1 } else { 0 }
}

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis() as i64
}
