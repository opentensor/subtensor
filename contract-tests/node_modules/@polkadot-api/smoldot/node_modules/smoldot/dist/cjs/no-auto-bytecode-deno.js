"use strict";
// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.startWithBytecode = exports.JsonRpcDisabledError = exports.QueueFullError = exports.CrashError = exports.AlreadyDestroyedError = exports.AddChainError = void 0;
const client_js_1 = require("./internals/client.js");
var public_types_js_1 = require("./public-types.js");
Object.defineProperty(exports, "AddChainError", { enumerable: true, get: function () { return public_types_js_1.AddChainError; } });
Object.defineProperty(exports, "AlreadyDestroyedError", { enumerable: true, get: function () { return public_types_js_1.AlreadyDestroyedError; } });
Object.defineProperty(exports, "CrashError", { enumerable: true, get: function () { return public_types_js_1.CrashError; } });
Object.defineProperty(exports, "QueueFullError", { enumerable: true, get: function () { return public_types_js_1.QueueFullError; } });
Object.defineProperty(exports, "JsonRpcDisabledError", { enumerable: true, get: function () { return public_types_js_1.JsonRpcDisabledError; } });
/**
 * Initializes a new client. This is a pre-requisite to connecting to a blockchain.
 *
 * Can never fail.
 *
 * @param options Configuration of the client.
 */
function startWithBytecode(options) {
    options.forbidWebRtc = true;
    return (0, client_js_1.start)(options || {}, options.bytecode, {
        performanceNow: () => {
            return performance.now();
        },
        getRandomValues: (buffer) => {
            const crypto = globalThis.crypto;
            if (!crypto)
                throw new Error('randomness not available');
            crypto.getRandomValues(buffer);
        },
        connect: (config) => {
            return connect(config);
        }
    });
}
exports.startWithBytecode = startWithBytecode;
/**
 * Tries to open a new connection using the given configuration.
 *
 * @see Connection
 * @throws {@link ConnectionError} If the multiaddress couldn't be parsed or contains an invalid protocol.
 */
