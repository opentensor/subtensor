// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
import { startWithBytecode } from './no-auto-bytecode-nodejs.js';
import { compileBytecode } from './bytecode-nodejs.js';
export { AddChainError, AlreadyDestroyedError, CrashError, QueueFullError, JsonRpcDisabledError } from './public-types.js';
/**
 * Initializes a new client. This is a pre-requisite to connecting to a blockchain.
 *
 * Can never fail.
 *
 * @param options Configuration of the client. Defaults to `{}`.
 */
export function start(options) {
    options = options || {};
    return startWithBytecode(Object.assign({ bytecode: compileBytecode() }, options));
}
