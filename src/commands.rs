use anyhow::{bail, Context, Result};
use clap::Subcommand;
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::agent::AgentKind;
use crate::issues::{
    append_resolution, count_open_issues, filter_issues, issue_path, list_issues, new_issue,
    save_issue, sort_issues, IssueFilter, IssuePriority, IssueSource, IssueStatus,
    IssueStatusFilter, IssueType,
};
use crate::model::Model;
use crate::prompt::{issues_text, parallelism_text, render_prompt, PromptContext};
use crate::state::{
    claim_task, create_session, create_task_state, has_active_claim, has_active_session,
    list_tasks, load_session, load_task, save_session, update_session, update_task, SessionState,
    SessionStatus, TaskState, TaskStatus,
};
use crate::util::{
    confirm, get_agent_root, home_dir, now_iso, read_text, task_dir, task_state_path,
    validate_task_name, write_text, TerminalGuard,
};

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
fn link_prompt(target: &Path, link: &Path) -> Result<()> {
    if link.exists() {
        fs::remove_file(link).ok();
    }
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("Failed to link {}", link.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn link_prompt(target: &Path, link: &Path) -> Result<()> {
    if link.exists() {
        fs::remove_file(link).ok();
    }
    fs::copy(target, link).with_context(|| format!("Failed to copy {}", link.display()))?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ModelChoice {
    pub model: Model,
    pub explicit: bool,
    pub force_model: bool,
}

#[derive(Subcommand)]
pub enum IssueCommands {
    List {
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        unassigned: bool,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long = "type")]
        issue_type: Option<String>,
        #[arg(long)]
        source: Option<String>,
    },
    Add {
        #[arg(long)]
        title: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long = "type")]
        issue_type: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        stdin_body: bool,
    },
    Resolve {
        #[arg(help = "Issue ID (use `metagent issues` to list IDs)")]
        id: String,
        #[arg(long)]
        resolution: Option<String>,
    },
    Assign {
        #[arg(help = "Issue ID (use `metagent issues` to list IDs)")]
        id: String,
        #[arg(long)]
        task: String,
        #[arg(long)]
        stage: Option<String>,
    },
    Show {
        #[arg(help = "Issue ID (use `metagent issues` to list IDs)")]
        id: String,
    },
}

#[derive(Clone, Debug)]
pub struct CommandContext {
    pub agent: AgentKind,
    pub model_choice: ModelChoice,
    pub repo_root: PathBuf,
    pub agent_root: PathBuf,
    pub prompt_root: PathBuf,
    pub host: String,
}

impl CommandContext {
    pub fn new(agent: AgentKind, model_choice: ModelChoice, repo_root: PathBuf) -> Result<Self> {
        let agent_root = get_agent_root(&repo_root, agent.name())?;
        let prompt_root = home_dir()?.join(".metagent").join(agent.name());
        let host = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Ok(Self {
            agent,
            model_choice,
            repo_root,
            agent_root,
            prompt_root,
            host,
        })
    }
}

#[cfg(target_os = "macos")]
fn macos_detect_codesign_identity() -> Option<String> {
    let output = Command::new("security")
        .args(["find-identity", "-p", "codesigning", "-v"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut identities = Vec::new();
    for line in stdout.lines() {
        let start = line.find('"');
        let end = line.rfind('"');
        if let (Some(start), Some(end)) = (start, end) {
            if end > start {
                identities.push(line[start + 1..end].to_string());
            }
        }
    }
    if identities.is_empty() {
        return None;
    }
    for prefix in ["Developer ID Application:", "Developer ID:"] {
        if let Some(identity) = identities.iter().find(|id| id.starts_with(prefix)) {
            return Some(identity.clone());
        }
    }
    identities.into_iter().next()
}

#[cfg(target_os = "macos")]
fn macos_run_codesign(dest: &Path, identity: Option<&str>) -> bool {
    let mut cmd = Command::new("codesign");
    cmd.arg("--force")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match identity {
        Some(identity) => {
            cmd.args(["--options", "runtime", "--timestamp", "-s", identity]);
        }
        None => {
            cmd.args(["-s", "-"]);
        }
    }
    cmd.arg(dest)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn macos_post_install(dest: &Path) {
    if env::var_os("METAGENT_SKIP_CODESIGN").is_some() {
        return;
    }

    let _ = Command::new("xattr")
        .args(["-d", "com.apple.quarantine"])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("xattr")
        .args(["-d", "com.apple.provenance"])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let explicit_identity = env::var("METAGENT_CODESIGN_ID").ok();
    let detected_identity = explicit_identity
        .clone()
        .or_else(macos_detect_codesign_identity);
    let mut signed = macos_run_codesign(dest, detected_identity.as_deref());
    if !signed && explicit_identity.is_none() && detected_identity.is_some() {
        signed = macos_run_codesign(dest, None);
    }
    if matches!(explicit_identity, Some(_)) && !signed {
        eprintln!("Warning: codesign failed for {}.", dest.display());
    }
}

#[cfg(not(target_os = "macos"))]
fn macos_post_install(_: &Path) {}

pub fn cmd_install() -> Result<()> {
    let home = home_dir()?;
    let bin_dir = home.join(".local/bin");
    fs::create_dir_all(&bin_dir)?;
    let exe = env::current_exe().context("Unable to locate current executable")?;
    let dest = bin_dir.join("metagent");
    fs::copy(&exe, &dest).context("Failed to install metagent binary")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;
    }

    macos_post_install(&dest);

    // Verify the installed binary works (catches macOS code signing issues)
    let verify = Command::new(&dest)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match verify {
        Ok(status) if !status.success() => {
            let code = status.code().unwrap_or(-1);
            if code == 137 || code == -9 {
                bail!(
                    "Installed binary was killed (exit {}). This may be a macOS code signing issue.\n\
                     Try: xattr -cr {} && codesign -s - {}",
                    code, dest.display(), dest.display()
                );
            }
        }
        Err(e) => {
            bail!("Failed to verify installed binary: {}", e);
        }
        _ => {}
    }

    let metagent_dir = home.join(".metagent");
    for agent in [AgentKind::Code, AgentKind::Writer] {
        let agent_dir = metagent_dir.join(agent.name());
        fs::create_dir_all(&agent_dir)?;
        for (file, content) in agent.install_prompts() {
            write_text(&agent_dir.join(file), content)?;
        }
    }

    let claude_commands = home.join(".claude/commands");
    let codex_commands = home.join(".codex/prompts");
    for dir in [&claude_commands, &codex_commands] {
        fs::create_dir_all(dir)?;
    }
    for agent in [AgentKind::Code, AgentKind::Writer] {
        let prompt_dir = metagent_dir.join(agent.name());
        for (prompt_file, command_name) in agent.slash_commands() {
            let target = prompt_dir.join(prompt_file);
            if !target.exists() {
                continue;
            }
            for commands_dir in [&claude_commands, &codex_commands] {
                let link = commands_dir.join(format!("{command_name}.md"));
                link_prompt(&target, &link)?;
            }
        }
    }

    if let Ok(path) = env::var("PATH") {
        let bin_str = bin_dir.display().to_string();
        if !path.split(':').any(|entry| entry == bin_str) {
            println!("Note: {} is not in your PATH", bin_dir.display());
            println!("Add this to your shell profile:");
            println!("  export PATH=\"$HOME/.local/bin:$PATH\"");
        }
    }

    println!("Installed metagent to {}", dest.display());
    Ok(())
}

pub fn cmd_uninstall() -> Result<()> {
    let home = home_dir()?;
    let bin_dir = home.join(".local/bin/metagent");
    let metagent_dir = home.join(".metagent");
    let claude_commands = home.join(".claude/commands");
    let codex_commands = home.join(".codex/prompts");

    if bin_dir.exists() {
        fs::remove_file(&bin_dir)?;
        println!("Removed {}", bin_dir.display());
    }

    for dir in [&claude_commands, &codex_commands] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Ok(target) = fs::read_link(&path) {
                if target.starts_with(&metagent_dir) {
                    fs::remove_file(&path)?;
                }
            }
        }
    }

    if metagent_dir.exists() {
        fs::remove_dir_all(&metagent_dir)?;
        println!("Removed {}", metagent_dir.display());
    }

    Ok(())
}

pub fn cmd_init(
    agent: AgentKind,
    target: Option<PathBuf>,
    model_choice: ModelChoice,
) -> Result<()> {
    let target = match target {
        Some(path) => fs::canonicalize(path)?,
        None => env::current_dir()?,
    };

    if !target.join(".git").is_dir() {
        let proceed = confirm("Warning: Target is not a git repository. Continue? (y/N) ")?;
        if !proceed {
            println!("Aborted.");
            return Ok(());
        }
    }

    let agent_dir = target.join(".agents").join(agent.name());
    let mut overwrite = false;
    if agent_dir.exists() {
        overwrite = confirm(&format!(
            "Warning: .agents/{}/ already exists. Overwrite templates? (y/N) ",
            agent.name()
        ))?;
        if !overwrite {
            println!("Aborted.");
            return Ok(());
        }
    }

    fs::create_dir_all(agent_dir.join("tasks"))?;
    if agent == AgentKind::Code {
        fs::create_dir_all(agent_dir.join("issues"))?;
    }
    for (file, content) in agent.template_files() {
        let dest = agent_dir.join(file);
        if dest.exists() && !overwrite {
            continue;
        }
        write_text(&dest, content)?;
    }

    println!("Initialized {} agent in {}", agent.name(), target.display());

    if agent == AgentKind::Code {
        let ctx = CommandContext::new(agent, model_choice, target)?;
        if bootstrap_needed(&ctx.agent_root)? {
            println!("Bootstrap not detected. Running bootstrap prompt...");
            run_bootstrap(&ctx)?;
        }
    }
    Ok(())
}

