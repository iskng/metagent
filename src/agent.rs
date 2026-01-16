use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::assets;
use crate::model::Model;
use crate::util::{today_date, write_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    Code,
    Writer,
}

impl AgentKind {
    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "code" => Ok(Self::Code),
            "writer" => Ok(Self::Writer),
            _ => bail!("Unknown agent: {value}"),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Writer => "writer",
        }
    }

    pub fn stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &["spec", "spec-review", "planning", "build", "review", "completed"],
            Self::Writer => &["init", "plan", "write", "completed"],
        }
    }

    #[allow(dead_code)]
    pub fn orchestrated_stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &["spec", "planning"],
            Self::Writer => &["init", "plan", "write"],
        }
    }

    pub fn handoff_stage(&self) -> Option<&'static str> {
        match self {
            Self::Code => Some("build"),
            Self::Writer => None,
        }
    }

    pub fn initial_stage(&self) -> &'static str {
        match self {
            Self::Code => "spec",
            Self::Writer => "init",
        }
    }

    pub fn next_stage(&self, stage: &str) -> Option<&'static str> {
        match self {
            Self::Code => match stage {
                "spec" => Some("planning"),
                "spec-review" => Some("planning"),
                "planning" => Some("build"),
                "build" => Some("review"),
                "review" => Some("completed"),
                "task" => Some("completed"),
                _ => None,
            },
            Self::Writer => match stage {
                "init" => Some("plan"),
                "plan" => Some("write"),
                "write" => Some("completed"),
                _ => None,
            },
        }
    }

    pub fn valid_finish_stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &["spec", "spec-review", "planning", "build", "review", "task"],
            Self::Writer => &["init", "plan", "write"],
        }
    }

    pub fn stage_label(&self, stage: &str) -> String {
        match self {
            Self::Code => match stage {
                "spec" => "Spec",
                "spec-review" => "Spec Review",
                "planning" => "Planning",
                "build" => "Build",
                "review" => "Review",
                "completed" => "Completed",
                _ => stage,
            },
            Self::Writer => match stage {
                "init" => "Init",
                "plan" => "Plan",
                "write" => "Write",
                "completed" => "Completed",
                _ => stage,
            },
        }
        .to_string()
    }

    pub fn prompt_file_for_stage(&self, stage: &str, task: Option<&str>, repo_root: &Path) -> Option<PathBuf> {
        match self {
            Self::Code => match stage {
                "spec" => Some(PathBuf::from("SPEC_PROMPT.md")),
                "spec-review" => Some(PathBuf::from("SPEC_REVIEW_PROMPT.md")),
                "planning" => Some(PathBuf::from("PLANNING_PROMPT.md")),
                "build" => task.map(|task| repo_root.join(".agents/code/tasks").join(task).join("PROMPT.md")),
                "review" => Some(PathBuf::from("REVIEW_PROMPT.md")),
                _ => None,
            },
            Self::Writer => match stage {
                "init" => Some(PathBuf::from("INIT_PROMPT.md")),
                "plan" => Some(PathBuf::from("PLANNING_PROMPT.md")),
                "write" => Some(PathBuf::from("PROMPT.md")),
                _ => None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn review_prompt_name(&self) -> Option<&'static str> {
        match self {
            Self::Code => Some("REVIEW_PROMPT.md"),
            Self::Writer => None,
        }
    }

    #[allow(dead_code)]
    pub fn spec_review_prompt_name(&self) -> Option<&'static str> {
        match self {
            Self::Code => Some("SPEC_REVIEW_PROMPT.md"),
            Self::Writer => None,
        }
    }

    pub fn model_for_stage(&self, stage: &str) -> Option<Model> {
        match self {
            Self::Code => match stage {
                "spec-review" | "planning" | "build" | "review" => Some(Model::Codex),
                _ => None,
            },
            Self::Writer => None,
        }
    }

    pub fn embedded_prompt(&self, file_name: &str) -> Option<&'static str> {
        match self {
            Self::Code => match file_name {
                "BOOTSTRAP_PROMPT.md" => Some(assets::CODE_BOOTSTRAP_PROMPT),
                "SPEC_PROMPT.md" => Some(assets::CODE_SPEC_PROMPT),
                "PLANNING_PROMPT.md" => Some(assets::CODE_PLANNING_PROMPT),
                "DEBUG_PROMPT.md" => Some(assets::CODE_DEBUG_PROMPT),
                "SUBMIT_ISSUE_PROMPT.md" => Some(assets::CODE_SUBMIT_ISSUE_PROMPT),
                "RECOVERY_PROMPT.md" => Some(assets::CODE_RECOVERY_PROMPT),
                "REFRESH_PROMPT.md" => Some(assets::CODE_REFRESH_PROMPT),
                "REVIEW_PROMPT.md" => Some(assets::CODE_REVIEW_PROMPT),
                "SPEC_REVIEW_PROMPT.md" => Some(assets::CODE_SPEC_REVIEW_PROMPT),
                _ => None,
            },
            Self::Writer => match file_name {
                "INIT_PROMPT.md" => Some(assets::WRITER_INIT_PROMPT),
                "PLANNING_PROMPT.md" => Some(assets::WRITER_PLANNING_PROMPT),
                "PROMPT.md" => Some(assets::WRITER_PROMPT),
                _ => None,
            },
        }
    }

    pub fn install_prompts(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("BOOTSTRAP_PROMPT.md", assets::CODE_BOOTSTRAP_PROMPT),
                ("SPEC_PROMPT.md", assets::CODE_SPEC_PROMPT),
                ("PLANNING_PROMPT.md", assets::CODE_PLANNING_PROMPT),
                ("DEBUG_PROMPT.md", assets::CODE_DEBUG_PROMPT),
                ("SUBMIT_ISSUE_PROMPT.md", assets::CODE_SUBMIT_ISSUE_PROMPT),
                ("RECOVERY_PROMPT.md", assets::CODE_RECOVERY_PROMPT),
                ("REFRESH_PROMPT.md", assets::CODE_REFRESH_PROMPT),
                ("REVIEW_PROMPT.md", assets::CODE_REVIEW_PROMPT),
                ("SPEC_REVIEW_PROMPT.md", assets::CODE_SPEC_REVIEW_PROMPT),
                ("agent.sh", assets::CODE_AGENT_SH),
            ],
            Self::Writer => vec![
                ("INIT_PROMPT.md", assets::WRITER_INIT_PROMPT),
                ("PLANNING_PROMPT.md", assets::WRITER_PLANNING_PROMPT),
                ("PROMPT.md", assets::WRITER_PROMPT),
                ("agent.sh", assets::WRITER_AGENT_SH),
            ],
        }
    }

    pub fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("AGENTS.md", assets::CODE_TEMPLATE_AGENTS),
                ("SPEC.md", assets::CODE_TEMPLATE_SPEC),
                ("TECHNICAL_STANDARDS.md", assets::CODE_TEMPLATE_TECHNICAL_STANDARDS),
            ],
            Self::Writer => vec![("AGENTS.md", assets::WRITER_TEMPLATE_AGENTS)],
        }
    }

    pub fn create_task(&self, task_dir: &Path, task: &str) -> Result<()> {
        match self {
            Self::Code => {
                std::fs::create_dir_all(task_dir.join("spec"))?;
                let plan = format!(
                    "# Implementation Plan - {task}\n\n> Generated: {}\n> Status: PENDING_SPEC\n\n- [ ] (tasks will be added during planning phase)\n",
                    today_date()
                );
                write_text(&task_dir.join("plan.md"), &plan)?;

                let prompt = code_build_prompt(task);
                write_text(&task_dir.join("PROMPT.md"), &prompt)?;
            }
            Self::Writer => {
                std::fs::create_dir_all(task_dir.join("content"))?;
                std::fs::create_dir_all(task_dir.join("outline"))?;
                std::fs::create_dir_all(task_dir.join("style"))?;
                std::fs::create_dir_all(task_dir.join("research"))?;
                let editorial = format!(
                    "# Editorial Plan - {task}\n\n> Generated: {}\n> Status: Awaiting project setup\n\n## Current Task\n\nRun /writer-init to set up the project.\n\n## Section Status\n\n| Section | Status | Progress | Notes |\n|---------|--------|----------|-------|\n| (sections added after init) | - | - | - |\n\n## Issues & Blockers\n\n(none yet)\n",
                    today_date()
                );
                write_text(&task_dir.join("editorial_plan.md"), &editorial)?;
            }
        }
        Ok(())
    }

    pub fn slash_commands(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("bootstrap.md", "BOOTSTRAP_PROMPT.md"),
                ("spec.md", "SPEC_PROMPT.md"),
                ("planner.md", "PLANNING_PROMPT.md"),
                ("debug.md", "DEBUG_PROMPT.md"),
                ("submit-issue.md", "SUBMIT_ISSUE_PROMPT.md"),
            ],
            Self::Writer => vec![
                ("writer-init.md", "INIT_PROMPT.md"),
                ("writer-plan.md", "PLANNING_PROMPT.md"),
                ("writer.md", "PROMPT.md"),
            ],
        }
    }
}

