mod session;
mod storage;

use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tracing_subscriber::EnvFilter;

use session::{SessionManager, SessionStateDto};

struct AppState {
    session: Arc<Mutex<SessionManager>>,
    pool: Option<SqlitePool>,
}

#[tauri::command]
fn get_state(state: tauri::State<'_, AppState>) -> SessionStateDto {
    match state.session.lock() {
        Ok(manager) => manager.get_state_dto(storage::now_ms()),
        Err(_) => SessionStateDto::idle(),
    }
}

#[tauri::command]
async fn start_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    planned_duration_sec: Option<u64>,
) -> Result<SessionStateDto, String> {
    let manager_ref = state.session.clone();
    let pool = state.pool.clone();
    let dto = {
        let mut manager = state
            .session
            .lock()
            .map_err(|_| "session lock poisoned".to_string())?;
        manager.start(&app, planned_duration_sec, manager_ref, pool.clone())?
    };

    if dto.status == "running" {
        if let (Some(pool), Some(id), Some(start_ts_ms)) =
            (state.pool.as_ref(), dto.session_id.clone(), dto.start_ts_ms)
        {
            let input = storage::models::CreateSessionInput {
                id,
                start_ts_ms,
                planned_duration_sec: dto.planned_duration_sec as i64,
                capture_enabled: false,
                mode: "leetcode".to_string(),
                task_title: None,
            };

            if let Err(error) = storage::create_session(pool, input).await {
                tracing::warn!("DB create_session failed: {error}");
            }
        }
    }

    Ok(dto)
}

#[tauri::command]
async fn stop_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SessionStateDto, String> {
    let end_ts_ms = storage::now_ms();
    let (ended_id, dto) = match state.session.lock() {
        Ok(mut manager) => {
            let ended_id = manager.get_state_dto(end_ts_ms).session_id.clone();
            let dto = manager.stop(&app);
            (ended_id, dto)
        }
        Err(_) => return Ok(SessionStateDto::idle()),
    };

    if let (Some(pool), Some(id)) = (state.pool.as_ref(), ended_id) {
        if let Err(error) = storage::end_session(pool, &id, end_ts_ms, "manual").await {
            tracing::warn!("DB end_session failed: {error}");
        }
    }

    Ok(dto)
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_state, start_session, stop_session, db_smoke]);
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_state, start_session, stop_session]);

    builder
        .setup(|app| {
            let session_state = Arc::new(Mutex::new(SessionManager::new()));
            let handle = app.handle();
            let pool = tauri::async_runtime::block_on(async { storage::init_pool(&handle).await });
            let pool = match pool {
                Ok(pool) => Some(pool),
                Err(error) => {
                    tracing::warn!(
                        "DB init failed (app will run without persistence): {error}"
                    );
                    None
                }
            };

            app.manage(AppState {
                session: session_state,
                pool,
            });

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: 360.0,
                    height: 420.0,
                }));
                let _ = window.hide();
            }

            let icon = app
                .default_window_icon()
                .cloned()
                .expect("failed to load default window icon");

            TrayIconBuilder::new()
                .icon(icon)
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
                    if let TrayIconEvent::Click { button_state, .. } = event {
                        if button_state == MouseButtonState::Up {
                            toggle_main_window(tray.app_handle());
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(debug_assertions)]
#[tauri::command]
async fn db_smoke(pool_state: tauri::State<'_, AppState>) -> Result<(), String> {
    let Some(pool) = pool_state.pool.as_ref() else {
        return Err("no DB pool available".to_string());
    };

    let id = uuid::Uuid::new_v4().to_string();
    let start = storage::now_ms();
    let input = storage::models::CreateSessionInput {
        id: id.clone(),
        start_ts_ms: start,
        planned_duration_sec: 10,
        capture_enabled: false,
        mode: "leetcode".to_string(),
        task_title: Some("smoke".to_string()),
    };

    storage::create_session(pool, input)
        .await
        .map_err(|error| error.to_string())?;
    storage::end_session(pool, &id, storage::now_ms(), "manual")
        .await
        .map_err(|error| error.to_string())?;
    let row = storage::get_session(pool, &id)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!("db_smoke row={row:?}");
    Ok(())
}
