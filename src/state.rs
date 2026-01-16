use anyhow::{bail, Context, Result};
use chrono::Utc;
use fs2::FileExt;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::util::{claim_path, now_iso, session_state_path, task_state_path};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Incomplete,
    Failed,
    Completed,
    Issues,
}

impl TaskStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Running => "●",
            Self::Incomplete => "◐",
            Self::Failed => "✗",
            Self::Completed => "✓",
            Self::Issues => "!",
        }
    }

    pub fn styled(&self) -> String {
        let symbol = self.symbol();
        match self {
            Self::Pending => symbol.dimmed().to_string(),
            Self::Running => symbol.yellow().bold().to_string(),
            Self::Incomplete => symbol.yellow().to_string(),
            Self::Failed => symbol.red().bold().to_string(),
            Self::Completed => symbol.green().to_string(),
            Self::Issues => symbol.magenta().bold().to_string(),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Incomplete => "incomplete",
            Self::Failed => "failed",
            Self::Completed => "completed",
            Self::Issues => "issues",
        };
        write!(f, "{value}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskState {
    pub task: String,
    pub agent: String,
    pub stage: String,
    pub status: TaskStatus,
    pub added_at: String,
    pub updated_at: String,
    pub last_session: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Running,
    Finished,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub task: Option<String>,
    pub agent: String,
    pub stage: String,
    pub status: SessionStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub next_stage: Option<String>,
    pub pid: u32,
    pub host: String,
    pub repo_root: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClaimState {
    pub task: String,
    pub agent: String,
    pub pid: u32,
    pub host: String,
    pub started_at: String,
    pub ttl_seconds: u64,
}

pub struct ClaimGuard {
    path: PathBuf,
}

impl ClaimGuard {
    #[allow(dead_code)]
    pub fn release(self) -> Result<()> {
        fs::remove_file(&self.path).ok();
        Ok(())
    }
}

impl Drop for ClaimGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn lock_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "state".into());
    path.with_file_name(format!("{file_name}.lock"))
}

fn with_lock<T>(path: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock_path = lock_path(path);
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let lock_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("Failed to open lock file {}", lock_path.display()))?;
    lock_file
        .lock_exclusive()
        .with_context(|| format!("Failed to lock {}", lock_path.display()))?;
    let result = f();
    lock_file.unlock().ok();
    result
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let data = serde_json::to_string_pretty(value)?;
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "state".into());
    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));
    if let Some(parent) = tmp_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(&tmp_path, data).with_context(|| format!("Failed to write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("Failed to rename {}", path.display()))?;
    Ok(())
}

pub fn load_task(path: &Path) -> Result<TaskState> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read task state {}", path.display()))?;
    let task: TaskState = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse task state {}", path.display()))?;
    Ok(task)
}

pub fn save_task(path: &Path, task: &TaskState) -> Result<()> {
    with_lock(path, || write_json_atomic(path, task))
}

pub fn update_task(path: &Path, update: impl FnOnce(&mut TaskState) -> Result<()>) -> Result<()> {
    with_lock(path, || {
        let mut task = load_task(path)?;
        update(&mut task)?;
        write_json_atomic(path, &task)
    })
}

pub fn load_session(path: &Path) -> Result<SessionState> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read session {}", path.display()))?;
    let session: SessionState = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse session {}", path.display()))?;
    Ok(session)
}

pub fn save_session(path: &Path, session: &SessionState) -> Result<()> {
    with_lock(path, || write_json_atomic(path, session))
}

pub fn update_session(path: &Path, update: impl FnOnce(&mut SessionState) -> Result<()>) -> Result<()> {
    with_lock(path, || {
        let mut session = load_session(path)?;
        update(&mut session)?;
        write_json_atomic(path, &session)
    })
}

pub fn list_tasks(agent_root: &Path) -> Vec<TaskState> {
    let tasks_dir = agent_root.join("tasks");
    let mut tasks = Vec::new();
    let entries = match fs::read_dir(&tasks_dir) {
        Ok(entries) => entries,
        Err(_) => return tasks,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let task_path = path.join("task.json");
        if !task_path.exists() {
            continue;
        }
        if let Ok(task) = load_task(&task_path) {
            tasks.push(task);
        }
    }

    tasks
}

pub fn new_session_id() -> String {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("{}-{}", epoch, std::process::id())
}

