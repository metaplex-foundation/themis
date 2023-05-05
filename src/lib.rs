use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::fmt;

pub mod args;
pub mod config;
pub mod processor;

pub enum Cluster {
    Devnet,
    Mainnet,
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cluster::Devnet => write!(f, "devnet"),
            Cluster::Mainnet => write!(f, "mainnet-beta"),
        }
    }
}

pub const GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!("mrgTA4fqsDqtvizQBoTMGXosiwruTmu2yXZxmPNLKiJ");
pub const REALM_ID: Pubkey = pubkey!("7UyZEvibjTmYDfbhuE3Hi4kuKB4eDfYpZkN7V9XNDeex");
pub const GOVERNANCE_ID: Pubkey = pubkey!("FBpepvDmgVyVhiNit56kxGed7NSwDadvpPkfcTagkMcg");
