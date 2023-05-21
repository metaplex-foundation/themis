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
        /// Proposal pubkey
        proposal_id: Pubkey,

        /// Vote: true = yes, false = no
        vote_choice: Vote,

        #[arg(short, long, default_value = "council")]
        mint_type: MintType,
    },
}
