use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tempfile::TempDir;

fn resolve_binary() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_metagent") {
        return PathBuf::from(path);
    }

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let mut candidate = manifest_dir.join("target/debug/metagent");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }

    if candidate.exists() {
        return candidate;
    }

    let status = Command::new("cargo")
        .args(["build"])
        .current_dir(&manifest_dir)
        .status()
        .expect("cargo build");
    assert!(status.success(), "cargo build failed");

    if candidate.exists() {
        return candidate;
    }

    panic!("metagent binary not found");
}

struct TestEnv {
    home: TempDir,
    repo: PathBuf,
    bin: PathBuf,
    stub_bin: PathBuf,
    path: String,
}

impl TestEnv {
    fn new() -> Self {
        let home = TempDir::new().expect("temp home");
        let repo = home.path().join("repo");
        fs::create_dir_all(repo.join(".git")).expect("create .git");

        let bin = resolve_binary();
        let stub_bin = home.path().join("bin");
        fs::create_dir_all(&stub_bin).expect("stub bin");
        let path = std::env::var("PATH").unwrap_or_default();

        Self {
            home,
            repo,
            bin,
            stub_bin,
            path,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        cmd.env("HOME", self.home.path());
        cmd.env("PATH", format!("{}:{}", self.stub_bin.display(), self.path));
        cmd.current_dir(&self.repo);
        cmd
    }

    fn run(&self, args: &[&str]) {
        let status = self
            .command()
            .args(args)
            .status()
            .unwrap_or_else(|err| panic!("failed to run {args:?}: {err}"));
        assert!(status.success(), "command failed: {args:?}");
    }

    fn output(&self, args: &[&str]) -> String {
        let output = self
            .command()
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap_or_else(|err| panic!("failed to run {args:?}: {err}"));
        assert!(output.status.success(), "command failed: {args:?}");
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    fn install_stub_loop(&self, name: &str) {
        let path = self.stub_bin.join(name);
        let script = "#!/bin/sh\ntrap 'exit 0' INT TERM\nwhile true; do sleep 1; done\n";
        fs::write(&path, script).expect("write stub");
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod");
    }

    fn install_stub_spawn_tree(&self, name: &str) {
        let path = self.stub_bin.join(name);
        let script = r#"#!/bin/sh
(
  trap '' INT TERM
  while true; do sleep 1; done
) &
child=$!
if [ -n "$METAGENT_CHILD_PID_FILE" ]; then
  printf '%s\n' "$child" > "$METAGENT_CHILD_PID_FILE"
fi
trap 'exit 0' INT TERM
while true; do sleep 1; done
"#;
        fs::write(&path, script).expect("write stub");
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod");
    }

    fn install_stub_capture(&self, name: &str) {
        let path = self.stub_bin.join(name);
        let script = "#!/bin/sh\nif [ -n \"$METAGENT_PROMPT_FILE\" ]; then\n  printf '%s' \"$*\" > \"$METAGENT_PROMPT_FILE\"\nfi\nexit 0\n";
        fs::write(&path, script).expect("write stub");
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod");
    }
}

fn wait_for_session(agent_root: &Path) -> String {
    let sessions_dir = agent_root.join("sessions");
    let deadline = Instant::now() + Duration::from_secs(10);

    while Instant::now() < deadline {
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                let session_id = entry.file_name().to_string_lossy().to_string();
                let session_path = entry.path().join("session.json");
                if !session_path.exists() {
                    continue;
                }
                if let Ok(data) = fs::read_to_string(&session_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                        if json["status"] == "running" {
                            return session_id;
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    panic!("Timed out waiting for session");
}

fn wait_for_session_for_task(agent_root: &Path, task: &str) -> String {
    let sessions_dir = agent_root.join("sessions");
    let deadline = Instant::now() + Duration::from_secs(10);

    while Instant::now() < deadline {
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                let session_id = entry.file_name().to_string_lossy().to_string();
                let session_path = entry.path().join("session.json");
                if !session_path.exists() {
                    continue;
                }
                if let Ok(data) = fs::read_to_string(&session_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                        if json["status"] == "running" && json["task"] == task {
                            return session_id;
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    panic!("Timed out waiting for session for task {task}");
}

fn wait_for_running_session(agent_root: &Path) -> Option<(String, String)> {
    let sessions_dir = agent_root.join("sessions");
    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let session_id = entry.file_name().to_string_lossy().to_string();
            let session_path = entry.path().join("session.json");
            if !session_path.exists() {
                continue;
            }
            if let Ok(data) = fs::read_to_string(&session_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&data) {
                    if json["status"] == "running" {
                        let task = json["task"].as_str().unwrap_or("").to_string();
                        return Some((session_id, task));
                    }
                }
            }
        }
    }
    None
}

fn wait_for_exit(child: &mut std::process::Child) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    panic!("Timed out waiting for metagent run to exit");
}

fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[test]
fn install_and_uninstall() {
    let env = TestEnv::new();

    env.run(&["install"]);

    let home = env.home.path();
    assert!(home.join(".local/bin/metagent").exists());
    assert!(home.join(".metagent/code/SPEC_PROMPT.md").exists());
    assert!(home.join(".claude/commands/spec.md").exists());
    assert!(home.join(".codex/prompts/spec.md").exists());
    assert!(home.join(".claude/commands/submit-issue.md").exists());
    assert!(home.join(".codex/prompts/submit-issue.md").exists());
    assert!(home.join(".claude/commands/submit-task.md").exists());
    assert!(home.join(".codex/prompts/submit-task.md").exists());
    assert!(home.join(".claude/commands/submit-hold-task.md").exists());
    assert!(home.join(".codex/prompts/submit-hold-task.md").exists());

    env.run(&["uninstall"]);

    assert!(!home.join(".local/bin/metagent").exists());
    assert!(!home.join(".metagent").exists());
}

#[test]
fn init_runs_bootstrap_when_needed() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");

    let prompt_file = env.home.path().join("bootstrap_prompt.txt");
    let status = env
        .command()
        .args(["init"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("init");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(prompt.contains("Configure Workflow for Repository"));
}

#[test]
fn init_task_queue_dequeue() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");

    env.run(&["init"]);

    let agent_root = env.repo.join(".agents/code");
    assert!(agent_root.join("AGENTS.md").exists());
    assert!(agent_root.join("SPEC.md").exists());
    assert!(agent_root.join("TECHNICAL_STANDARDS.md").exists());

    env.run(&["task", "my-task"]);
    assert!(agent_root.join("tasks/my-task/task.json").exists());

    let output = env.output(&["queue"]);
    assert!(output.contains("my-task"));

    env.run(&["dequeue", "my-task"]);
    assert!(!agent_root.join("tasks/my-task").exists());
}

#[test]
fn set_stage_updates_task() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");

    env.run(&["init"]);
    env.run(&["task", "stage-task"]);

    env.run(&["set-stage", "stage-task", "planning"]);

    let agent_root = env.repo.join(".agents/code");
    let task_state =
        fs::read_to_string(agent_root.join("tasks/stage-task/task.json")).expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["stage"], "planning");
    assert_eq!(task_json["status"], "pending");

    env.run(&["set-stage", "stage-task", "review", "--status", "running"]);
    let task_state =
        fs::read_to_string(agent_root.join("tasks/stage-task/task.json")).expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["stage"], "review");
    assert_eq!(task_json["status"], "running");
}

#[test]
fn plan_command_lists_canonical_steps() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");

