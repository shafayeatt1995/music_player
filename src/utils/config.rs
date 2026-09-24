use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub volume: f32,
    pub pan: f32,
    pub speed: f32,
    pub eq_enabled: bool,
    pub eq_preamp: f32,
    pub eq_bands: [f32; 10],
    pub eq_preset_name: String,
    pub visualizer_mode: usize,
    pub repeat_mode: u8, // 0: Off, 1: Repeat All, 2: Repeat One
    pub shuffle: bool,
    pub theme_dark: bool,
    pub last_directory: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            speed: 1.0,
            eq_enabled: true,
            eq_preamp: 0.0,
            eq_bands: [0.0; 10],
            eq_preset_name: "Flat".to_string(),
            visualizer_mode: 0,
            repeat_mode: 0,
            shuffle: false,
            theme_dark: true,
            last_directory: None,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("xorin-player");
            p.push("config.json");
            p
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(&path, json);
            }
        }
    }
}
