// Copyright 2021-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
import { Feature, HexEncodedString } from '../..';
import type { BasicOutputBuilderParams } from './basic-output-params';

/**
 * Options for building an Alias Output.
 */
export interface AliasOutputBuilderParams extends BasicOutputBuilderParams {
    /**
     * TODO.
     */
    aliasId: HexEncodedString;
    /**
     * TODO.
     */
    stateIndex?: number;
    /**
     * TODO.
     */
    stateMetadata?: HexEncodedString;
    /**
     * TODO.
     */
    foundryCounter?: number;
    /**
     * TODO.
     */
    immutableFeatures?: Feature[];
}
