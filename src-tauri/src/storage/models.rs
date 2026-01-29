use serde::Serialize;

#[derive(Debug, Clone)]
pub struct CreateSessionInput {
    pub id: String,
    pub start_ts_ms: i64,
    pub planned_duration_sec: i64,
    pub capture_enabled: bool,
    pub mode: String,
    pub task_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: String,
    pub start_ts_ms: i64,
    pub end_ts_ms: Option<i64>,
    pub planned_duration_sec: i64,
    pub ended_reason: Option<String>,
    pub capture_enabled: bool,
    pub mode: String,
    pub task_title: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
