use std::path::PathBuf;

use clap::{Parser, Subcommand};
use solana_program::pubkey::Pubkey;

use crate::{processor::MintType, Vote};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Path to the keypair file.
    #[arg(short, long, global = true)]
    pub keypair_path: Option<PathBuf>,

    /// RPC URL for the Solana cluster.
    #[arg(short, long, global = true)]
    pub rpc_url: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    Propose {
        /// Proposal name
        #[arg(short, long)]
        source_buffer: Pubkey,

        /// Proposal description
        #[arg(short, long)]
        spill_account: Option<Pubkey>,

        /// Proposal name
        #[arg(short, long)]
        name: String,

        /// Proposal description
        #[arg(short, long)]
        description: String,

        /// Mint type: Member or Council
        #[arg(short, long, default_value = "council")]
        mint_type: MintType,

        #[arg(short, long)]
        options: Vec<String>,
    },
    Vote {
        /// Vote: true = yes, false = no
        vote_choice: Vote,

        /// Proposal pubkey
        #[arg(short, long)]
        proposal_id: Option<Pubkey>,

        /// Vote on the most recent proposal
        #[arg(short, long)]
        latest: bool,

        #[arg(short, long, default_value = "council")]
        mint_type: MintType,
    },
    Execute {
        /// Proposal pubkey
        #[arg(short, long)]
        proposal_id: Option<Pubkey>,

        /// Vote on the most recent proposal
        #[arg(short, long)]
        latest: bool,

        #[arg(short, long, default_value = "council")]
        mint_type: MintType,
    },
    Deposit {
        /// Amount of governance tokens to deposit
        amount: u64,

        #[arg(short, long, default_value = "council")]
        mint_type: MintType,
    },
    Withdraw {
        #[arg(short, long, default_value = "council")]
        mint_type: MintType,
    },
    Update {
        /// Mint type: Member or Council
        #[arg(short, long, default_value = "council")]
        mint_type: MintType,

        #[arg(long)]
        vote_threshold_percentage: Option<u8>,

        #[arg(long)]
        min_council_weight_to_create_proposal: Option<u64>,

        #[arg(long)]
        min_transaction_hold_up_time: Option<u32>,

        /// Max voting time in seconds
        #[arg(long)]
        max_voting_time: Option<u32>,

        #[arg(long)]
        proposal_cool_off_time: Option<u32>,

        #[arg(long)]
        min_comunity_weight_to_create_proposal: Option<u64>,
    },
}