pub fn cmd_task(
    ctx: &CommandContext,
    task: &str,
    hold: bool,
    description: Option<String>,
) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    let task_dir_path = task_dir(&ctx.agent_root, task);

    if task_path.exists() {
        if let Some(description) = description.as_ref() {
            update_task(&task_path, |task_state| {
                task_state.description = Some(description.clone());
                task_state.updated_at = now_iso();
                Ok(())
            })?;
        }
        let task_state = load_task(&task_path)?;
        println!("Task '{}' already exists", task);
        println!("  Stage: {}", task_state.stage);
        if task_state.held {
            println!("  Status: held (backlog)");
        }
        if let Some(description) = task_state.description.as_ref() {
            println!("  Description: {}", description);
        } else {
            println!("  Description: (none)");
        }
        let history = build_task_history(&ctx.agent_root, task)?;
        if history.is_empty() {
            println!("  History: (none yet)");
        } else {
            println!("  History: {}", history);
        }
        println!("  Directory: {}", task_dir_path.display());
        return Ok(());
    }

    ctx.agent.create_task(&task_dir_path, task)?;
    let timestamp = now_iso();
    create_task_state(
        &ctx.agent_root,
        ctx.agent.name(),
        task,
        ctx.agent.initial_stage(),
        &timestamp,
        hold,
        description.clone(),
    )?;

    println!("Created task: {}", task);
    println!("  Directory: {}", task_dir_path.display());
    println!("  Stage: {}", ctx.agent.initial_stage());
    if hold {
        println!("  Status: held (backlog)");
    }
    if let Some(description) = description {
        println!("  Description: {}", description);
    }
    Ok(())
}

pub fn cmd_hold(ctx: &CommandContext, task: &str) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    update_task(&task_path, |task_state| {
        if task_state.status == TaskStatus::Running {
            bail!("Task '{}' is running. Finish it before holding.", task);
        }
        task_state.held = true;
        task_state.updated_at = now_iso();
        Ok(())
    })?;
    println!("Held '{}'", task);
    Ok(())
}

pub fn cmd_activate(ctx: &CommandContext, task: &str) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    update_task(&task_path, |task_state| {
        task_state.held = false;
        task_state.updated_at = now_iso();
        Ok(())
    })?;
    sync_task_status_for_issues(&ctx.agent_root, task)?;
    println!("Activated '{}'", task);
    Ok(())
}

pub fn cmd_queue(ctx: &CommandContext, task: Option<&str>) -> Result<()> {
    if let Some(task) = task {
        validate_task_name(task)?;
        let task_path = task_state_path(&ctx.agent_root, task);
        if task_path.exists() {
            let task_state = load_task(&task_path)?;
            println!("Task '{}' already exists", task);
            println!("  Stage: {}", task_state.stage);
            if task_state.held {
                println!("  Status: held (backlog)");
            }
            return Ok(());
        }

        let dir = task_dir(&ctx.agent_root, task);
        if !dir.exists() {
            bail!(
                "Task '{}' not found. Create it with 'metagent task {}'",
                task,
                task
            );
        }

        let timestamp = now_iso();
        create_task_state(
            &ctx.agent_root,
            ctx.agent.name(),
            task,
            ctx.agent.initial_stage(),
            &timestamp,
            false,
            None,
        )?;
        println!("Queued '{}' (stage: {})", task, ctx.agent.initial_stage());
        return Ok(());
    }

    let tasks = list_tasks(&ctx.agent_root);
    if tasks.is_empty() {
        println!("{}", "No tasks".dimmed());
        return Ok(());
    }

    let issue_counts = match list_issues(&ctx.agent_root) {
        Ok(issues) => count_open_issues(&issues),
        Err(err) => {
            eprintln!("Warning: failed to load issues: {}", err);
            Default::default()
        }
    };
    if issue_counts.unassigned > 0 {
        println!(
            "Unassigned issues: {} (run 'metagent issues --unassigned')",
            issue_counts.unassigned
        );
    }

    let mut backlog: Vec<&TaskState> = tasks.iter().filter(|t| t.held).collect();
    println!("{}", "Tasks:".bold());
    for stage in ctx.agent.stages() {
        if *stage == "completed" {
            continue;
        }
        let mut stage_tasks: Vec<&TaskState> = tasks
            .iter()
            .filter(|t| !t.held && t.stage == *stage)
            .collect();
        if stage_tasks.is_empty() {
            continue;
        }
        if *stage == "build" {
            stage_tasks.sort_by(|a, b| {
                let ar = a.queue_rank.unwrap_or(i64::MAX);
                let br = b.queue_rank.unwrap_or(i64::MAX);
                ar.cmp(&br).then_with(|| a.added_at.cmp(&b.added_at))
            });
        } else {
            stage_tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));
        }
        println!("{}:", ctx.agent.stage_label(stage));
        for task in stage_tasks {
            let issue_count = issue_counts.per_task.get(&task.task).copied().unwrap_or(0);
            if issue_count > 0 {
                println!(
                    "  {} {} [issues: {}]",
                    task.status.styled(),
                    task.task,
                    issue_count
                );
            } else {
                println!("  {} {}", task.status.styled(), task.task);
            }
        }
        println!();
    }

    let mut completed: Vec<&TaskState> = tasks
        .iter()
        .filter(|t| !t.held && t.stage == "completed")
        .collect();
    if !completed.is_empty() {
        completed.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let total_completed = completed.len();
        println!("{}:", ctx.agent.stage_label("completed").dimmed());
        for task in completed.into_iter().take(10) {
            let issue_count = issue_counts.per_task.get(&task.task).copied().unwrap_or(0);
            if issue_count > 0 {
                println!(
                    "  {} {} [issues: {}]",
                    task.status.styled(),
                    task.task.dimmed(),
                    issue_count
                );
            } else {
                println!("  {} {}", task.status.styled(), task.task.dimmed());
            }
        }
        if total_completed > 10 {
            println!("  ... and {} more", total_completed - 10);
        }
    }

    if !backlog.is_empty() {
        backlog.sort_by(|a, b| a.added_at.cmp(&b.added_at));
        println!("\nBacklog:");
        for task in backlog {
            let issue_count = issue_counts.per_task.get(&task.task).copied().unwrap_or(0);
            if issue_count > 0 {
                println!(
                    "  {} {} [issues: {}] (stage: {})",
                    task.status.styled(),
                    task.task,
                    issue_count,
                    ctx.agent.stage_label(&task.stage)
                );
            } else {
                println!(
                    "  {} {} (stage: {})",
                    task.status.styled(),
                    task.task,
                    ctx.agent.stage_label(&task.stage)
                );
            }
        }
    }

    Ok(())
}

pub fn cmd_plan(ctx: &CommandContext, task: &str) -> Result<()> {
    validate_task_name(task)?;
    let file_name = if ctx.agent == AgentKind::Code {
        "plan.md"
    } else {
        "editorial_plan.md"
    };
    let plan_path = task_dir(&ctx.agent_root, task).join(file_name);
    if !plan_path.exists() {
        bail!(
            "{} not found for task '{}': {}",
            file_name,
            task,
            plan_path.display()
        );
    }

    let content = read_text(&plan_path)?;
    let mut canonical_steps = Vec::new();
    let mut checklist_steps = Vec::new();
    let mut id_lines: HashMap<u32, Vec<usize>> = HashMap::new();

    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if let Some(step) = parse_canonical_plan_step(line, line_number) {
            id_lines.entry(step.id).or_default().push(line_number);
            canonical_steps.push(step);
            continue;
        }
        if let Some(step) = parse_checklist_step(line, line_number) {
            checklist_steps.push(step);
        }
    }

    if canonical_steps.is_empty() && checklist_steps.is_empty() {
        println!(
            "{}",
            format!("No checklist steps found in {}", plan_path.display()).dimmed()
        );
        return Ok(());
    }

    println!("Plan '{}': {}", task, plan_path.display());
    let mut open = 0usize;
    let mut done = 0usize;

    if !canonical_steps.is_empty() {
        println!("Canonical steps:");
        for step in &canonical_steps {
            let marker = if step.done { "x" } else { " " };
            if step.done {
                done += 1;
            } else {
                open += 1;
            }
            println!(
                "  L{} - [{}] [{}][{}][T{}] {}",
                step.line, marker, step.priority, step.complexity, step.id, step.title
            );
        }
    }

    if !checklist_steps.is_empty() {
        println!("Other checklist lines:");
        for step in &checklist_steps {
            let marker = if step.done { "x" } else { " " };
            if step.done {
                done += 1;
            } else {
                open += 1;
            }
            println!("  L{} - [{}] {}", step.line, marker, step.title);
        }
    }

    let total = open + done;
    println!();
    println!("Summary: {} total ({} open, {} done)", total, open, done);

    let mut duplicates: Vec<(u32, Vec<usize>)> = id_lines
        .into_iter()
        .filter_map(|(id, mut lines)| {
            if lines.len() <= 1 {
                return None;
            }
            lines.sort_unstable();
            Some((id, lines))
        })
        .collect();
    duplicates.sort_by_key(|(id, _)| *id);
    if !duplicates.is_empty() {
        println!();
        println!("Warnings:");
        for (id, lines) in duplicates {
            let joined = lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("  duplicate T{} at lines {}", id, joined);
        }
    }

    Ok(())
}

