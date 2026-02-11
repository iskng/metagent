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
            Self::Code => &[
                "spec",
                "spec-review",
                "spec-review-issues",
                "planning",
                "build",
                "review",
                "completed",
            ],
            Self::Writer => &["init", "plan", "write", "edit", "completed"],
        }
    }

    #[allow(dead_code)]
    pub fn orchestrated_stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &["spec", "planning"],
            Self::Writer => &["init", "plan", "write", "edit"],
        }
    }

    pub fn handoff_stage(&self) -> Option<&'static str> {
        match self {
            Self::Code => Some("build"),
            Self::Writer => None,
        }
    }

    /// Stages that run-queue will process (no spec/planning)
    pub fn queue_stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &["spec-review-issues", "build", "review"],
            Self::Writer => &["write", "edit"],
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
                "spec-review-issues" => Some("planning"),
                "planning" => Some("build"),
                "build" => Some("review"),
                "review" => Some("completed"),
                "task" => Some("completed"),
                _ => None,
            },
            Self::Writer => match stage {
                "init" => Some("plan"),
                "plan" => Some("write"),
                "write" => Some("edit"),
                "edit" => Some("completed"),
                _ => None,
            },
        }
    }

    pub fn valid_finish_stages(&self) -> &'static [&'static str] {
        match self {
            Self::Code => &[
                "spec",
                "spec-review",
                "spec-review-issues",
                "planning",
                "build",
                "review",
                "task",
            ],
            Self::Writer => &["init", "plan", "write", "edit"],
        }
    }

    pub fn stage_label(&self, stage: &str) -> String {
        match self {
            Self::Code => match stage {
                "spec" => "Spec",
                "spec-review" => "Spec Review",
                "spec-review-issues" => "Spec Review Issues",
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
                "edit" => "Edit",
                "completed" => "Completed",
                _ => stage,
            },
        }
        .to_string()
    }

    pub fn prompt_file_for_stage(&self, stage: &str, task: Option<&str>) -> Option<PathBuf> {
        match self {
            Self::Code => match stage {
                "spec" => {
                    if task.is_some() {
                        Some(PathBuf::from("SPEC_EXISTING_TASK_PROMPT.md"))
                    } else {
                        Some(PathBuf::from("SPEC_PROMPT.md"))
                    }
                }
                "spec-review" => Some(PathBuf::from("SPEC_REVIEW_PROMPT.md")),
                "spec-review-issues" => Some(PathBuf::from("SPEC_REVIEW_ISSUES_PROMPT.md")),
                "planning" => Some(PathBuf::from("PLANNING_PROMPT.md")),
                "build" => Some(PathBuf::from("BUILD_PROMPT.md")),
                "review" => Some(PathBuf::from("REVIEW_PROMPT.md")),
                _ => None,
            },
            Self::Writer => match stage {
                "init" => Some(PathBuf::from("INIT_PROMPT.md")),
                "plan" => Some(PathBuf::from("PLANNING_PROMPT.md")),
                "write" => Some(PathBuf::from("PROMPT.md")),
                "edit" => Some(PathBuf::from("EDITOR_PROMPT.md")),
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
                "spec" | "spec-review" | "spec-review-issues" | "planning" | "build" | "review" => {
                    Some(Model::Codex)
                }
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
                "SPEC_EXISTING_TASK_PROMPT.md" => Some(assets::CODE_SPEC_EXISTING_PROMPT),
                "PLANNING_PROMPT.md" => Some(assets::CODE_PLANNING_PROMPT),
                "BUILD_PROMPT.md" => Some(assets::CODE_BUILD_PROMPT),
                "DEBUG_PROMPT.md" => Some(assets::CODE_DEBUG_PROMPT),
                "SUBMIT_ISSUE_PROMPT.md" => Some(assets::CODE_SUBMIT_ISSUE_PROMPT),
                "SUBMIT_TASK_PROMPT.md" => Some(assets::CODE_SUBMIT_TASK_PROMPT),
                "SUBMIT_HOLD_TASK_PROMPT.md" => Some(assets::CODE_SUBMIT_HOLD_TASK_PROMPT),
                "RECOVERY_PROMPT.md" => Some(assets::CODE_RECOVERY_PROMPT),
                "REFRESH_PROMPT.md" => Some(assets::CODE_REFRESH_PROMPT),
                "REVIEW_PROMPT.md" => Some(assets::CODE_REVIEW_PROMPT),
                "SPEC_REVIEW_PROMPT.md" => Some(assets::CODE_SPEC_REVIEW_PROMPT),
                "SPEC_REVIEW_ISSUES_PROMPT.md" => Some(assets::CODE_SPEC_REVIEW_ISSUES_PROMPT),
                "RESEARCH_PROMPT.md" => Some(assets::CODE_RESEARCH_PROMPT),
                "how/commit.md" => Some(assets::CODE_HOW_COMMIT),
                "how/plan-update.md" => Some(assets::CODE_HOW_PLAN_UPDATE),
                _ => None,
            },
            Self::Writer => match file_name {
                "INIT_PROMPT.md" => Some(assets::WRITER_INIT_PROMPT),
                "PLANNING_PROMPT.md" => Some(assets::WRITER_PLANNING_PROMPT),
                "PROMPT.md" => Some(assets::WRITER_PROMPT),
                "EDITOR_PROMPT.md" => Some(assets::WRITER_EDITOR_PROMPT),
                _ => None,
            },
        }
    }

    pub fn install_prompts(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("BOOTSTRAP_PROMPT.md", assets::CODE_BOOTSTRAP_PROMPT),
                ("SPEC_PROMPT.md", assets::CODE_SPEC_PROMPT),
                (
                    "SPEC_EXISTING_TASK_PROMPT.md",
                    assets::CODE_SPEC_EXISTING_PROMPT,
                ),
                ("PLANNING_PROMPT.md", assets::CODE_PLANNING_PROMPT),
                ("BUILD_PROMPT.md", assets::CODE_BUILD_PROMPT),
                ("DEBUG_PROMPT.md", assets::CODE_DEBUG_PROMPT),
                ("SUBMIT_ISSUE_PROMPT.md", assets::CODE_SUBMIT_ISSUE_PROMPT),
                ("SUBMIT_TASK_PROMPT.md", assets::CODE_SUBMIT_TASK_PROMPT),
                (
                    "SUBMIT_HOLD_TASK_PROMPT.md",
                    assets::CODE_SUBMIT_HOLD_TASK_PROMPT,
                ),
                ("RECOVERY_PROMPT.md", assets::CODE_RECOVERY_PROMPT),
                ("REFRESH_PROMPT.md", assets::CODE_REFRESH_PROMPT),
                ("REVIEW_PROMPT.md", assets::CODE_REVIEW_PROMPT),
                ("SPEC_REVIEW_PROMPT.md", assets::CODE_SPEC_REVIEW_PROMPT),
                ("RESEARCH_PROMPT.md", assets::CODE_RESEARCH_PROMPT),
                ("how/commit.md", assets::CODE_HOW_COMMIT),
                ("how/plan-update.md", assets::CODE_HOW_PLAN_UPDATE),
            ],
            Self::Writer => vec![
                ("INIT_PROMPT.md", assets::WRITER_INIT_PROMPT),
                ("PLANNING_PROMPT.md", assets::WRITER_PLANNING_PROMPT),
                ("PROMPT.md", assets::WRITER_PROMPT),
                ("EDITOR_PROMPT.md", assets::WRITER_EDITOR_PROMPT),
            ],
        }
    }

    pub fn how_topics(&self) -> Vec<&'static str> {
        match self {
            Self::Code => vec!["commit", "plan-update"],
            Self::Writer => Vec::new(),
        }
    }

    pub fn slash_commands(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("BOOTSTRAP_PROMPT.md", "bootstrap"),
                ("SPEC_PROMPT.md", "spec"),
                ("PLANNING_PROMPT.md", "planner"),
                ("DEBUG_PROMPT.md", "debug"),
                ("SUBMIT_ISSUE_PROMPT.md", "submit-issue"),
                ("SUBMIT_TASK_PROMPT.md", "submit-task"),
                ("SUBMIT_HOLD_TASK_PROMPT.md", "submit-hold-task"),
            ],
            Self::Writer => vec![
                ("INIT_PROMPT.md", "writer-init"),
                ("PLANNING_PROMPT.md", "writer-plan"),
                ("PROMPT.md", "writer"),
            ],
        }
    }

    pub fn template_files(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::Code => vec![
                ("AGENTS.md", assets::CODE_TEMPLATE_AGENTS),
                ("SPEC.md", assets::CODE_TEMPLATE_SPEC),
                (
                    "TECHNICAL_STANDARDS.md",
                    assets::CODE_TEMPLATE_TECHNICAL_STANDARDS,
                ),
            ],
            Self::Writer => vec![("AGENTS.md", assets::WRITER_TEMPLATE_AGENTS)],
        }
    }

    pub fn create_task(&self, task_dir: &Path, task: &str) -> Result<()> {
        match self {
            Self::Code => {
                std::fs::create_dir_all(task_dir.join("spec"))?;
                let spec_dir = task_dir.join("spec");
                for (file, title) in [
                    ("overview.md", "Overview"),
                    ("types.md", "Types"),
                    ("modules.md", "Modules"),
                    ("errors.md", "Errors"),
                ] {
                    let path = spec_dir.join(file);
                    if !path.exists() {
                        write_text(&path, &format!("# {title}\n\n"))?;
                    }
                }
                let plan = format!(
                    "# Implementation Plan - {task}\n\n> Generated: {}\n> Status: PENDING_SPEC\n\n- [ ] (tasks will be added during planning phase)\n",
                    today_date()
                );
                write_text(&task_dir.join("plan.md"), &plan)?;
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
}
