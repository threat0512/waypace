mod session;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};

use session::{SessionManager, SessionStateDto};

#[tauri::command]
fn get_state(state: tauri::State<Arc<Mutex<SessionManager>>>) -> SessionStateDto {
    match state.lock() {
        Ok(manager) => manager.get_state_dto(session::now_ms()),
        Err(_) => SessionStateDto::idle(),
    }
}

#[tauri::command]
fn start_session(
    app: tauri::AppHandle,
    state: tauri::State<Arc<Mutex<SessionManager>>>,
    planned_duration_sec: Option<u64>,
) -> Result<SessionStateDto, String> {
    let mut manager = state
        .lock()
        .map_err(|_| "session lock poisoned".to_string())?;
    manager.start(&app, planned_duration_sec)
}

#[tauri::command]
fn stop_session(
    app: tauri::AppHandle,
    state: tauri::State<Arc<Mutex<SessionManager>>>,
) -> SessionStateDto {
    match state.lock() {
        Ok(mut manager) => manager.stop(&app),
        Err(_) => SessionStateDto::idle(),
    }
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
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(SessionManager::new())))
        .invoke_handler(tauri::generate_handler![get_state, start_session, stop_session])
        .setup(|app| {
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
