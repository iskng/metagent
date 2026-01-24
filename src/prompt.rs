use crate::agent::AgentKind;
use crate::model::Model;
use crate::state::TaskStatus;

pub struct PromptContext<'a> {
    pub repo_root: &'a str,
    pub task: Option<&'a str>,
    pub session: Option<&'a str>,
    pub issues_header: &'a str,
    pub issues_mode: &'a str,
    pub parallelism_mode: &'a str,
    pub focus_section: &'a str,
}

pub fn render_prompt(template: &str, context: &PromptContext<'_>) -> String {
    let mut output = template.to_string();
    if let Some(task) = context.task {
        output = output.replace("{task}", task);
        output = output.replace("{taskname}", task);
    }
    if let Some(session) = context.session {
        output = output.replace("{session}", session);
    } else {
        output = output.replace("{session}", "");
    }
    output = output.replace("{repo}", context.repo_root);
    output = output.replace("{issues_header}", context.issues_header);
    output = output.replace("{issues_mode}", context.issues_mode);
    output = output.replace("{parallelism_mode}", context.parallelism_mode);
    output = output.replace("{focus_section}", context.focus_section);
    output
}

pub fn issues_text(agent: AgentKind, status: Option<&TaskStatus>, task: Option<&str>) -> (String, String) {
    if agent != AgentKind::Code {
        return (String::new(), String::new());
    }
    if status != Some(&TaskStatus::Issues) {
        return (String::new(), String::new());
    }
    let task = match task {
        Some(task) => task,
        None => return (String::new(), String::new()),
    };
    let header = format!(
        "0d. Check @.agents/code/tasks/{task}/issues.md - Open issues from review MUST be addressed first\n\n1. **PRIORITY: Review Issues** - If issues.md exists with open issues, address those FIRST. These are requirements clarifications or architectural decisions needed. Update issue status to \"resolved\" when addressed."
    );
    let mode = format!(
        "99999999999999. **REVIEW ISSUES:** This task returned from review with issues. All issues in @.agents/code/tasks/{task}/issues.md must be resolved before finishing this phase."
    );
    (header, mode)
}

pub fn parallelism_text(model: Model) -> String {
    if model != Model::Claude {
        return String::new();
    }
    "## Parallelism\n- Use subagents liberally for research before implementing\n- Codebase search: up to 100 subagents\n- File reading: up to 100 subagents\n- File writing: up to 10 subagents (independent files only)\n- Build/test: 1 subagent only\n- plan.md updates: 1 subagent"
        .to_string()
}
