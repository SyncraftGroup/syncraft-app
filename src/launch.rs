use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use syncraft_core::{
    client::{
        auth,
        config::{Account, ClientConfig, client_config},
        sync,
    },
    install::Installer,
    launcher::{LaunchContext, Launcher},
    paths::MinecraftPaths,
    progress::Progress,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::{api::LaunchEvent, config::AppData};

const EVENT: &str = "launch-progress";

/// Emits install/download progress to the UI at a bounded rate.
struct GuiProgress {
    app: AppHandle,
    phase: &'static str,
    start: f64,
    end: f64,
    stage: Mutex<String>,
    files_done: AtomicU64,
    files_total: AtomicU64,
    bytes_done: AtomicU64,
    bytes_total: AtomicU64,
    per_file: Mutex<HashMap<String, u64>>,
    last_emit: Mutex<Instant>,
}

impl GuiProgress {
    fn new(app: AppHandle, phase: &'static str, start: f64, end: f64, stage: &str) -> Self {
        Self {
            app,
            phase,
            start,
            end,
            stage: Mutex::new(stage.to_string()),
            files_done: AtomicU64::new(0),
            files_total: AtomicU64::new(0),
            bytes_done: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            per_file: Mutex::new(HashMap::new()),
            last_emit: Mutex::new(Instant::now() - Duration::from_secs(1)),
        }
    }

    fn set_stage(&self, stage: &str) {
        *self.stage.lock().unwrap() = stage.to_string();
        self.emit(true);
    }

    fn finish(&self, stage: &str) {
        *self.stage.lock().unwrap() = stage.to_string();
        self.emit_impl(true, Some(self.end));
    }

    fn emit_impl(&self, force: bool, progress_override: Option<f64>) {
        let now = Instant::now();
        {
            let mut last = self.last_emit.lock().unwrap();
            if !force && now.duration_since(*last) < Duration::from_millis(80) {
                return;
            }
            *last = now;
        }

        let done = self.files_done.load(Ordering::Relaxed);
        let total = self.files_total.load(Ordering::Relaxed);
        let local = if total > 0 {
            (done as f64 / total as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress = progress_override
            .unwrap_or(self.start + (self.end - self.start) * local)
            .clamp(0.0, 100.0);

        let _ = self.app.emit(
            EVENT,
            LaunchEvent {
                phase: self.phase.into(),
                progress,
                stage: self.stage.lock().unwrap().clone(),
                files_done: done,
                files_total: total,
                bytes_done: self.bytes_done.load(Ordering::Relaxed),
                bytes_total: self.bytes_total.load(Ordering::Relaxed),
                message: None,
            },
        );
    }

    fn emit(&self, force: bool) {
        self.emit_impl(force, None);
    }

    fn emit_with_progress(&self, force: bool, progress: f64) {
        self.emit_impl(force, Some(progress));
    }

    fn recalc_bytes_total(&self) {
        let total: u64 = self.per_file.lock().unwrap().values().sum();
        self.bytes_total.store(total, Ordering::Relaxed);
    }
}

impl Progress for GuiProgress {
    fn set_total(&self, total: u64) {
        self.files_done.store(0, Ordering::Relaxed);
        self.files_total.store(total, Ordering::Relaxed);
        self.emit(true);
    }

    fn add_total(&self, n: u64) {
        self.files_total.fetch_add(n, Ordering::Relaxed);
        self.emit(false);
    }

    fn overall_inc(&self) {
        self.files_done.fetch_add(1, Ordering::Relaxed);
        self.emit(false);
    }

    fn overall_finish(&self, msg: &str) {
        self.finish(msg);
    }

    fn spinner(&self, msg: &str) {
        // Core uses English stage strings; the flow sets Chinese ones itself.
        let _ = msg;
        self.emit_with_progress(true, self.start + (self.end - self.start) * 0.1);
    }

    fn spinner_done(&self, _msg: &str) {}

    fn file_start(&self, name: &str, size: u64) {
        self.per_file.lock().unwrap().insert(name.to_string(), size);
        self.recalc_bytes_total();
        self.emit(false);
    }

    fn file_total(&self, name: &str, total: u64) {
        self.per_file
            .lock()
            .unwrap()
            .insert(name.to_string(), total);
        self.recalc_bytes_total();
        self.emit(false);
    }

    fn file_inc(&self, _name: &str, bytes: u64) {
        self.bytes_done.fetch_add(bytes, Ordering::Relaxed);
        self.emit(false);
    }

    fn file_done(&self, _name: &str) {
        self.emit(false);
    }
}

fn emit_stage(app: &AppHandle, phase: &str, progress: f64, stage: &str) {
    let _ = app.emit(EVENT, LaunchEvent::stage(phase, progress, stage));
}

fn resolve_java(client: &ClientConfig, required: u8) -> Result<PathBuf, String> {
    if let Some(path) = &client.java_override {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    java_detect::get_java(required)
        .map(|java| java.path)
        .ok_or_else(|| format!("未找到可用的 Java {required}，请在设置中选择运行时"))
}

async fn resolve_account(client: &ClientConfig, link: &str) -> Result<Account, String> {
    match &client.account {
        Some(Account::Syncraft {
            username,
            uuid,
            email,
            client_token,
            access_token,
        }) => match auth::refresh_token(link, access_token, client_token).await {
            Ok((new_access, new_client)) => {
                let account = Account::Syncraft {
                    username: username.clone(),
                    uuid: uuid.clone(),
                    email: email.clone(),
                    client_token: new_client,
                    access_token: new_access,
                };
                client_config()
                    .update(|cfg| cfg.account = Some(account.clone()))
                    .map_err(|e| e.to_string())?;
                Ok(account)
            }
            Err(err) => Err(format!("Syncraft 登录已失效：{err}")),
        },
        Some(offline) => Ok(offline.clone()),
        None => Err("请先在账号页添加账号".into()),
    }
}

pub async fn run(app: AppHandle) {
    if let Err(err) = flow(&app).await {
        let _ = app.emit(EVENT, LaunchEvent::error(err));
    }
}

async fn flow(app: &AppHandle) -> Result<(), String> {
    let client = client_config().get();
    let link = client
        .syncraft_link
        .clone()
        .ok_or_else(|| "未配置服务器链接".to_string())?;

    let data = app.state::<AppData>();
    let game_dir = data.game_dir();

    emit_stage(app, "checking", 0.0, "正在检查更新");
    let cached = sync::cached_manifest(&game_dir);
    let manifest = sync::fetch_manifest(&link, cached.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    emit_stage(app, "checking", 3.0, "正在安装游戏文件");
    let account = resolve_account(&client, &link).await?;

    let paths = MinecraftPaths::new(&manifest.version.id, manifest.version.cache_key())
        .map_err(|e| e.to_string())?;

    let install = GuiProgress::new(app.clone(), "installing", 5.0, 55.0, "正在安装游戏文件");
    let vanilla = Installer::meta_vanilla(&manifest.version, &paths, Some(&install))
        .await
        .map_err(|e| e.to_string())?;

    let required = vanilla.java_version.unwrap_or(8);
    install.set_stage("正在查找 Java");
    let java_path = resolve_java(&client, required)?;

    install.set_stage("正在安装游戏文件");
    let version = Installer::restore_or_install(&manifest.version, vanilla, &paths, Some(&install))
        .await
        .map_err(|e| e.to_string())?;
    install.finish("游戏环境就绪");

    emit_stage(app, "syncing", 55.0, "正在同步服务器文件");
    let plan = sync::plan_sync(&manifest, paths.root())
        .await
        .map_err(|e| e.to_string())?;
    let syncing = GuiProgress::new(app.clone(), "syncing", 55.0, 95.0, "正在同步服务器文件");
    sync::download_files(&link, &manifest, &plan, &syncing)
        .await
        .map_err(|e| e.to_string())?;
    syncing.finish("文件同步完成");

    client_config()
        .update(|cfg| {
            cfg.version_hash = Some(manifest.version_hash.clone());
            cfg.commit = Some(manifest.commit.clone());
        })
        .map_err(|e| e.to_string())?;

    let (username, uuid, access_token, auth_server) = match &account {
        Account::Offline { username, uuid } => {
            (username.clone(), uuid.clone(), "0".to_string(), None)
        }
        Account::Syncraft {
            username,
            uuid,
            access_token,
            ..
        } => (
            username.clone(),
            uuid.clone(),
            access_token.clone(),
            Some(format!("{link}/yggdrasil")),
        ),
    };

    let ctx = LaunchContext {
        java_path,
        username,
        uuid,
        access_token,
        launcher_name: "syncraft".into(),
        launcher_version: env!("CARGO_PKG_VERSION").into(),
        auth_server,
    };

    emit_stage(app, "launching", 97.0, "正在启动游戏");

    // Keep the last lines of the game output so a crash before the window
    // appears can be surfaced instead of silently reporting "running".
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(512);
    let tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let tail_writer = tail.clone();
    let log_path = paths.root().join("logs").join("syncraft-launch.log");
    if let Some(parent) = log_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    tauri::async_runtime::spawn(async move {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        while let Some(line) = rx.recv().await {
            eprintln!("[minecraft] {line}");
            if let Some(file) = file.as_mut() {
                let _ = writeln!(file, "{line}");
            }
            let mut lines = tail_writer.lock().unwrap();
            if lines.len() >= 40 {
                lines.pop_front();
            }
            lines.push_back(line);
        }
    });

    let mut child = Launcher::launch(version, ctx, &paths, Some(tx))
        .await
        .map_err(|e| e.to_string())?;

    if data.app_settings().minimize_on_launch
        && let Some(window) = app.get_webview_window("main")
    {
        let _ = window.minimize();
    }

    let _ = app.emit(
        EVENT,
        LaunchEvent {
            phase: "running".into(),
            progress: 100.0,
            stage: "游戏运行中".into(),
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            message: None,
        },
    );

    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .map_err(|e| e.to_string())?;

    let tail_text = tail
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    match status {
        Ok(status) if status.success() => emit_stage(app, "idle", 0.0, "游戏已退出"),
        Ok(status) => {
            let code = status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".into());
            let message = if tail_text.is_empty() {
                format!("游戏进程退出（{code}），未捕获到输出")
            } else {
                format!("游戏进程退出（{code}）\n{tail_text}")
            };
            let _ = app.emit(EVENT, LaunchEvent::error(message));
        }
        Err(err) => {
            let _ = app.emit(EVENT, LaunchEvent::error(format!("游戏进程异常：{err}")));
        }
    }

    Ok(())
}
