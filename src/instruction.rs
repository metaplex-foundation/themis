use std::env;
use std::str::FromStr;

use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::{clock::ID as sysvar_clock, rent::ID as rent_sysvar};
use spl_governance::state::proposal_transaction::{AccountMetaData, InstructionData};

use crate::BPF_UPLOADER_ID;

pub fn create_upgrade_program_instruction(
    source_buffer: Pubkey,
    spill_account: Pubkey,
    upgrade_authority: Pubkey,
) -> Result<InstructionData> {
    let program_data = Pubkey::from_str(&env::var("PROGRAM_DATA")?)?;
    let program_id = Pubkey::from_str(&env::var("PROGRAM_ID")?)?;

    Ok(InstructionData {
        program_id: BPF_UPLOADER_ID,
        accounts: vec![
            AccountMetaData {
                pubkey: program_data,
                is_signer: false,
                is_writable: true,
            },
            AccountMetaData {
                pubkey: program_id,
                is_signer: false,
                is_writable: true,
            },
            AccountMetaData {
                pubkey: source_buffer,
                is_signer: false,
                is_writable: true,
            },
            AccountMetaData {
                pubkey: spill_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMetaData {
                pubkey: rent_sysvar,
                is_signer: false,
                is_writable: false,
            },
            AccountMetaData {
                pubkey: sysvar_clock,
                is_signer: false,
                is_writable: false,
            },
            AccountMetaData {
                pubkey: upgrade_authority,
                is_signer: true,
                is_writable: true,
            },
        ],
        data: vec![3, 0, 0, 0],
    })
}
