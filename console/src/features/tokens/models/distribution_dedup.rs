use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DynamoEntity, Default)]
pub struct DistributionDedup {
    pub pk: Partition,
    pub sk: EntityType,

    pub idempotency_key: String,
    pub round_id: String,
    pub created_at: i64,
}

impl DistributionDedup {
    pub fn new(project_id: Partition, idempotency_key: String, round_id: String) -> Self {
        Self {
            pk: project_id,
            sk: EntityType::DistributionDedup(idempotency_key.clone()),
            idempotency_key,
            round_id,
            created_at: crate::common::utils::time_utils::get_now(),
        }
    }

    pub fn keys(project_id: Partition, idempotency_key: String) -> (Partition, EntityType) {
        (project_id, EntityType::DistributionDedup(idempotency_key))
    }
}