fn code_build_prompt(task: &str) -> String {
    format!(
        r#"0a. Study all files in @.agents/code/tasks/{task}/spec/ - Task specifications and architecture
0b. Study @.agents/code/TECHNICAL_STANDARDS.md - Codebase patterns to follow
0c. Study @.agents/code/tasks/{task}/plan.md - Current task list
0d. Study @.agents/code/AGENTS.md - Build/test commands and learnings
{{issues_header}}

1. Your task is to implement {task} per the specifications. Study @plan.md, choose the most important uncompleted items that you can accomplish in one pass (max 5), research before implementing (NEVER assume code doesn't exist), implement according to specifications.

2. After implementing functionality or resolving problems, run the tests for that unit of code that was improved. If functionality is missing then it's your job to add it as per the application specifications.

3. When the tests pass update @plan.md, then add changed code and @plan.md with git add the relevant files you created/modified via bash then do git commit -m "feat({task}): [descriptive message]"

4. ALWAYS KEEP @plan.md up to date with your learnings about the task. After wrapping up/finishing your turn append a short session-x summary with what was accomplished and any relevant notes.

5. When you learn something new about how to run the build/tests make sure you update @.agents/code/AGENT.md but keep it brief.

999999. Important: We want single sources of truth, no migrations/adapters. If tests unrelated to your work fail then it's your job to resolve these tests as part of the increment of change.
99999999. Important: When authoring tests capture the WHY - document importance in docstrings.
999999999. IMPORTANT: When you discover a bug resolve it even if it is unrelated to the current piece of work after documenting it in @plan.md
9999999999. You may add extra logging if required to be able to debug the issues.
99999999999. If you find inconsistencies in the specs/* then use the oracle (think extra hard) and then update the specs.
999999999999. FULL IMPLEMENTATIONS ONLY. NO PLACEHOLDERS. NO STUBS. NO TODO COMMENTS. DO NOT IMPLEMENT PLACEHOLDER OR SIMPLE IMPLEMENTATIONS. WE WANT FULL IMPLEMENTATIONS. DO IT OR I WILL YELL AT YOU
9999999999999. SUPER IMPORTANT DO NOT IGNORE. DO NOT PLACE STATUS REPORT UPDATES INTO @.agents/code/AGENT.md
99999999999999. **WHEN ITEM DONE:** run `cd "{{repo}}" && METAGENT_SESSION="{{session}}" METAGENT_TASK="{{task}}" metagent --agent code finish --next build` to signal iteration complete (more items remain).
999999999999999. **WHEN ALL ASPECTS OF THE PLAN.md ARE COMPLETE:** run a full `cargo build` to verify everything compiles, then run `cd "{{repo}}" && METAGENT_SESSION="{{session}}" METAGENT_TASK="{{task}}" metagent --agent code finish` to signal task complete (all items done).
{{issues_mode}}
{{parallelism_mode}}
"#
    )
}
