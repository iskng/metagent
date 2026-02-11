use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::util::{ensure_dir, now_iso};

static ISSUE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    Resolved,
}

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "resolved" => Ok(Self::Resolved),
            other => bail!("Invalid issue status: {}", other),
        }
    }
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssuePriority {
    P0,
    P1,
    P2,
    P3,
}

impl IssuePriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        }
    }

    pub fn weight(&self) -> u8 {
        match self {
            Self::P0 => 0,
            Self::P1 => 1,
            Self::P2 => 2,
            Self::P3 => 3,
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        let normalized = value.trim().to_lowercase();
        let token = normalized.strip_prefix('p').unwrap_or(&normalized);
        match token {
            "0" => Ok(Self::P0),
            "1" => Ok(Self::P1),
            "2" => Ok(Self::P2),
            "3" => Ok(Self::P3),
            other => bail!("Invalid priority: {}", other),
        }
    }
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueType {
    Spec,
    Build,
    Bug,
    Test,
    Perf,
    Other,
}

impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Build => "build",
            Self::Bug => "bug",
            Self::Test => "test",
            Self::Perf => "perf",
            Self::Other => "other",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "spec" => Ok(Self::Spec),
            "build" => Ok(Self::Build),
            "bug" => Ok(Self::Bug),
            "test" => Ok(Self::Test),
            "perf" | "performance" => Ok(Self::Perf),
            "other" => Ok(Self::Other),
            other => bail!("Invalid issue type: {}", other),
        }
    }
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSource {
    Review,
    Debug,
    Submit,
    Manual,
}

impl IssueSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Debug => "debug",
            Self::Submit => "submit",
            Self::Manual => "manual",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "review" => Ok(Self::Review),
            "debug" => Ok(Self::Debug),
            "submit" => Ok(Self::Submit),
            "manual" => Ok(Self::Manual),
            other => bail!("Invalid issue source: {}", other),
        }
    }
}

impl std::fmt::Display for IssueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub task: Option<String>,
    pub issue_type: IssueType,
    pub source: IssueSource,
    pub created_at: String,
    pub updated_at: String,
    pub file: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueStatusFilter {
    Open,
    Resolved,
    All,
}

#[derive(Debug, Clone)]
pub struct IssueFilter {
    pub status: IssueStatusFilter,
    pub task: Option<String>,
    pub unassigned: bool,
    pub issue_type: Option<IssueType>,
    pub priority: Option<IssuePriority>,
    pub source: Option<IssueSource>,
}

#[derive(Debug, Default)]
pub struct IssueCounts {
    pub per_task: HashMap<String, usize>,
    pub unassigned: usize,
}

pub fn new_issue_id() -> String {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    let counter = ISSUE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}-{}-{}", epoch, std::process::id(), counter)
}

pub fn issues_dir(agent_root: &Path) -> PathBuf {
    agent_root.join("issues")
}

pub fn issue_path(agent_root: &Path, issue_id: &str) -> PathBuf {
    issues_dir(agent_root).join(format!("{issue_id}.md"))
}

pub fn load_issue(path: &Path) -> Result<Issue> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read issue {}", path.display()))?;
    parse_issue(&content).with_context(|| format!("Failed to parse issue {}", path.display()))
}

pub fn save_issue(path: &Path, issue: &Issue) -> Result<()> {
    let content = render_issue(issue);
    write_text_atomic(path, &content)
}

pub fn list_issues(agent_root: &Path) -> Result<Vec<Issue>> {
    let dir = issues_dir(agent_root);
    let mut issues = Vec::new();
    if !dir.exists() {
        return Ok(issues);
    }
    let entries = fs::read_dir(&dir)
        .with_context(|| format!("Failed to read issues directory {}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        match load_issue(&path) {
            Ok(issue) => issues.push(issue),
            Err(err) => {
                eprintln!("Warning: {} (skipping)", err);
            }
        }
    }
    Ok(issues)
}

pub fn filter_issues(mut issues: Vec<Issue>, filter: &IssueFilter) -> Vec<Issue> {
    issues.retain(|issue| {
        if filter.unassigned {
            if issue.task.is_some() {
                return false;
            }
        } else if let Some(task) = filter.task.as_ref() {
            if issue.task.as_deref() != Some(task.as_str()) {
                return false;
            }
        }

        if let Some(issue_type) = filter.issue_type.as_ref() {
            if &issue.issue_type != issue_type {
                return false;
            }
        }
        if let Some(priority) = filter.priority.as_ref() {
            if &issue.priority != priority {
                return false;
            }
        }
        if let Some(source) = filter.source.as_ref() {
            if &issue.source != source {
                return false;
            }
        }

        match filter.status {
            IssueStatusFilter::Open => issue.status == IssueStatus::Open,
            IssueStatusFilter::Resolved => issue.status == IssueStatus::Resolved,
            IssueStatusFilter::All => true,
        }
    });
    issues
}

