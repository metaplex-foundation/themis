use super::*;

pub struct ExecuteArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub proposal_id: Option<Pubkey>,
    pub latest: bool,
    pub mint_type: MintType,
}

pub fn execute(args: ExecuteArgs) -> Result<()> {
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

    println!(
        "governing token owner: {:?}",
        token_owner_record.governing_token_owner
    );

    // All our upgrade proposals should have option and instruction indexes of 0,
    // but to support more generically we should get the indexes.

    let option_index = (proposal
        .options
        .first()
        .expect("More than one proposal option found!")
        .transactions_next_index
        - 1) as u8;

    println!("Option index: {:?}", option_index);

    let proposal_transaction_pubkey = get_proposal_transaction_address(
        &GOVERNANCE_PROGRAM_ID,
        &proposal_id,
        &option_index.to_le_bytes(),
        &[0, 0],
    );

    println!("Proposal transaction: {}", proposal_transaction_pubkey);

    let proposal_transaction: ProposalTransactionV2 =
        get_governance_state(&config.client, &proposal_transaction_pubkey)?;

    if proposal_transaction.instructions.len() > 1 {
        return Err(anyhow!(
            "More than one instruction found in proposal transaction"
        ));
    }
    let instruction = proposal_transaction
        .instructions
        .first()
        .expect("No instructions found in proposal transaction");

    let instruction_program_id = instruction.program_id;

    // Convert from the SPL governance type to the Solana SDK type
    // Manually set the signer to false for the governance keypair since that gets
    // signed via CPI by the governance program.
    let instruction_accounts: Vec<AccountMeta> = instruction
        .accounts
        .clone()
        .into_iter()
        .map(|a| AccountMeta {
            pubkey: a.pubkey,
            is_signer: if a.pubkey == governance_id {
                false
            } else {
                a.is_signer
            },
            is_writable: a.is_writable,
        })
        .collect();

    println!("Instruction program ID: {}", instruction_program_id);

    let ix = execute_transaction(
        &GOVERNANCE_PROGRAM_ID,
        &governance_id,
        &proposal_id,
        &proposal_transaction_pubkey,
        &instruction_program_id,
        &instruction_accounts,
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
