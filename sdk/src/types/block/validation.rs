// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use super::{
    core::{verify_parents, BlockWrapper},
    parent::{ShallowLikeParents, StrongParents, WeakParents},
    protocol::{ProtocolParameters, ProtocolParametersHash, WorkScore, WorkScoreStructure},
    signature::{Ed25519Signature, Signature},
    slot::{SlotCommitmentId, SlotIndex},
    Block, BlockBuilder, Error, IssuerId,
};

pub type ValidationBlock = BlockWrapper<ValidationBlockData>;

impl BlockBuilder<ValidationBlock> {
    /// Creates a new [`BlockBuilder`] for a [`ValidationBlock`].
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        protocol_params: ProtocolParameters,
        issuing_time: u64,
        slot_commitment_id: SlotCommitmentId,
        latest_finalized_slot: SlotIndex,
        issuer_id: IssuerId,
        strong_parents: StrongParents,
        highest_supported_version: u8,
        protocol_parameters: &ProtocolParameters,
        signature: Ed25519Signature,
    ) -> Self {
        Self(BlockWrapper {
            protocol_params,
            issuing_time,
            slot_commitment_id,
            latest_finalized_slot,
            issuer_id,
            data: ValidationBlockData {
                strong_parents,
                weak_parents: Default::default(),
                shallow_like_parents: Default::default(),
                highest_supported_version,
                protocol_parameters_hash: protocol_parameters.hash(),
            },
            signature,
        })
    }

    /// Adds weak parents to a [`BlockBuilder`].
    #[inline(always)]
    pub fn with_weak_parents(mut self, weak_parents: impl Into<WeakParents>) -> Self {
        self.0.data.weak_parents = weak_parents.into();
        self
    }

    /// Adds shallow like parents to a [`BlockBuilder`].
    #[inline(always)]
    pub fn with_shallow_like_parents(mut self, shallow_like_parents: impl Into<ShallowLikeParents>) -> Self {
        self.0.data.shallow_like_parents = shallow_like_parents.into();
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationBlockData {
    /// Blocks that are strongly directly approved.
    pub(crate) strong_parents: StrongParents,
    /// Blocks that are weakly directly approved.
    pub(crate) weak_parents: WeakParents,
    /// Blocks that are directly referenced to adjust opinion.
    pub(crate) shallow_like_parents: ShallowLikeParents,
    /// The highest supported protocol version the issuer of this block supports.
    pub(crate) highest_supported_version: u8,
    /// The hash of the protocol parameters for the Highest Supported Version.
    pub(crate) protocol_parameters_hash: ProtocolParametersHash,
}

impl ValidationBlock {
    pub const KIND: u8 = 1;

    /// Returns the strong parents of a [`ValidationBlock`].
    #[inline(always)]
    pub fn strong_parents(&self) -> &StrongParents {
        &self.data.strong_parents
    }

    /// Returns the weak parents of a [`ValidationBlock`].
    #[inline(always)]
    pub fn weak_parents(&self) -> &WeakParents {
        &self.data.weak_parents
    }

    /// Returns the shallow like parents of a [`ValidationBlock`].
    #[inline(always)]
    pub fn shallow_like_parents(&self) -> &ShallowLikeParents {
        &self.data.shallow_like_parents
    }

    /// Returns the highest supported protocol version of a [`ValidationBlock`].
    #[inline(always)]
    pub fn highest_supported_version(&self) -> u8 {
        self.data.highest_supported_version
    }

    /// Returns the protocol parameters hash of a [`ValidationBlock`].
    #[inline(always)]
    pub fn protocol_parameters_hash(&self) -> ProtocolParametersHash {
        self.data.protocol_parameters_hash
    }

    /// Returns the work score of a [`ValidationBlock`].
    pub fn workscore(&self) -> u32 {
        let workscore_structure = self.protocol_parameters().work_score_structure;
        let workscore_bytes = workscore_structure.data_kilobyte * self.packed_len() as u32;
        let mut workscore_kilobytes = workscore_bytes / 1024;

        workscore_kilobytes += self.workscore_header(workscore_structure);
        workscore_kilobytes += self.data.workscore(workscore_structure);
        workscore_kilobytes += self.workscore_signature(workscore_structure);
        workscore_kilobytes
    }
}

impl WorkScore for ValidationBlockData {
    fn workscore(&self, _workscore_structure: WorkScoreStructure) -> u32 {
        // Validator blocks do not incur any work score as they do not burn mana
        0
    }
}

impl Packable for ValidationBlockData {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.strong_parents.pack(packer)?;
        self.weak_parents.pack(packer)?;
        self.shallow_like_parents.pack(packer)?;
        self.highest_supported_version.pack(packer)?;
        self.protocol_parameters_hash.pack(packer)?;
        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let strong_parents = StrongParents::unpack::<_, VERIFY>(unpacker, &())?;
        let weak_parents = WeakParents::unpack::<_, VERIFY>(unpacker, &())?;
        let shallow_like_parents = ShallowLikeParents::unpack::<_, VERIFY>(unpacker, &())?;

        if VERIFY {
            verify_parents(&strong_parents, &weak_parents, &shallow_like_parents).map_err(UnpackError::Packable)?;
        }

        let highest_supported_version = u8::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        let protocol_parameters_hash = ProtocolParametersHash::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        if VERIFY {
            validate_protocol_params_hash(&protocol_parameters_hash, visitor).map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            strong_parents,
            weak_parents,
            shallow_like_parents,
            highest_supported_version,
            protocol_parameters_hash,
        })
    }
}

