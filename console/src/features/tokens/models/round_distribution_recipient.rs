use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DynamoEntity, Default)]
pub struct RoundDistributionRecipient {
    pub pk: Partition,
    pub sk: EntityType,

    pub round_id: String,
    pub wallet_address: String,
    pub amount_raw: String,
    pub meta_user_id: Option<String>,
    pub created_at: i64,
}

impl RoundDistributionRecipient {
    pub fn new(
        project_id: Partition,
        round_id: String,
        sequence: usize,
        wallet_address: String,
        amount_raw: String,
        meta_user_id: Option<String>,
    ) -> Self {
        Self {
            pk: project_id,
            sk: EntityType::RoundDistributionRecipient(format!("{round_id}#{sequence:06}")),
            round_id,
            wallet_address,
            amount_raw,
            meta_user_id,
            created_at: crate::common::utils::time_utils::get_now(),
        }
    }
}
