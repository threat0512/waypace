use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tauri::async_runtime::JoinHandle;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

const DEFAULT_DURATION_SEC: u64 = 45 * 60;

// st
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Idle,
    Running,
}

#[derive(Debug, Clone)]
struct Session {
    id: Uuid,
    start_ts_ms: i64,
    planned_duration_sec: u64,
}

#[derive(Debug)]
pub struct SessionManager {
    status: Status,
    current: Option<Session>,
    tick_task: Option<JoinHandle<()>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateDto {
    pub status: String,
    pub session_id: Option<String>,
    pub planned_duration_sec: u64,
    pub start_ts_ms: Option<i64>,
    pub remaining_sec: u64,
}

impl SessionStateDto {
    pub fn idle() -> Self {
        Self {
            status: "idle".into(),
            session_id: None,
            planned_duration_sec: DEFAULT_DURATION_SEC,
            start_ts_ms: None,
            remaining_sec: 0,
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            status: Status::Idle,
            current: None,
            tick_task: None,
        }
    }

    pub fn get_state_dto(&self, now_ms: i64) -> SessionStateDto {
        match (&self.status, &self.current) {
            (Status::Running, Some(session)) => {
                let elapsed_ms = (now_ms - session.start_ts_ms).max(0) as u64;
                let elapsed_sec = elapsed_ms / 1000;
                let remaining = session.planned_duration_sec.saturating_sub(elapsed_sec);
                SessionStateDto {
                    status: "running".into(),
                    session_id: Some(session.id.to_string()),
                    planned_duration_sec: session.planned_duration_sec,
                    start_ts_ms: Some(session.start_ts_ms),
                    remaining_sec: remaining,
                }
            }
            _ => SessionStateDto::idle(),
        }
    }

    fn cancel_tick_task(&mut self) {
        if let Some(handle) = self.tick_task.take() {
            handle.abort();
        }
    }

    pub fn stop(&mut self, app: &AppHandle) -> SessionStateDto {
        self.cancel_tick_task();
        self.status = Status::Idle;
        self.current = None;

        let dto = self.get_state_dto(now_ms());
        let _ = app.emit("waypace://session_state", dto.clone());
        dto
    }

    pub fn start(
        &mut self,
        app: &AppHandle,
        planned_duration_sec: Option<u64>,
    ) -> Result<SessionStateDto, String> {
        if self.status == Status::Running {
            return Ok(self.get_state_dto(now_ms()));
        }

        self.cancel_tick_task();

        let planned_duration_sec = planned_duration_sec.unwrap_or(DEFAULT_DURATION_SEC);
        let session = Session {
            id: Uuid::new_v4(),
            start_ts_ms: now_ms(),
            planned_duration_sec,
        };

        self.status = Status::Running;
        self.current = Some(session.clone());

        let dto = self.get_state_dto(now_ms());
        let _ = app.emit("waypace://session_state", dto.clone());

        let app_handle = app.clone();
        let manager_ref = app
            .state::<Arc<Mutex<SessionManager>>>()
            .inner()
            .clone();
        let session_id = session.id.to_string();

        let task = tauri::async_runtime::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;

                let mut manager = match manager_ref.lock() {
                    Ok(manager) => manager,
                    Err(_) => return,
                };

                let dto = manager.get_state_dto(now_ms());
                if dto.status != "running" || dto.session_id.as_deref() != Some(&session_id) {
                    return;
                }

                let _ = app_handle.emit(
                    "waypace://session_tick",
                    serde_json::json!({
                        "sessionId": session_id,
                        "remainingSec": dto.remaining_sec,
                    }),
                );

                if dto.remaining_sec == 0 {
                    manager.cancel_tick_task();
                    manager.status = Status::Idle;
                    manager.current = None;

                    let end_dto = manager.get_state_dto(now_ms());
                    let _ = app_handle.emit("waypace://session_state", end_dto);
                    return;
                }
            }
        });

        self.tick_task = Some(task);
        Ok(dto)
    }
}

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis() as i64
}
