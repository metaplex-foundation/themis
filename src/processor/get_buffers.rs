use super::*;

#[derive(Debug, Clone)]
pub struct GetBuffersArgs {
    pub keypair_path: Option<PathBuf>,
    pub rpc_url: Option<String>,
    pub authority: Pubkey,
}

impl From<CloseBuffersArgs> for GetBuffersArgs {
    fn from(args: CloseBuffersArgs) -> Self {
        Self {
            keypair_path: args.keypair_path,
            rpc_url: args.rpc_url,
            authority: args.authority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeableBuffer {
    pub address: Pubkey,
    pub authority: Pubkey,
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
    for (address, account) in results.into_iter() {
        if let Ok(UpgradeableLoaderState::Buffer { authority_address }) = account.state() {
            // Skip if no authority is set as it cannot be closed.
            if authority_address.is_none() {
                continue;
            }

            buffers.push(UpgradeableBuffer {
                address,
                authority: authority_address.unwrap(),
                data_len: account.data.len(),
                lamports: account.lamports,
            });
        } else {
            return Err(anyhow!("Error parsing Buffer account {}", address));
        }
    }
    Ok(buffers)
}
