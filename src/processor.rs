use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_governance::{
    instruction::{add_signatory, create_proposal, insert_transaction, sign_off_proposal},
    state::{
        governance::GovernanceV2, proposal::get_proposal_address,
        token_owner_record::get_token_owner_record_address,
    },
    state::{proposal::VoteType, realm::RealmV2},
};

use crate::{
    config, instruction::create_upgrade_program_instruction, GOVERNANCE_ID, GOVERNANCE_PROGRAM_ID,
    REALM_ID,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MintType {
    Member,
    Council,
}

impl FromStr for MintType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "member" => Ok(MintType::Member),
            "council" => Ok(MintType::Council),
            _ => Err(anyhow!("Invalid mint type")),
        }
    }
}

pub struct ProposeArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub source_buffer: Pubkey,
    pub spill_account: Option<Pubkey>,
    pub name: String,
    pub description: String,
    pub mint_type: MintType,
    pub vote_type: VoteType,
    pub options: Vec<String>,
}

pub fn propose(args: ProposeArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    let realm = get_realm_data(&config.client, &REALM_ID)?;
    let governance = get_governance_data(&config.client, &GOVERNANCE_ID)?;

    let governing_token_mint = match args.mint_type {
        MintType::Member => realm.community_mint,
        MintType::Council => realm
            .config
            .council_mint
            .ok_or_else(|| anyhow!("Council mint not found"))?,
    };

    println!("Options: {:?}", args.options);

    let proposal_index: u32 = governance.proposals_count;

    println!("Proposal index: {}", proposal_index);

    let proposal_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &REALM_ID,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

    let create_ix = create_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &GOVERNANCE_ID,
        &proposal_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        None,
        &REALM_ID,
        args.name,
        args.description,
        &governing_token_mint,
        args.vote_type,
        args.options,
        true,
        proposal_index,
    );

    let proposal_address = get_proposal_address(
        &GOVERNANCE_PROGRAM_ID,
        &GOVERNANCE_ID,
        &governing_token_mint,
        &proposal_index.to_le_bytes(),
    );

    let token_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &REALM_ID,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

    println!("proposal address: {}", proposal_address);

    let add_signatory_ix = add_signatory(
        &GOVERNANCE_PROGRAM_ID,
        &proposal_address,
        &token_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
    );

    // Empirically determined from existing proposals. Not sure the significance of these yet.
    let option_index = 0;
    let index = 0;
    let hold_up_time = 0;

    let program_upgrade_instruction = create_upgrade_program_instruction(
        args.source_buffer,
        args.spill_account.unwrap_or(config.keypair.pubkey()),
        config.keypair.pubkey(),
    );

    let insert_ix = insert_transaction(
        &GOVERNANCE_PROGRAM_ID,
        &GOVERNANCE_ID,
        &proposal_address,
        &token_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        option_index,
        index,
        hold_up_time,
        vec![program_upgrade_instruction],
    );

    let sign_off_ix = sign_off_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &REALM_ID,
        &GOVERNANCE_ID,
        &proposal_address,
        &config.keypair.pubkey(),
        None,
    );

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_ix, add_signatory_ix, insert_ix, sign_off_ix],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        config.client.get_latest_blockhash()?,
    );

    config.client.send_and_confirm_transaction(&tx)?;

    Ok(())
}

fn get_realm_data(client: &RpcClient, realm: &Pubkey) -> Result<RealmV2> {
    let account = client.get_account(realm)?;
    let realm_data = RealmV2::deserialize(&mut account.data.as_slice())?;
    Ok(realm_data)
}

fn get_governance_data(client: &RpcClient, governance: &Pubkey) -> Result<GovernanceV2> {
    let account = client.get_account(governance)?;
    let governance_data = GovernanceV2::deserialize(&mut account.data.as_slice())?;
    Ok(governance_data)
}
