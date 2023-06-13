use super::*;

pub struct GetGovConfigArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
}

pub fn get_gov_config(args: GetGovConfigArgs) -> Result<()> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    let governance = get_governance_data(&config.client, &config.governance_id)?;
    let governance_config = governance.config;

    println!("Current Governance Config: {:#?}", governance_config);

    Ok(())
}
