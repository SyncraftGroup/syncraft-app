use std::path::Path;

use java_detect::JavaInstallation;
use serde::Serialize;
use syncraft_core::client::config::Account;
use syncraft_protocol::{Loader, Manifest};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSummary {
    pub link: String,
    pub name: String,
    pub version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub file_count: usize,
    pub version_hash: String,
    pub commit: String,
    pub updates: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub kind: String,
    pub name: String,
    pub email: Option<String>,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaSummary {
    pub path: String,
    pub version: u8,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub server: Option<ServerSummary>,
    pub link: Option<String>,
    pub server_error: Option<String>,
    pub account: Option<AccountSummary>,
    pub game_dir: String,
    pub java: Option<JavaSummary>,
    pub detected_java: Vec<JavaSummary>,
    pub source: String,
    pub concurrency: Option<usize>,
    pub minimize_on_launch: bool,
    pub auto_launch: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchEvent {
    pub phase: String,
    pub progress: f64,
    pub stage: String,
    pub files_done: u64,
    pub files_total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl LaunchEvent {
    pub fn stage(phase: &str, progress: f64, stage: &str) -> Self {
        Self {
            phase: phase.into(),
            progress,
            stage: stage.into(),
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            phase: "error".into(),
            progress: 0.0,
            stage: "启动失败".into(),
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            message: Some(message.into()),
        }
    }
}

pub fn loader_fields(loader: &Loader) -> (String, Option<String>) {
    match loader {
        Loader::Vanilla => ("Vanilla".into(), None),
        Loader::Fabric { loader_version } => ("Fabric".into(), Some(loader_version.clone())),
        Loader::Forge { loader_version } => ("Forge".into(), Some(loader_version.clone())),
        Loader::NeoForge { loader_version } => ("NeoForge".into(), Some(loader_version.clone())),
    }
}

pub fn server_summary(link: &str, manifest: &Manifest, updates: Vec<String>) -> ServerSummary {
    let (loader, loader_version) = loader_fields(&manifest.version.loader);
    ServerSummary {
        link: link.to_string(),
        name: manifest.version.name.clone(),
        version: manifest.version.vanilla_version.clone(),
        loader,
        loader_version,
        file_count: manifest.files.len(),
        version_hash: manifest.version_hash.clone(),
        commit: manifest.commit.clone(),
        updates,
    }
}

pub fn account_summary(account: &Account) -> AccountSummary {
    match account {
        Account::Offline { username, uuid } => AccountSummary {
            kind: "offline".into(),
            name: username.clone(),
            email: None,
            uuid: uuid.clone(),
        },
        Account::Syncraft {
            username,
            uuid,
            email,
            ..
        } => AccountSummary {
            kind: "syncraft".into(),
            name: username.clone(),
            email: Some(email.clone()),
            uuid: uuid.clone(),
        },
    }
}

pub fn java_summary(java: &JavaInstallation, selected: Option<&Path>) -> JavaSummary {
    JavaSummary {
        path: java.path.to_string_lossy().into_owned(),
        version: java.version,
        selected: selected.is_some_and(|path| path == java.path),
    }
}
