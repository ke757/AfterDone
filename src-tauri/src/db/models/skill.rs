use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    Requested,
    Generated,
    Verified,
    Failed,
}

impl std::fmt::Display for SkillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillStatus::Requested => write!(f, "requested"),
            SkillStatus::Generated => write!(f, "generated"),
            SkillStatus::Verified => write!(f, "verified"),
            SkillStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TryFrom<&str> for SkillStatus {
    type Error = String;
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "requested" => Ok(SkillStatus::Requested),
            "generated" => Ok(SkillStatus::Generated),
            "verified" => Ok(SkillStatus::Verified),
            "failed" => Ok(SkillStatus::Failed),
            _ => Err(format!("Unknown skill status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: String,
    pub milestone_id: String,
    pub name: String,
    pub description: String,
    pub harness_skill_id: Option<String>,
    pub parameters_schema: Option<String>,
    pub test_report: Option<String>,
    pub status: String,
    pub source_code_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
