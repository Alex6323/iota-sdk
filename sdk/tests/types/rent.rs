// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO: Re-enable when rent is figured out

// use iota_sdk::types::block::{
//     output::{Output, Rent},
//     protocol::protocol_parameters,
//     rand::output::{rand_account_output, rand_basic_output, rand_foundry_output, rand_nft_output},
// };

// fn output_in_range(output: Output, range: std::ops::RangeInclusive<u64>) {
//     let cost = output.storage_score(Default::default());
//     assert!(range.contains(&cost), "{output:#?} has a required storage cost of {cost}");
// }

// #[test]
// fn valid_rent_cost_range() {
//     let token_supply = protocol_parameters().token_supply();

//     output_in_range(Output::Account(rand_account_output(token_supply)), 445..=29_620);
//     output_in_range(Output::Basic(rand_basic_output(token_supply)), 414..=13_485);
//     output_in_range(Output::Foundry(rand_foundry_output(token_supply)), 496..=21_365);
//     output_in_range(Output::Nft(rand_nft_output(token_supply)), 435..=21_734);
// }
