use iota_sdk::{
    client::secret::SecretManager,
    wallet::account_manager::AccountManager,
};

/// Restore a wallet.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct RestoreWallet {
    /// TODO
    #[arg(short, long)]
    backup_path: String,
}

impl RestoreWallet {
    pub async fn exec(&self, secret_manager: SecretManager, storage_path: &str, password: String) -> eyre::Result<AccountManager> {
        // Note: node url and coin type will be set while restoring
        let account_manager = AccountManager::builder()
            .with_secret_manager(secret_manager)
            .with_storage_path(storage_path)
            .finish()
            .await?;

        account_manager
            .restore_backup(self.backup_path.clone().into(), password, None)
            .await?;

        Ok(account_manager)
    }
}
