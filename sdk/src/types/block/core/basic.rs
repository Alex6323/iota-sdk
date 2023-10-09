// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::types::block::{
    core::{parent::verify_parents_sets, Block, Parents},
    payload::{OptionalPayload, Payload},
    protocol::{ProtocolParameters, WorkScore, WorkScoreStructure},
    Error,
};

pub type StrongParents = Parents<1, 8>;
pub type WeakParents = Parents<0, 8>;
pub type ShallowLikeParents = Parents<0, 8>;

/// A builder for a [`BasicBlock`].
pub struct BasicBlockBuilder {
    strong_parents: StrongParents,
    weak_parents: WeakParents,
    shallow_like_parents: ShallowLikeParents,
    payload: OptionalPayload,
    max_burned_mana: u64,
}

impl BasicBlockBuilder {
    /// Creates a new [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn new(strong_parents: StrongParents, max_burned_mana: u64) -> Self {
        Self {
            strong_parents,
            weak_parents: WeakParents::default(),
            shallow_like_parents: ShallowLikeParents::default(),
            payload: OptionalPayload::default(),
            max_burned_mana,
        }
    }

    /// Adds strong parents to a [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn with_strong_parents(mut self, strong_parents: impl Into<StrongParents>) -> Self {
        self.strong_parents = strong_parents.into();
        self
    }

    /// Adds weak parents to a [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn with_weak_parents(mut self, weak_parents: impl Into<WeakParents>) -> Self {
        self.weak_parents = weak_parents.into();
        self
    }

    /// Adds shallow like parents to a [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn with_shallow_like_parents(mut self, shallow_like_parents: impl Into<ShallowLikeParents>) -> Self {
        self.shallow_like_parents = shallow_like_parents.into();
        self
    }

    /// Adds a payload to a [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn with_payload(mut self, payload: impl Into<OptionalPayload>) -> Self {
        self.payload = payload.into();
        self
    }

    /// Adds max burned mana to a [`BasicBlockBuilder`].
    #[inline(always)]
    pub fn with_max_burned_mana(mut self, max_burned_mana: u64) -> Self {
        self.max_burned_mana = max_burned_mana;
        self
    }

    /// Finishes the builder into a [`BasicBlock`].
    pub fn finish(self) -> Result<BasicBlock, Error> {
        verify_parents_sets(&self.strong_parents, &self.weak_parents, &self.shallow_like_parents)?;

        Ok(BasicBlock {
            strong_parents: self.strong_parents,
            weak_parents: self.weak_parents,
            shallow_like_parents: self.shallow_like_parents,
            payload: self.payload,
            max_burned_mana: self.max_burned_mana,
        })
    }

    /// Finishes the builder into a [`Block`].
    pub fn finish_block(self) -> Result<Block, Error> {
        Ok(Block::from(self.finish()?))
    }
}

impl From<BasicBlock> for BasicBlockBuilder {
    fn from(value: BasicBlock) -> Self {
        Self {
            strong_parents: value.strong_parents,
            weak_parents: value.weak_parents,
            shallow_like_parents: value.shallow_like_parents,
            payload: value.payload,
            max_burned_mana: value.max_burned_mana,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    /// Blocks that are strongly directly approved.
    strong_parents: StrongParents,
    /// Blocks that are weakly directly approved.
    weak_parents: WeakParents,
    /// Blocks that are directly referenced to adjust opinion.
    shallow_like_parents: ShallowLikeParents,
    /// The optional [`Payload`] of the block.
    payload: OptionalPayload,
    /// The amount of Mana the Account identified by [`IssuerId`](super::IssuerId) is at most willing to burn for this
    /// block.
    max_burned_mana: u64,
}

impl BasicBlock {
    pub const KIND: u8 = 0;

    /// Returns the strong parents of a [`BasicBlock`].
    #[inline(always)]
    pub fn strong_parents(&self) -> &StrongParents {
        &self.strong_parents
    }

    /// Returns the weak parents of a [`BasicBlock`].
    #[inline(always)]
    pub fn weak_parents(&self) -> &WeakParents {
        &self.weak_parents
    }

    /// Returns the shallow like parents of a [`BasicBlock`].
    #[inline(always)]
    pub fn shallow_like_parents(&self) -> &ShallowLikeParents {
        &self.shallow_like_parents
    }

    /// Returns the optional payload of a [`BasicBlock`].
    #[inline(always)]
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    /// Returns the max burned mana of a [`BasicBlock`].
    #[inline(always)]
    pub fn max_burned_mana(&self) -> u64 {
        self.max_burned_mana
    }
}

impl WorkScore for BasicBlock {
    fn work_score(&self, workscore_structure: WorkScoreStructure) -> u32 {
        let mut score = workscore_structure.block;
        let min_strong_parents_threshold = workscore_structure.min_strong_parents_threshold as usize;
        if self.strong_parents.len() < min_strong_parents_threshold {
            let missing_parents_count = min_strong_parents_threshold - self.strong_parents.len();
            score += workscore_structure.missing_parent * missing_parents_count as u32;
        }
        if let Some(payload) = &*self.payload {
            score += payload.work_score(workscore_structure);
        }
        score
    }
}

impl Packable for BasicBlock {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.strong_parents.pack(packer)?;
        self.weak_parents.pack(packer)?;
        self.shallow_like_parents.pack(packer)?;
        self.payload.pack(packer)?;
        self.max_burned_mana.pack(packer)?;

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
            verify_parents_sets(&strong_parents, &weak_parents, &shallow_like_parents)
                .map_err(UnpackError::Packable)?;
        }

        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker, visitor)?;

        let max_burned_mana = u64::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        Ok(Self {
            strong_parents,
            weak_parents,
            shallow_like_parents,
            payload,
            max_burned_mana,
        })
    }
}

#[cfg(feature = "serde")]
pub(crate) mod dto {
    use alloc::collections::BTreeSet;

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::types::{
        block::{payload::dto::PayloadDto, BlockId, Error},
        TryFromDto, ValidationParams,
    };

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BasicBlockDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub strong_parents: BTreeSet<BlockId>,
        pub weak_parents: BTreeSet<BlockId>,
        pub shallow_like_parents: BTreeSet<BlockId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub payload: Option<PayloadDto>,
        #[serde(with = "crate::utils::serde::string")]
        pub max_burned_mana: u64,
    }

    impl From<&BasicBlock> for BasicBlockDto {
        fn from(value: &BasicBlock) -> Self {
            Self {
                kind: BasicBlock::KIND,
                strong_parents: value.strong_parents.to_set(),
                weak_parents: value.weak_parents.to_set(),
                shallow_like_parents: value.shallow_like_parents.to_set(),
                payload: value.payload.as_ref().map(Into::into),
                max_burned_mana: value.max_burned_mana,
            }
        }
    }

    impl TryFromDto for BasicBlock {
        type Dto = BasicBlockDto;
        type Error = Error;

        fn try_from_dto_with_params_inner(dto: Self::Dto, params: ValidationParams<'_>) -> Result<Self, Self::Error> {
            BasicBlockBuilder::new(StrongParents::from_set(dto.strong_parents)?, dto.max_burned_mana)
                .with_weak_parents(WeakParents::from_set(dto.weak_parents)?)
                .with_shallow_like_parents(ShallowLikeParents::from_set(dto.shallow_like_parents)?)
                .with_payload(
                    dto.payload
                        .map(|payload| Payload::try_from_dto_with_params_inner(payload, params))
                        .transpose()?,
                )
                .finish()
        }
    }
}