function connect(config) {
    if (config.address.ty === "websocket") {
        const socket = new WebSocket(config.address.url);
        socket.binaryType = 'arraybuffer';
        const bufferedAmountCheck = { quenedUnreportedBytes: 0, nextTimeout: 10 };
        const checkBufferedAmount = () => {
            if (socket.readyState != 1)
                return;
            // Note that we might expect `bufferedAmount` to always be <= the sum of the lengths
            // of all the data that has been sent, but that might not be the case. For this
            // reason, we use `bufferedAmount` as a hint rather than a correct value.
            const bufferedAmount = socket.bufferedAmount;
            let wasSent = bufferedAmountCheck.quenedUnreportedBytes - bufferedAmount;
            if (wasSent < 0)
                wasSent = 0;
            bufferedAmountCheck.quenedUnreportedBytes -= wasSent;
            if (bufferedAmountCheck.quenedUnreportedBytes != 0) {
                setTimeout(checkBufferedAmount, bufferedAmountCheck.nextTimeout);
                bufferedAmountCheck.nextTimeout *= 2;
                if (bufferedAmountCheck.nextTimeout > 500)
                    bufferedAmountCheck.nextTimeout = 500;
            }
            // Note: it is important to call `onWritableBytes` at the very end, as it might
            // trigger a call to `send`.
            if (wasSent != 0)
                config.onWritableBytes(wasSent);
        };
        socket.onopen = () => {
            config.onWritableBytes(1024 * 1024);
        };
        socket.onclose = (event) => {
            const message = "Error code " + event.code + (!!event.reason ? (": " + event.reason) : "");
            config.onConnectionReset(message);
        };
        socket.onmessage = (msg) => {
            config.onMessage(new Uint8Array(msg.data));
        };
        return {
            reset: () => {
                // We can't set these fields to null because the TypeScript definitions don't
                // allow it, but we can set them to dummy values.
                socket.onopen = () => { };
                socket.onclose = () => { };
                socket.onmessage = () => { };
                socket.onerror = () => { };
                socket.close();
            },
            send: (data) => {
                // The WebSocket library that we use seems to spontaneously transition connections
                // to the "closed" state but not call the `onclosed` callback immediately. Calling
                // `send` on that object throws an exception. In order to avoid panicking smoldot,
                // we thus absorb any exception thrown here.
                // See also <https://github.com/paritytech/smoldot/issues/2937>.
                try {
                    if (bufferedAmountCheck.quenedUnreportedBytes == 0) {
                        bufferedAmountCheck.nextTimeout = 10;
                        setTimeout(checkBufferedAmount, 10);
                    }
                    for (const buffer of data) {
                        bufferedAmountCheck.quenedUnreportedBytes += buffer.length;
                    }
                    socket.send(new Blob(data));
                }
                catch (_error) { }
            },
            closeSend: () => { throw new Error('Wrong connection type'); },
            openOutSubstream: () => { throw new Error('Wrong connection type'); }
        };
    }
    else if (config.address.ty === "tcp") {
        const socket = {
            destroyed: false,
            inner: Deno.connect({
                hostname: config.address.hostname,
                port: config.address.port,
            }).catch((error) => {
                socket.destroyed = true;
                config.onConnectionReset(error.toString());
                return null;
            })
        };
        socket.inner = socket.inner.then((established) => {
            if (socket.destroyed)
                return established;
            established === null || established === void 0 ? void 0 : established.setNoDelay();
            config.onWritableBytes(1024 * 1024);
            // Spawns an asynchronous task that continuously reads from the socket.
            // Every time data is read, the task re-executes itself in order to continue reading.
            // The task ends automatically if an EOF or error is detected, which should also happen
            // if the user calls `close()`.
            const read = (readBuffer) => __awaiter(this, void 0, void 0, function* () {
                if (socket.destroyed || established === null)
                    return;
                let outcome = null;
                try {
                    outcome = yield established.read(readBuffer);
                }
                catch (error) {
                    // The type of `error` is unclear, but we assume that it implements `Error`
                    outcome = error.toString();
                }
                if (socket.destroyed)
                    return;
                if (typeof outcome !== 'number' || outcome === null) {
                    // The socket is reported closed, but `socket.destroyed` is still `false` (see
                    // check above). As such, we must inform the inner layers.
                    socket.destroyed = true;
                    config.onConnectionReset(outcome === null ? "EOF when reading socket" : outcome);
                    return;
                }
                console.assert(outcome !== 0); // `read` guarantees to return a non-zero value.
                config.onMessage(readBuffer.slice(0, outcome));
                return read(readBuffer);
            });
            read(new Uint8Array(32768));
            return established;
        });
        return {
            reset: () => {
                socket.destroyed = true;
                socket.inner.then((connec) => connec.close());
            },
            send: (data) => {
                let dataCopy = data.map((buf) => Uint8Array.from(buf)); // Deep copy of the data
                socket.inner = socket.inner.then((c) => __awaiter(this, void 0, void 0, function* () {
                    for (let buffer of dataCopy) {
                        while (buffer.length > 0) {
                            if (socket.destroyed || c === null)
                                return c;
                            let outcome;
                            try {
                                outcome = yield c.write(buffer);
                                config.onWritableBytes(buffer.length);
                            }
                            catch (error) {
                                // The type of `error` is unclear, but we assume that it
                                // implements `Error`
                                outcome = error.toString();
                            }
                            if (typeof outcome !== 'number') {
                                // The socket is reported closed, but `socket.destroyed` is still
                                // `false` (see check above). As such, we must inform the
                                // inner layers.
                                socket.destroyed = true;
                                config.onConnectionReset(outcome);
                                return c;
                            }
                            // Note that, contrary to `read`, it is possible for `outcome` to be 0.
                            // This happen if the write had to be interrupted, and the only thing
                            // we have to do is try writing again.
                            buffer = buffer.slice(outcome);
                        }
                    }
                    return c;
                }));
            },
            closeSend: () => {
                socket.inner = socket.inner.then((c) => __awaiter(this, void 0, void 0, function* () {
                    yield (c === null || c === void 0 ? void 0 : c.closeWrite());
                    return c;
                }));
            },
            openOutSubstream: () => { throw new Error('Wrong connection type'); }
        };
    }
    else {
        // Should never happen, as we tweak the options to refuse connection types that
        // we don't support.
        throw new Error();
    }
}
