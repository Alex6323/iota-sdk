mod account;
mod init;
mod mnemonic;
mod restore;
mod wallet;

pub use account::*;
pub use init::*;
use iota_sdk::wallet::{account::AccountHandle, account_manager::AccountManager};
pub use mnemonic::GenerateMnemonic;
pub use restore::*;
pub use wallet::*;

#[async_trait::async_trait]
pub trait WalletCommand {
    type Context: Sync + Send;
    type Output;

    async fn exec(&self, account_manager: &AccountManager, ctx: &Self::Context) -> eyre::Result<Self::Output>;
}

#[async_trait::async_trait]
pub trait AccountCommand {
    async fn exec(&self, account: &AccountHandle) -> eyre::Result<()>;
}
