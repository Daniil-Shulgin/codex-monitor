use crate::{model::Snapshot, settings::Settings};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub struct Shared {
    pub snapshot: Snapshot,
    pub settings: Settings,
    pub dir: PathBuf,
    pub error: Option<String>,
}
pub type SharedState = Arc<Mutex<Shared>>;
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    snapshot: Snapshot,
    settings: Settings,
    error: Option<String>,
}
#[tauri::command]
pub fn bootstrap(state: tauri::State<'_, SharedState>) -> Result<Bootstrap, String> {
    let s = state.lock().map_err(|_| "State unavailable")?;
    Ok(Bootstrap {
        snapshot: s.snapshot.clone(),
        settings: s.settings.clone(),
        error: s.error.clone(),
    })
}

pub fn clamp(window: &WebviewWindow, desired: Option<(i32, i32)>) -> Result<(), String> {
    let Ok(monitors) = window.available_monitors() else {
        // Some Windows desktop contexts transiently deny monitor enumeration.
        // The widget is still usable; retain its native position instead of failing startup.
        return Ok(());
    };
    let position = desired
        .or_else(|| window.outer_position().ok().map(|p| (p.x, p.y)))
        .unwrap_or((24, 24));
    let selected = monitors
        .iter()
        .find(|m| {
            let a = m.work_area();
            position.0 >= a.position.x
                && position.1 >= a.position.y
                && position.0 < a.position.x + a.size.width as i32
                && position.1 < a.position.y + a.size.height as i32
        })
        .or_else(|| monitors.first());
    if let Some(m) = selected {
        let a = m.work_area();
        let size = window.outer_size().map_err(|_| "Window size unavailable")?;
        let x = position.0.clamp(
            a.position.x,
            (a.position.x + a.size.width as i32 - size.width as i32).max(a.position.x),
        );
        let y = position.1.clamp(
            a.position.y,
            (a.position.y + a.size.height as i32 - size.height as i32).max(a.position.y),
        );
        window
            .set_position(PhysicalPosition::new(x, y))
            .map_err(|_| "Cannot move window")?;
    }
    Ok(())
}
fn native(window: &WebviewWindow, s: &Settings, resize: bool) -> Result<(), String> {
    window
        .set_always_on_top(s.pinned)
        .map_err(|_| "Cannot change Always on Top")?;
    window
        .set_ignore_cursor_events(s.click_through)
        .map_err(|_| "Cannot change click-through")?;
    if resize {
        window
            .set_size(LogicalSize::new(
                if s.expanded { 330.0 } else { 290.0 },
                if s.expanded { 160.0 } else { 40.0 },
            ))
            .map_err(|_| "Cannot resize window")?;
    }
    clamp(window, None)
}
pub fn report(app: &AppHandle, message: String) {
    if let Some(state) = app.try_state::<SharedState>() {
        if let Ok(mut s) = state.lock() {
            s.error = Some(message.clone());
            crate::diagnostics::log(&s.dir, "native_operation_failed");
        }
    }
    let _ = app.emit("runtime-error", message);
}
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: tauri::State<'_, SharedState>,
    mut settings: Settings,
) -> Result<Settings, String> {
    settings.validate()?;
    let old = {
        let s = state.lock().map_err(|_| "State unavailable")?;
        settings.position = s.settings.position;
        s.settings.clone()
    };
    let window = app.get_webview_window("main").ok_or("Widget unavailable")?;
    let shortcut_changed = !settings.shortcut.eq_ignore_ascii_case(&old.shortcut);
    if shortcut_changed {
        app.global_shortcut()
            .register(settings.shortcut.as_str())
            .map_err(|_| "Shortcut unavailable or already in use. Previous shortcut retained.")?;
    }
    let result = (|| {
        if settings.launch_at_login != old.launch_at_login {
            if settings.launch_at_login {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            }
            .map_err(|_| "Cannot change Windows startup")?;
        }
        native(&window, &settings, false)?;
        let mut s = state.lock().map_err(|_| "State unavailable")?;
        settings.save(&s.dir)?;
        s.settings = settings.clone();
        s.error = None;
        Ok::<(), String>(())
    })();
    if let Err(e) = result {
        if shortcut_changed {
            let _ = app.global_shortcut().unregister(settings.shortcut.as_str());
        }
        let _ = native(&window, &old, false);
        if settings.launch_at_login != old.launch_at_login {
            let _ = if old.launch_at_login {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            };
        }
        return Err(e);
    }
    if shortcut_changed {
        let _ = app.global_shortcut().unregister(old.shortcut.as_str());
    }
    let _ = app.emit("settings", &settings);
    update_menu(&app, &settings);
    Ok(settings)
}
#[tauri::command]
pub fn drag_widget(window: WebviewWindow) -> Result<(), String> {
    window
        .start_dragging()
        .map_err(|_| "Cannot start window dragging".into())
}
#[tauri::command]
pub fn resize_widget(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    let state = app.state::<SharedState>();
    let expanded = state
        .lock()
        .map_err(|_| "State unavailable")?
        .settings
        .expanded;
    let w = app.get_webview_window("main").ok_or("Widget unavailable")?;
    let width = if expanded {
        330.0
    } else {
        width.clamp(290.0, 390.0)
    };
    let height = if expanded {
        height.clamp(132.0, 430.0)
    } else {
        40.0
    };
    let old_position = w.outer_position().ok();
    let old_size = w.outer_size().ok();
    w.set_size(LogicalSize::new(width, height))
        .map_err(|_| "Cannot resize widget")?;
    if let (Some(position), Some(old_size), Ok(new_size)) = (old_position, old_size, w.outer_size())
    {
        if old_size != new_size {
            w.set_position(PhysicalPosition::new(
                position
                    .x
                    .saturating_add(old_size.width as i32)
                    .saturating_sub(new_size.width as i32),
                position
                    .y
                    .saturating_add(old_size.height as i32)
                    .saturating_sub(new_size.height as i32),
            ))
            .map_err(|_| "Cannot anchor widget to bottom-right corner")?;
        }
    }
    clamp(&w, None)
}
#[tauri::command]
pub fn open_settings(app: AppHandle) {
    action(&app, "settings");
}
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
pub fn present_widget(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window
            .show()
            .and_then(|_| window.unminimize())
            .and_then(|_| window.set_focus())
        {
            report(app, format!("Cannot bring widget to front: {error}"));
        }
    }
}
fn menu(app: &AppHandle, s: &Settings) -> tauri::Result<Menu<tauri::Wry>> {
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "Show / Hide Widget", true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "expand",
                if s.expanded { "Collapse" } else { "Expand" },
                true,
                None::<&str>,
            )?,
            &CheckMenuItem::with_id(app, "pin", "Always on Top", true, s.pinned, None::<&str>)?,
            &CheckMenuItem::with_id(
                app,
                "click",
                "Click-through",
                true,
                s.click_through,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?,
            &CheckMenuItem::with_id(
                app,
                "startup",
                "Start with Windows",
                true,
                s.launch_at_login,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?,
        ],
    )
}
fn update_menu(app: &AppHandle, s: &Settings) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("monitor"), menu(app, s)) {
        let _ = tray.set_menu(Some(menu));
    }
}
pub fn action(app: &AppHandle, id: &str) {
    if id == "exit" {
        app.exit(0);
        return;
    }
    if id == "show" {
        if let Some(w) = app.get_webview_window("main") {
            if w.is_visible().unwrap_or(false) {
                let _ = w.hide();
            } else {
                let _ = w.show();
            }
        }
        return;
    }
    if id == "settings" {
        if let Some(w) = app.get_webview_window("settings") {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.set_focus();
        } else {
            report(app, "Settings window unavailable".into());
        }
        return;
    }
    let state = app.state::<SharedState>();
    let Ok(s) = state.lock() else { return };
    let mut settings = s.settings.clone();
    drop(s);
    match id {
        "expand" => settings.expanded = !settings.expanded,
        "pin" => settings.pinned = !settings.pinned,
        "click" => settings.click_through = !settings.click_through,
        "startup" => settings.launch_at_login = !settings.launch_at_login,
        _ => return,
    }
    if let Err(e) = save_settings(app.clone(), state, settings) {
        report(app, e);
    }
}
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_config_dir()?;
    let mut settings = Settings::load(&dir);
    let mut error = None;
    if app
        .global_shortcut()
        .register(settings.shortcut.as_str())
        .is_err()
    {
        settings.click_through = false;
        error=Some("Shortcut registration failed. Click-through disabled; use tray Settings to choose another shortcut.".into());
        crate::diagnostics::log(&dir, "shortcut_registration_failed");
    }
    if let Ok(enabled) = app.autolaunch().is_enabled() {
        settings.launch_at_login = enabled;
    }
    let shared = Arc::new(Mutex::new(Shared {
        snapshot: Snapshot::default(),
        settings: settings.clone(),
        dir: dir.clone(),
        error,
    }));
    app.manage(shared.clone());
    let window = app
        .get_webview_window("main")
        .ok_or("Main window missing")?;
    native(&window, &settings, true)?;
    clamp(&window, settings.position)?;
    TrayIconBuilder::with_id("monitor")
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/icon.png"
        ))?)
        .tooltip("Codex Monitor")
        .menu(&menu(app.handle(), &settings)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| action(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                present_widget(tray.app_handle());
            }
        })
        .build(app)?;
    window.show()?;
    let home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".codex")))
        .ok_or("User home unavailable")?;
    crate::diagnostics::log(&dir, "monitor_started");
    crate::provider::start(app.handle().clone(), shared, home);
    if std::env::args().any(|a| a == "--settings") {
        action(app.handle(), "settings");
    }
    Ok(())
}