    env.run(&["init"]);
    env.run(&["task", "plan-task"]);

    let plan_path = env.repo.join(".agents/code/tasks/plan-task/plan.md");
    fs::write(
        &plan_path,
        r#"# Implementation Plan - plan-task

> Status: READY

- [ ] [P1][M][T17] Implement token validation
- [x] [P2][S][T18] Add regression tests
"#,
    )
    .expect("write plan");

    let output = env.output(&["plan", "plan-task"]);
    assert!(output.contains("Canonical steps:"));
    assert!(output.contains("[P1][M][T17] Implement token validation"));
    assert!(output.contains("[P2][S][T18] Add regression tests"));
    assert!(output.contains("Summary: 2 total (1 open, 1 done)"));
}

#[test]
fn run_and_finish() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.install_stub_loop("claude");

    env.run(&["task", "runner-task"]);

    let mut cmd = env.command();
    cmd.args(["run", "runner-task"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn run");

    let agent_root = env.repo.join(".agents/code");
    let session_id = wait_for_session(&agent_root);

    let status = env
        .command()
        .args([
            "finish",
            "spec",
            "--next",
            "completed",
            "--session",
            &session_id,
            "--task",
            "runner-task",
        ])
        .status()
        .expect("finish");
    assert!(status.success());

    wait_for_exit(&mut child);

    let task_state =
        fs::read_to_string(agent_root.join("tasks/runner-task/task.json")).expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["stage"], "completed");
    assert_eq!(task_json["status"], "completed");
}

