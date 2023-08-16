use solana_program::instruction::Instruction;
use spl_governance::state::proposal_transaction::{AccountMetaData, InstructionData};

use super::*;

#[derive(Debug, Clone)]
pub struct CloseBuffersArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub authority: Pubkey,
    pub recipient: Pubkey,
    pub spill_account: Option<Pubkey>,
    pub name: String,
    pub description: String,
    pub mint_type: MintType,
    pub vote_type: VoteType,
    pub options: Vec<String>,
}

const BATCH_SIZE: usize = 10;

pub fn close_buffers(args: CloseBuffersArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path.clone(), args.rpc_url.clone())?;

    let buffers = get_buffers(args.clone().into())?;

    let realm = get_realm_data(&config.client, &config.realm_id)?;
    let governance = get_governance_data(&config.client, &config.governance_id)?;

    let governing_token_mint = match args.mint_type {
        MintType::Member => realm.community_mint,
        MintType::Council => realm
            .config
            .council_mint
            .ok_or_else(|| anyhow!("Council mint not found"))?,
    };

    let proposal_index: u32 = governance.proposals_count;

    let proposal_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &config.realm_id,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

    let mut instructions = vec![];

    for buffer in buffers.iter().take(BATCH_SIZE) {
        instructions.push(bpf_loader_upgradeable::close_any(
            &buffer.address,
            &args.recipient,
            Some(&args.authority),
            None,
        ));
    }

    let instructions = instructions
        .into_iter()
        .map(into_instruction_data)
        .collect();

    let create_ix = create_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &config.governance_id,
        &proposal_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        None,
        &config.realm_id,
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
        &config.governance_id,
        &governing_token_mint,
        &proposal_index.to_le_bytes(),
    );

    let token_owner_record = get_token_owner_record_address(
        &GOVERNANCE_PROGRAM_ID,
        &config.realm_id,
        &governing_token_mint,
        &config.keypair.pubkey(),
    );

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

    let insert_ix = insert_transaction(
        &GOVERNANCE_PROGRAM_ID,
        &config.governance_id,
        &proposal_address,
        &token_owner_record,
        &config.keypair.pubkey(),
        &config.keypair.pubkey(),
        option_index,
        index,
        hold_up_time,
        instructions,
    );

    let sign_off_ix = sign_off_proposal(
        &GOVERNANCE_PROGRAM_ID,
        &config.realm_id,
        &config.governance_id,
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

    config
        .client
        .send_and_confirm_transaction_with_spinner(&tx)?;

    Ok(())
}

fn into_instruction_data(instruction: Instruction) -> InstructionData {
    let accounts = instruction
        .accounts
        .into_iter()
        .map(into_account_meta_data)
        .collect();

    InstructionData {
        program_id: instruction.program_id,
        accounts,
        data: instruction.data,
    }
}

fn into_account_meta_data(account: AccountMeta) -> AccountMetaData {
    AccountMetaData {
        pubkey: account.pubkey,
        is_signer: account.is_signer,
        is_writable: account.is_writable,
    }
}