pub fn cmd_issues(
    ctx: &CommandContext,
    task: Option<String>,
    unassigned: bool,
    status: Option<String>,
    priority: Option<String>,
    issue_type: Option<String>,
    source: Option<String>,
) -> Result<()> {
    ensure_code_agent(ctx)?;
    if unassigned && task.is_some() {
        bail!("Use --task or --unassigned, not both");
    }
    if let Some(task) = task.as_deref() {
        validate_task_name(task)?;
    }
    let status_filter = parse_status_filter(status.as_deref())?;
    let priority = parse_priority(priority.as_deref())?;
    let issue_type = parse_issue_type(issue_type.as_deref())?;
    let source = parse_issue_source(source.as_deref())?;

    let filter = IssueFilter {
        status: status_filter,
        task,
        unassigned,
        issue_type,
        priority,
        source,
    };

    let issues = list_issues(&ctx.agent_root)?;
    let mut issues = filter_issues(issues, &filter);
    sort_issues(&mut issues);

    if issues.is_empty() {
        println!("{}", "No issues".dimmed());
        return Ok(());
    }

    let heading = match status_filter {
        IssueStatusFilter::Open => "Open issues",
        IssueStatusFilter::Resolved => "Resolved issues",
        IssueStatusFilter::All => "Issues",
    };
    println!("{}:", heading);
    for (index, issue) in issues.iter().enumerate() {
        let task_label = issue.task.as_deref().unwrap_or("unassigned");
        println!("  id: {}", issue.id);
        println!("  [{}] {}: {}", issue.priority, task_label, issue.title);
        if status_filter == IssueStatusFilter::All {
            println!("      status: {}", issue.status);
        }
        if index + 1 < issues.len() {
            println!();
        }
    }
    Ok(())
}

pub fn cmd_issue(ctx: &CommandContext, command: IssueCommands) -> Result<()> {
    ensure_code_agent(ctx)?;
    match command {
        IssueCommands::List {
            task,
            unassigned,
            status,
            priority,
            issue_type,
            source,
        } => cmd_issues(ctx, task, unassigned, status, priority, issue_type, source),
        IssueCommands::Add {
            title,
            task,
            priority,
            issue_type,
            source,
            file,
            stage,
            body,
            stdin_body,
        } => cmd_issue_add(
            ctx, title, task, priority, issue_type, source, file, stage, body, stdin_body,
        ),
        IssueCommands::Resolve { id, resolution } => cmd_issue_resolve(ctx, &id, resolution),
        IssueCommands::Assign { id, task, stage } => cmd_issue_assign(ctx, &id, &task, stage),
        IssueCommands::Show { id } => cmd_issue_show(ctx, &id),
    }
}

pub fn cmd_delete(ctx: &CommandContext, task: &str, force: bool) -> Result<()> {
    validate_task_name(task)?;
    let dir = task_dir(&ctx.agent_root, task);
    if !dir.exists() {
        println!("Task '{}' not found", task);
        return Ok(());
    }

    let issues = list_issues(&ctx.agent_root)?;
    let open_issue_ids: Vec<_> = issues
        .iter()
        .filter(|issue| issue.status == IssueStatus::Open && issue.task.as_deref() == Some(task))
        .map(|issue| issue.id.clone())
        .collect();

    if !open_issue_ids.is_empty() && !force {
        bail!(
            "Task '{}' has open issues ({}). Re-run with --force to delete and unassign them.",
            task,
            open_issue_ids.len()
        );
    }

    if force && !open_issue_ids.is_empty() {
        for mut issue in issues {
            if issue.status == IssueStatus::Open && issue.task.as_deref() == Some(task) {
                issue.task = None;
                issue.updated_at = now_iso();
                let path = issue_path(&ctx.agent_root, &issue.id);
                save_issue(&path, &issue)?;
            }
        }
    }

    fs::remove_dir_all(&dir)?;
    println!("Removed '{}'", task);
    Ok(())
}

pub fn cmd_reorder(ctx: &CommandContext, task: &str, position: usize) -> Result<()> {
    validate_task_name(task)?;
    if position == 0 {
        bail!("Position must be 1 or greater");
    }
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    let task_state = load_task(&task_path)?;
    if task_state.stage != "build" {
        bail!("Reorder is only supported for build stage tasks");
    }
    if task_state.held {
        bail!("Task '{}' is held. Activate it before reordering.", task);
    }

    let mut stage_tasks: Vec<TaskState> = list_tasks(&ctx.agent_root)
        .into_iter()
        .filter(|t| !t.held && t.stage == "build")
        .collect();
    if stage_tasks.is_empty() {
        bail!("No build tasks to reorder");
    }

    stage_tasks.sort_by(|a, b| {
        let ar = a.queue_rank.unwrap_or(i64::MAX);
        let br = b.queue_rank.unwrap_or(i64::MAX);
        ar.cmp(&br).then_with(|| a.added_at.cmp(&b.added_at))
    });

    let current_index = stage_tasks
        .iter()
        .position(|t| t.task == task)
        .ok_or_else(|| anyhow::anyhow!("Task '{}' is not in the build queue", task))?;

    let mut ordered = Vec::with_capacity(stage_tasks.len());
    for (idx, item) in stage_tasks.into_iter().enumerate() {
        if idx != current_index {
            ordered.push(item);
        }
    }
    let insert_index = std::cmp::min(position - 1, ordered.len());
    ordered.insert(insert_index, task_state);

    for (idx, item) in ordered.iter().enumerate() {
        let new_rank = (idx + 1) as i64;
        if item.queue_rank == Some(new_rank) {
            continue;
        }
        let path = task_state_path(&ctx.agent_root, &item.task);
        update_task(&path, |task_state| {
            task_state.queue_rank = Some(new_rank);
            task_state.updated_at = now_iso();
            Ok(())
        })?;
    }

    println!(
        "Reordered '{}' to position {} in build queue.",
        task,
        insert_index + 1
    );
    let mut build_tasks: Vec<TaskState> = list_tasks(&ctx.agent_root)
        .into_iter()
        .filter(|t| !t.held && t.stage == "build")
        .collect();
    build_tasks.sort_by(|a, b| {
        let ar = a.queue_rank.unwrap_or(i64::MAX);
        let br = b.queue_rank.unwrap_or(i64::MAX);
        ar.cmp(&br).then_with(|| a.added_at.cmp(&b.added_at))
    });
    let issue_counts = match list_issues(&ctx.agent_root) {
        Ok(issues) => count_open_issues(&issues),
        Err(err) => {
            eprintln!("Warning: failed to load issues: {}", err);
            Default::default()
        }
    };
    println!("{}:", ctx.agent.stage_label("build"));
    for task in build_tasks {
        let issue_count = issue_counts.per_task.get(&task.task).copied().unwrap_or(0);
        if issue_count > 0 {
            println!(
                "  {} {} [issues: {}]",
                task.status.styled(),
                task.task,
                issue_count
            );
        } else {
            println!("  {} {}", task.status.styled(), task.task);
        }
    }
    Ok(())
}

pub fn cmd_start(ctx: &CommandContext) -> Result<()> {
    let mut task_name: Option<String> = None;
    let mut stage = ctx.agent.initial_stage().to_string();
    let handoff_stage = ctx.agent.handoff_stage();

    loop {
        if let Some(task) = task_name.as_ref() {
            let task_path = task_state_path(&ctx.agent_root, task);
            if task_path.exists() {
                update_task(&task_path, |task_state| {
                    // Preserve Issues status so issue injection works in run_stage
                    if task_state.status != TaskStatus::Issues {
                        task_state.status = TaskStatus::Running;
                    }
                    task_state.updated_at = now_iso();
                    Ok(())
                })?;
            }
        }

        let result = run_stage(
            ctx,
            task_name.as_deref(),
            &stage,
            None,
            ReviewFinishMode::Queue,
        )?;
        match result {
            StageResult::Finished(session) => {
                if task_name.is_none() {
                    if let Some(task) = session.task.clone() {
                        task_name = Some(task);
                    }
                }
                let next_stage = session
                    .next_stage
                    .clone()
                    .or_else(|| ctx.agent.next_stage(&stage).map(|s| s.to_string()));
                if let Some(next_stage) = next_stage {
                    if let Some(handoff) = handoff_stage {
                        if next_stage == handoff {
                            if let Some(task) = task_name.as_ref() {
                                println!("Task '{}' is ready.", task);
                                println!(
                                    "Run 'metagent run {}' or 'metagent run-queue' to start.",
                                    task
                                );
                            }
                            return Ok(());
                        }
                    }
                    if next_stage == "completed" {
                        println!("Task completed.");
                        return Ok(());
                    }
                    stage = next_stage;
                    continue;
                }

                bail!("No next stage provided.");
            }
            StageResult::Interrupted => {
                if let Some(task) = task_name.as_ref() {
                    let task_path = task_state_path(&ctx.agent_root, task);
                    if task_path.exists() {
                        update_task(&task_path, |task_state| {
                            task_state.status = TaskStatus::Incomplete;
                            task_state.updated_at = now_iso();
                            Ok(())
                        })?;
                    }
                }
                return Ok(());
            }
            StageResult::NoFinish => {
                if let Some(task) = task_name.as_ref() {
                    let task_path = task_state_path(&ctx.agent_root, task);
                    if task_path.exists() {
                        update_task(&task_path, |task_state| {
                            task_state.status = TaskStatus::Failed;
                            task_state.updated_at = now_iso();
                            Ok(())
                        })?;
                    }
                    bail!("Task '{}' exited without completing stage {}", task, stage);
                } else {
                    bail!("Interview ended without creating a task");
                }
            }
        }
    }
}

