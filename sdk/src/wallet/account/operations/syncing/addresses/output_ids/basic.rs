// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(not(target_family = "wasm"))]
use std::collections::HashSet;

#[cfg(not(target_family = "wasm"))]
use futures::FutureExt;

#[cfg(not(target_family = "wasm"))]
use crate::types::api::plugins::indexer::OutputIdsResponse;
use crate::{
    client::node_api::indexer::query_parameters::QueryParameter,
    types::block::{address::Bech32Address, output::OutputId, ConvertTo},
    wallet::Account,
};

impl Account {
    /// Returns output ids of basic outputs that have only the address unlock condition
    pub(crate) async fn get_basic_output_ids_with_address_unlock_condition_only(
        &self,
        bech32_address: impl ConvertTo<Bech32Address>,
    ) -> crate::client::Result<Vec<OutputId>> {
        let bech32_address = bech32_address.convert()?;
        // Only request basic outputs with `AddressUnlockCondition` only
        Ok(self
            .client()
            .basic_output_ids([
                QueryParameter::Address(bech32_address),
                QueryParameter::HasExpiration(false),
                QueryParameter::HasTimelock(false),
                QueryParameter::HasStorageDepositReturn(false),
            ])
            .await?
            .items)
    }

    /// Returns output ids of basic outputs that have the address in the `AddressUnlockCondition`,
    /// `ExpirationUnlockCondition` or `StorageDepositReturnUnlockCondition`
    pub(crate) async fn get_basic_output_ids_with_any_unlock_condition(
        &self,
        bech32_address: impl ConvertTo<Bech32Address>,
    ) -> crate::wallet::Result<Vec<OutputId>> {
        let bech32_address = bech32_address.convert()?;
        // aliases and foundries
        #[cfg(target_family = "wasm")]
        {
            let mut output_ids = Vec::new();
            output_ids.extend(
                self.client()
                    .basic_output_ids([QueryParameter::Address(bech32_address)])
                    .await?
                    .items,
            );
            output_ids.extend(
                self.client()
                    .basic_output_ids([QueryParameter::StorageDepositReturnAddress(bech32_address)])
                    .await?
                    .items,
            );
            output_ids.extend(
                self.client()
                    .basic_output_ids([QueryParameter::ExpirationReturnAddress(bech32_address)])
                    .await?
                    .items,
            );

            Ok(output_ids)
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let client = self.client();
            let tasks = [
                // Get basic outputs
                async move {
                    let client = client.clone();
                    tokio::spawn(async move {
                        client
                            .basic_output_ids([QueryParameter::Address(bech32_address)])
                            .await
                            .map_err(From::from)
                    })
                    .await
                }
                .boxed(),
                // Get outputs where the address is in the storage deposit return unlock condition
                async move {
                    let client = client.clone();
                    tokio::spawn(async move {
                        client
                            .basic_output_ids([QueryParameter::StorageDepositReturnAddress(bech32_address)])
                            .await
                            .map_err(From::from)
                    })
                    .await
                }
                .boxed(),
                // Get outputs where the address is in an expired expiration unlock condition
                async move {
                    let client = client.clone();
                    tokio::spawn(async move {
                        client
                            .basic_output_ids([QueryParameter::ExpirationReturnAddress(bech32_address)])
                            .await
                            .map_err(From::from)
                    })
                    .await
                }
                .boxed(),
            ];

            // Get all results
            let mut output_ids = HashSet::new();
            let results: Vec<crate::wallet::Result<OutputIdsResponse>> = futures::future::try_join_all(tasks).await?;

            for res in results {
                let found_output_ids = res?;
                output_ids.extend(found_output_ids.items);
            }

            Ok(output_ids.into_iter().collect())
        }
    }
}
