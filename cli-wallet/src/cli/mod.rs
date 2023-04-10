mod commands;

const DEFAULT_NODE_URL: &str = "https://api.testnet.shimmer.network";
const DEFAULT_WALLET_DATABASE_PATH: &str = "./stardust-cli-wallet-db";
const DEFAULT_STRONGHOLD_PATH: &str = "./stardust-cli-wallet.stronghold";

use dialoguer::Password;
use iota_sdk::{
    client::secret::{stronghold::StrongholdSecretManager, SecretManager},
    wallet::account_manager::AccountManager,
};

use self::commands::WalletCommand;
use crate::{cli::commands::InitializeWallet, println_log_info};

/// TODO
#[derive(clap::Parser, Debug)]
// #[command(author, version, about, next_display_order = None)]
#[command(author, version, about)]
pub struct ClArgs {
    /// Set the path to the wallet database.
    #[arg(long, value_name = "PATH", env = "WALLET_DATABASE_PATH", default_value = DEFAULT_WALLET_DATABASE_PATH)]
    pub wallet_db_path: String,
    /// Set the path to stronghold file.
    #[arg(long, value_name = "PATH", env = "STRONGHOLD_PATH", default_value = DEFAULT_STRONGHOLD_PATH)]
    pub stronghold_path: String,
    pub account: Option<String>,
    /// Subcommands.
    #[command(subcommand)]
    pub commands: Option<WalletCommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum WalletCommands {
    Init(commands::InitializeWallet),
    Mnemonic(commands::GenerateMnemonic),
    Restore(commands::RestoreWallet),
    Account(ManageAccountsArgs),
}

/// Manage the accounts of the wallet.
#[derive(Debug, clap::Parser)]
pub struct ManageAccountsArgs {
    /// Subcommands.
    #[command(subcommand)]
    pub commands: Option<AccountManagementCommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum AccountManagementCommands {
    Backup(commands::CreateBackup),
    ChangeNode(commands::ChangeNode),
    ChangePassword(commands::ChangePassword),
    Create(commands::CreateAccount),
    Sync(commands::SynchronizeAccounts),
}

pub enum PostCommand {
    Start(AccountManager),
    Exit,
}

impl ClArgs {
    pub async fn process_command(&self) -> eyre::Result<PostCommand> {
        let storage_path = self.wallet_db_path.clone();
        let storage_exists = std::path::Path::new(&storage_path).exists();
        let stronghold_path = self.stronghold_path.clone();
        let stronghold_exists = std::path::Path::new(&stronghold_path).exists();
        let password = get_password("Stronghold password", !stronghold_exists)?;
        let secret_manager = SecretManager::Stronghold(
            StrongholdSecretManager::builder()
                .password(&password)
                .build(stronghold_path)?,
        );

        if let Some(cmd) = &self.commands {
            match cmd {
                WalletCommands::Init(cmd) => {
                    let account_manager = if storage_exists {
                        println_log_info!("Wallet already initialized.");
                        AccountManager::builder()
                            .with_secret_manager(secret_manager)
                            .with_storage_path(&storage_path)
                            .finish()
                            .await?
                    } else {
                        cmd.exec(secret_manager, &self).await?
                    };
                    Ok(PostCommand::Start(account_manager))
                }
                WalletCommands::Restore(cmd) => {
                    let account_manager = cmd.exec(secret_manager, &storage_path, password).await?;
                    Ok(PostCommand::Start(account_manager))
                }
                WalletCommands::Mnemonic(cmd) => {
                    cmd.exec().await?;
                    Ok(PostCommand::Exit)
                }
                WalletCommands::Account(args) => {
                    if !storage_exists {
                        println_log_info!("Please initialize or restore a wallet first.");
                        Ok(PostCommand::Exit)
                    } else {
                        let account_manager = AccountManager::builder()
                            .with_secret_manager(secret_manager)
                            .with_storage_path(&storage_path)
                            .finish()
                            .await?;
                        if let Some(cmd) = &args.commands {
                            match cmd {
                                AccountManagementCommands::Backup(cmd) => {
                                    cmd.exec(&account_manager, &password).await?;
                                }
                                AccountManagementCommands::ChangeNode(cmd) => {
                                    cmd.exec(&account_manager, &()).await?;
                                }
                                AccountManagementCommands::ChangePassword(cmd) => {
                                    cmd.exec(&account_manager, &password).await?;
                                }
                                AccountManagementCommands::Create(cmd) => {
                                    cmd.exec(&account_manager, &()).await?;
                                }
                                AccountManagementCommands::Sync(cmd) => {
                                    cmd.exec(&account_manager, &()).await?;
                                }
                            }
                        }
                        Ok(PostCommand::Start(account_manager))
                    }
                }
            }
        } else if stronghold_exists {
            let account_manager = AccountManager::builder()
                .with_secret_manager(secret_manager)
                .with_storage_path(&storage_path)
                .finish()
                .await?;
            Ok(PostCommand::Start(account_manager))
            
        } else {
            use clap::Parser;
            println_log_info!("Initializing wallet with default values.");
            let account_manager = InitializeWallet::parse().exec(secret_manager, &self).await?;
            Ok(PostCommand::Start(account_manager))
        }
    }
}

pub fn get_password(prompt: &str, confirmation: bool) -> eyre::Result<String> {
    let mut password = Password::new();
    password.with_prompt(prompt);
    if confirmation {
        password.with_confirmation("Confirm password", "Password mismatch");
    }
    Ok(password.interact()?)
}
