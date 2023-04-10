use iota_sdk::wallet::account_manager::AccountManager;

use crate::{
    cli::{commands::WalletCommand, get_password},
    println_log_info,
};

/// Create a stronghold backup file.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct CreateBackup {
    path: String,
}

#[async_trait::async_trait]
impl WalletCommand for CreateBackup {
    type Context = String;
    type Output = ();

    async fn exec(&self, account_manager: &AccountManager, password: &Self::Context) -> eyre::Result<Self::Output> {
        account_manager
            .backup(self.path.clone().into(), password.clone())
            .await?;

        println_log_info!("Wallet has been backed up to \"{}\".", self.path);

        Ok(())
    }
}

/// Create a new account with an optional alias.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct CreateAccount {
    alias: Option<String>,
}

#[async_trait::async_trait]
impl WalletCommand for CreateAccount {
    type Context = ();
    type Output = ();

    async fn exec(&self, account_manager: &AccountManager, _ctx: &Self::Context) -> eyre::Result<Self::Output> {
        let mut builder = account_manager.create_account();

        if let Some(alias) = &self.alias {
            builder = builder.with_alias(alias.clone());
        }
        let account_handle = builder.finish().await?;
        let alias = account_handle.read().await.alias().to_string();

        println_log_info!("Created account \"{alias}\"");

        Ok(())
    }
}

/// Change the stronghold password.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct ChangePassword {}

#[async_trait::async_trait]
impl WalletCommand for ChangePassword {
    type Context = String;
    type Output = ();

    async fn exec(
        &self,
        account_manager: &AccountManager,
        current_password: &Self::Context,
    ) -> eyre::Result<Self::Output> {
        let new_password = &get_password("Stronghold new password", true)?;
        account_manager
            .change_stronghold_password(current_password, new_password)
            .await?;
        Ok(())
    }
}

/// Change the node to connect to.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct ChangeNode {
    node_url: String,
}

#[async_trait::async_trait]
impl WalletCommand for ChangeNode {
    type Context = ();
    type Output = ();

    async fn exec(
        &self,
        account_manager: &AccountManager,
        _ctx: &Self::Context,
    ) -> eyre::Result<Self::Output> {
        account_manager
            .set_client_options(iota_sdk::wallet::ClientOptions::new().with_node(&self.node_url)?)
            .await?;

        Ok(())
    }
}
/// Synchronize all accounts.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SynchronizeAccounts;

#[async_trait::async_trait]
impl WalletCommand for SynchronizeAccounts {
    type Context = ();
    type Output = ();

    async fn exec(&self, account_manager: &AccountManager, _ctx: &Self::Context) -> eyre::Result<Self::Output> {
        let total_balance = account_manager.sync(None).await?;

        println_log_info!("Synchronized all accounts: {:?}", total_balance);
        Ok(())
    }
}