impl Packable for ValidationBlock {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.protocol_version().pack(packer)?;
        self.network_id().pack(packer)?;
        self.issuing_time.pack(packer)?;
        self.slot_commitment_id.pack(packer)?;
        self.latest_finalized_slot.pack(packer)?;
        self.issuer_id.pack(packer)?;
        Self::KIND.pack(packer)?;
        self.data.pack(packer)?;
        Signature::Ed25519(self.signature).pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        protocol_params: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let start_opt = unpacker.read_bytes();

        let protocol_version = u8::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        if VERIFY && protocol_version != protocol_params.version() {
            return Err(UnpackError::Packable(Error::ProtocolVersionMismatch {
                expected: protocol_params.version(),
                actual: protocol_version,
            }));
        }

        let network_id = u64::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        if VERIFY && network_id != protocol_params.network_id() {
            return Err(UnpackError::Packable(Error::NetworkIdMismatch {
                expected: protocol_params.network_id(),
                actual: network_id,
            }));
        }

        let issuing_time = u64::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        let slot_commitment_id = SlotCommitmentId::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        let latest_finalized_slot = SlotIndex::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        let issuer_id = IssuerId::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        let kind = u8::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        if kind != Self::KIND {
            return Err(Error::InvalidBlockKind(kind)).map_err(UnpackError::Packable);
        }

        let data = ValidationBlockData::unpack::<_, VERIFY>(unpacker, protocol_params)?;

        let Signature::Ed25519(signature) = Signature::unpack::<_, VERIFY>(unpacker, &())?;

        let block = Self {
            protocol_params: protocol_params.clone(),
            issuing_time,
            slot_commitment_id,
            latest_finalized_slot,
            issuer_id,
            data,
            signature,
        };

        if VERIFY {
            let block_len = if let (Some(start), Some(end)) = (start_opt, unpacker.read_bytes()) {
                end - start
            } else {
                block.packed_len()
            };

            if block_len > Block::LENGTH_MAX {
                return Err(UnpackError::Packable(Error::InvalidBlockLength(block_len)));
            }
        }

        Ok(block)
    }
}

fn validate_protocol_params_hash(hash: &ProtocolParametersHash, params: &ProtocolParameters) -> Result<(), Error> {
    let params_hash = params.hash();
    if hash != &params_hash {
        return Err(Error::InvalidProtocolParametersHash {
            expected: params_hash,
            actual: *hash,
        });
    }
    Ok(())
}

#[cfg(feature = "serde")]
pub(crate) mod dto {
    use alloc::collections::BTreeSet;

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::types::{
        block::{BlockId, Error},
        TryFromDto, ValidationParams,
    };

    /// A special type of block used by validators to secure the network.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ValidationBlockDataDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub strong_parents: BTreeSet<BlockId>,
        pub weak_parents: BTreeSet<BlockId>,
        pub shallow_like_parents: BTreeSet<BlockId>,
        pub highest_supported_version: u8,
        pub protocol_parameters_hash: ProtocolParametersHash,
    }

    impl From<&ValidationBlockData> for ValidationBlockDataDto {
        fn from(value: &ValidationBlockData) -> Self {
            Self {
                kind: ValidationBlock::KIND,
                strong_parents: value.strong_parents.to_set(),
                weak_parents: value.weak_parents.to_set(),
                shallow_like_parents: value.shallow_like_parents.to_set(),
                highest_supported_version: value.highest_supported_version,
                protocol_parameters_hash: value.protocol_parameters_hash,
            }
        }
    }

    impl TryFromDto for ValidationBlockData {
        type Dto = ValidationBlockDataDto;
        type Error = Error;

        fn try_from_dto_with_params_inner(dto: Self::Dto, params: ValidationParams<'_>) -> Result<Self, Self::Error> {
            if let Some(protocol_params) = params.protocol_parameters() {
                validate_protocol_params_hash(&dto.protocol_parameters_hash, protocol_params)?;
            }
            Ok(Self {
                strong_parents: StrongParents::from_set(dto.strong_parents)?,
                weak_parents: WeakParents::from_set(dto.weak_parents)?,
                shallow_like_parents: ShallowLikeParents::from_set(dto.shallow_like_parents)?,
                highest_supported_version: dto.highest_supported_version,
                protocol_parameters_hash: dto.protocol_parameters_hash,
            })
        }
    }
}