pub fn sort_issues(issues: &mut [Issue]) {
    issues.sort_by(|a, b| {
        let status_weight = match a.status {
            IssueStatus::Open => 0,
            IssueStatus::Resolved => 1,
        };
        let status_weight_b = match b.status {
            IssueStatus::Open => 0,
            IssueStatus::Resolved => 1,
        };
        status_weight
            .cmp(&status_weight_b)
            .then_with(|| a.priority.weight().cmp(&b.priority.weight()))
            .then_with(|| a.created_at.cmp(&b.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

pub fn count_open_issues(issues: &[Issue]) -> IssueCounts {
    let mut counts = IssueCounts::default();
    for issue in issues.iter().filter(|i| i.status == IssueStatus::Open) {
        if let Some(task) = issue.task.as_ref() {
            *counts.per_task.entry(task.clone()).or_insert(0) += 1;
        } else {
            counts.unassigned += 1;
        }
    }
    counts
}

pub fn parse_issue(content: &str) -> Result<Issue> {
    let (frontmatter, body) = parse_frontmatter(content);
    let id = frontmatter
        .get("id")
        .cloned()
        .ok_or_else(|| anyhow!("Missing id"))?;
    let title = frontmatter
        .get("title")
        .cloned()
        .ok_or_else(|| anyhow!("Missing title"))?;
    let status = IssueStatus::from_str(
        frontmatter
            .get("status")
            .ok_or_else(|| anyhow!("Missing status"))?,
    )?;
    let priority = IssuePriority::from_str(
        frontmatter
            .get("priority")
            .ok_or_else(|| anyhow!("Missing priority"))?,
    )?;
    let issue_type = IssueType::from_str(
        frontmatter
            .get("type")
            .ok_or_else(|| anyhow!("Missing type"))?,
    )?;
    let source = IssueSource::from_str(
        frontmatter
            .get("source")
            .ok_or_else(|| anyhow!("Missing source"))?,
    )?;
    let created_at = frontmatter
        .get("created_at")
        .cloned()
        .ok_or_else(|| anyhow!("Missing created_at"))?;
    let updated_at = frontmatter
        .get("updated_at")
        .cloned()
        .ok_or_else(|| anyhow!("Missing updated_at"))?;
    let task = frontmatter.get("task").and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "-" {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let file = frontmatter.get("file").and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "-" {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let body = if body.trim().is_empty() {
        None
    } else {
        Some(body.trim().to_string())
    };

    Ok(Issue {
        id,
        title,
        status,
        priority,
        task,
        issue_type,
        source,
        created_at,
        updated_at,
        file,
        body,
    })
}

pub fn render_issue(issue: &Issue) -> String {
    let task = issue.task.as_deref().unwrap_or("-");
    let file = issue.file.as_deref().unwrap_or("-");
    let mut lines = Vec::new();
    lines.push("---".to_string());
    lines.push(format!("id: {}", issue.id));
    lines.push(format!("title: {}", issue.title));
    lines.push(format!("status: {}", issue.status));
    lines.push(format!("priority: {}", issue.priority));
    lines.push(format!("task: {}", task));
    lines.push(format!("type: {}", issue.issue_type));
    lines.push(format!("source: {}", issue.source));
    lines.push(format!("created_at: {}", issue.created_at));
    lines.push(format!("updated_at: {}", issue.updated_at));
    lines.push(format!("file: {}", file));
    lines.push("---".to_string());
    if let Some(body) = issue.body.as_ref() {
        if !body.trim().is_empty() {
            lines.push(String::new());
            lines.push(body.trim().to_string());
        }
    }
    lines.join("\n")
}

fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    let mut lines = content.lines();
    let mut frontmatter = HashMap::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = false;

    if let Some(first) = lines.next() {
        if first.trim() == "---" {
            in_frontmatter = true;
        } else {
            body_lines.push(first);
        }
    }

    if in_frontmatter {
        for line in lines.by_ref() {
            if line.trim() == "---" {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                frontmatter.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        body_lines.extend(lines);
    } else {
        body_lines.extend(lines);
    }

    (frontmatter, body_lines.join("\n"))
}

fn write_text_atomic(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "issue".into());
    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));
    fs::write(&tmp_path, content)
        .with_context(|| format!("Failed to write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn append_resolution(body: Option<String>, resolution: &str) -> String {
    let mut result = body.unwrap_or_default();
    let resolution = resolution.trim();
    if resolution.is_empty() {
        return result;
    }
    if !result.is_empty() {
        result.push('\n');
        result.push('\n');
    }
    result.push_str("## Resolution\n");
    result.push_str(resolution);
    result.trim().to_string()
}

pub fn new_issue(
    title: String,
    status: IssueStatus,
    priority: IssuePriority,
    task: Option<String>,
    issue_type: IssueType,
    source: IssueSource,
    file: Option<String>,
    body: Option<String>,
) -> Issue {
    let now = now_iso();
    Issue {
        id: new_issue_id(),
        title,
        status,
        priority,
        task,
        issue_type,
        source,
        created_at: now.clone(),
        updated_at: now,
        file,
        body,
    }
}