pub fn cmd_run(ctx: &CommandContext, task: &str) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!(
            "Task '{}' not found. Run 'metagent queue {}' to add it first.",
            task,
            task
        );
    }
    reconcile_running_tasks(&ctx.agent_root)?;
    let claim = claim_task(&ctx.agent_root, task, 3600, &ctx.host)?;
    let Some(_guard) = claim else {
        bail!("Task '{}' is already claimed.", task);
    };

    loop {
        let task_state = load_task(&task_path)?;
        if task_state.stage == "completed" {
            println!("Task '{}' completed.", task);
            return Ok(());
        }

        if task_state.held {
            update_task(&task_path, |task_state| {
                task_state.held = false;
                task_state.updated_at = now_iso();
                Ok(())
            })?;
            println!("Activating held task '{}'", task);
        }

        update_task(&task_path, |task_state| {
            // Preserve Issues status so issue injection works in run_stage
            if task_state.status != TaskStatus::Issues {
                task_state.status = TaskStatus::Running;
            }
            task_state.updated_at = now_iso();
            Ok(())
        })?;

        let result = run_stage(
            ctx,
            Some(task),
            &task_state.stage,
            None,
            ReviewFinishMode::Queue,
        )?;
        match result {
            StageResult::Finished(_) => continue,
            StageResult::Interrupted => {
                update_task(&task_path, |task_state| {
                    task_state.status = TaskStatus::Incomplete;
                    task_state.updated_at = now_iso();
                    Ok(())
                })?;
                return Ok(());
            }
            StageResult::NoFinish => {
                update_task(&task_path, |task_state| {
                    task_state.status = TaskStatus::Incomplete;
                    task_state.updated_at = now_iso();
                    Ok(())
                })?;
                println!("Session ended. Run 'metagent run {}' to continue.", task);
                return Ok(());
            }
        }
    }
}

pub fn cmd_run_queue(ctx: &CommandContext, loop_limit: usize) -> Result<()> {
    let tasks = list_tasks(&ctx.agent_root);
    if tasks.is_empty() {
        println!("No tasks");
        return Ok(());
    }
    reconcile_running_tasks(&ctx.agent_root)?;

    let mut current_task: Option<String> = None;
    let mut current_claim: Option<crate::state::ClaimGuard> = None;
    let mut review_loops = 0usize;
    let loop_limit = if loop_limit == 0 { 100 } else { loop_limit };

    loop {
        if let Some(task_name) = current_task.clone() {
            let task_path = task_state_path(&ctx.agent_root, &task_name);
            if !task_path.exists() {
                current_task = None;
                current_claim = None;
                continue;
            }
            let task_state = load_task(&task_path)?;
            if task_state.held {
                current_task = None;
                current_claim = None;
                continue;
            }
            if task_state.stage == "completed" {
                current_task = None;
                current_claim = None;
                continue;
            }
            if !ctx
                .agent
                .queue_stages()
                .contains(&task_state.stage.as_str())
            {
                println!(
                    "Task '{}' moved to stage '{}' (not handled by run-queue).",
                    task_state.task, task_state.stage
                );
                return Ok(());
            }
            if current_claim.is_none() {
                let claim = claim_task(&ctx.agent_root, &task_state.task, 3600, &ctx.host)?;
                let Some(guard) = claim else {
                    println!("Task '{}' is already claimed.", task_state.task);
                    return Ok(());
                };
                current_claim = Some(guard);
            }

            update_task(&task_path, |task_state| {
                // Preserve Issues status so issue injection works in run_stage
                if task_state.status != TaskStatus::Issues {
                    task_state.status = TaskStatus::Running;
                }
                task_state.updated_at = now_iso();
                Ok(())
            })?;

            let stage_name = task_state.stage.clone();
            let result = run_stage(
                ctx,
                Some(&task_state.task),
                &task_state.stage,
                None,
                ReviewFinishMode::Queue,
            )?;
            match result {
                StageResult::Finished(_) => {
                    if stage_name == "review" {
                        let task_state = load_task(&task_path)?;
                        if task_state.stage == "build" {
                            review_loops += 1;
                            if review_loops >= loop_limit {
                                update_task(&task_path, |task_state| {
                                    task_state.held = true;
                                    task_state.updated_at = now_iso();
                                    Ok(())
                                })?;
                                println!(
                                    "Task '{}' exceeded review/build loop limit ({}); moving to backlog.",
                                    task_state.task, loop_limit
                                );
                                current_task = None;
                                current_claim = None;
                                review_loops = 0;
                                continue;
                            }
                        }
                    }
                    continue;
                }
                StageResult::Interrupted => {
                    update_task(&task_path, |task_state| {
                        task_state.status = TaskStatus::Incomplete;
                        task_state.updated_at = now_iso();
                        Ok(())
                    })?;
                    return Ok(());
                }
                StageResult::NoFinish => {
                    update_task(&task_path, |task_state| {
                        task_state.status = TaskStatus::Failed;
                        task_state.updated_at = now_iso();
                        Ok(())
                    })?;
                    return Ok(());
                }
            }
        }

        let tasks = list_tasks(&ctx.agent_root);
        let Some(task_state) = next_eligible_task(ctx.agent, &tasks) else {
            println!("Queue processing complete.");
            return Ok(());
        };

        let claim = claim_task(&ctx.agent_root, &task_state.task, 3600, &ctx.host)?;
        let Some(guard) = claim else {
            continue;
        };
        current_claim = Some(guard);
        current_task = Some(task_state.task);
        review_loops = 0;
    }
}

pub fn cmd_run_next(ctx: &CommandContext, task: Option<&str>) -> Result<()> {
    let tasks = list_tasks(&ctx.agent_root);
    if tasks.is_empty() {
        println!("No tasks");
        return Ok(());
    }
    reconcile_running_tasks(&ctx.agent_root)?;

    if let Some(task) = task {
        validate_task_name(task)?;
        let task_path = task_state_path(&ctx.agent_root, task);
        if !task_path.exists() {
            bail!("Task '{}' not found", task);
        }
        let task_state = load_task(&task_path)?;
        if task_state.stage == "completed" {
            println!("Task '{}' completed.", task);
            return Ok(());
        }
        if task_state.status == TaskStatus::Running {
            bail!("Task '{}' is currently running", task);
        }
        if task_state.held {
            update_task(&task_path, |task_state| {
                task_state.held = false;
                task_state.updated_at = now_iso();
                Ok(())
            })?;
            println!("Activating held task '{}'", task);
        }
        update_task(&task_path, |task_state| {
            // Preserve Issues status so issue injection works in run_stage
            if task_state.status != TaskStatus::Issues {
                task_state.status = TaskStatus::Running;
            }
            task_state.updated_at = now_iso();
            Ok(())
        })?;

        let result = run_stage(
            ctx,
            Some(task),
            &task_state.stage,
            None,
            ReviewFinishMode::Queue,
        )?;
        match result {
            StageResult::Finished(_) => {}
            StageResult::Interrupted => {
                update_task(&task_path, |task_state| {
                    task_state.status = TaskStatus::Incomplete;
                    task_state.updated_at = now_iso();
                    Ok(())
                })?;
            }
            StageResult::NoFinish => {
                update_task(&task_path, |task_state| {
                    task_state.status = TaskStatus::Failed;
                    task_state.updated_at = now_iso();
                    Ok(())
                })?;
            }
        }
        return Ok(());
    }

    let tasks = list_tasks(&ctx.agent_root);
    let Some(task_state) = next_eligible_task(ctx.agent, &tasks) else {
        println!("No eligible tasks.");
        return Ok(());
    };

    let claim = claim_task(&ctx.agent_root, &task_state.task, 3600, &ctx.host)?;
    let Some(_guard) = claim else {
        println!("Task '{}' is already claimed.", task_state.task);
        return Ok(());
    };

    let task_path = task_state_path(&ctx.agent_root, &task_state.task);
    update_task(&task_path, |task_state| {
        // Preserve Issues status so issue injection works in run_stage
        if task_state.status != TaskStatus::Issues {
            task_state.status = TaskStatus::Running;
        }
        task_state.updated_at = now_iso();
        Ok(())
    })?;

    let result = run_stage(
        ctx,
        Some(&task_state.task),
        &task_state.stage,
        None,
        ReviewFinishMode::Queue,
    )?;
    match result {
        StageResult::Finished(_) => {}
        StageResult::Interrupted => {
            update_task(&task_path, |task_state| {
                task_state.status = TaskStatus::Incomplete;
                task_state.updated_at = now_iso();
                Ok(())
            })?;
        }
        StageResult::NoFinish => {
            update_task(&task_path, |task_state| {
                task_state.status = TaskStatus::Failed;
                task_state.updated_at = now_iso();
                Ok(())
            })?;
        }
    }

    Ok(())
}