#[test]
fn finish_terminates_model_process_tree() {
    let env = TestEnv::new();
    env.install_stub_capture("codex");
    env.install_stub_capture("claude");

    env.run(&["init"]);
    env.install_stub_spawn_tree("claude");
    env.run(&["task", "tree-task"]);

    let child_pid_file = env.home.path().join("child_pid.txt");
    let mut cmd = env.command();
    cmd.args(["run", "tree-task"])
        .env("METAGENT_MODEL", "claude")
        .env("METAGENT_CHILD_PID_FILE", &child_pid_file)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut run_child = cmd.spawn().expect("spawn run");

    let agent_root = env.repo.join(".agents/code");
    let session_id = wait_for_session_for_task(&agent_root, "tree-task");

    let child_pid = {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if child_pid_file.exists() {
                let text = fs::read_to_string(&child_pid_file).expect("read child pid");
                let pid = text.trim().parse::<i32>().expect("parse child pid");
                break pid;
            }
            if Instant::now() >= deadline {
                panic!("Timed out waiting for child pid file");
            }
            thread::sleep(Duration::from_millis(50));
        }
    };

    let status = env
        .command()
        .args([
            "finish",
            "spec",
            "--next",
            "completed",
            "--session",
            &session_id,
            "--task",
            "tree-task",
        ])
        .status()
        .expect("finish");
    assert!(status.success());

    wait_for_exit(&mut run_child);

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && pid_alive(child_pid) {
        thread::sleep(Duration::from_millis(50));
    }
    if pid_alive(child_pid) {
        unsafe {
            let _ = libc::kill(child_pid, libc::SIGKILL);
        }
    }
    assert!(
        !pid_alive(child_pid),
        "expected descendant process {child_pid} to be terminated"
    );
}

#[test]
fn finish_without_session_env() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.install_stub_loop("claude");

    env.run(&["task", "no-session"]);

    let mut cmd = env.command();
    cmd.args(["run", "no-session"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn run");

    let agent_root = env.repo.join(".agents/code");
    let _session_id = wait_for_session_for_task(&agent_root, "no-session");

    let status = env
        .command()
        .args([
            "finish",
            "spec",
            "--next",
            "completed",
            "--task",
            "no-session",
        ])
        .status()
        .expect("finish");
    assert!(status.success());

    wait_for_exit(&mut child);

    let task_state =
        fs::read_to_string(agent_root.join("tasks/no-session/task.json")).expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["stage"], "completed");
    assert_eq!(task_json["status"], "completed");
}

