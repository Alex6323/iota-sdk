// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

use hashbrown::HashSet;
use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

use crate::types::block::{
    input::{Input, INPUT_COUNT_RANGE},
    output::{InputsCommitment, NativeTokens, Output, OUTPUT_COUNT_RANGE},
    payload::{OptionalPayload, Payload},
    protocol::ProtocolParameters,
    Error,
};

/// A builder to build a [`RegularTransactionEssence`].
#[derive(Debug, Clone)]
#[must_use]
pub struct RegularTransactionEssenceBuilder {
    network_id: u64,
    inputs: Vec<Input>,
    inputs_commitment: InputsCommitment,
    outputs: Vec<Output>,
    payload: OptionalPayload,
}

impl RegularTransactionEssenceBuilder {
    /// Creates a new [`RegularTransactionEssenceBuilder`].
    pub fn new(network_id: u64, inputs_commitment: InputsCommitment) -> Self {
        Self {
            network_id,
            inputs: Vec::new(),
            inputs_commitment,
            outputs: Vec::new(),
            payload: OptionalPayload::default(),
        }
    }

    /// Adds inputs to a [`RegularTransactionEssenceBuilder`].
    pub fn with_inputs(mut self, inputs: impl Into<Vec<Input>>) -> Self {
        self.inputs = inputs.into();
        self
    }

    /// Add an input to a [`RegularTransactionEssenceBuilder`].
    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add outputs to a [`RegularTransactionEssenceBuilder`].
    pub fn with_outputs(mut self, outputs: impl Into<Vec<Output>>) -> Self {
        self.outputs = outputs.into();
        self
    }

    /// Add an output to a [`RegularTransactionEssenceBuilder`].
    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add a payload to a [`RegularTransactionEssenceBuilder`].
    pub fn with_payload(mut self, payload: impl Into<OptionalPayload>) -> Self {
        self.payload = payload.into();
        self
    }

    /// Finishes a [`RegularTransactionEssenceBuilder`] into a [`RegularTransactionEssence`].
    pub fn finish(self, protocol_parameters: &ProtocolParameters) -> Result<RegularTransactionEssence, Error> {
        if self.network_id != protocol_parameters.network_id() {
            return Err(Error::NetworkIdMismatch {
                expected: protocol_parameters.network_id(),
                actual: self.network_id,
            });
        }

        let inputs: BoxedSlicePrefix<Input, InputCount> = self
            .inputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidInputCount)?;

        verify_inputs::<true>(&inputs)?;

        let outputs: BoxedSlicePrefix<Output, OutputCount> = self
            .outputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidOutputCount)?;

        verify_outputs::<true>(&outputs, protocol_parameters)?;

        verify_payload::<true>(&self.payload)?;

        Ok(RegularTransactionEssence {
            network_id: self.network_id,
            inputs,
            inputs_commitment: self.inputs_commitment,
            outputs,
            payload: self.payload,
        })
    }

    ///
    pub fn finish_unverified(self) -> Result<RegularTransactionEssence, Error> {
        let inputs: BoxedSlicePrefix<Input, InputCount> = self
            .inputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidInputCount)?;

        verify_inputs::<true>(&inputs)?;

        let outputs: BoxedSlicePrefix<Output, OutputCount> = self
            .outputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidOutputCount)?;

        verify_outputs_unverified::<true>(&outputs)?;

        verify_payload::<true>(&self.payload)?;

        Ok(RegularTransactionEssence {
            network_id: self.network_id,
            inputs,
            inputs_commitment: self.inputs_commitment,
            outputs,
            payload: self.payload,
        })
    }
}

pub(crate) type InputCount = BoundedU16<{ *INPUT_COUNT_RANGE.start() }, { *INPUT_COUNT_RANGE.end() }>;
pub(crate) type OutputCount = BoundedU16<{ *OUTPUT_COUNT_RANGE.start() }, { *OUTPUT_COUNT_RANGE.end() }>;

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct RegularTransactionEssence {
    /// The unique value denoting whether the block was meant for mainnet, testnet, or a private network.
    #[packable(verify_with = verify_network_id)]
    network_id: u64,
    #[packable(verify_with = verify_inputs_packable)]
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidInputCount(p.into())))]
    inputs: BoxedSlicePrefix<Input, InputCount>,
    /// BLAKE2b-256 hash of the serialized outputs referenced in inputs by their OutputId.
    inputs_commitment: InputsCommitment,
    #[packable(verify_with = verify_outputs)]
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidOutputCount(p.into())))]
    outputs: BoxedSlicePrefix<Output, OutputCount>,
    #[packable(verify_with = verify_payload_packable)]
    payload: OptionalPayload,
}

