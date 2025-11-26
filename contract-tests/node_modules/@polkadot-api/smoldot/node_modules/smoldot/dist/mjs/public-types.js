// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
/**
 * Thrown in case of a problem when initializing the chain.
 */
export class AddChainError extends Error {
    constructor(message) {
        super(message);
        this.name = "AddChainError";
    }
}
/**
 * Thrown in case the API user tries to use a chain or client that has already been destroyed.
 */
export class AlreadyDestroyedError extends Error {
    constructor() {
        super();
        this.name = "AlreadyDestroyedError";
    }
}
/**
 * Thrown when trying to send a JSON-RPC message to a chain whose JSON-RPC system hasn't been
 * enabled.
 */
export class JsonRpcDisabledError extends Error {
    constructor() {
        super();
        this.name = "JsonRpcDisabledError";
    }
}
/**
 * Thrown in case the underlying client encounters an unexpected crash.
 *
 * This is always an internal bug in smoldot and is never supposed to happen.
 */
export class CrashError extends Error {
    constructor(message) {
        super(message);
    }
}
/**
 * Thrown in case the buffer of JSON-RPC requests is full and cannot accept any more request.
 */
export class QueueFullError extends Error {
    constructor() {
        super("JSON-RPC requests queue is full");
    }
}
