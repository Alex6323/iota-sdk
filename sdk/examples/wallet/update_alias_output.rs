// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will update the state metadata of an alias output.
//!
//! Make sure that `example.stronghold` and `example.walletdb` already exist by
//! running the `create_wallet` example!
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --all-features --example update_alias_output
//! ```

use std::{env::var, str::FromStr, time::Instant};

use iota_sdk::{
    types::block::output::{AliasId, AliasOutput, AliasOutputBuilder, Output},
    wallet::{account::FilterOptions, Account, Result, Wallet},
};

// Replace with an alias id held in an unspent output of the account
const ALIAS_ID: &str = "0xc94fc4d280d63c7de09c8cc49ecefba6192e104d200ab7472db9e943e0feef7c";
// Replace with the correct increment of the current state index of your alias
const NEW_STATE_INDEX: u32 = 2;
// The metadata for the next state
const NEW_STATE_METADATA: &str = "new state metadata";

#[tokio::main]
async fn main() -> Result<()> {
    //  This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    // Get the account we generated with `01_create_wallet`
    let wallet = Wallet::builder()
        .with_storage_path(&var("WALLET_DB_PATH").unwrap())
        .finish()
        .await?;
    let account = wallet.get_account(&var("ACCOUNT_ALIAS_1").unwrap()).await?;

    sync_and_print_balance(&account).await?;

    // Set the stronghold password
    wallet
        .set_stronghold_password(&var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // Get the alias output by its alias id
    let alias_id = AliasId::from_str(ALIAS_ID)?;
    if let Some(unspent_alias_output) = account
        .unspent_outputs(Some(FilterOptions {
            output_types: Some(vec![AliasOutput::KIND]),
            ..Default::default()
        }))
        .await?
        .into_iter()
        .find_map(|output_data| match &output_data.output {
            Output::Alias(alias_output) => {
                let output_alias_id = alias_output.alias_id_non_null(&output_data.output_id);
                (output_alias_id == alias_id).then_some(output_data)
            }
            _ => None,
        })
    {
        println!(
            "Alias '{ALIAS_ID}' found in unspent output: '{}'",
            unspent_alias_output.output_id
        );

        let token_supply = account.client().get_token_supply().await?;
        let rent_structure = account.client().get_rent_structure().await?;

        let updated_alias_output = AliasOutputBuilder::from(unspent_alias_output.output.as_alias())
            // Minimum required storage deposit will change if the new metadata has a different size, so we will update
            // the amount
            .with_minimum_storage_deposit(rent_structure)
            .with_state_index(NEW_STATE_INDEX)
            .with_state_metadata(NEW_STATE_METADATA.as_bytes().to_vec())
            .finish_output(token_supply)?;

        println!("Sending transaction...",);
        send_and_wait_for_inclusion(&account, vec![updated_alias_output]).await?;
    } else {
        panic!("alias doesn't exist or is not unspent");
    }

    println!("Example finished successfully");
    Ok(())
}

async fn sync_and_print_balance(account: &Account) -> Result<()> {
    let alias = account.alias().await;
    let now = Instant::now();
    let balance = account.sync(None).await?;
    println!("{alias}'s account synced in: {:.2?}", now.elapsed());
    println!("{alias}'s base coin balance:\n{:#?}", balance.base_coin());
    println!("{alias}'s aliases:\n{:#?}", balance.aliases());
    Ok(())
}

async fn send_and_wait_for_inclusion(account: &Account, outputs: Vec<Output>) -> Result<()> {
    let transaction = account.send(outputs, None).await?;
    println!(
        "Transaction sent: {}/transaction/{}",
        var("EXPLORER_URL").unwrap(),
        transaction.transaction_id
    );
    // Wait for transaction to get included
    let block_id = account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    println!(
        "Transaction included: {}/block/{}",
        var("EXPLORER_URL").unwrap(),
        block_id
    );
    Ok(())
}