fn cmd_issue_add(
    ctx: &CommandContext,
    title: String,
    task: Option<String>,
    priority: Option<String>,
    issue_type: Option<String>,
    source: Option<String>,
    file: Option<String>,
    stage: Option<String>,
    body: Option<String>,
    stdin_body: bool,
) -> Result<()> {
    if stdin_body && body.is_some() {
        bail!("Use --body or --stdin-body, not both");
    }
    if title.trim().is_empty() {
        bail!("Issue title cannot be empty");
    }
    let body = if stdin_body {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    } else {
        body.unwrap_or_default()
    };
    let body = if body.trim().is_empty() {
        None
    } else {
        Some(body.trim().to_string())
    };

    let priority = parse_priority(priority.as_deref())?.unwrap_or(IssuePriority::P2);
    let issue_type = parse_issue_type(issue_type.as_deref())?.unwrap_or(IssueType::Build);
    let source = parse_issue_source(source.as_deref())?.unwrap_or(IssueSource::Manual);
    let task = if let Some(task) = task {
        validate_task_name(&task)?;
        Some(task)
    } else {
        None
    };

    let issue = new_issue(
        title,
        IssueStatus::Open,
        priority,
        task.clone(),
        issue_type.clone(),
        source,
        file,
        body,
    );
    let path = issue_path(&ctx.agent_root, &issue.id);
    crate::issues::save_issue(&path, &issue)?;

    if let Some(task) = task {
        if let Some(stage) = stage.as_deref() {
            validate_issue_stage(ctx.agent, stage)?;
        }
        let default_stage = issue_default_stage(ctx.agent, &issue_type);
        update_task_for_issue(
            &ctx.agent_root,
            &task,
            stage.as_deref(),
            default_stage.as_deref(),
        )?;
    }

    println!("Created issue {}", issue.id);
    Ok(())
}

fn cmd_issue_resolve(ctx: &CommandContext, id: &str, resolution: Option<String>) -> Result<()> {
    let path = issue_path(&ctx.agent_root, id);
    if !path.exists() {
        bail!(
            "Issue '{}' not found (run `metagent issues` to list IDs)",
            id
        );
    }
    let mut issue = crate::issues::load_issue(&path)?;
    issue.status = IssueStatus::Resolved;
    issue.updated_at = now_iso();
    if let Some(resolution) = resolution {
        issue.body = Some(append_resolution(issue.body.take(), &resolution));
    }
    crate::issues::save_issue(&path, &issue)?;

    if let Some(task) = issue.task.as_ref() {
        sync_task_status_for_issues(&ctx.agent_root, task)?;
    }

    println!("Resolved issue {}", id);
    Ok(())
}

fn cmd_issue_assign(
    ctx: &CommandContext,
    id: &str,
    task: &str,
    stage: Option<String>,
) -> Result<()> {
    validate_task_name(task)?;
    let path = issue_path(&ctx.agent_root, id);
    if !path.exists() {
        bail!(
            "Issue '{}' not found (run `metagent issues` to list IDs)",
            id
        );
    }
    let mut issue = crate::issues::load_issue(&path)?;
    issue.task = Some(task.to_string());
    issue.updated_at = now_iso();
    crate::issues::save_issue(&path, &issue)?;

    if issue.status == IssueStatus::Resolved {
        println!("Assigned resolved issue {} to {}", id, task);
        return Ok(());
    }

    if let Some(stage) = stage.as_deref() {
        validate_issue_stage(ctx.agent, stage)?;
    }
    let default_stage = issue_default_stage(ctx.agent, &issue.issue_type);
    update_task_for_issue(
        &ctx.agent_root,
        task,
        stage.as_deref(),
        default_stage.as_deref(),
    )?;
    println!("Assigned issue {} to {}", id, task);
    Ok(())
}

fn cmd_issue_show(ctx: &CommandContext, id: &str) -> Result<()> {
    let path = issue_path(&ctx.agent_root, id);
    if !path.exists() {
        bail!(
            "Issue '{}' not found (run `metagent issues` to list IDs)",
            id
        );
    }
    let content = read_text(&path)?;
    println!("{}", content);
    Ok(())
}

pub fn cmd_finish(
    ctx: &CommandContext,
    stage: Option<String>,
    next_stage: Option<String>,
    session_id: Option<String>,
    task_arg: Option<String>,
) -> Result<()> {
    let stage = stage.unwrap_or_else(|| "task".to_string());
    if !ctx.agent.valid_finish_stages().contains(&stage.as_str()) {
        bail!("Unknown stage: {}", stage);
    }

    if let Some(ref next_stage) = next_stage {
        if !ctx.agent.stages().contains(&next_stage.as_str()) {
            bail!("Unknown next stage: {}", next_stage);
        }
    }

    let session_id = crate::state::resolve_session_id(&ctx.agent_root, session_id)?;
    let session_path = crate::util::session_state_path(&ctx.agent_root, &session_id);
    if !session_path.exists() {
        bail!("Session not found: {}", session_id);
    }

    let mut session = load_session(&session_path)?;

    let task = task_arg
        .or_else(|| env::var("METAGENT_TASK").ok())
        .or_else(|| session.task.clone());

    let task = if stage != "task" {
        if let Some(task) = task {
            task
        } else {
            find_unique_task(&ctx.agent_root, &stage)?.ok_or_else(|| {
                anyhow::anyhow!(
                    "METAGENT_TASK not set and no unique task found for stage '{}'",
                    stage
                )
            })?
        }
    } else {
        task.unwrap_or_default()
    };

    let resolved_next = if let Some(next) = next_stage.clone() {
        next
    } else if stage == "task" {
        "completed".to_string()
    } else {
        ctx.agent
            .next_stage(&stage)
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No next stage for {}", stage))?
    };

    session.status = SessionStatus::Finished;
    session.finished_at = Some(now_iso());
    session.next_stage = Some(resolved_next.clone());
    if !task.is_empty() {
        session.task = Some(task.clone());
    }
    save_session(&session_path, &session)?;

    let has_open_issues = if !task.is_empty() {
        task_has_open_issues(&ctx.agent_root, &task)?
    } else {
        false
    };

    // Don't allow moving to completed if there are open issues
    let resolved_next = if has_open_issues && resolved_next == "completed" {
        "build".to_string()
    } else {
        resolved_next
    };

    if !task.is_empty() {
        let task_path = task_state_path(&ctx.agent_root, &task);
        if !task_path.exists() {
            bail!("Task '{}' not found", task);
        }
        update_task(&task_path, |task_state| {
            task_state.stage = resolved_next.clone();
            task_state.updated_at = now_iso();
            task_state.last_session = Some(session_id.clone());
            task_state.status = determine_next_status(
                &stage,
                next_stage.is_some(),
                &resolved_next,
                has_open_issues,
            );
            Ok(())
        })?;
    }

    println!("Advanced stage to {}", resolved_next);
    Ok(())
}

pub fn cmd_review(ctx: &CommandContext, task: &str, focus: Option<String>) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    let focus_section = focus.map(|text| {
        format!(
            "## FOCUS AREA\n\nThe user has requested special attention to:\n> {text}\n\nPrioritize investigating this area first, then continue with full review."
        )
    });
    run_stage(
        ctx,
        Some(task),
        "review",
        focus_section.as_deref(),
        ReviewFinishMode::Manual,
    )?;
    Ok(())
}

pub fn cmd_spec_review(ctx: &CommandContext, task: &str) -> Result<()> {
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    run_stage(
        ctx,
        Some(task),
        "spec-review",
        None,
        ReviewFinishMode::Queue,
    )?;
    Ok(())
}

pub fn cmd_research(ctx: &CommandContext, task: &str, focus: Option<String>) -> Result<()> {
    ensure_code_agent(ctx)?;
    validate_task_name(task)?;
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }

    let prompt = load_prompt_by_name(ctx, "RESEARCH_PROMPT.md")?;
    let repo_root_str = ctx.repo_root.display().to_string();
    let focus_section = focus.map(|text| {
        format!(
            "## FOCUS AREA\n\nFocus on the following:\n> {text}\n\nPrioritize this area first, then continue with full research."
        )
    });
    let context = PromptContext {
        repo_root: &repo_root_str,
        task: Some(task),
        session: None,
        issues_header: "",
        issues_mode: "",
        review_finish_instructions: "",
        parallelism_mode: "",
        focus_section: focus_section.as_deref().unwrap_or(""),
    };
    let rendered = render_prompt(&prompt, &context);

    let _terminal_guard = TerminalGuard::capture();
    let model = resolve_model(&ctx.model_choice, ctx.agent, "build", None);
    let (cmd, args) = model.command();
    let status = Command::new(cmd)
        .args(args)
        .arg(rendered)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&ctx.repo_root)
        .env("METAGENT_AGENT", ctx.agent.name())
        .env("METAGENT_REPO_ROOT", ctx.repo_root.as_os_str())
        .env("METAGENT_TASK", task)
        .status()
        .context("Failed to start research model")?;

    if !status.success() {
        bail!("Research command failed");
    }
    Ok(())
}

