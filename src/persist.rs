use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct PersistentConfig {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub mode: Option<String>,
    pub time_duration: Option<u64>,
    pub word_count: Option<usize>,
    pub punctuation: Option<bool>,
    pub numbers: Option<bool>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("typingterm").join("config.json"))
}

impl PersistentConfig {
    pub fn load() -> Self {
        let path = match config_path() {
            Some(p) => p,
            None => return Self::default(),
        };
        match fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = match config_path() {
            Some(p) => p,
            None => return,
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }
}
