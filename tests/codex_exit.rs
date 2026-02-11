use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore]
fn codex_exit_methods() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CODEX_EXIT_TEST").ok().as_deref() != Some("1") {
        eprintln!("Skipping codex exit test (set CODEX_EXIT_TEST=1 to run).");
        return Ok(());
    }

    let mode = SpawnMode::from_env();
    println!("spawn mode: {}", mode.label());

    for method in methods_for_mode(mode) {
        let result = run_once(method, mode)?;
        println!("{}", result);
    }

    Ok(())
}

enum Method {
    Key(&'static str, &'static [u8]),
    Signal(&'static str, i32),
    CloseStdin,
}

#[derive(Clone, Copy)]
enum SpawnMode {
    Pty,
    Inherit,
}

impl SpawnMode {
    fn from_env() -> Self {
        match std::env::var("CODEX_SPAWN_MODE").ok().as_deref() {
            Some("pty") => SpawnMode::Pty,
            Some("inherit") => SpawnMode::Inherit,
            None => SpawnMode::Inherit,
            Some(other) => {
                eprintln!("Unknown CODEX_SPAWN_MODE='{other}', defaulting to inherit.");
                SpawnMode::Inherit
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            SpawnMode::Pty => "pty",
            SpawnMode::Inherit => "inherit",
        }
    }
}

struct ExitResult {
    name: &'static str,
    status: Option<ExitStatus>,
    elapsed: Duration,
    note: &'static str,
}

impl std::fmt::Display for ExitResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self.status {
            Some(status) => describe_status(status),
            None => "still running".to_string(),
        };
        write!(
            f,
            "{:<10} -> {:<15} ({:?}) {}",
            self.name, status, self.elapsed, self.note
        )
    }
}

fn run_once(method: Method, mode: SpawnMode) -> io::Result<ExitResult> {
    let (mut child, mut master_opt) = spawn_codex(mode)?;
    let pid = child.id() as i32;

    if let Some(status) = wait_for_exit(&mut child, Duration::from_secs(2), master_opt.as_ref())? {
        return Ok(ExitResult {
            name: method_name(&method),
            status: Some(status),
            elapsed: Duration::from_secs(0),
            note: "exited early",
        });
    }

    let start = Instant::now();
    match method {
        Method::Key(_, bytes) => {
            if let Some(master) = master_opt.as_mut() {
                master.write_all(bytes)?;
                master.flush()?;
            }
        }
        Method::Signal(_, sig) => {
            send_signal(pid, sig);
        }
        Method::CloseStdin => {
            master_opt.take();
        }
    }

    let status = wait_for_exit(&mut child, Duration::from_secs(6), master_opt.as_ref())?;
    let note = if status.is_some() {
        "exited"
    } else {
        force_kill(pid, &mut child, master_opt.as_ref());
        "forced kill"
    };

    Ok(ExitResult {
        name: method_name(&method),
        status,
        elapsed: start.elapsed(),
        note,
    })
}

fn spawn_codex(mode: SpawnMode) -> io::Result<(Child, Option<File>)> {
    match mode {
        SpawnMode::Pty => spawn_codex_pty(),
        SpawnMode::Inherit => spawn_codex_inherit(),
    }
}

fn spawn_codex_pty() -> io::Result<(Child, Option<File>)> {
    let mut master_fd = 0;
    let mut slave_fd = 0;
    let open_result = unsafe {
        libc::openpty(
            &mut master_fd,
            &mut slave_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut::<libc::termios>(),
            std::ptr::null_mut::<libc::winsize>(),
        )
    };
    if open_result != 0 {
        return Err(io::Error::last_os_error());
    }

    let master = unsafe { File::from_raw_fd(master_fd) };
    set_nonblocking(master.as_raw_fd())?;
    let slave = unsafe { File::from_raw_fd(slave_fd) };

    let mut cmd = Command::new("codex");
    cmd.arg("--dangerously-bypass-approvals-and-sandbox")
        .arg(test_prompt())
        .stdin(slave.try_clone()?)
        .stdout(slave.try_clone()?)
        .stderr(slave)
        .env("TERM", "xterm-256color");
    apply_mung_env(&mut cmd);

    unsafe {
        cmd.pre_exec(move || {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            let _ = libc::ioctl(0, libc::TIOCSCTTY as libc::c_ulong, 0);
            Ok(())
        });
    }

    let child = cmd.spawn()?;
    Ok((child, Some(master)))
}

fn spawn_codex_inherit() -> io::Result<(Child, Option<File>)> {
    let mut cmd = Command::new("codex");
    cmd.arg("--dangerously-bypass-approvals-and-sandbox")
        .arg(test_prompt())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    apply_mung_env(&mut cmd);

    let child = cmd.spawn()?;
    Ok((child, None))
}

fn wait_for_exit(
    child: &mut Child,
    timeout: Duration,
    master_ref: Option<&File>,
) -> io::Result<Option<ExitStatus>> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if let Some(master) = master_ref {
            drain_master(master);
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(None)
}

fn send_signal(pid: i32, sig: i32) {
    unsafe {
        let _ = libc::kill(pid, sig);
    }
}

fn force_kill(pid: i32, child: &mut Child, master_ref: Option<&File>) {
    send_signal(pid, libc::SIGTERM);
    let _ = wait_for_exit(child, Duration::from_secs(2), master_ref);
    send_signal(pid, libc::SIGKILL);
    let _ = wait_for_exit(child, Duration::from_secs(2), master_ref);
    let _ = child.kill();
    let _ = wait_for_exit(child, Duration::from_secs(2), master_ref);
}

fn set_nonblocking(fd: i32) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let next = flags | libc::O_NONBLOCK;
    if unsafe { libc::fcntl(fd, libc::F_SETFL, next) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn drain_master(master: &File) {
    let mut buf = [0u8; 4096];
    loop {
        match (&*master).read(&mut buf) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
}

fn describe_status(status: ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("exit {code}");
    }
    if let Some(signal) = status.signal() {
        return format!("signal {signal}");
    }
    "unknown".to_string()
}

fn method_name(method: &Method) -> &'static str {
    match method {
        Method::Key(name, _) => name,
        Method::Signal(name, _) => name,
        Method::CloseStdin => "close_stdin",
    }
}

fn methods_for_mode(mode: SpawnMode) -> Vec<Method> {
    let mut methods = Vec::new();
    match mode {
        SpawnMode::Pty => {
            methods.push(Method::Key("ctrl_c", b"\x03"));
            methods.push(Method::Key("q", b"q"));
            methods.push(Method::Key("esc", b"\x1b"));
            methods.push(Method::Key("ctrl_d", b"\x04"));
            methods.push(Method::Signal("sigint", libc::SIGINT));
            methods.push(Method::Signal("sigterm", libc::SIGTERM));
            methods.push(Method::Signal("sighup", libc::SIGHUP));
            methods.push(Method::Signal("sigquit", libc::SIGQUIT));
            methods.push(Method::CloseStdin);
        }
        SpawnMode::Inherit => {
            methods.push(Method::Signal("sigint", libc::SIGINT));
        }
    }
    methods
}

fn test_prompt() -> &'static str {
    "Task: codex-exit-test\n\nStart, then wait for shutdown."
}

fn apply_mung_env(cmd: &mut Command) {
    cmd.env("MUNG_AGENT", "code")
        .env("MUNG_SESSION", "exit-test-session")
        .env("MUNG_TASK", "codex-exit-test");
    if let Ok(cwd) = std::env::current_dir() {
        cmd.env("MUNG_REPO_ROOT", cwd);
    }
}
