// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { callUtilsMethod } from '../../../bindings';
import { HexEncodedString } from '../../utils';
import { OutputId } from '../output';

/**
 * All of the input types.
 */
enum InputType {
    /** TODO. */
    UTXO = 0,
    /** TODO. */
    Treasury = 1,
}

/**
 * TODO.
 */
abstract class Input {
    readonly type: InputType;

    /** TODO. */
    constructor(type: InputType) {
        this.type = type;
    }

    /**
     * Get the type of input.
     */
    getType(): InputType {
        return this.type;
    }
}

/**
 * Treasury Input.
 */
class TreasuryInput extends Input {
    /**
     * The milestone id of the input.
     */
    milestoneId: HexEncodedString;

    constructor(milestoneId: HexEncodedString) {
        super(InputType.Treasury);
        this.milestoneId = milestoneId;
    }
}

/**
 * UTXO Transaction Input.
 */
class UTXOInput extends Input {
    /**
     * The transaction Id.
     */
    transactionId: HexEncodedString;
    /**
     * The output index.
     */
    transactionOutputIndex: number;

    /** TODO. */
    constructor(
        transactionId: HexEncodedString,
        transactionOutputIndex: number,
    ) {
        super(InputType.UTXO);
        this.transactionId = transactionId;
        this.transactionOutputIndex = transactionOutputIndex;
    }

    /**
     * Creates a `UTXOInput` from an output id.
     */
    static fromOutputId(outputId: OutputId): UTXOInput {
        const input = callUtilsMethod({
            name: 'outputIdToUtxoInput',
            data: {
                outputId,
            },
        });
        return new UTXOInput(input.transactionId, input.transactionOutputIndex);
    }
}

const InputDiscriminator = {
    property: 'type',
    subTypes: [
        { value: TreasuryInput, name: InputType.Treasury as any },
        { value: UTXOInput, name: InputType.UTXO as any },
    ],
};

export { InputDiscriminator, InputType, Input, TreasuryInput, UTXOInput };
