use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use serde::Serialize;
use syncraft_core::{
    Source,
    client::{
        auth,
        config::{Account, client_config},
        sync,
    },
};
use tauri::{AppHandle, State};

use crate::{
    api::{
        AccountSummary, JavaSummary, ServerSummary, Snapshot, account_summary, java_summary,
        server_summary,
    },
    config::{AppData, normalize_link, save_link},
    launch,
};

fn link_or_err() -> Result<String, String> {
    client_config()
        .get()
        .syncraft_link
        .filter(|link| !link.is_empty())
        .ok_or_else(|| "请先连接服务器".to_string())
}

#[tauri::command]
pub fn snapshot(data: State<'_, AppData>) -> Result<Snapshot, String> {
    let client = client_config().get();
    let core = syncraft_core::config().get();
    let game_dir = data.game_dir();
    let settings = data.app_settings();

    let server = client
        .syncraft_link
        .as_deref()
        .filter(|link| !link.is_empty())
        .and_then(|link| {
            sync::cached_manifest(&game_dir).map(|m| server_summary(link, &m, vec![]))
        });

    let link = client.syncraft_link.clone().filter(|link| !link.is_empty());

    let detected = java_detect::detect();
    let override_path = client.java_override.clone();
    let detected_java: Vec<JavaSummary> = detected
        .iter()
        .map(|java| java_summary(java, override_path.as_deref()))
        .collect();

    let java = override_path.map(|path| {
        detected_java
            .iter()
            .find(|java| Path::new(&java.path) == path)
            .cloned()
            .unwrap_or_else(|| JavaSummary {
                path: path.to_string_lossy().into_owned(),
                version: probe_java_version(&path).unwrap_or(0),
                selected: true,
            })
    });

    let source = match core.selected_source {
        Source::Default => "default",
        Source::BMCLAPI => "bmclapi",
    };

    Ok(Snapshot {
        server,
        link,
        server_error: None,
        account: client.account.as_ref().map(account_summary),
        game_dir: game_dir.to_string_lossy().into_owned(),
        java,
        detected_java,
        source: source.into(),
        concurrency: client.download_concurrency,
        minimize_on_launch: settings.minimize_on_launch,
        auto_launch: settings.auto_launch,
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

#[tauri::command]
pub async fn refresh_server(data: State<'_, AppData>) -> Result<ServerSummary, String> {
    let client = client_config().get();
    let link = client
        .syncraft_link
        .clone()
        .ok_or_else(|| "未配置服务器链接".to_string())?;
    let game_dir = data.game_dir();
    let cached = sync::cached_manifest(&game_dir);

    let manifest = sync::fetch_manifest(&link, cached.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let updates = match client.commit.as_deref() {
        Some(commit) if !manifest.commit.is_empty() && commit != manifest.commit => {
            sync::fetch_update_info(&link, commit)
                .await
                .map(|items| items.into_iter().map(|item| item.message).collect())
                .unwrap_or_default()
        }
        _ => Vec::new(),
    };

    Ok(server_summary(&link, &manifest, updates))
}

#[tauri::command]
pub async fn set_server_link(
    link: String,
    data: State<'_, AppData>,
) -> Result<ServerSummary, String> {
    let link = normalize_link(&link);
    if link.is_empty() {
        return Err("服务器链接不能为空".into());
    }

    let game_dir = data.game_dir();
    let previous = client_config().get().syncraft_link;
    let switching = previous.as_deref() != Some(link.as_str());
    if switching {
        // A new server is a new trust boundary: drop the pinned signing key
        // and the last-applied version before the first request.
        client_config()
            .update(|cfg| {
                cfg.server_key = None;
                cfg.version_hash = None;
                cfg.commit = None;
            })
            .map_err(|e| e.to_string())?;
    }

    let cached = if switching {
        None
    } else {
        sync::cached_manifest(&game_dir)
    };
    let manifest = sync::fetch_manifest(&link, cached.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    save_link(&link)?;
    Ok(server_summary(&link, &manifest, vec![]))
}

#[tauri::command]
pub async fn login(email: String, password: String) -> Result<AccountSummary, String> {
    let link = link_or_err()?;
    let account = auth::authenticate(&link, email.trim(), &password)
        .await
        .map_err(|e| e.to_string())?;
    client_config()
        .update(|cfg| cfg.account = Some(account.clone()))
        .map_err(|e| e.to_string())?;
    Ok(account_summary(&account))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResult {
    pub id: String,
}

#[tauri::command]
pub async fn register(
    username: String,
    email: String,
    password: String,
) -> Result<RegisterResult, String> {
    let link = link_or_err()?;
    let username = username.trim().to_string();
    if !auth::is_valid_minecraft_username(&username) {
        return Err("角色名需为 3–16 位字母、数字或下划线".into());
    }
    if password.len() < 8 {
        return Err("密码至少 8 位".into());
    }

    let result = auth::register(&link, &username, email.trim(), &password)
        .await
        .map_err(|e| e.to_string())?;
    Ok(RegisterResult { id: result.id })
}

#[tauri::command]
pub fn set_offline_account(username: String) -> Result<AccountSummary, String> {
    // Accounts only make sense with a selected server; the UI enforces this
    // too, but the command must not be usable around it.
    link_or_err()?;

    let username = username.trim().to_string();
    if !auth::is_valid_minecraft_username(&username) {
        return Err("角色名需为 3–16 位字母、数字或下划线".into());
    }
    let account = Account::Offline {
        uuid: auth::offline_uuid(&username),
        username,
    };
    client_config()
        .update(|cfg| cfg.account = Some(account.clone()))
        .map_err(|e| e.to_string())?;
    Ok(account_summary(&account))
}

#[tauri::command]
pub fn logout() -> Result<(), String> {
    client_config()
        .update(|cfg| cfg.account = None)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_java() -> Vec<JavaSummary> {
    let selected = client_config().get().java_override;
    java_detect::detect()
        .iter()
        .map(|java| java_summary(java, selected.as_deref()))
        .collect()
}

#[tauri::command]
pub fn set_java(path: Option<String>) -> Result<(), String> {
    let path = path
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(path) = &path {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err("找不到该 Java 可执行文件".into());
        }
        client_config()
            .update(|cfg| cfg.java_override = Some(path))
            .map_err(|e| e.to_string())?;
    } else {
        client_config()
            .update(|cfg| cfg.java_override = None)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_game_dir(path: String, data: State<'_, AppData>) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("游戏目录不能为空".into());
    }
    let mut dir = PathBuf::from(trimmed);
    if !dir.is_absolute() {
        dir = data.base.join(dir);
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;

    syncraft_core::config()
        .update(|cfg| cfg.game_dir = dir.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn set_source(source: String) -> Result<(), String> {
    let selected = match source.as_str() {
        "default" => Source::Default,
        "bmclapi" => Source::BMCLAPI,
        other => return Err(format!("未知下载源：{other}")),
    };
    syncraft_core::config()
        .update(|cfg| cfg.selected_source = selected)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_concurrency(value: Option<usize>) -> Result<(), String> {
    let value = value.map(|value| value.clamp(1, 16));
    client_config()
        .update(|cfg| cfg.download_concurrency = value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_toggle(name: String, value: bool, data: State<'_, AppData>) -> Result<(), String> {
    data.update_settings(|settings| match name.as_str() {
        "minimizeOnLaunch" => settings.minimize_on_launch = value,
        "autoLaunch" => settings.auto_launch = value,
        _ => {}
    })
}

#[tauri::command]
pub fn start_launch(app: AppHandle, data: State<'_, AppData>) -> Result<(), String> {
    if data.launching.swap(true, Ordering::SeqCst) {
        return Err("正在启动中".into());
    }

    let flag = data.launching.clone();
    tauri::async_runtime::spawn(async move {
        launch::run(app).await;
        flag.store(false, Ordering::SeqCst);
    });

    Ok(())
}

fn probe_java_version(path: &Path) -> Option<u8> {
    let output = std::process::Command::new(path)
        .arg("-version")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stderr);
    let start = text.find("version \"")? + "version \"".len();
    let rest = &text[start..];
    let end = rest.find('"')?;
    let version = &rest[..end];
    let major = version
        .strip_prefix("1.")
        .unwrap_or(version)
        .split('.')
        .next()?;
    major.parse().ok()
}