pub fn cmd_how(ctx: &CommandContext, topic: Option<&str>) -> Result<()> {
    let topics = list_how_topics(ctx)?;
    if topic.is_none() {
        if topics.is_empty() {
            println!("{}", "No how topics available".dimmed());
        } else {
            println!("{}", "How topics:".bold());
            for topic in topics {
                println!("  {topic}");
            }
        }
        return Ok(());
    }

    let normalized = normalize_how_topic(topic.unwrap());
    if normalized.is_empty() {
        bail!("Topic cannot be empty");
    }

    let content = load_how_prompt(ctx, &normalized)?;
    println!("{content}");
    Ok(())
}

fn build_task_history(agent_root: &Path, task: &str) -> Result<String> {
    let sessions_dir = agent_root.join("sessions");
    let entries = match fs::read_dir(&sessions_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(String::new()),
    };

    let mut sessions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path().join("session.json");
        if !path.exists() {
            continue;
        }
        if let Ok(session) = load_session(&path) {
            if session.task.as_deref() == Some(task) {
                sessions.push((session.started_at, session.stage));
            }
        }
    }
    if sessions.is_empty() {
        return Ok(String::new());
    }
    sessions.sort_by(|a, b| a.0.cmp(&b.0));

    let mut parts: Vec<String> = Vec::new();
    let mut current_stage = String::new();
    let mut current_count = 0usize;
    for (_, stage) in sessions {
        if current_count == 0 {
            current_stage = stage;
            current_count = 1;
            continue;
        }
        if stage == current_stage {
            current_count += 1;
        } else {
            parts.push(format_stage_history(&current_stage, current_count));
            current_stage = stage;
            current_count = 1;
        }
    }
    if current_count > 0 {
        parts.push(format_stage_history(&current_stage, current_count));
    }

    Ok(parts.join("->"))
}

fn format_stage_history(stage: &str, count: usize) -> String {
    if count > 1 {
        format!("{stage}({count}x)")
    } else {
        stage.to_string()
    }
}

fn list_how_topics(ctx: &CommandContext) -> Result<Vec<String>> {
    let how_dir = ctx.prompt_root.join("how");
    let mut topics = Vec::new();
    if let Ok(entries) = fs::read_dir(&how_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if ext != "md" {
                    continue;
                }
            } else {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                topics.push(stem.to_string());
            }
        }
    }
    if topics.is_empty() {
        topics = ctx
            .agent
            .how_topics()
            .into_iter()
            .map(|t| t.to_string())
            .collect();
    }
    topics.sort();
    Ok(topics)
}

fn normalize_how_topic(raw: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in raw.trim().chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if ch == '-' || ch == '_' || ch.is_whitespace() {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn load_how_prompt(ctx: &CommandContext, topic: &str) -> Result<String> {
    let file_name = format!("{topic}.md");
    let prompt_path = ctx.prompt_root.join("how").join(&file_name);
    if prompt_path.exists() {
        return read_text(&prompt_path);
    }
    let embedded_key = format!("how/{file_name}");
    if let Some(embedded) = ctx.agent.embedded_prompt(&embedded_key) {
        return Ok(embedded.to_string());
    }
    bail!(
        "No how prompt found for '{}'. Run 'metagent how' to list topics.",
        topic
    );
}

pub fn cmd_set_stage(
    ctx: &CommandContext,
    task: &str,
    stage: &str,
    status: Option<String>,
) -> Result<()> {
    validate_task_name(task)?;
    if !ctx.agent.stages().contains(&stage) {
        bail!("Unknown stage: {}", stage);
    }
    let task_path = task_state_path(&ctx.agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }

    let resolved_status = if let Some(status) = status {
        TaskStatus::from_str(&status)?
    } else {
        let has_open_issues = if ctx.agent == AgentKind::Code {
            task_has_open_issues(&ctx.agent_root, task)?
        } else {
            false
        };
        if has_open_issues {
            TaskStatus::Issues
        } else if stage == "completed" {
            TaskStatus::Completed
        } else {
            TaskStatus::Pending
        }
    };

    let status_for_update = resolved_status.clone();
    update_task(&task_path, |task_state| {
        task_state.stage = stage.to_string();
        task_state.status = status_for_update;
        task_state.updated_at = now_iso();
        Ok(())
    })?;

    println!(
        "Set '{}' to stage '{}' (status: {})",
        task, stage, resolved_status
    );
    Ok(())
}

pub fn cmd_debug(
    ctx: &CommandContext,
    bug: Vec<String>,
    file: Option<PathBuf>,
    stdin: bool,
) -> Result<()> {
    let _terminal_guard = TerminalGuard::capture();
    if file.is_some() && stdin {
        bail!("Use --file or --stdin, not both");
    }

    let bug_text = if stdin {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    } else if let Some(path) = file {
        read_text(&path)?
    } else if !bug.is_empty() {
        bug.join(" ")
    } else {
        String::new()
    };

    let prompt = load_prompt_by_name(ctx, "DEBUG_PROMPT.md")?;
    let repo_root_str = ctx.repo_root.display().to_string();
    let parallelism_mode = parallelism_text(Model::Codex);
    let context = PromptContext {
        repo_root: &repo_root_str,
        task: None,
        session: None,
        issues_header: "",
        issues_mode: "",
        review_finish_instructions: "",
        parallelism_mode: &parallelism_mode,
        focus_section: "",
    };
    let mut rendered = render_prompt(&prompt, &context);
    if !bug_text.trim().is_empty() {
        let bug_block = format!("## Bug Report & Logs\n{}\n\n", bug_text.trim());
        rendered = format!("{bug_block}{rendered}");
    }

    let (cmd, args) = Model::Codex.command();
    let status = Command::new(cmd)
        .args(args)
        .arg(rendered)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&ctx.repo_root)
        .env("METAGENT_AGENT", ctx.agent.name())
        .env("METAGENT_REPO_ROOT", ctx.repo_root.as_os_str())
        .status()
        .context("Failed to start debug model")?;

    if !status.success() {
        bail!("Debug command failed");
    }
    Ok(())
}

fn run_stage(
    ctx: &CommandContext,
    task: Option<&str>,
    stage: &str,
    focus_section: Option<&str>,
    review_mode: ReviewFinishMode,
) -> Result<StageResult> {
    let _terminal_guard = TerminalGuard::capture();
    let task_status = task.and_then(|task_name| {
        let path = task_state_path(&ctx.agent_root, task_name);
        load_task(&path).ok().map(|task| task.status)
    });
    let has_open_issues = if let Some(task_name) = task {
        match task_has_open_issues(&ctx.agent_root, task_name) {
            Ok(has_open) => has_open,
            Err(err) => {
                eprintln!("Warning: failed to load issues: {}", err);
                false
            }
        }
    } else {
        false
    };
    let effective_status = if has_open_issues {
        Some(TaskStatus::Issues)
    } else {
        task_status.clone()
    };
    let model = resolve_model(
        &ctx.model_choice,
        ctx.agent,
        stage,
        effective_status.as_ref(),
    );

    let session_id = crate::state::new_session_id();
    let session = create_session(
        &ctx.agent_root,
        &session_id,
        ctx.agent.name(),
        stage,
        task,
        &ctx.repo_root,
        &ctx.host,
    )?;

    let prompt_template = load_stage_prompt(ctx, stage, task)?;
    let issues_context_status = if stage == "review" {
        None
    } else {
        effective_status.as_ref()
    };
    let (issues_header, issues_mode) = issues_text(ctx.agent, issues_context_status, task);
    let review_finish_instructions = if stage == "review" {
        build_review_finish_instructions(review_mode, &ctx.repo_root, task, &session.session_id)
    } else {
        String::new()
    };
    let parallelism_mode = parallelism_text(model);
    let focus_section = focus_section.unwrap_or("");
    let repo_root_str = ctx.repo_root.display().to_string();
    let prompt_context = PromptContext {
        repo_root: &repo_root_str,
        task,
        session: Some(&session.session_id),
        issues_header: &issues_header,
        issues_mode: &issues_mode,
        review_finish_instructions: &review_finish_instructions,
        parallelism_mode: &parallelism_mode,
        focus_section,
    };

    let mut rendered = render_prompt(&prompt_template, &prompt_context);
    if let Some(task) = task {
        rendered = format!("Task: {task}\n\n{rendered}");
    }

    let (cmd, args) = model.command();
    let mut child = Command::new(cmd);
    child.args(args);
    child.arg(rendered);
    child.stdin(Stdio::inherit());
    child.stdout(Stdio::inherit());
    child.stderr(Stdio::inherit());
    child.current_dir(&ctx.repo_root);
    child.env("METAGENT_AGENT", ctx.agent.name());
    child.env("METAGENT_SESSION", &session_id);
    child.env("METAGENT_REPO_ROOT", ctx.repo_root.as_os_str());
    if let Some(task) = task {
        child.env("METAGENT_TASK", task);
    }
    let mut child = child.spawn().context("Failed to start model process")?;

    let session_path = crate::util::session_state_path(&ctx.agent_root, &session_id);
    loop {
        if INTERRUPTED.load(Ordering::SeqCst) {
            terminate_child(&mut child);
            return Ok(StageResult::Interrupted);
        }

        if let Ok(session_state) = load_session(&session_path) {
            if session_state.status == SessionStatus::Finished {
                terminate_child(&mut child);
                return Ok(StageResult::Finished(session_state));
            }
        }

        if let Some(_status) = child.try_wait()? {
            break;
        }

        thread::sleep(Duration::from_millis(500));
    }

    if let Ok(session_state) = load_session(&session_path) {
        if session_state.status == SessionStatus::Finished {
            return Ok(StageResult::Finished(session_state));
        }
    }

    update_session(&session_path, |session_state| {
        session_state.status = SessionStatus::Failed;
        session_state.finished_at = Some(now_iso());
        Ok(())
    })
    .ok();

    Ok(StageResult::NoFinish)
}

fn bootstrap_needed(agent_root: &Path) -> Result<bool> {
    let agents_path = agent_root.join("AGENTS.md");
    let spec_path = agent_root.join("SPEC.md");
    let tech_path = agent_root.join("TECHNICAL_STANDARDS.md");

    if !agents_path.exists() || !spec_path.exists() || !tech_path.exists() {
        return Ok(true);
    }

    let agents = read_text(&agents_path).unwrap_or_default();
    let spec = read_text(&spec_path).unwrap_or_default();
    let tech = read_text(&tech_path).unwrap_or_default();

    let agents_markers = [
        "{PROJECT_NAME}",
        "{LANGUAGE}",
        "{FRAMEWORK}",
        "{BUILD_TOOL}",
        "{TEST_FRAMEWORK}",
        "{PACKAGE_MANAGER}",
    ];
    let spec_markers = [
        "{PROJECT_DESCRIPTION}",
        "{WHY_THIS_EXISTS}",
        "{ARCHITECTURE_DIAGRAM}",
        "{DATA_FLOW_DESCRIPTION}",
        "{MAIN_FEATURES}",
    ];
    let tech_markers = [
        "{LANGUAGE}",
        "{LANGUAGE_VERSION}",
        "{STYLE_GUIDE}",
        "{FILE_CONVENTION}",
        "{ASYNC_PATTERNS}",
    ];

    let needs_agents = agents_markers.iter().any(|marker| agents.contains(marker));
    let needs_spec = spec_markers.iter().any(|marker| spec.contains(marker));
    let needs_tech = tech_markers.iter().any(|marker| tech.contains(marker));

    Ok(needs_agents || needs_spec || needs_tech)
}

fn run_bootstrap(ctx: &CommandContext) -> Result<()> {
    let _terminal_guard = TerminalGuard::capture();
    let prompt = load_prompt_by_name(ctx, "BOOTSTRAP_PROMPT.md")?;
    let model = ctx.model_choice.model;
    let parallelism_mode = parallelism_text(model);
    let repo_root_str = ctx.repo_root.display().to_string();
    let context = PromptContext {
        repo_root: &repo_root_str,
        task: None,
        session: None,
        issues_header: "",
        issues_mode: "",
        review_finish_instructions: "",
        parallelism_mode: &parallelism_mode,
        focus_section: "",
    };
    let prompt_text = render_prompt(&prompt, &context);

    let (cmd, args) = model.command();
    let status = Command::new(cmd)
        .args(args)
        .arg(prompt_text)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&ctx.repo_root)
        .env("METAGENT_AGENT", ctx.agent.name())
        .env("METAGENT_REPO_ROOT", ctx.repo_root.as_os_str())
        .status()
        .context("Failed to start bootstrap model")?;

    if !status.success() {
        bail!("Bootstrap command failed");
    }
    Ok(())
}

