mod history;

use dialoguer::{theme::ColorfulTheme, Select, console::Term, Input};
use iota_sdk::wallet::account_manager::AccountManager;

use crate::{println_log_info, prompt::history::PromptHistory};

pub struct SelectAccountPrompt<'a> {
    pub account_manager: &'a AccountManager,
}

impl SelectAccountPrompt<'_> {
    pub async fn run(&self) -> eyre::Result<Option<u32>> {
        let accounts = self.account_manager.get_accounts().await?;

        let mut select_items = Vec::with_capacity(accounts.len() + 1);
        for account_handle in accounts {
            select_items.push(account_handle.read().await.alias().clone());
        }
        select_items.push("__quit__".to_string());

        let index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select account:")
            .items(&select_items)
            .default(0)
            .interact_on(&Term::stderr())?;

        if index == select_items.len() - 1 {
            // Quit
            Ok(None)
        } else {
            // Select this account
            Ok(Some(index as u32))
        }
    }
}

pub struct AccountPrompt<'a> {
    pub account_manager: &'a AccountManager,
    pub account_id: u32,
}

impl AccountPrompt<'_> {
    pub async fn run(&self) -> eyre::Result<()> {
        let alias = self.account_manager.get_account(self.account_id).await?.alias().await;
        println_log_info!("Accessing account: {}", alias);

        let mut history = PromptHistory::default();
        while self.prompt(&alias, &mut history).await? {}
        Ok(())
    }

    async fn prompt(&self, alias: &str, history: &mut PromptHistory) -> eyre::Result<bool> {
        let command: String = Input::new()
            .with_prompt(format!("Account \"{}\"", alias))
            .history_with(history)
            .interact_text()?;

        if ["exit", "close"].contains(&command.as_str()) {
            return Ok(false);
        }

        if ["clear"].contains(&command.as_str()) {
            clear();
            return Ok(true);
        }

        if ["help", "h", "?"].contains(&command.as_str()) {

            return Ok(true);
        }

        Ok(true)
    }
}

fn clear() {
    std::process::Command::new("clear").status().ok();
}
