use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn today_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("Failed to resolve home directory")
}

pub fn get_repo_root(start: Option<PathBuf>) -> Result<PathBuf> {
    if let Ok(root) = env::var("METAGENT_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }

    let mut dir = match start {
        Some(path) => path,
        None => env::current_dir().context("Failed to read current directory")?,
    };

    loop {
        if dir.join(".agents").is_dir() || dir.join(".git").is_dir() {
            return Ok(dir);
        }

        if !dir.pop() {
            break;
        }
    }

    bail!("No repo found (missing .agents/ or .git). Run 'metagent init' in a repo.")
}

pub fn get_agent_root(repo_root: &Path, agent: &str) -> Result<PathBuf> {
    let agents_dir = repo_root.join(".agents");
    if !agents_dir.is_dir() {
        bail!(".agents/ not found in repo. Run 'metagent init' first.");
    }

    Ok(agents_dir.join(agent))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("Failed to create directory: {}", path.display()))
}

pub fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
}

pub fn read_text(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(buf)
}

pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let reply = input.trim();
    Ok(matches!(reply, "y" | "Y"))
}

pub fn validate_task_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Task name required");
    }
    if name.len() > 100 {
        bail!("Task name too long (max 100 chars)");
    }
    if name.contains("..") || name.starts_with('.') {
        bail!("Invalid task name '{name}'");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("Invalid task name '{name}'");
    }
    Ok(())
}

pub fn task_dir(agent_root: &Path, task: &str) -> PathBuf {
    agent_root.join("tasks").join(task)
}

pub fn task_state_path(agent_root: &Path, task: &str) -> PathBuf {
    task_dir(agent_root, task).join("task.json")
}

pub fn session_dir(agent_root: &Path, session_id: &str) -> PathBuf {
    agent_root.join("sessions").join(session_id)
}

pub fn session_state_path(agent_root: &Path, session_id: &str) -> PathBuf {
    session_dir(agent_root, session_id).join("session.json")
}

pub fn claim_path(agent_root: &Path, task: &str) -> PathBuf {
    agent_root.join("claims").join(format!("{task}.lock"))
}