pub fn create_session(
    agent_root: &Path,
    session_id: &str,
    agent: &str,
    stage: &str,
    task: Option<&str>,
    repo_root: &Path,
    host: &str,
) -> Result<SessionState> {
    let session = SessionState {
        session_id: session_id.to_string(),
        task: task.map(|t| t.to_string()),
        agent: agent.to_string(),
        stage: stage.to_string(),
        status: SessionStatus::Running,
        started_at: now_iso(),
        finished_at: None,
        next_stage: None,
        pid: std::process::id(),
        host: host.to_string(),
        repo_root: repo_root.display().to_string(),
    };

    let session_path = session_state_path(agent_root, session_id);
    if let Some(parent) = session_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    save_session(&session_path, &session)?;
    Ok(session)
}

pub fn resolve_session_id(agent_root: &Path, explicit: Option<String>) -> Result<String> {
    if let Some(session) = explicit {
        return Ok(session);
    }
    if let Ok(session) = std::env::var("METAGENT_SESSION") {
        if !session.is_empty() {
            return Ok(session);
        }
    }

    let sessions_dir = agent_root.join("sessions");
    let entries = match fs::read_dir(&sessions_dir) {
        Ok(entries) => entries,
        Err(_) => bail!("METAGENT_SESSION not set and no active session found"),
    };

    let mut running = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path().join("session.json");
        if !path.exists() {
            continue;
        }
        if let Ok(session) = load_session(&path) {
            if session.status == SessionStatus::Running {
                running.push(session.session_id);
            }
        }
    }

    if running.len() == 1 {
        return Ok(running.remove(0));
    }

    bail!("METAGENT_SESSION not set and no unique active session found")
}

pub fn write_task_state(path: &Path, task: &TaskState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    save_task(path, task)
}

pub fn create_task_state(
    agent_root: &Path,
    agent: &str,
    task: &str,
    stage: &str,
    added_at: &str,
) -> Result<TaskState> {
    let task_state = TaskState {
        task: task.to_string(),
        agent: agent.to_string(),
        stage: stage.to_string(),
        status: TaskStatus::Pending,
        added_at: added_at.to_string(),
        updated_at: added_at.to_string(),
        last_session: None,
        last_error: None,
    };

    let task_path = task_state_path(agent_root, task);
    write_task_state(&task_path, &task_state)?;
    Ok(task_state)
}

pub fn claim_task(agent_root: &Path, task: &str, ttl_seconds: u64, host: &str) -> Result<Option<ClaimGuard>> {
    let path = claim_path(agent_root, task);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            let claim = ClaimState {
                task: task.to_string(),
                agent: agent_root
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".into()),
                pid: std::process::id(),
                host: host.to_string(),
                started_at: now_iso(),
                ttl_seconds,
            };
            let data = serde_json::to_string_pretty(&claim)?;
            file.write_all(data.as_bytes())?;
            return Ok(Some(ClaimGuard { path }));
        }
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            // Check for stale claim
        }
        Err(err) => return Err(err.into()),
    }

    if let Ok(existing) = read_claim(&path) {
        if is_claim_stale(&existing, host) {
            fs::remove_file(&path).ok();
            if let Ok(mut file) = OpenOptions::new().write(true).create_new(true).open(&path) {
                let claim = ClaimState {
                    task: task.to_string(),
                    agent: existing.agent,
                    pid: std::process::id(),
                    host: host.to_string(),
                    started_at: now_iso(),
                    ttl_seconds,
                };
                let data = serde_json::to_string_pretty(&claim)?;
                file.write_all(data.as_bytes())?;
                return Ok(Some(ClaimGuard { path }));
            }
        }
    }

    Ok(None)
}

fn read_claim(path: &Path) -> Result<ClaimState> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read claim {}", path.display()))?;
    let claim = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse claim {}", path.display()))?;
    Ok(claim)
}

fn is_claim_stale(claim: &ClaimState, host: &str) -> bool {
    if let Ok(started_at) = chrono::DateTime::parse_from_rfc3339(&claim.started_at) {
        let elapsed = Utc::now().signed_duration_since(started_at.with_timezone(&Utc));
        if elapsed.num_seconds() as u64 > claim.ttl_seconds {
            return true;
        }
    }

    if claim.host == host {
        return !is_pid_alive(claim.pid);
    }

    false
}

fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
