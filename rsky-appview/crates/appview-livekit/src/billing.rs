use appview_core::error::{AppViewError, Result};
use appview_db::PgPool;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Subscription tier for a DID
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// Free tier — viewer only, no hosting
    Free,
    /// Creator tier — can host sessions up to N concurrent viewers
    Creator,
    /// Pro tier — unlimited viewers, AI agents, gated content
    Pro,
}

impl SubscriptionTier {
    pub fn can_host(&self) -> bool {
        matches!(self, SubscriptionTier::Creator | SubscriptionTier::Pro)
    }

    pub fn can_use_agents(&self) -> bool {
        matches!(self, SubscriptionTier::Pro)
    }

    pub fn can_gate_content(&self) -> bool {
        matches!(self, SubscriptionTier::Pro)
    }

    pub fn max_viewers(&self) -> u32 {
        match self {
            SubscriptionTier::Free => 0,
            SubscriptionTier::Creator => 100,
            SubscriptionTier::Pro => 0, // unlimited
        }
    }
}

/// Result of a billing gate check
#[derive(Debug, Clone)]
pub struct GateResult {
    pub allowed: bool,
    pub tier: SubscriptionTier,
    pub reason: Option<String>,
}

impl GateResult {
    pub fn allow(tier: SubscriptionTier) -> Self {
        Self {
            allowed: true,
            tier,
            reason: None,
        }
    }

    pub fn deny(tier: SubscriptionTier, reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            tier,
            reason: Some(reason.into()),
        }
    }
}

/// Billing gate — checks subscription before minting tokens or creating rooms
pub struct BillingGate {
    pool: PgPool,
}

impl BillingGate {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if a DID can host a live session
    pub async fn check_can_host(&self, did: &str) -> Result<GateResult> {
        let tier = self.get_tier(did).await?;

        if tier.can_host() {
            debug!("{} is allowed to host (tier: {:?})", did, tier);
            Ok(GateResult::allow(tier))
        } else {
            Ok(GateResult::deny(
                tier,
                "hosting requires Creator or Pro subscription",
            ))
        }
    }

    /// Check if a DID can join a room (all tiers can join as viewers)
    pub async fn check_can_join(
        &self,
        _did: &str,
        gated: bool,
        session_uri: &str,
    ) -> Result<GateResult> {
        // Free tier can always join non-gated sessions
        if !gated {
            return Ok(GateResult::allow(SubscriptionTier::Free));
        }

        // For gated sessions, check if they have purchased access
        let has_access = self.check_purchase(session_uri).await?;
        if has_access {
            Ok(GateResult::allow(SubscriptionTier::Free))
        } else {
            Ok(GateResult::deny(
                SubscriptionTier::Free,
                "purchase required to join this session",
            ))
        }
    }

    /// Check if a DID can use AI agents
    pub async fn check_can_use_agents(&self, did: &str) -> Result<GateResult> {
        let tier = self.get_tier(did).await?;

        if tier.can_use_agents() {
            Ok(GateResult::allow(tier))
        } else {
            Ok(GateResult::deny(tier, "AI agents require Pro subscription"))
        }
    }

    /// Get the subscription tier for a DID from the DB
    async fn get_tier(&self, did: &str) -> Result<SubscriptionTier> {
        #[derive(sqlx::FromRow)]
        struct TierRow {
            tier: String,
        }

        let row = sqlx::query_as::<_, TierRow>(
            "SELECT tier FROM subscription WHERE did = $1 AND expires_at > NOW()",
        )
        .bind(did)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppViewError::Storage(format!("failed to query subscription tier: {e}")))?;

        Ok(match row.as_ref().map(|r| r.tier.as_str()) {
            Some("pro") => SubscriptionTier::Pro,
            Some("creator") => SubscriptionTier::Creator,
            _ => SubscriptionTier::Free,
        })
    }

    /// Check if a viewer has purchased access to a gated session
    async fn check_purchase(&self, session_uri: &str) -> Result<bool> {
        #[derive(sqlx::FromRow)]
        struct PurchaseRow {
            #[allow(dead_code)]
            id: i64,
        }

        let row = sqlx::query_as::<_, PurchaseRow>(
            "SELECT id FROM session_purchase WHERE session_uri = $1 LIMIT 1",
        )
        .bind(session_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppViewError::Storage(format!("failed to query session purchase: {e}")))?;

        Ok(row.is_some())
    }
}
