// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example quorum --release

use std::env;

use dotenv::dotenv;
use iota_client::{
    node_api::indexer::query_parameters::QueryParameter,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Client, Result,
};

/// In this example we will get the account balance of a known seed with quorum, which will compare the responses from
/// the nodes

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe/")?
        .with_node("http://localhost:14265")?
        .with_node("https://api.thin-hornet-1.h.chrysalis-devnet.iota.cafe/")?
        .with_quorum(true)
        .with_min_quorum_size(3)
        .with_quorum_threshold(66)
        .finish()
        .await?;

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(
        &env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap(),
    )?);

    // Generate the first address
    let addresses = client
        .get_addresses(&secret_manager)
        .with_account_index(0)
        .with_range(0..1)
        .finish()
        .await?;

    // Get output ids of outputs that can be controlled by this address without further unlock constraints
    let output_ids = client
        .output_ids(vec![
            QueryParameter::Address(addresses[0].clone()),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ])
        .await?;
    println!("Address outputs: {:?}", output_ids);

    Ok(())
}
