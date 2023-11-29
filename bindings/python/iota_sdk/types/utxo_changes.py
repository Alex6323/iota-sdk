# Copyright 2023 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

from typing import List

from dataclasses import dataclass
from iota_sdk.types.common import HexStr, json


@json
@dataclass
class UtxoChanges():
    """Response of GET /api/core/v2/milestone/{milestone_index}/utxo-changes.
    Returns all UTXO changes that happened at a specific milestone.
    """
    index: int
    created_outputs: List[HexStr]
    consumed_outputs: List[HexStr]