impl RegularTransactionEssence {
    /// The essence kind of a [`RegularTransactionEssence`].
    pub const KIND: u8 = 1;

    /// Creates a new [`RegularTransactionEssenceBuilder`] to build a [`RegularTransactionEssence`].
    pub fn builder(network_id: u64, inputs_commitment: InputsCommitment) -> RegularTransactionEssenceBuilder {
        RegularTransactionEssenceBuilder::new(network_id, inputs_commitment)
    }

    /// Returns the network ID of a [`RegularTransactionEssence`].
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the inputs of a [`RegularTransactionEssence`].
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Returns the inputs commitment of a [`RegularTransactionEssence`].
    pub fn inputs_commitment(&self) -> &InputsCommitment {
        &self.inputs_commitment
    }

    /// Returns the outputs of a [`RegularTransactionEssence`].
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Returns the optional payload of a [`RegularTransactionEssence`].
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }
}

fn verify_network_id<const VERIFY: bool>(network_id: &u64, visitor: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY {
        let expected = visitor.network_id();

        if *network_id != expected {
            return Err(Error::NetworkIdMismatch {
                expected,
                actual: *network_id,
            });
        }
    }

    Ok(())
}

fn verify_inputs<const VERIFY: bool>(inputs: &[Input]) -> Result<(), Error> {
    if VERIFY {
        let mut seen_utxos = HashSet::new();

        for input in inputs.iter() {
            match input {
                Input::Utxo(utxo) => {
                    if !seen_utxos.insert(utxo) {
                        return Err(Error::DuplicateUtxo(*utxo));
                    }
                }
                _ => return Err(Error::InvalidInputKind(input.kind())),
            }
        }
    }

    Ok(())
}

fn verify_inputs_packable<const VERIFY: bool>(inputs: &[Input], _visitor: &ProtocolParameters) -> Result<(), Error> {
    verify_inputs::<VERIFY>(inputs)
}

