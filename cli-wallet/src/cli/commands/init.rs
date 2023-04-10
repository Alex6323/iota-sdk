use iota_sdk::{
    client::{constants::SHIMMER_COIN_TYPE, secret::SecretManager},
    wallet::{account_manager::AccountManager, ClientOptions},
};

use crate::cli::{commands::GenerateMnemonic, ClArgs, DEFAULT_NODE_URL};

/// Set up a new wallet.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct InitializeWallet {
    /// Set the mnemonic for the new wallet.
    #[arg(short, long)]
    pub mnemonic: Option<String>,
    /// Set the node to connect to with this wallet.
    #[arg(long, value_name = "URL", env = "NODE_URL", default_value = DEFAULT_NODE_URL)]
    pub node_url: String,
    /// Set the coin type held by this wallet.
    #[arg(long, default_value_t = SHIMMER_COIN_TYPE)]
    pub coin_type: u32,
}

impl InitializeWallet {
    pub async fn exec(&self, secret_manager: SecretManager, args: &ClArgs) -> eyre::Result<AccountManager> {
        let account_manager = AccountManager::builder()
            .with_secret_manager(secret_manager)
            .with_client_options(ClientOptions::new().with_node(&self.node_url)?)
            .with_storage_path(&args.wallet_db_path)
            .with_coin_type(self.coin_type)
            .finish()
            .await?;

        let mnemonic = if let Some(mnemonic) = &self.mnemonic {
            mnemonic.clone()
        } else {
            GenerateMnemonic.exec().await?
        };

        if let SecretManager::Stronghold(secret_manager) = &mut *account_manager.get_secret_manager().write().await {
            secret_manager.store_mnemonic(mnemonic).await?;
        } else {
            eyre::bail!("cli-wallet only supports Stronghold-backed secret managers at the moment.");
        }
        Ok(account_manager)
    }
}
