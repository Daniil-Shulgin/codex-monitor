use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub theme: String,
    pub opacity: f64,
    pub pinned: bool,
    pub expanded: bool,
    pub click_through: bool,
    pub shortcut: String,
    pub launch_at_login: bool,
    pub notify_action: bool,
    pub notify_done: bool,
    pub sound: bool,
    pub volume: u8,
    pub refresh_seconds: u64,
    pub position: Option<(i32, i32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            opacity: 0.88,
            pinned: true,
            expanded: false,
            click_through: false,
            shortcut: "Ctrl+Shift+C".into(),
            launch_at_login: false,
            notify_action: true,
            notify_done: true,
            sound: true,
            volume: 35,
            refresh_seconds: 30,
            position: None,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !["system", "dark", "light"].contains(&self.theme.as_str())
            || !self.opacity.is_finite()
            || !(0.55..=1.0).contains(&self.opacity)
            || self.volume > 100
            || ![10, 30, 60].contains(&self.refresh_seconds)
            || self.shortcut.len() > 80
        {
            return Err("Invalid settings".into());
        }
        Ok(())
    }
    pub fn load(dir: &Path) -> Self {
        std::fs::read(dir.join("settings.json"))
            .ok()
            .filter(|b| b.len() < 16384)
            .and_then(|b| serde_json::from_slice::<Self>(&b).ok())
            .filter(|s| s.validate().is_ok())
            .unwrap_or_default()
    }
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|_| "Cannot create settings directory")?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|_| "Cannot encode settings")?;
        let temp = dir.join("settings.tmp");
        std::fs::write(&temp, bytes).map_err(|_| "Cannot save settings")?;
        // std::fs::rename replaces the destination atomically on Windows.
        std::fs::rename(temp, dir.join("settings.json"))
            .map_err(|_| "Cannot replace settings".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_settings_survive_atomic_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings::default();
        settings.save(dir.path()).unwrap();
        settings.position = Some((-300, 42));
        settings.opacity = 0.7;
        settings.theme = "light".into();
        settings.save(dir.path()).unwrap();
        assert_eq!(Settings::load(dir.path()), settings);
    }
    #[test]
    fn corrupt_settings_fall_back_and_invalid_values_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), "broken").unwrap();
        assert_eq!(Settings::load(dir.path()), Settings::default());
        let settings = Settings {
            opacity: f64::NAN,
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
    }
}
