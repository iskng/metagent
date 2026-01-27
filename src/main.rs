use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

mod agent;
mod assets;
mod commands;
mod model;
mod issues;
mod prompt;
mod state;
mod util;

use agent::AgentKind;
use commands::{
    cmd_debug, cmd_dequeue, cmd_finish, cmd_init, cmd_install, cmd_queue, cmd_run, cmd_run_queue,
    cmd_spec_review, cmd_start, cmd_task, cmd_review, cmd_uninstall, CommandContext, ModelChoice,
    IssueCommands, INTERRUPTED,
};
use model::Model;
use util::get_repo_root;

#[derive(Parser)]
#[command(name = "metagent")]
#[command(version)]
#[command(about = "Agent workflow manager", long_about = None)]
struct Cli {
    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    model: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Uninstall,
    Init { path: Option<PathBuf> },
    Start,
    Task {
        name: String,
        #[arg(long)]
        hold: bool,
    },
    Hold { name: String },
    Activate { name: String },
    Finish {
        stage: Option<String>,
        #[arg(long)]
        next: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        task: Option<String>,
    },
    Run { name: String },
    #[command(name = "run-next", alias = "rn")]
    RunNext,
    #[command(alias = "q")]
    Queue { task: Option<String> },
    Dequeue { name: String },
    #[command(name = "run-queue", alias = "rq")]
    RunQueue,
    Review { task: String, focus: Option<String> },
    #[command(name = "spec-review")]
    SpecReview { task: String },
    #[command(name = "set-stage")]
    SetStage {
        name: String,
        stage: String,
        #[arg(long)]
        status: Option<String>,
    },
    Issues {
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
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    Debug {
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        stdin: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        bug: Vec<String>,
    },
}

fn main() -> Result<()> {
    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .context("Failed to install CTRL-C handler")?;

    let cli = Cli::parse();
    let agent_value = cli
        .agent
        .or_else(|| env::var("METAGENT_AGENT").ok())
        .unwrap_or_else(|| "code".to_string());
    let agent = AgentKind::from_str(&agent_value)?;

    let model_choice = resolve_model_choice(cli.model)?;

    match cli.command.unwrap_or(Commands::Start) {
        Commands::Install => cmd_install(),
        Commands::Uninstall => cmd_uninstall(),
        Commands::Init { path } => cmd_init(agent, path, model_choice),
        Commands::Start => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_start(&ctx)
        }
        Commands::Task { name, hold } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_task(&ctx, &name, hold)
        }
        Commands::Hold { name } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_hold(&ctx, &name)
        }
        Commands::Activate { name } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_activate(&ctx, &name)
        }
        Commands::Finish {
            stage,
            next,
            session,
            task,
        } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_finish(&ctx, stage, next, session, task)
        }
        Commands::Run { name } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_run(&ctx, &name)
        }
        Commands::RunNext => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_run_next(&ctx)
        }
        Commands::Queue { task } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_queue(&ctx, task.as_deref())
        }
        Commands::Dequeue { name } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_dequeue(&ctx, &name)
        }
        Commands::RunQueue => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_run_queue(&ctx)
        }
        Commands::Review { task, focus } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_review(&ctx, &task, focus)
        }
        Commands::SpecReview { task } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_spec_review(&ctx, &task)
        }
        Commands::SetStage { name, stage, status } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_set_stage(&ctx, &name, &stage, status)
        }
        Commands::Issues {
            task,
            unassigned,
            status,
            priority,
            issue_type,
            source,
        } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_issues(&ctx, task, unassigned, status, priority, issue_type, source)
        }
        Commands::Issue { command } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_issue(&ctx, command)
        }
        Commands::Debug { file, stdin, bug } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_debug(&ctx, bug, file, stdin)
        }
    }
}

fn resolve_model_choice(flag: Option<String>) -> Result<ModelChoice> {
    let env_model = env::var("METAGENT_MODEL").ok();
    if let Some(flag) = flag {
        return Ok(ModelChoice {
            model: Model::from_str(&flag)?,
            explicit: true,
        });
    }
    if let Some(env_model) = env_model {
        return Ok(ModelChoice {
            model: Model::from_str(&env_model)?,
            explicit: true,
        });
    }
    Ok(ModelChoice {
        model: Model::Claude,
        explicit: false,
    })
}
