use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

mod agent;
mod assets;
mod commands;
mod issues;
mod model;
mod prompt;
mod state;
mod util;

use agent::AgentKind;
use commands::{
    cmd_debug, cmd_delete, cmd_finish, cmd_init, cmd_install, cmd_plan, cmd_queue, cmd_review,
    cmd_run, cmd_run_queue, cmd_spec_review, cmd_start, cmd_task, cmd_uninstall, CommandContext,
    IssueCommands, ModelChoice, INTERRUPTED,
};
use model::Model;
use util::{env_var, get_repo_root};

#[derive(Parser)]
#[command(name = "mung")]
#[command(version)]
#[command(about = "Agent workflow manager", long_about = None)]
struct Cli {
    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    model: Option<String>,

    #[arg(long)]
    force_model: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Uninstall,
    Init {
        path: Option<PathBuf>,
    },
    Start,
    Task {
        name: String,
        #[arg(long)]
        hold: bool,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
    },
    Hold {
        name: String,
    },
    Activate {
        name: String,
    },
    Finish {
        stage: Option<String>,
        #[arg(long)]
        next: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        task: Option<String>,
    },
    Run {
        name: String,
    },
    #[command(name = "run-next", alias = "rn")]
    RunNext {
        name: Option<String>,
    },
    #[command(alias = "q")]
    Queue {
        task: Option<String>,
    },
    Plan {
        task: String,
    },
    #[command(name = "delete", alias = "dequeue")]
    Delete {
        name: String,
        #[arg(long)]
        force: bool,
    },
    Reorder {
        name: String,
        position: usize,
    },
    #[command(name = "run-queue", alias = "rq")]
    RunQueue {
        #[arg(
            long,
            default_value_t = 4,
            help = "Max review->build loops before holding (0 = 100)"
        )]
        r#loop: usize,
    },
    Review {
        task: String,
        focus: Option<String>,
    },
    #[command(name = "spec-review")]
    SpecReview {
        task: String,
    },
    Research {
        task: String,
        focus: Option<String>,
    },
    How {
        topic: Option<String>,
    },
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
        .or_else(|| env_var("MUNG_AGENT", "METAGENT_AGENT"))
        .unwrap_or_else(|| "code".to_string());
    let agent = AgentKind::from_str(&agent_value)?;

    let model_choice = resolve_model_choice(cli.model, cli.force_model)?;

    match cli.command.unwrap_or(Commands::Start) {
        Commands::Install => cmd_install(),
        Commands::Uninstall => cmd_uninstall(),
        Commands::Init { path } => cmd_init(agent, path, model_choice),
        Commands::Start => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_start(&ctx)
        }
        Commands::Task {
            name,
            hold,
            description,
            prompt,
        } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_task(&ctx, &name, hold, description, prompt)
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
        Commands::RunNext { name } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_run_next(&ctx, name.as_deref())
        }
        Commands::Queue { task } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_queue(&ctx, task.as_deref())
        }
        Commands::Plan { task } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_plan(&ctx, &task)
        }
        Commands::Delete { name, force } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_delete(&ctx, &name, force)
        }
        Commands::Reorder { name, position } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_reorder(&ctx, &name, position)
        }
        Commands::RunQueue { r#loop } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            cmd_run_queue(&ctx, r#loop)
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
        Commands::Research { task, focus } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_research(&ctx, &task, focus)
        }
        Commands::How { topic } => {
            let repo_root = get_repo_root(None)?;
            let ctx = CommandContext::new(agent, model_choice, repo_root)?;
            commands::cmd_how(&ctx, topic.as_deref())
        }
        Commands::SetStage {
            name,
            stage,
            status,
        } => {
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

fn resolve_model_choice(flag: Option<String>, force_model_flag: bool) -> Result<ModelChoice> {
    let env_model = env_var("MUNG_MODEL", "METAGENT_MODEL");
    let env_force = env_var("MUNG_FORCE_MODEL", "METAGENT_FORCE_MODEL")
        .map(|value| matches!(value.trim().to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    let force_model = force_model_flag || env_force;

    if let Some(flag) = flag {
        return Ok(ModelChoice {
            model: Model::from_str(&flag)?,
            explicit: true,
            force_model,
        });
    }
    if let Some(env_model) = env_model {
        return Ok(ModelChoice {
            model: Model::from_str(&env_model)?,
            explicit: true,
            force_model,
        });
    }
    Ok(ModelChoice {
        model: Model::Claude,
        explicit: false,
        force_model,
    })
}
