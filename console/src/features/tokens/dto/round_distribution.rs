use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, api_doc_macros::ApiDocSchema)]
#[cfg_attr(feature = "server", derive(schemars::JsonSchema))]
pub struct RoundDistributionAllocation {
    pub wallet_address: String,
    pub amount_raw: String,
    #[serde(default)]
    pub meta_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, api_doc_macros::ApiDocSchema)]
#[cfg_attr(feature = "server", derive(schemars::JsonSchema))]
pub struct RoundDistributionResponse {
    pub round_id: String,
    pub idempotency_key: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub contract_address: String,
    pub chain_id: u64,
    pub total_amount_raw: String,
    pub recipient_count: i64,
    pub submitted_at: i64,
    pub updated_at: i64,
}

impl From<crate::features::tokens::RoundDistribution> for RoundDistributionResponse {
    fn from(value: crate::features::tokens::RoundDistribution) -> Self {
        Self {
            round_id: value.round_id,
            idempotency_key: value.idempotency_key,
            status: value.status,
            tx_hash: value.tx_hash,
            contract_address: value.contract_address,
            chain_id: value.chain_id,
            total_amount_raw: value.total_amount_raw,
            recipient_count: value.recipient_count,
            submitted_at: value.submitted_at,
            updated_at: value.updated_at,
        }
    }
}