fn verify_outputs<const VERIFY: bool>(outputs: &[Output], visitor: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY {
        let mut amount_sum: u64 = 0;
        let mut native_tokens_count: u8 = 0;
        let mut chain_ids = HashSet::new();

        for output in outputs.iter() {
            let (amount, native_tokens, chain_id) = match output {
                Output::Basic(output) => (output.amount(), output.native_tokens(), None),
                Output::Alias(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                Output::Foundry(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                Output::Nft(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                _ => return Err(Error::InvalidOutputKind(output.kind())),
            };

            amount_sum = amount_sum
                .checked_add(amount)
                .ok_or(Error::InvalidTransactionAmountSum(amount_sum as u128 + amount as u128))?;

            // Accumulated output balance must not exceed the total supply of tokens.
            if amount_sum > visitor.token_supply() {
                return Err(Error::InvalidTransactionAmountSum(amount_sum as u128));
            }

            native_tokens_count = native_tokens_count.checked_add(native_tokens.len() as u8).ok_or(
                Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16 + native_tokens.len() as u16),
            )?;

            if native_tokens_count > NativeTokens::COUNT_MAX {
                return Err(Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16));
            }

            if let Some(chain_id) = chain_id {
                if !chain_id.is_null() && !chain_ids.insert(chain_id) {
                    return Err(Error::DuplicateOutputChain(chain_id));
                }
            }

            output.verify_storage_deposit(*visitor.rent_structure(), visitor.token_supply())?;
        }
    }

    Ok(())
}

// TODO: find a solution to avoid this duplication.
fn verify_outputs_unverified<const VERIFY: bool>(outputs: &[Output]) -> Result<(), Error> {
    if VERIFY {
        let mut amount_sum: u64 = 0;
        let mut native_tokens_count: u8 = 0;
        let mut chain_ids = HashSet::new();

        for output in outputs.iter() {
            let (amount, native_tokens, chain_id) = match output {
                Output::Basic(output) => (output.amount(), output.native_tokens(), None),
                Output::Alias(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                Output::Foundry(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                Output::Nft(output) => (output.amount(), output.native_tokens(), Some(output.chain_id())),
                _ => return Err(Error::InvalidOutputKind(output.kind())),
            };

            amount_sum = amount_sum
                .checked_add(amount)
                .ok_or(Error::InvalidTransactionAmountSum(amount_sum as u128 + amount as u128))?;

            native_tokens_count = native_tokens_count.checked_add(native_tokens.len() as u8).ok_or(
                Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16 + native_tokens.len() as u16),
            )?;

            if native_tokens_count > NativeTokens::COUNT_MAX {
                return Err(Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16));
            }

            if let Some(chain_id) = chain_id {
                if !chain_id.is_null() && !chain_ids.insert(chain_id) {
                    return Err(Error::DuplicateOutputChain(chain_id));
                }
            }
        }
    }

    Ok(())
}

fn verify_payload<const VERIFY: bool>(payload: &OptionalPayload) -> Result<(), Error> {
    if VERIFY {
        match &payload.0 {
            Some(Payload::TaggedData(_)) | None => Ok(()),
            Some(payload) => Err(Error::InvalidPayloadKind(payload.kind())),
        }
    } else {
        Ok(())
    }
}

fn verify_payload_packable<const VERIFY: bool>(
    payload: &OptionalPayload,
    _visitor: &ProtocolParameters,
) -> Result<(), Error> {
    verify_payload::<VERIFY>(payload)
}

#[allow(missing_docs)]
pub mod dto {
    use alloc::{
        boxed::Box,
        string::{String, ToString},
    };
    use core::str::FromStr;

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::types::block::{input::dto::InputDto, output::dto::OutputDto, payload::dto::PayloadDto, Error};

    /// Describes the essence data making up a transaction by defining its inputs and outputs and an optional payload.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RegularTransactionEssenceDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub network_id: String,
        pub inputs: Vec<InputDto>,
        pub inputs_commitment: String,
        pub outputs: Vec<OutputDto>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub payload: Option<PayloadDto>,
    }

    impl From<&RegularTransactionEssence> for RegularTransactionEssenceDto {
        fn from(value: &RegularTransactionEssence) -> Self {
            Self {
                kind: RegularTransactionEssence::KIND,
                network_id: value.network_id().to_string(),
                inputs: value.inputs().iter().map(Into::into).collect::<Vec<_>>(),
                inputs_commitment: value.inputs_commitment().to_string(),
                outputs: value.outputs().iter().map(Into::into).collect::<Vec<_>>(),
                payload: match value.payload() {
                    Some(Payload::TaggedData(i)) => Some(PayloadDto::TaggedData(Box::new(i.as_ref().into()))),
                    Some(_) => unimplemented!(),
                    None => None,
                },
            }
        }
    }

    impl RegularTransactionEssence {
        fn _try_from_dto(
            value: &RegularTransactionEssenceDto,
            outputs: Vec<Output>,
        ) -> Result<RegularTransactionEssenceBuilder, Error> {
            let network_id = value
                .network_id
                .parse::<u64>()
                .map_err(|_| Error::InvalidField("network_id"))?;
            let inputs = value
                .inputs
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Input>, Error>>()?;

            let mut builder = Self::builder(network_id, InputsCommitment::from_str(&value.inputs_commitment)?)
                .with_inputs(inputs)
                .with_outputs(outputs);

            builder = if let Some(p) = &value.payload {
                if let PayloadDto::TaggedData(i) = p {
                    builder.with_payload(Payload::TaggedData(Box::new((i.as_ref()).try_into()?)))
                } else {
                    return Err(Error::InvalidField("payload"));
                }
            } else {
                builder
            };

            Ok(builder)
        }

        pub fn try_from_dto(
            value: &RegularTransactionEssenceDto,
            protocol_parameters: &ProtocolParameters,
        ) -> Result<Self, Error> {
            let outputs = value
                .outputs
                .iter()
                .map(|o| Output::try_from_dto(o, protocol_parameters.token_supply()))
                .collect::<Result<Vec<Output>, Error>>()?;

            let builder = Self::_try_from_dto(value, outputs)?;

            builder.finish(protocol_parameters).map_err(Into::into)
        }

        pub fn try_from_dto_unverified(value: &RegularTransactionEssenceDto) -> Result<Self, Error> {
            let outputs = value
                .outputs
                .iter()
                .map(Output::try_from_dto_unverified)
                .collect::<Result<Vec<Output>, Error>>()?;

            let builder = Self::_try_from_dto(value, outputs)?;

            builder.finish_unverified().map_err(Into::into)
        }
    }
}