#[test]
fn run_queue_completes_tasks_with_stale_claim() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.install_stub_loop("claude");

    env.run(&["task", "alpha"]);
    env.run(&["task", "beta"]);
    env.run(&["set-stage", "alpha", "build"]);
    env.run(&["set-stage", "beta", "build"]);

    let agent_root = env.repo.join(".agents/code");
    let stale_claim = agent_root.join("claims/alpha.lock");
    fs::create_dir_all(stale_claim.parent().unwrap()).expect("claims dir");
    let stale = json!({
        "task": "alpha",
        "agent": "code",
        "pid": 999999,
        "host": "localhost",
        "started_at": "2000-01-01T00:00:00Z",
        "ttl_seconds": 3600
    });
    fs::write(&stale_claim, serde_json::to_string_pretty(&stale).unwrap()).expect("stale claim");

    let mut cmd = env.command();
    cmd.args(["run-queue"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn run-queue");

    let mut completed = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while completed < 2 && Instant::now() < deadline {
        if let Some((session_id, task)) = wait_for_running_session(&agent_root) {
            if task.is_empty() {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            let status = env
                .command()
                .args([
                    "finish",
                    "spec",
                    "--next",
                    "completed",
                    "--task",
                    &task,
                    "--session",
                    &session_id,
                ])
                .status()
                .expect("finish");
            assert!(status.success());
            completed += 1;
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }

    wait_for_exit(&mut child);

    let alpha_state =
        fs::read_to_string(agent_root.join("tasks/alpha/task.json")).expect("alpha task.json");
    let beta_state =
        fs::read_to_string(agent_root.join("tasks/beta/task.json")).expect("beta task.json");
    let alpha_json: Value = serde_json::from_str(&alpha_state).expect("alpha parse");
    let beta_json: Value = serde_json::from_str(&beta_state).expect("beta parse");
    assert_eq!(alpha_json["status"], "completed");
    assert_eq!(beta_json["status"], "completed");
}

#[test]
fn review_focus_injected_into_prompt() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);

    env.run(&["task", "review-task"]);

    let prompt_file = env.home.path().join("prompt.txt");
    let status = env
        .command()
        .args(["review", "review-task", "Focus on caching"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("review");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(prompt.contains("FOCUS AREA"), "missing focus header");
    assert!(prompt.contains("Focus on caching"), "missing focus text");
}

#[test]
fn spec_review_renders_prompt() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);

    env.run(&["task", "spec-review-task"]);

    let prompt_file = env.home.path().join("spec_review_prompt.txt");
    let status = env
        .command()
        .args(["spec-review", "spec-review-task"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("spec review");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(
        prompt.contains("@.agents/code/tasks/spec-review-task/spec/"),
        "prompt missing task path"
    );
}

#[test]
fn debug_includes_bug_context() {
    let env = TestEnv::new();
    env.install_stub_capture("codex");

    let status = env
        .command()
        .args(["init"])
        .env("METAGENT_MODEL", "codex")
        .status()
        .expect("init");
    assert!(status.success());

    let prompt_file = env.home.path().join("debug_prompt.txt");
    let status = env
        .command()
        .args(["debug", "login", "fails", "500"])
        .env("METAGENT_MODEL", "claude")
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("debug");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(prompt.contains("Bug Report & Logs"));
    assert!(prompt.contains("login fails 500"));
}

#[test]
fn reorder_build_queue_position() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.run(&["task", "alpha"]);
    env.run(&["task", "beta"]);
    env.run(&["task", "gamma"]);

    env.run(&["set-stage", "alpha", "build"]);
    env.run(&["set-stage", "beta", "build"]);
    env.run(&["set-stage", "gamma", "build"]);

    env.run(&["reorder", "beta", "1"]);

    let prompt_file = env.home.path().join("reorder_prompt.txt");
    let status = env
        .command()
        .args(["run-next"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("run-next");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(prompt.contains("Task: beta"), "expected beta to run first");
}

#[test]
fn issues_add_list_resolve() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");

    env.run(&["init"]);
    env.run(&["task", "issue-task"]);

    let output = env.output(&[
        "issue",
        "add",
        "--title",
        "Login fails",
        "--task",
        "issue-task",
        "--priority",
        "P1",
        "--type",
        "build",
        "--source",
        "manual",
        "--body",
        "Repro steps here",
    ]);
    assert!(output.contains("Created issue"));

    let issues_dir = env.repo.join(".agents/code/issues");
    let mut entries: Vec<PathBuf> = fs::read_dir(&issues_dir)
        .expect("issues dir")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .collect();
    assert_eq!(entries.len(), 1);
    let issue_path = entries.pop().expect("issue path");
    let issue_id = issue_path
        .file_stem()
        .expect("issue stem")
        .to_string_lossy()
        .to_string();

    let list_output = env.output(&["issues", "--task", "issue-task"]);
    assert!(list_output.contains("Login fails"));

    let task_state = fs::read_to_string(env.repo.join(".agents/code/tasks/issue-task/task.json"))
        .expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["status"], "issues");

    env.run(&["issue", "resolve", &issue_id, "--resolution", "fixed"]);

    let issue_content = fs::read_to_string(&issue_path).expect("issue content");
    assert!(issue_content.contains("status: resolved"));

    let task_state = fs::read_to_string(env.repo.join(".agents/code/tasks/issue-task/task.json"))
        .expect("task.json");
    let task_json: Value = serde_json::from_str(&task_state).expect("parse task.json");
    assert_eq!(task_json["status"], "pending");
}

#[test]
fn run_next_injects_issues_even_if_status_drifts() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.run(&["task", "issue-task"]);

    env.run(&[
        "issue",
        "add",
        "--title",
        "Login fails",
        "--task",
        "issue-task",
        "--priority",
        "P1",
        "--type",
        "build",
        "--source",
        "manual",
        "--body",
        "Repro steps here",
    ]);

    let task_path = env.repo.join(".agents/code/tasks/issue-task/task.json");
    let mut task_json: Value =
        serde_json::from_str(&fs::read_to_string(&task_path).expect("task.json"))
            .expect("parse task.json");
    task_json["status"] = Value::String("running".to_string());
    fs::write(
        &task_path,
        serde_json::to_string_pretty(&task_json).expect("serialize task.json"),
    )
    .expect("write task.json");

    let prompt_file = env.home.path().join("issues_prompt.txt");
    let status = env
        .command()
        .args(["run-next", "issue-task"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("run-next");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(
        prompt.contains("REVIEW ISSUES"),
        "expected issues prompt injection"
    );
}

#[test]
fn run_held_task_uses_existing_spec_prompt() {
    let env = TestEnv::new();
    env.install_stub_capture("claude");
    env.install_stub_capture("codex");

    env.run(&["init"]);
    env.run(&["task", "held-task", "--hold"]);

    let prompt_file = env.home.path().join("spec_existing_prompt.txt");
    let status = env
        .command()
        .args(["run", "held-task"])
        .env("METAGENT_PROMPT_FILE", &prompt_file)
        .status()
        .expect("run held task");
    assert!(status.success());

    let prompt = fs::read_to_string(&prompt_file).expect("prompt content");
    assert!(
        prompt.contains("Task already exists: held-task"),
        "expected existing-task spec prompt"
    );
}
