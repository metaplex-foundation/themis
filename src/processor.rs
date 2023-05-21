use std::{env, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_governance::{
    instruction::{
        add_signatory, cast_vote, create_proposal, insert_transaction, sign_off_proposal,
    },
    state::{
        governance::GovernanceV2,
        proposal::{get_proposal_address, ProposalV2},
        token_owner_record::{get_token_owner_record_address, TokenOwnerRecordV2},
        vote_record::Vote as SplVote,
    },
    state::{proposal::VoteType, realm::RealmV2},
};

use crate::{config, instruction::create_upgrade_program_instruction, Vote, GOVERNANCE_PROGRAM_ID};

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

    let realm_id_var = env::var("REALM_ID").map_err(|_| anyhow!("Missing REALM_ID env var."))?;
    let governance_id_var =
        env::var("GOVERNANCE_ID").map_err(|_| anyhow!("Missing GOVERNANCE_ID env var."))?;

    println!("Realm ID: {}", realm_id_var);
    println!("Governance ID: {}", governance_id_var);

    let realm_id = Pubkey::from_str(&realm_id_var)?;
    let governance_id = Pubkey::from_str(&governance_id_var)?;

    let realm = get_realm_data(&config.client, &realm_id)?;
    let governance = get_governance_data(&config.client, &governance_id)?;

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
        &realm_id,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

    let create_ix = create_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &governance_id,
        &proposal_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        None,
        &realm_id,
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
        &governance_id,
        &governing_token_mint,
        &proposal_index.to_le_bytes(),
    );

    let token_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &realm_id,
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
        governance_id,
    )?;

    let insert_ix = insert_transaction(
        &GOVERNANCE_PROGRAM_ID,
        &governance_id,
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
        &realm_id,
        &governance_id,
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

pub struct VoteArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub proposal_id: Option<Pubkey>,
    pub vote_choice: Vote,
    pub mint_type: MintType,
    pub latest: bool,
}

pub fn vote(args: VoteArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    println!("Authority: {}", config.keypair.pubkey());

    let realm_id_var = env::var("REALM_ID").map_err(|_| anyhow!("Missing REALM_ID env var."))?;
    let governance_id_var =
        env::var("GOVERNANCE_ID").map_err(|_| anyhow!("Missing GOVERNANCE_ID env var."))?;

    println!("Realm ID: {}", realm_id_var);
    println!("Governance ID: {}", governance_id_var);

    let realm_id = Pubkey::from_str(&realm_id_var)?;
    let governance_id = Pubkey::from_str(&governance_id_var)?;

    let realm: RealmV2 = get_governance_state(&config.client, &realm_id)?;

    let governing_token_mint = match args.mint_type {
        MintType::Member => realm.community_mint,
        MintType::Council => realm
            .config
            .council_mint
            .ok_or_else(|| anyhow!("Council mint not found"))?,
    };

    let proposal_id = if args.latest {
        let governance: GovernanceV2 = get_governance_state(&config.client, &governance_id)?;
        let proposal_index = governance.proposals_count - 1;

        println!("Proposal index: {}", proposal_index);

        get_proposal_address(
            &GOVERNANCE_PROGRAM_ID,
            &governance_id,
            &governing_token_mint,
            &proposal_index.to_le_bytes(),
        )
    } else if let Some(proposal_id) = args.proposal_id {
        proposal_id
    } else {
        return Err(anyhow!("Either --latest or --proposal-id must be provided"));
    };

    println!("Proposal ID: {}", proposal_id);

    // We need to find the owner of the proposal to find the correct proposal_owner_record
    // as this will only be the voter if the voter also created the proposal.

    let proposal: ProposalV2 = get_governance_state(&config.client, &proposal_id)?;

    let token_owner_record: TokenOwnerRecordV2 =
        get_governance_state(&config.client, &proposal.token_owner_record)?;

    println!("Token owner record: {:?}", token_owner_record);

    let proposal_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &realm_id,
        &governing_token_mint,
        &token_owner_record.governing_token_owner,
    );

    println!("Proposal owner record: {}", proposal_owner_record);

    let voter_token_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &realm_id,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

    println!("Voter token owner record: {}", voter_token_owner_record);

    let vote: SplVote = args.vote_choice.into();

    let ix = cast_vote(
        &GOVERNANCE_PROGRAM_ID,
        &realm_id,
        &governance_id,
        &proposal_id,
        &proposal_owner_record,
        &voter_token_owner_record,
        &config.keypair.pubkey(),
        &governing_token_mint,
        &config.keypair.pubkey(),
        None,
        None,
        vote,
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

fn get_governance_state<T>(client: &RpcClient, governance: &Pubkey) -> Result<T>
where
    T: borsh::BorshDeserialize,
{
    let account = client.get_account(governance)?;
    let governance_data = T::deserialize(&mut account.data.as_slice())?;
    Ok(governance_data)
}
