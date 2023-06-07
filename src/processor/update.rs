use spl_governance::state::enums::VoteThresholdPercentage;

use super::*;

pub struct UpdateArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub vote_threshold_percentage: Option<u8>,
    pub min_council_weight_to_create_proposal: Option<u64>,
    pub min_transaction_hold_up_time: Option<u32>,
    pub max_voting_time: Option<u32>,
    pub proposal_cool_off_time: Option<u32>,
    pub min_comunity_weight_to_create_proposal: Option<u64>,
}

pub fn update(args: UpdateArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    let governance = get_governance_data(&config.client, &config.governance_id)?;
    let mut governance_config = governance.config;

    if let Some(vote_threshold_percentage) = args.vote_threshold_percentage {
        governance_config.vote_threshold_percentage =
            VoteThresholdPercentage::YesVote(vote_threshold_percentage);
    }
    if let Some(min_council_weight_to_create_proposal) = args.min_council_weight_to_create_proposal
    {
        governance_config.min_council_weight_to_create_proposal =
            min_council_weight_to_create_proposal;
    }
    if let Some(min_transaction_hold_up_time) = args.min_transaction_hold_up_time {
        governance_config.min_transaction_hold_up_time = min_transaction_hold_up_time;
    }
    if let Some(max_voting_time) = args.max_voting_time {
        governance_config.max_voting_time = max_voting_time;
    }
    if let Some(proposal_cool_off_time) = args.proposal_cool_off_time {
        governance_config.proposal_cool_off_time = proposal_cool_off_time;
    }
    if let Some(min_comunity_weight_to_create_proposal) =
        args.min_comunity_weight_to_create_proposal
    {
        governance_config.min_community_weight_to_create_proposal =
            min_comunity_weight_to_create_proposal;
    }

    let ix = set_governance_config(
        &GOVERNANCE_PROGRAM_ID,
        &config.governance_id,
        governance_config,
    );

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        config.client.get_latest_blockhash()?,
    );

    config.client.send_and_confirm_transaction(&tx)?;

    Ok(())
}
