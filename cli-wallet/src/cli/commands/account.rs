use iota_sdk::wallet::account::{types::AccountAddress, AccountHandle};

use super::AccountCommand;
use crate::println_log_info;

/// Creates a new address.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct NewAddress;

/// Lists all addresses.
#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct ListAddresses;

#[async_trait::async_trait]
impl AccountCommand for ListAddresses {
    async fn exec(&self, account: &AccountHandle) -> eyre::Result<()> {
        let addresses = account.addresses().await?;

        if addresses.is_empty() {
            println_log_info!("No addresses found");
        } else {
            for address in addresses {
                print_address(account, &address).await?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct MintNft;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct BurnNft;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct Balance;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SyncBalance;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct Claim;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct Consolidate;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct CreateAliasOutput;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct MintNativeToken;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct BurnNativeToken;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct IncreaseNativeToken;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct DecreaseNativeToken;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct DestroyAlias;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct DestroyFoundry;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct Faucet;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct GetOutput;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct GetOutputs;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct GetUnspentOutputs;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SendAmount;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SendMicroTransaction;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SendNativeToken;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct SendNft;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct GetTransactions;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct Vote;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct StopParticipating;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct ParticipationOverview;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct VotingPower;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct IncreaseVotingPower;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct DecreaseVotingPower;

#[derive(Clone, Debug, PartialEq, Eq, clap::Parser)]
pub struct VotingOutput;

async fn print_address(account_handle: &AccountHandle, address: &AccountAddress) -> eyre::Result<()> {
    let mut log = format!("Address {}: {}", address.key_index(), address.address().to_bech32());

    if *address.internal() {
        log = format!("{log}\nChange address");
    }

    let addresses = account_handle.addresses_with_unspent_outputs().await?;

    if let Ok(index) = addresses.binary_search_by_key(&(address.key_index(), address.internal()), |a| {
        (a.key_index(), a.internal())
    }) {
        log = format!("{log}\nOutputs: {:#?}", addresses[index].output_ids());
    }

    println_log_info!("{log}");

    Ok(())
}
