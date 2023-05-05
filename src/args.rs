use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::processor::MintType;

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
        name: String,

        /// Proposal description
        #[arg(short, long)]
        description: String,

        /// Mint type: Member or Council
        #[arg(short, long)]
        mint_type: MintType,
    },
}
