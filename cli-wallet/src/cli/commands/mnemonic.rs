use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use iota_sdk::client::generate_mnemonic;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::println_log_info;

const MNEMONIC_FILE_NAME: &str = "mnemonic.txt";

/// Generate a random mnemonic.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct GenerateMnemonic;

impl GenerateMnemonic {
    pub async fn exec(&self) -> eyre::Result<String> {
        let mnemonic = generate_mnemonic()?;
        println_log_info!("Mnemonic has been generated.");

        let items = vec!["Writh to Console", "Write to File", "Write to Both", "Keep Hidden"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match index {
                0 => println!("{}", mnemonic),
                1 => write_to_file(MNEMONIC_FILE_NAME, &mnemonic).await?,
                2 => {
                    println!("{}", mnemonic);
                    write_to_file(MNEMONIC_FILE_NAME, &mnemonic).await?;
                }
                _ => {}
            },
            None => {}
        }
        Ok(mnemonic)
    }
}

async fn write_to_file(path: &str, mnemonic: &str) -> eyre::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path).await?;
    file.write_all(format!("mnemonic_command: {mnemonic}\n").as_bytes())
        .await?;

    println_log_info!(
        "IMPORTANT: mnemonic has been written to '{}', handle it safely.",
        MNEMONIC_FILE_NAME
    );
    println_log_info!(
        "It is the only way to recover your account if you ever forget your password and/or lose the stronghold file."
    );

    Ok(())
}
