use crate::com::atproto::admin::RepoRef;
use crate::com::atproto::repo::StrongRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ReasonType {
    #[serde(rename = "com.atproto.moderation.defs#reasonSpam")]
    Spam,
    #[serde(rename = "com.atproto.moderation.defs#reasonViolation")]
    Violation,
    #[serde(rename = "com.atproto.moderation.defs#reasonMisleading")]
    Misleading,
    #[serde(rename = "com.atproto.moderation.defs#reasonSexual")]
    Sexual,
    #[serde(rename = "com.atproto.moderation.defs#reasonRude")]
    Rude,
    #[serde(rename = "com.atproto.moderation.defs#reasonOther")]
    Other,
    #[serde(rename = "com.atproto.moderation.defs#reasonAppeal")]
    Appeal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubjectType {
    Account,
    Record,
    Chat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateReportInput {
    pub reason_type: ReasonType,
    pub reason: Option<String>,
    #[serde(rename = "subject")]
    pub subject: CreateReportSubject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_tool: Option<ModTool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum CreateReportSubject {
    #[serde(rename = "com.atproto.admin.defs#repoRef")]
    RepoRef(RepoRef),
    #[serde(rename = "com.atproto.repo.strongRef")]
    StrongRef(StrongRef),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ModTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateReportOutput {
    pub id: i64,
    pub reason_type: ReasonType,
    pub reason: Option<String>,
    #[serde(rename = "subject")]
    pub subject: CreateReportSubject,
    pub reported_by: String,
    pub created_at: String,
}
