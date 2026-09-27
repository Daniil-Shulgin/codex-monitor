mod alerts;
mod diagnostics;
pub mod lifecycle;
pub mod metadata;
pub mod model;
mod process_job;
pub mod provider;
pub mod quotas;
pub mod rpc;
mod runtime;
pub mod scanner;
mod settings;

pub fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if args.iter().any(|a| a == "--settings") {
                runtime::action(app, "settings");
            } else {
                runtime::present_widget(app);
            }
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        runtime::action(app, "click");
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(runtime::setup)
        .invoke_handler(tauri::generate_handler![
            runtime::bootstrap,
            runtime::save_settings,
            runtime::drag_widget,
            runtime::resize_widget,
            runtime::open_settings,
            runtime::quit_app
        ])
        .on_window_event(|window, event| {
            if window.label() == "settings" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
                return;
            }
            if window.label() != "main" {
                return;
            }
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                tauri::WindowEvent::Moved(position) => {
                    if let Some(state) = window.app_handle().try_state::<runtime::SharedState>() {
                        if let Ok(mut s) = state.lock() {
                            s.settings.position = Some((position.x, position.y));
                            let _ = s.settings.save(&s.dir);
                        }
                    }
                }
                tauri::WindowEvent::ScaleFactorChanged { .. } => {
                    if let Some(w) = window.app_handle().get_webview_window("main") {
                        let _ = runtime::clamp(&w, None);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("Codex Monitor could not initialize its native window");
}