fn resolve_model(
    choice: &ModelChoice,
    agent: AgentKind,
    stage: &str,
    task_status: Option<&TaskStatus>,
) -> Model {
    if task_status == Some(&TaskStatus::Issues) && !(choice.force_model && choice.explicit) {
        return Model::Codex;
    }
    if choice.explicit {
        return choice.model;
    }
    if let Some(stage_model) = agent.model_for_stage(stage) {
        return stage_model;
    }
    choice.model
}

fn reconcile_running_tasks(agent_root: &Path) -> Result<()> {
    let tasks = list_tasks(agent_root);
    for task in tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Running && t.stage != "completed")
    {
        if has_active_claim(agent_root, &task.task)? || has_active_session(agent_root, &task.task)?
        {
            continue;
        }
        let task_path = task_state_path(agent_root, &task.task);
        update_task(&task_path, |task_state| {
            task_state.status = TaskStatus::Incomplete;
            task_state.updated_at = now_iso();
            Ok(())
        })?;
    }
    Ok(())
}

fn load_stage_prompt(ctx: &CommandContext, stage: &str, task: Option<&str>) -> Result<String> {
    let prompt_path = ctx
        .agent
        .prompt_file_for_stage(stage, task)
        .ok_or_else(|| anyhow::anyhow!("No prompt for stage: {}", stage))?;

    if prompt_path.is_absolute() || prompt_path.components().count() > 1 {
        if !prompt_path.exists() {
            bail!("Prompt file not found: {}", prompt_path.display());
        }
        return read_text(&prompt_path);
    }

    let prompt_file = ctx.prompt_root.join(&prompt_path);
    if prompt_file.exists() {
        return read_text(&prompt_file);
    }

    let file_name = prompt_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    if let Some(embedded) = ctx.agent.embedded_prompt(&file_name) {
        return Ok(embedded.to_string());
    }

    bail!("Prompt file not found: {}", prompt_file.display())
}

fn load_prompt_by_name(ctx: &CommandContext, name: &str) -> Result<String> {
    let prompt_file = ctx.prompt_root.join(name);
    if prompt_file.exists() {
        return read_text(&prompt_file);
    }
    if let Some(embedded) = ctx.agent.embedded_prompt(name) {
        return Ok(embedded.to_string());
    }
    bail!("Prompt file not found: {}", prompt_file.display());
}

fn find_unique_task(agent_root: &Path, stage: &str) -> Result<Option<String>> {
    let tasks = list_tasks(agent_root);
    let mut matches: Vec<TaskState> = tasks
        .into_iter()
        .filter(|task| {
            task.stage == stage
                && matches!(
                    task.status,
                    TaskStatus::Running
                        | TaskStatus::Pending
                        | TaskStatus::Incomplete
                        | TaskStatus::Issues
                )
        })
        .collect();
    if matches.len() == 1 {
        return Ok(Some(matches.remove(0).task));
    }
    Ok(None)
}

fn determine_next_status(
    stage: &str,
    override_next: bool,
    next_stage: &str,
    has_open_issues: bool,
) -> TaskStatus {
    if has_open_issues {
        return TaskStatus::Issues;
    }
    if next_stage == "completed" {
        return TaskStatus::Completed;
    }
    if stage == "review" && override_next {
        if next_stage == "spec-review-issues" {
            return TaskStatus::Pending;
        }
        return TaskStatus::Issues;
    }
    TaskStatus::Pending
}

fn ensure_code_agent(ctx: &CommandContext) -> Result<()> {
    if ctx.agent != AgentKind::Code {
        bail!("Issue tracking is only supported for the code agent");
    }
    Ok(())
}

fn parse_status_filter(value: Option<&str>) -> Result<IssueStatusFilter> {
    let value = value.unwrap_or("open");
    match value.trim().to_lowercase().as_str() {
        "open" => Ok(IssueStatusFilter::Open),
        "resolved" => Ok(IssueStatusFilter::Resolved),
        "all" => Ok(IssueStatusFilter::All),
        other => bail!("Invalid status filter: {}", other),
    }
}

fn parse_priority(value: Option<&str>) -> Result<Option<IssuePriority>> {
    match value {
        Some(value) => Ok(Some(IssuePriority::from_str(value)?)),
        None => Ok(None),
    }
}

fn parse_issue_type(value: Option<&str>) -> Result<Option<IssueType>> {
    match value {
        Some(value) => Ok(Some(IssueType::from_str(value)?)),
        None => Ok(None),
    }
}

fn parse_issue_source(value: Option<&str>) -> Result<Option<IssueSource>> {
    match value {
        Some(value) => Ok(Some(IssueSource::from_str(value)?)),
        None => Ok(None),
    }
}

#[derive(Debug)]
struct CanonicalPlanStep {
    line: usize,
    done: bool,
    priority: String,
    complexity: String,
    id: u32,
    title: String,
}

#[derive(Debug)]
struct ChecklistStep {
    line: usize,
    done: bool,
    title: String,
}

fn parse_checklist_prefix(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("- [")?;
    let status = rest.chars().next()?;
    if status != ' ' && status != 'x' {
        return None;
    }
    let rest = &rest[status.len_utf8()..];
    let rest = rest.strip_prefix("] ")?;
    Some((status == 'x', rest))
}

fn parse_bracket_tag(input: &str) -> Option<(&str, &str)> {
    let inner = input.strip_prefix('[')?;
    let end = inner.find(']')?;
    let tag = &inner[..end];
    let rest = &inner[end + 1..];
    Some((tag, rest))
}

