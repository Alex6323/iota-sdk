// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

use derive_more::From;

pub use self::ed25519::Ed25519Signature;
use crate::types::block::Error;

/// A `Signature` contains a signature which is used to unlock a transaction input.
///
/// This is defined as part of the Unspent Transaction Output (UTXO) transaction protocol.
///
/// RFC: <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#signature-unlock-block>
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, From, packable::Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidSignatureKind)]
pub enum Signature {
    /// An Ed25519 signature.
    #[packable(tag = Ed25519Signature::KIND)]
    Ed25519(Ed25519Signature),
}

impl core::fmt::Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ed25519(signature) => signature.fmt(f),
        }
    }
}

impl Signature {
    /// Returns the signature kind of a `Signature`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Signature::KIND,
        }
    }
}

#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    pub use super::ed25519::dto::Ed25519SignatureDto;
    use super::*;
    use crate::types::block::Error;

    /// Describes all the different signature types.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, From)]
    #[serde(untagged)]
    pub enum SignatureDto {
        Ed25519(Ed25519SignatureDto),
    }

    impl From<&Signature> for SignatureDto {
        fn from(value: &Signature) -> Self {
            match value {
                Signature::Ed25519(s) => Self::Ed25519(s.into()),
            }
        }
    }

    impl TryFrom<&SignatureDto> for Signature {
        type Error = Error;

        fn try_from(value: &SignatureDto) -> Result<Self, Self::Error> {
            match value {
                SignatureDto::Ed25519(s) => Ok(Self::Ed25519(s.try_into()?)),
            }
        }
    }
}
