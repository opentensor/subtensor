// Smoldot
// Copyright (C) 2023  Pierre Krieger
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
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
/// <reference lib="dom" />
import * as instance from './internals/remote-instance.js';
/**
 * Runs the CPU-heavy parts of smoldot. Must be passed a port whose other end is passed to
 * `ClientOptions.portToWorker`.
 *
 * Returns a `Promise` that is ready when the smoldot client is shut down (either because it
 * crashes or intentionally with a call to `Client.terminate`).
 * Since this function is asynchronous, this `Promise` is wrapped around another `Promise`. In
 * other words, the outer `Promise` is ready when execution starts, and the inner `Promise` is
 * ready when execution ends.
 */
export function run(messagePort) {
    return __awaiter(this, void 0, void 0, function* () {
        const whenShutdown = yield instance.startInstanceServer({
            envVars: [],
            performanceNow: () => {
                return performance.now();
            },
            getRandomValues: (buffer) => {
                const crypto = globalThis.crypto;
                if (!crypto)
                    throw new Error('randomness not available');
                // Browsers have this completely undocumented behavior (it's not even part of a spec)
                // that for some reason `getRandomValues` can't be called on arrayviews back by
                // `SharedArrayBuffer`s and they throw an exception if you try.
                if (buffer.buffer instanceof ArrayBuffer)
                    crypto.getRandomValues(buffer);
                else {
                    const tmpArray = new Uint8Array(buffer.length);
                    crypto.getRandomValues(tmpArray);
                    buffer.set(tmpArray);
                }
            },
        }, messagePort);
        return whenShutdown;
    });
}
