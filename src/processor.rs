use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_governance::{
    instruction::{create_proposal, sign_off_proposal},
    state::{proposal::get_proposal_address, token_owner_record::get_token_owner_record_address},
    state::{proposal::VoteType, realm::RealmV2},
};

use crate::{config, GOVERNANCE_ID, GOVERNANCE_PROGRAM_ID, REALM_ID};

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
    pub name: String,
    pub description: String,
    pub mint_type: MintType,
    pub vote_type: VoteType,
    pub options: Vec<String>,
}

pub fn propose(args: ProposeArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    let realm = get_realm_data(&config.client, &REALM_ID)?;

    let governing_token_mint = match args.mint_type {
        MintType::Member => realm.community_mint,
        MintType::Council => realm
            .config
            .council_mint
            .ok_or_else(|| anyhow!("Council mint not found"))?,
    };

    let proposal_index: u32 = realm.voting_proposal_count.into();

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

    println!("proposal address: {}", proposal_address);

    let sign_off_ix = sign_off_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &REALM_ID,
        &GOVERNANCE_ID,
        &proposal_address,
        &config.keypair.pubkey(),
        Some(&proposal_owner_record),
    );

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_ix, sign_off_ix],
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
