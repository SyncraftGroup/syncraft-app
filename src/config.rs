use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use serde::{Deserialize, Serialize};
use syncraft_core::{
    Config, Source,
    client::config::{ClientConfig, client_config, try_init_client_config},
};

/// Initial server is intentionally empty: the user must add an address before
/// anything that needs the server becomes available.
pub const DEFAULT_SERVER_LINK: Option<&str> = None;

const CORE_CONFIG: &str = "core.json";
const CLIENT_CONFIG: &str = "client.json";
const APP_CONFIG: &str = "app.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub minimize_on_launch: bool,
    pub auto_launch: bool,
}

pub struct AppData {
    pub base: PathBuf,
    settings: Mutex<AppSettings>,
    pub launching: Arc<AtomicBool>,
}

impl AppData {
    pub fn initialize(base: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;

        // The game directory lives next to the working directory (e.g. when the
        // app runs from `syncraft-app/`, this is the workspace `.minecraft/`).
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let game_dir = cwd.parent().unwrap_or(cwd.as_path()).join(".minecraft");
        fs::create_dir_all(&game_dir).map_err(|e| e.to_string())?;

        init_core_config(&base.join(CORE_CONFIG), &game_dir)?;
        init_client_config(&base.join(CLIENT_CONFIG))?;

        let settings = load_app_settings(&base.join(APP_CONFIG));

        Ok(Self {
            base,
            settings: Mutex::new(settings),
            launching: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn app_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings<F: FnOnce(&mut AppSettings)>(&self, f: F) -> Result<(), String> {
        {
            let mut settings = self.settings.lock().unwrap();
            f(&mut settings);
        }
        let settings = self.settings.lock().unwrap().clone();
        let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        fs::write(self.base.join(APP_CONFIG), json).map_err(|e| e.to_string())
    }

    /// Absolute game directory (the configured relative path is resolved
    /// against the current directory by core, so store it absolutely).
    pub fn game_dir(&self) -> PathBuf {
        let configured = PathBuf::from(syncraft_core::config().get().game_dir);
        if configured.is_absolute() {
            configured
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| self.base.clone())
                .join(configured)
        }
    }
}

fn init_core_config(path: &Path, game_dir: &Path) -> Result<(), String> {
    let default = || Config {
        game_dir: game_dir.to_string_lossy().into_owned(),
        java_paths: Default::default(),
        selected_source: Source::Default,
    };

    if !path.exists() {
        let json = serde_json::to_string_pretty(&default()).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
    }

    match syncraft_core::try_set_config(path) {
        Ok(()) => Ok(()),
        Err(err) => {
            // A corrupt config must not make the launcher unstartable: keep a
            // backup and fall back to defaults.
            let backup = path.with_extension("json.corrupt");
            let _ = fs::rename(path, &backup);
            let json = serde_json::to_string_pretty(&default()).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())?;
            syncraft_core::try_set_config(path).map_err(|e| format!("{err}; retry: {e}"))
        }
    }
}

fn init_client_config(path: &Path) -> Result<(), String> {
    let default = || ClientConfig {
        syncraft_link: DEFAULT_SERVER_LINK.map(str::to_string),
        ..Default::default()
    };

    if !path.exists() {
        let json = serde_json::to_string_pretty(&default()).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
    }

    match try_init_client_config(path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let backup = path.with_extension("json.corrupt");
            let _ = fs::rename(path, &backup);
            let json = serde_json::to_string_pretty(&default()).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())?;
            try_init_client_config(path).map_err(|e| format!("{err}; retry: {e}"))
        }
    }
}

fn load_app_settings(path: &Path) -> AppSettings {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Normalize a user-entered server address.
pub fn normalize_link(link: &str) -> String {
    link.trim().trim_end_matches('/').to_string()
}

pub fn save_link(link: &str) -> Result<String, String> {
    let link = normalize_link(link);
    if link.is_empty() {
        return Err("服务器链接不能为空".into());
    }
    client_config()
        .update(|cfg| cfg.syncraft_link = Some(link.clone()))
        .map_err(|e| e.to_string())?;
    Ok(link)
}
