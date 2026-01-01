use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PasteBehavior {
    DirectPaste, // 选中即粘贴
    SelectOnly,  // 选中仅复制，手工粘贴
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortOrder {
    LastCopied,
    FirstCopied,
    CopyCount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub autostart: bool,
    pub hotkey: String,
    pub paste_behavior: PasteBehavior,
    pub clear_on_exit: bool,
}

impl GeneralConfig {
    pub fn update_autostart(&self) -> anyhow::Result<()> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let app_path = std::env::current_exe()?;
            let app_name = "Snapaste";
            
            if self.autostart {
                // Add to login items - Check if it already exists first or just try to delete then add
                let _ = Command::new("osascript")
                    .arg("-e")
                    .arg(format!("tell application \"System Events\" to delete login item \"{}\"", app_name))
                    .output();
                    
                Command::new("osascript")
                    .arg("-e")
                    .arg(format!(
                        "tell application \"System Events\" to make login item at end with properties {{path:\"{}\", name:\"{}\", hidden:false}}",
                        app_path.display(), app_name
                    ))
                    .output()?;
            } else {
                // Remove from login items
                Command::new("osascript")
                    .arg("-e")
                    .arg(format!(
                        "tell application \"System Events\" to delete login item \"{}\"",
                        app_name
                    ))
                    .output()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub store_text: bool,
    pub store_image: bool,
    pub store_file: bool,
    pub limit: usize,
    pub sort_order: SortOrder,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub storage: StorageConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                autostart: false,
                hotkey: "Cmd+Shift+V".to_string(), // 默认为 Mac 常见的快捷键或用户偏好
                paste_behavior: PasteBehavior::DirectPaste,
                clear_on_exit: false,
            },
            storage: StorageConfig {
                store_text: true,
                store_image: true,
                store_file: true,
                limit: 500,
                sort_order: SortOrder::LastCopied,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "snapaste", "snapaste")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }
}
