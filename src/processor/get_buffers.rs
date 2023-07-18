use super::*;

pub struct GetBuffersArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub authority: Pubkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeableBuffer {
    pub address: String,
    pub authority: String,
    pub data_len: usize,
    pub lamports: u64,
}

const ACCOUNT_TYPE_SIZE: usize = 4;
const SLOT_SIZE: usize = size_of::<u64>();
const OPTION_SIZE: usize = 1;
const PUBKEY_LEN: usize = 32;

pub fn get_buffers(args: GetBuffersArgs) -> Result<Vec<UpgradeableBuffer>> {
    let config = config::CliConfig::new(args.keypair_path, args.rpc_url)?;

    let mut filters = vec![RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &[1, 0, 0, 0],
    ))];
    filters.push(RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        ACCOUNT_TYPE_SIZE,
        &[1],
    )));
    filters.push(RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        ACCOUNT_TYPE_SIZE + OPTION_SIZE,
        args.authority.as_ref(),
    )));

    let length = ACCOUNT_TYPE_SIZE + SLOT_SIZE + OPTION_SIZE + PUBKEY_LEN;

    let results = config.client.get_program_accounts_with_config(
        &bpf_loader_upgradeable::id(),
        RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: Some(UiDataSliceConfig { offset: 0, length }),
                ..RpcAccountInfoConfig::default()
            },
            ..RpcProgramAccountsConfig::default()
        },
    )?;

    let mut buffers = vec![];
    for (address, account) in results.iter() {
        if let Ok(UpgradeableLoaderState::Buffer { authority_address }) = account.state() {
            buffers.push(UpgradeableBuffer {
                address: address.to_string(),
                authority: authority_address
                    .map(|pubkey| pubkey.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                data_len: account.data.len(),
                lamports: account.lamports,
            });
        } else {
            return Err(anyhow!("Error parsing Buffer account {}", address));
        }
    }
    Ok(buffers)
}