fn parse_canonical_plan_step(line: &str, line_number: usize) -> Option<CanonicalPlanStep> {
    let (done, rest) = parse_checklist_prefix(line)?;
    let (priority, rest) = parse_bracket_tag(rest)?;
    if !matches!(priority, "P0" | "P1" | "P2" | "P3") {
        return None;
    }
    let (complexity, rest) = parse_bracket_tag(rest)?;
    if !matches!(complexity, "S" | "M" | "L") {
        return None;
    }
    let (id_tag, rest) = parse_bracket_tag(rest)?;
    let id_part = id_tag.strip_prefix('T')?;
    if id_part.is_empty()
        || !id_part.chars().all(|c| c.is_ascii_digit())
        || (id_part.len() > 1 && id_part.starts_with('0'))
    {
        return None;
    }
    let id = id_part.parse::<u32>().ok()?;
    let title = rest.strip_prefix(' ')?.trim();
    if title.is_empty() {
        return None;
    }

    Some(CanonicalPlanStep {
        line: line_number,
        done,
        priority: priority.to_string(),
        complexity: complexity.to_string(),
        id,
        title: title.to_string(),
    })
}

fn parse_checklist_step(line: &str, line_number: usize) -> Option<ChecklistStep> {
    let (done, rest) = parse_checklist_prefix(line)?;
    let title = rest.trim();
    if title.is_empty() {
        return None;
    }
    Some(ChecklistStep {
        line: line_number,
        done,
        title: title.to_string(),
    })
}

fn issue_default_stage(agent: AgentKind, issue_type: &IssueType) -> Option<String> {
    if agent != AgentKind::Code {
        return None;
    }
    match issue_type {
        IssueType::Spec => Some("spec-review-issues".to_string()),
        _ => Some("build".to_string()),
    }
}

fn validate_issue_stage(agent: AgentKind, stage: &str) -> Result<()> {
    if !agent.stages().contains(&stage) {
        bail!("Unknown stage: {}", stage);
    }
    if stage == "completed" {
        bail!("Issues cannot target the completed stage");
    }
    Ok(())
}

fn update_task_for_issue(
    agent_root: &Path,
    task: &str,
    stage_override: Option<&str>,
    default_stage: Option<&str>,
) -> Result<()> {
    let task_path = task_state_path(agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    update_task(&task_path, |task_state| {
        if let Some(stage) = stage_override {
            task_state.stage = stage.to_string();
        } else if task_state.stage == "completed" {
            if let Some(stage) = default_stage {
                task_state.stage = stage.to_string();
            }
        }
        task_state.status = TaskStatus::Issues;
        task_state.updated_at = now_iso();
        Ok(())
    })?;
    Ok(())
}

fn sync_task_status_for_issues(agent_root: &Path, task: &str) -> Result<()> {
    let task_path = task_state_path(agent_root, task);
    if !task_path.exists() {
        bail!("Task '{}' not found", task);
    }
    let issues = list_issues(agent_root)?;
    let has_open = issues
        .iter()
        .any(|issue| issue.status == IssueStatus::Open && issue.task.as_deref() == Some(task));
    update_task(&task_path, |task_state| {
        if has_open {
            task_state.status = TaskStatus::Issues;
        } else if task_state.stage == "completed" {
            task_state.status = TaskStatus::Completed;
        } else if task_state.status == TaskStatus::Issues {
            task_state.status = TaskStatus::Pending;
        }
        task_state.updated_at = now_iso();
        Ok(())
    })?;
    Ok(())
}

fn task_has_open_issues(agent_root: &Path, task: &str) -> Result<bool> {
    let issues = list_issues(agent_root)?;
    Ok(issues
        .iter()
        .any(|issue| issue.status == IssueStatus::Open && issue.task.as_deref() == Some(task)))
}

fn next_eligible_task(agent: AgentKind, tasks: &[TaskState]) -> Option<TaskState> {
    for stage in agent.queue_stages() {
        let mut stage_tasks: Vec<TaskState> = tasks
            .iter()
            .filter(|t| {
                !t.held
                    && t.stage == *stage
                    && matches!(
                        t.status,
                        TaskStatus::Pending | TaskStatus::Incomplete | TaskStatus::Issues
                    )
            })
            .cloned()
            .collect();
        if stage_tasks.is_empty() {
            continue;
        }
        if *stage == "build" {
            stage_tasks.sort_by(|a, b| {
                let ar = a.queue_rank.unwrap_or(i64::MAX);
                let br = b.queue_rank.unwrap_or(i64::MAX);
                ar.cmp(&br).then_with(|| a.added_at.cmp(&b.added_at))
            });
        } else {
            stage_tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));
        }
        return stage_tasks.into_iter().next();
    }
    // Safety net: pick up completed tasks that still have Issues status
    let mut issues_tasks: Vec<TaskState> = tasks
        .iter()
        .filter(|t| !t.held && t.stage == "completed" && t.status == TaskStatus::Issues)
        .cloned()
        .collect();
    if !issues_tasks.is_empty() {
        issues_tasks.sort_by(|a, b| a.added_at.cmp(&b.added_at));
        // Override stage to build since completed has no prompt
        return issues_tasks.into_iter().next().map(|mut t| {
            t.stage = "build".to_string();
            t
        });
    }
    None
}

fn send_signal(child: &mut std::process::Child, signal: i32) {
    let pid = child.id() as i32;
    send_signal_to_pid(pid, signal);
}

fn send_signal_to_pid(pid: i32, signal: i32) {
    unsafe {
        let _ = libc::kill(pid, signal);
    }
}

fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

fn collect_descendant_pids(root_pid: i32) -> Vec<i32> {
    let output = match Command::new("ps").args(["-axo", "pid=,ppid="]).output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut children_by_parent: HashMap<i32, Vec<i32>> = HashMap::new();
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let pid = parts.next().and_then(|value| value.parse::<i32>().ok());
        let ppid = parts.next().and_then(|value| value.parse::<i32>().ok());
        if let (Some(pid), Some(ppid)) = (pid, ppid) {
            children_by_parent.entry(ppid).or_default().push(pid);
        }
    }

    let mut descendants = Vec::new();
    let mut stack = vec![root_pid];
    while let Some(parent) = stack.pop() {
        if let Some(children) = children_by_parent.get(&parent) {
            for child in children {
                descendants.push(*child);
                stack.push(*child);
            }
        }
    }
    descendants.sort_unstable();
    descendants.dedup();
    descendants
}

fn signal_process_tree(
    child: &mut std::process::Child,
    signal: i32,
    known_descendants: &mut HashSet<i32>,
) {
    let root_pid = child.id() as i32;
    known_descendants.extend(collect_descendant_pids(root_pid));

    // Signal descendants first so wrapper exits don't orphan deeper children.
    let mut descendants: Vec<i32> = known_descendants
        .iter()
        .copied()
        .filter(|pid| pid_alive(*pid))
        .collect();
    descendants.sort_unstable();
    descendants.reverse();
    for pid in descendants {
        send_signal_to_pid(pid, signal);
    }

    send_signal(child, signal);
}

fn wait_for_process_tree_exit(
    child: &mut std::process::Child,
    known_descendants: &mut HashSet<i32>,
    timeout: Duration,
) -> bool {
    let start = Instant::now();
    let mut root_exited = false;
    while start.elapsed() < timeout {
        if !root_exited {
            match child.try_wait() {
                Ok(Some(_)) => root_exited = true,
                Ok(None) => {}
                Err(_) => root_exited = true,
            }
        }
        known_descendants.retain(|pid| pid_alive(*pid));
        if root_exited && known_descendants.is_empty() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

fn terminate_child(child: &mut std::process::Child) {
    const SIGINT_ATTEMPTS: usize = 3;
    let mut known_descendants = HashSet::new();
    for _ in 0..SIGINT_ATTEMPTS {
        signal_process_tree(child, libc::SIGINT, &mut known_descendants);
        if wait_for_process_tree_exit(child, &mut known_descendants, Duration::from_millis(500)) {
            return;
        }
    }

    signal_process_tree(child, libc::SIGTERM, &mut known_descendants);
    if wait_for_process_tree_exit(child, &mut known_descendants, Duration::from_secs(1)) {
        return;
    }

    signal_process_tree(child, libc::SIGKILL, &mut known_descendants);
    let _ = wait_for_process_tree_exit(child, &mut known_descendants, Duration::from_secs(1));
    let _ = child.kill();
    let _ = wait_for_process_tree_exit(child, &mut known_descendants, Duration::from_secs(1));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReviewFinishMode {
    Queue,
    Manual,
}

#[derive(Debug)]
enum StageResult {
    Finished(SessionState),
    Interrupted,
    NoFinish,
}

fn build_review_finish_instructions(
    mode: ReviewFinishMode,
    repo_root: &Path,
    task: Option<&str>,
    session_id: &str,
) -> String {
    if mode == ReviewFinishMode::Manual {
        return "7. Manual review: do not run `metagent finish`. End after the report.".to_string();
    }
    let task = match task {
        Some(task) => task,
        None => return String::new(),
    };
    let repo = repo_root.display();
    format!(
        "7. Signal next stage:\n\
- Spec issues exist (any open) or spec needs revision: `cd \"{repo}\" && METAGENT_TASK=\"{task}\" metagent --agent code finish review --session \"{session_id}\" --next spec-review-issues`\n\
- Only build issues (no spec issues): `cd \"{repo}\" && METAGENT_TASK=\"{task}\" metagent --agent code finish review --session \"{session_id}\" --next build`\n\
- Pass (no issues): `cd \"{repo}\" && METAGENT_TASK=\"{task}\" metagent --agent code finish review --session \"{session_id}\"`"
    )
}
