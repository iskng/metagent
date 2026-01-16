use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Claude,
    Codex,
}

impl Model {
    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            _ => bail!("Unknown model: {value}"),
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    pub fn command(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Claude => ("claude", &["--dangerously-skip-permissions"]),
            Self::Codex => ("codex", &["--dangerously-bypass-approvals-and-sandbox"]),
        }
    }
}
