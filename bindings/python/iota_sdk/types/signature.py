# Copyright 2023 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

from dataclasses import dataclass
from iota_sdk.types.common import HexStr

@dataclass
class Ed25519Signature():
    """ED25519 signature.

    Attributes:
        publicKey (HexStr): the public key as hex string
        signature (HexStr): the signature as hex string
        type (int): signature type
    """
    publicKey: HexStr
    signature: HexStr
    type: int = 0
