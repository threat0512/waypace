use tauri::{AppHandle, Manager};
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};

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
