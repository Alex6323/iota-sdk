// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will spam transactions from multiple threads simultaneously to our own address.
//!
//! `cargo run --example threads --release`

use std::{env::var, time::Duration};

use iota_sdk::{
    client::{
        constants::SHIMMER_COIN_TYPE,
        request_funds_from_faucet,
        secret::{mnemonic::MnemonicSecretManager, SecretManager},
    },
    wallet::{account::types::AccountAddress, Account, ClientOptions, Result, SendAmountParams, Wallet},
};
use tokio::{task::JoinSet, time::sleep};

// The account alias used in this example.
const ACCOUNT_ALIAS: &str = "spammer";
// The number of spamming rounds.
const NUM_ROUNDS: usize = 1000;
// The amount to send in each transaction.
const SEND_AMOUNT: u64 = 1_000_000;
// The number of concurrent transactions.
const NUM_CONCURRENT_TXS: usize = 32;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    println!(
        "{} cores detected. Running spammer with {NUM_CONCURRENT_TXS} concurrent transactions...",
        num_cpus::get(),
    );

    // Restore wallet from a mnemonic phrase.
    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;
    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;
    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::Mnemonic(secret_manager))
        .with_storage_path(&var("WALLET_DB_PATH").unwrap())
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Ensure there's some base coin balance
    let account = get_or_create_account(&wallet, ACCOUNT_ALIAS).await?;
    let addresses = generate_max_addresses(&account, NUM_CONCURRENT_TXS).await?;

    // Ensure there are enough available funds for spamming on each address
    let amount = may_request_funds(&account, addresses[0].address().to_string().as_str()).await?;
    account.sync(None).await?;

    let num_addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?.len();
    println!("Addresses with balance count (before): {num_addresses_with_unspent_outputs}",);

    println!("Splitting available funds...");
    let split_amount = amount / NUM_CONCURRENT_TXS as u64;

    for address in addresses.iter().map(|addr| addr.address().to_string()) {
        println!("Sending {split_amount} to {address}");
        let outputs = vec![SendAmountParams::new(address, split_amount)];
        let transaction = account.send_amount(outputs, None).await?;

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

        account.sync(None).await?;
    }

    let num_addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?.len();
    println!("Addresses with balance count (after): {num_addresses_with_unspent_outputs}",);

    println!("Spamming...");
    for i in 1..=NUM_ROUNDS {
        println!("ROUND #{i}/{NUM_ROUNDS}");

        let mut tasks = JoinSet::<std::result::Result<(), (usize, iota_sdk::wallet::Error)>>::new();

        for n in 0..NUM_CONCURRENT_TXS {
            let account_clone = account.clone();
            let recv_address_clone = addresses[(n + 1) % NUM_CONCURRENT_TXS].address().to_string();

            tasks.spawn(async move {
                println!("Thread {n}: Sending {SEND_AMOUNT} to {recv_address_clone}");
                let outputs = vec![SendAmountParams::new(recv_address_clone, SEND_AMOUNT)];
                let transaction = account_clone.send_amount(outputs, None).await.map_err(|err| (n, err))?;

                println!(
                    "Thread {n}: Transaction sent: {}/transaction/{}",
                    var("EXPLORER_URL").unwrap(),
                    transaction.transaction_id
                );

                Ok(())
            });
        }

        let mut error_state: std::result::Result<(), ()> = Ok(());
        while let Some(Ok(res)) = tasks.join_next().await {
            match res {
                Ok(()) => {}
                Err((n, err)) => {
                    println!("Thread {n}: Failure: {err}");
                    error_state = Err(());
                }
            }
        }

        if error_state.is_err() {
            // Sync when getting an error, because that's probably when no outputs are available anymore
            let mut balance = account.sync(None).await?;
            while balance.base_coin().available() == 0 {
                println!("No funds available");
                sleep(Duration::from_secs(2)).await;
                balance = account.sync(None).await?;
                println!("Account synced");
            }
        }
    }
    Ok(())
}

async fn get_or_create_account(wallet: &Wallet, alias: &str) -> Result<Account> {
    Ok(if let Ok(account) = wallet.get_account(alias).await {
        account
    } else {
        println!("Creating account '{alias}'");
        wallet.create_account().with_alias(alias.to_string()).finish().await?
    })
}

async fn generate_max_addresses(account: &Account, max: usize) -> Result<Vec<AccountAddress>> {
    let alias = account.alias().await;
    if account.addresses().await?.len() < max {
        let num_addresses_to_generate = max - account.addresses().await?.len();
        println!("Generating {num_addresses_to_generate} addresses for account '{alias}'...");
        account
            .generate_addresses(num_addresses_to_generate as u32, None)
            .await?;
    }
    account.addresses().await
}

async fn may_request_funds(account: &Account, bech32_address: &str) -> Result<u64> {
    let balance = account.sync(None).await?;
    let available_funds_before = balance.base_coin().available();
    println!("Current available funds: {available_funds_before}");

    if available_funds_before < NUM_CONCURRENT_TXS as u64 * SEND_AMOUNT {
        println!("Requesting funds from faucet...");
        let faucet_response = request_funds_from_faucet(&var("FAUCET_URL").unwrap(), bech32_address).await?;
        println!("Response from faucet: {}", faucet_response.trim_end());
        if faucet_response.contains("error") {
            panic!("Requesting funds failed (error response)");
        }

        println!("Waiting for funds (timeout=60s)...");
        // Check for changes to the balance
        let start = std::time::Instant::now();
        let available_funds_after = loop {
            if start.elapsed().as_secs() > 60 {
                panic!("Requesting funds failed (timeout)");
            };
            let balance = account.sync(None).await?;
            let available_funds_after = balance.base_coin().available();
            if available_funds_after > available_funds_before {
                break available_funds_after;
            } else {
                tokio::time::sleep(instant::Duration::from_secs(2)).await;
            }
        };
        println!("New available funds: {available_funds_after}");
        Ok(available_funds_after)
    } else {
        println!("No faucet request necessary");
        Ok(available_funds_before)
    }
}
