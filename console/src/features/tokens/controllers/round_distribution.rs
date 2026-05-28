use crate::common::{ProjectPartition, Result};
use crate::features::tokens::{RoundDistributionAllocation, RoundDistributionResponse};
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::common::{CommonConfig, EntityType, ProjectAdminAuth, ProjectViewerAuth};
#[cfg(feature = "server")]
use crate::features::tokens::{
    DistributionDedup, ProjectToken, RoundDistribution, RoundDistributionRecipient, TokenError,
};

#[api_doc_macros::api_doc(
    group = "Tokens",
    summary = "Create round distribution",
    summary_ko = "라운드 토큰 일괄 지급"
)]
#[post("/v1/projects/:project_id/tokens/round-distributions", auth: ProjectAdminAuth)]
pub async fn create_round_distribution_handler(
    #[allow(unused_variables)] project_id: ProjectPartition,
    round_id: String,
    idempotency_key: String,
    allocations: Vec<RoundDistributionAllocation>,
) -> Result<RoundDistributionResponse> {
    let config = CommonConfig::default();
    let cli = config.dynamodb();
    let project = auth.project;

    let round_id = normalize_required("round_id", round_id)?;
    let idempotency_key = normalize_required("idempotency_key", idempotency_key)?;

    if let Some(dedup) = DistributionDedup::get(
        cli,
        &project.pk,
        Some(EntityType::DistributionDedup(idempotency_key.clone())),
    )
    .await?
    {
        let (pk, sk) = RoundDistribution::keys(project.pk.clone(), dedup.round_id);
        let existing = RoundDistribution::get(cli, &pk, Some(sk))
            .await?
            .ok_or(TokenError::TokenNotFound)?;
        return Ok(existing.into());
    }

    if let Some(existing) = RoundDistribution::get(
        cli,
        &project.pk,
        Some(EntityType::RoundDistribution(round_id.clone())),
    )
    .await?
    {
        return Ok(existing.into());
    }

    let normalized = validate_allocations(allocations)?;

    let (token_pk, token_sk) = ProjectToken::keys(project.pk.clone());
    let token = ProjectToken::get(cli, &token_pk, Some(token_sk))
        .await?
        .ok_or(TokenError::TokenNotFound)?;
    let contract_address = token
        .contract_address
        .clone()
        .ok_or(TokenError::NotDeployed)?;
    let chain_id = token.chain_id.ok_or(TokenError::NotDeployed)?;

    let tx_hash = submit_round_distribution(chain_id, &contract_address, &round_id, &normalized)
        .await
        .map_err(TokenError::RoundDistributionFailed)?;

    let total_amount = normalized.iter().try_fold(0u128, |acc, item| {
        parse_amount(&item.amount_raw).map(|v| acc + v)
    })?;

    let distribution = RoundDistribution::new(
        project.pk.clone(),
        round_id.clone(),
        idempotency_key.clone(),
        contract_address,
        chain_id,
        total_amount.to_string(),
        normalized.len() as i64,
        Some(tx_hash),
    );
    let dedup = DistributionDedup::new(project.pk.clone(), idempotency_key, round_id.clone());

    distribution.create(cli).await?;
    dedup.create(cli).await?;
    for (idx, allocation) in normalized.into_iter().enumerate() {
        RoundDistributionRecipient::new(
            project.pk.clone(),
            round_id.clone(),
            idx,
            allocation.wallet_address,
            allocation.amount_raw,
            allocation.meta_user_id,
        )
        .create(cli)
        .await?;
    }

    Ok(distribution.into())
}

#[get(
    "/v1/projects/:project_id/tokens/round-distributions/:round_id",
    auth: ProjectViewerAuth
)]
pub async fn get_round_distribution_handler(
    #[allow(unused_variables)] project_id: ProjectPartition,
    round_id: String,
) -> Result<RoundDistributionResponse> {
    let config = CommonConfig::default();
    let cli = config.dynamodb();
    let round_id = normalize_required("round_id", round_id)?;
    let (pk, sk) = RoundDistribution::keys(auth.project.pk, round_id);
    let distribution = RoundDistribution::get(cli, &pk, Some(sk))
        .await?
        .ok_or(TokenError::TokenNotFound)?;

    Ok(distribution.into())
}

#[cfg(feature = "server")]
fn normalize_required(field: &str, value: String) -> Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(TokenError::InvalidRoundDistribution(format!("{field} is required")).into());
    }
    Ok(value)
}

#[cfg(feature = "server")]
fn validate_allocations(
    allocations: Vec<RoundDistributionAllocation>,
) -> Result<Vec<RoundDistributionAllocation>> {
    if allocations.is_empty() {
        return Err(TokenError::InvalidRoundDistribution(
            "allocations must not be empty".to_string(),
        )
        .into());
    }

    allocations
        .into_iter()
        .map(|mut allocation| {
            allocation.wallet_address = allocation.wallet_address.trim().to_string();
            allocation.amount_raw = allocation.amount_raw.trim().to_string();
            if !is_wallet_address(&allocation.wallet_address) {
                return Err(TokenError::InvalidRoundDistribution(format!(
                    "invalid wallet address: {}",
                    allocation.wallet_address
                ))
                .into());
            }
            parse_amount(&allocation.amount_raw)?;
            Ok(allocation)
        })
        .collect()
}

#[cfg(feature = "server")]
fn is_wallet_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value.as_bytes()[2..].iter().all(|b| b.is_ascii_hexdigit())
}

#[cfg(feature = "server")]
fn parse_amount(value: &str) -> Result<u128> {
    let amount = value.parse::<u128>().map_err(|_| {
        TokenError::InvalidRoundDistribution(format!("invalid amount_raw: {value}"))
    })?;
    if amount == 0 {
        return Err(TokenError::InvalidRoundDistribution(
            "amount_raw must be greater than zero".to_string(),
        )
        .into());
    }
    Ok(amount)
}

#[cfg(all(feature = "server", feature = "disable-chain"))]
async fn submit_round_distribution(
    _chain_id: u64,
    _contract_address: &str,
    round_id: &str,
    allocations: &[RoundDistributionAllocation],
) -> std::result::Result<String, String> {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!("{round_id}:{}", allocations.len()).as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("0x{:064x}", hash))
}

#[cfg(all(feature = "server", not(feature = "disable-chain")))]
async fn submit_round_distribution(
    chain_id: u64,
    contract_address: &str,
    _round_id: &str,
    allocations: &[RoundDistributionAllocation],
) -> std::result::Result<String, String> {
    let mut last_tx = None;
    for allocation in allocations {
        let amount = ethers::types::U256::from_dec_str(&allocation.amount_raw)
            .map_err(|e| format!("invalid amount_raw: {e}"))?;
        let tx = crate::common::blockchain::transfer_brand_token(
            chain_id,
            contract_address,
            &allocation.wallet_address,
            amount,
        )
        .await?;
        last_tx = Some(format!("{tx:#x}"));
    }
    last_tx.ok_or_else(|| "no allocations submitted".to_string())
}
