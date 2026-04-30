use serde::{Deserialize, Serialize};

/// Pricing model for an agent service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPricing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_minute_usd_cents: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_session_usd_cents: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_monthly_usd_cents: Option<i64>,
}

/// `tools.know-me.agent.profile` — agent capability descriptor stored in PDS repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// E.g. `["text", "voice", "video", "image"]`
    pub modalities: Vec<String>,

    /// E.g. `"gpt-4o"`, `"claude-3-5-sonnet"`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<AgentPricing>,

    /// DID of the human/entity responsible for billing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_did: Option<String>,

    /// E.g. `["livekit-voice", "livekit-video", "content-gen"]`
    pub capabilities: Vec<String>,

    pub created_at: String,
}

/// `tools.know-me.agent.output` — record produced by an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOutput {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// DID of the agent that produced this output.
    pub generated_by_did: String,

    pub model: String,

    /// SHA-256 of the prompt for auditability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,

    /// AT URI of the output record (e.g. a `video.post` or text record).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_uri: Option<String>,

    /// `"text"` | `"image"` | `"video"` | `"audio"`
    pub output_type: String,

    pub created_at: String,
}

/// A specific service offered by an agent DID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentService {
    pub name: String,
    pub description: String,

    /// `"voice-call"` | `"content-gen"` | `"live-agent"`
    pub service_type: String,

    pub pricing: AgentPricing,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub livekit_agent_name: Option<String>,

    pub created_at: String,
}

/// A hydrated agent view combining DID, handle, profile, and services.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    pub did: String,
    pub handle: String,
    pub profile: AgentProfile,
    pub services: Vec<AgentService>,
}

/// `tools.know-me.agent.getAgents` — response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAgentsOutput {
    pub agents: Vec<AgentView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// `tools.know-me.agent.requestSession` — input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestSessionInput {
    pub agent_did: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
}

/// `tools.know-me.agent.requestSession` — response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestSessionOutput {
    pub session_uri: String,
    pub room_url: String,
    pub participant_token: String,
}
