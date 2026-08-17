mod app_state;
mod call_detection;
mod commands;
mod error;
mod models;
mod notify;
mod overlay;
mod persistence;
mod scheduler;
mod tray;

use std::sync::{Arc, Mutex};

use tauri::{Manager, WindowEvent};

use app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            commands::open_settings_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init());

    // On macOS, notifications go through `mac-usernotifications` (see
    // `notify.rs`) instead — the legacy NSUserNotification API this plugin
    // uses by default there doesn't reliably deliver.
    #[cfg(not(target_os = "macos"))]
    let builder = builder.plugin(tauri_plugin_notification::init());

    builder
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();

            let app_data = persistence::load_app_data(&handle)?;
            let scheduler_state = persistence::load_state(&handle)?;
            let paused = app_data.settings.paused;

            let data = Arc::new(Mutex::new(app_data));
            let sched = Arc::new(Mutex::new(scheduler_state));
            let call_detector = call_detection::platform_detector();

            app.manage(AppState {
                data: data.clone(),
                scheduler_state: sched.clone(),
            });

            tray::build_tray(&handle, paused)?;

            notify::request_permission(&handle);

            scheduler::spawn(handle, data, sched, call_detector);

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::breaks::list_breaks,
            commands::breaks::create_break,
            commands::breaks::update_break,
            commands::breaks::delete_break,
            commands::files::pick_image,
            commands::files::get_image_path,
            commands::break_actions::postpone_break,
            commands::break_actions::cancel_break,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
