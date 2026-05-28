use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DynamoEntity, Default)]
pub struct RoundDistribution {
    pub pk: Partition,
    pub sk: EntityType,

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

impl RoundDistribution {
    pub fn new(
        project_id: Partition,
        round_id: String,
        idempotency_key: String,
        contract_address: String,
        chain_id: u64,
        total_amount_raw: String,
        recipient_count: i64,
        tx_hash: Option<String>,
    ) -> Self {
        let now = crate::common::utils::time_utils::get_now();
        Self {
            pk: project_id,
            sk: EntityType::RoundDistribution(round_id.clone()),
            round_id,
            idempotency_key,
            status: "submitted".to_string(),
            tx_hash,
            contract_address,
            chain_id,
            total_amount_raw,
            recipient_count,
            submitted_at: now,
            updated_at: now,
        }
    }

    pub fn keys(project_id: Partition, round_id: String) -> (Partition, EntityType) {
        (project_id, EntityType::RoundDistribution(round_id))
    }
}
