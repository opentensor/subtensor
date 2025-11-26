"use strict";
// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
Object.defineProperty(exports, "__esModule", { value: true });
exports.startWithBytecode = exports.JsonRpcDisabledError = exports.QueueFullError = exports.CrashError = exports.AlreadyDestroyedError = exports.AddChainError = void 0;
const client_js_1 = require("./internals/client.js");
const ws_1 = require("ws");
const node_perf_hooks_1 = require("node:perf_hooks");
const node_net_1 = require("node:net");
const node_crypto_1 = require("node:crypto");
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
            return node_perf_hooks_1.performance.now();
        },
        getRandomValues: (buffer) => {
            if (buffer.length >= 1024 * 1024)
                throw new Error('getRandomValues buffer too large');
            (0, node_crypto_1.randomFillSync)(buffer);
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
        const socket = new ws_1.WebSocket(config.address.url);
        socket.binaryType = 'arraybuffer';
        const bufferedAmountCheck = { quenedUnreportedBytes: 0, nextTimeout: 10 };
        const checkBufferedAmount = () => {
            if (socket.readyState != 1)
                return;
            // Note that we might expect `bufferedAmount` to always be <= the sum of the lengths
            // of all the data that has been sent, but that seems to not be the case. It is
            // unclear whether this is intended or a bug, but is is likely that `bufferedAmount`
            // also includes WebSocket headers. For this reason, we use `bufferedAmount` as a hint
            // rather than a correct value.
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
            socket.onopen = () => { };
            socket.onclose = () => { };
            socket.onmessage = () => { };
            socket.onerror = () => { };
        };
        socket.onerror = (event) => {
            config.onConnectionReset(event.message);
            socket.onopen = () => { };
            socket.onclose = () => { };
            socket.onmessage = () => { };
            socket.onerror = () => { };
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
                if (bufferedAmountCheck.quenedUnreportedBytes == 0) {
                    bufferedAmountCheck.nextTimeout = 10;
                    setTimeout(checkBufferedAmount, 10);
                }
                for (const buffer of data) {
                    socket.send(buffer);
                    bufferedAmountCheck.quenedUnreportedBytes += buffer.length;
                }
            },
            closeSend: () => { throw new Error('Wrong connection type'); },
            openOutSubstream: () => { throw new Error('Wrong connection type'); }
        };
    }
    else if (config.address.ty === "tcp") {
        const socket = (0, node_net_1.createConnection)({
            host: config.address.hostname,
            port: config.address.port,
        });
        // Number of bytes queued using `socket.write` and where `write` has returned false.
        const drainingBytes = { num: 0 };
        socket.setNoDelay();
        socket.on('connect', () => {
            if (socket.destroyed)
                return;
            config.onWritableBytes(socket.writableHighWaterMark);
        });
        socket.on('close', (hasError) => {
            if (socket.destroyed)
                return;
            // NodeJS doesn't provide a reason why the closing happened, but only
            // whether it was caused by an error.
            const message = hasError ? "Error" : "Closed gracefully";
            config.onConnectionReset(message);
        });
        socket.on('error', () => { });
        socket.on('data', (message) => {
            if (socket.destroyed)
                return;
            config.onMessage(new Uint8Array(message.buffer));
        });
        socket.on('drain', () => {
            // The bytes queued using `socket.write` and where `write` has returned false have now
            // been sent. Notify the API that it can write more data.
            if (socket.destroyed)
                return;
            const val = drainingBytes.num;
            drainingBytes.num = 0;
            config.onWritableBytes(val);
        });
        return {
            reset: () => {
                socket.destroy();
            },
            send: (data) => {
                for (const buffer of data) {
                    const bufferLen = buffer.length;
                    const allWritten = socket.write(buffer);
                    if (allWritten) {
                        setImmediate(() => {
                            if (!socket.writable)
                                return;
                            config.onWritableBytes(bufferLen);
                        });
                    }
                    else {
                        drainingBytes.num += bufferLen;
                    }
                }
            },
            closeSend: () => {
                socket.end();
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
