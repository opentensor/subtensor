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
// Contains an implementation of `Instance` that is remote.
//
// In terms of implementation, the logic is pretty straight forward, with two exceptions:
//
// - Connections are tracked on both sides in order to handle situations where one side has
//   reset a connection or stream but the other is sending messages about this connection/stream.
//
// - JSON-RPC requests aren't sent back lazily one at a time. Instead, the client indicates that it
//   is ready to accept more JSON-RPC responses, after which the server can send responses at any
//   time and the client queues them locally.
import * as instance from './local-instance.js';
// Implementation note: it is unclear even in the official specification
// (https://html.spec.whatwg.org/multipage/web-messaging.html) whether both sides of a
// `MessagePort` should be closed, or if one is enough.
//
// It has been noticed that doing `port.postMessage(...); port.close();` doesn't deliver the
// message on Firefox (but it does on Chrome). The code below takes note of this, and only closes
// a port upon *receiving* the last possible message. It therefore assumes that closing only one
// side is enough. It is unclear whether this causes any memory leak.
export function connectToInstanceServer(config) {
    return __awaiter(this, void 0, void 0, function* () {
        // Send the wasm module and configuration to the server.
        // Note that we await the `wasmModule` `Promise` here.
        // If instead we used `wasmModule.then(...)`, the user would be able to start using the
        // returned instance before the module has been sent to the server.
        // In order to simplify the implementation, we create new ports and send one of them to
        // the server. This is necessary so that the server can pause receiving messages while the
        // instance is being initialized.
        const { port1: portToServer, port2: serverToClient } = new MessageChannel();
        const initialPort = config.portToServer;
        const initialMessage = {
            wasmModule: yield config.wasmModule,
            serverToClient,
            maxLogLevel: config.maxLogLevel,
            cpuRateLimit: config.cpuRateLimit,
            forbidWs: config.forbidWs,
            forbidWss: config.forbidWss,
            forbidNonLocalWs: config.forbidNonLocalWs,
            forbidTcp: config.forbidTcp,
            forbidWebRtc: config.forbidWebRtc
        };
        initialPort.postMessage(initialMessage, [serverToClient]);
        // Note that closing `initialPort` here will lead to the message not being delivered on Firefox
        // for some reason. It is therefore closed only on shutdown.
        const state = {
            jsonRpcResponses: new Map(),
            connections: new Map(),
        };
        portToServer.onmessage = (messageEvent) => {
            const message = messageEvent.data;
            // Update some local state.
            switch (message.ty) {
                case "wasm-panic":
                case "executor-shutdown": {
                    portToServer.close();
                    initialPort.close();
                    break;
                }
                case "add-chain-result": {
                    if (message.success) {
                        state.jsonRpcResponses.set(message.chainId, new Array);
                        const moreAccepted = { ty: "accept-more-json-rpc-answers", chainId: message.chainId };
                        for (let i = 0; i < 10; ++i)
                            portToServer.postMessage(moreAccepted);
                    }
                    break;
                }
                case "new-connection": {
                    state.connections.set(message.connectionId, new Set());
                    break;
                }
                case "connection-reset": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    state.connections.delete(message.connectionId);
                    break;
                }
                case "connection-stream-open": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    break;
                }
                case "connection-stream-reset": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (!state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    break;
                }
                case "stream-send": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (message.streamId && !state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    break;
                }
                case "stream-send-close": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (message.streamId && !state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    break;
                }
                case "json-rpc-response": {
                    const queue = state.jsonRpcResponses.get(message.chainId);
                    // The chain might have been removed locally in the past.
                    if (queue) {
                        queue.push(message.response);
                        config.eventCallback({ ty: "json-rpc-responses-non-empty", chainId: message.chainId });
                    }
                    return;
                }
            }
            config.eventCallback(message);
        };
        return {
            addChain(chainSpec, databaseContent, potentialRelayChains, disableJsonRpc, jsonRpcMaxPendingRequests, jsonRpcMaxSubscriptions) {
                return __awaiter(this, void 0, void 0, function* () {
                    const msg = { ty: "add-chain", chainSpec, databaseContent, potentialRelayChains, disableJsonRpc, jsonRpcMaxPendingRequests, jsonRpcMaxSubscriptions };
                    portToServer.postMessage(msg);
                });
            },
            removeChain(chainId) {
                state.jsonRpcResponses.delete(chainId);
                const msg = { ty: "remove-chain", chainId };
                portToServer.postMessage(msg);
            },
            request(request, chainId) {
                const msg = { ty: "request", chainId, request };
                portToServer.postMessage(msg);
                return 0; // TODO: wrong return value
            },
            peekJsonRpcResponse(chainId) {
                const item = state.jsonRpcResponses.get(chainId).shift();
                if (!item)
                    return null;
                const msg = { ty: "accept-more-json-rpc-answers", chainId };
                portToServer.postMessage(msg);
                return item;
            },
            shutdownExecutor() {
                const msg = { ty: "shutdown" };
                portToServer.postMessage(msg);
            },
            connectionReset(connectionId, message) {
                state.connections.delete(connectionId);
                const msg = { ty: "connection-reset", connectionId, message };
                portToServer.postMessage(msg);
            },
            connectionMultiStreamSetHandshakeInfo(connectionId, info) {
                const msg = { ty: "connection-multistream-set-info", connectionId, info };
                portToServer.postMessage(msg);
            },
            streamMessage(connectionId, message, streamId) {
                const msg = { ty: "stream-message", connectionId, message, streamId };
                portToServer.postMessage(msg);
            },
            streamOpened(connectionId, streamId, direction) {
                state.connections.get(connectionId).add(streamId);
                const msg = { ty: "stream-opened", connectionId, streamId, direction };
                portToServer.postMessage(msg);
            },
            streamWritableBytes(connectionId, numExtra, streamId) {
                const msg = { ty: "stream-writable-bytes", connectionId, numExtra, streamId };
                portToServer.postMessage(msg);
            },
            streamReset(connectionId, streamId, message) {
                state.connections.get(connectionId).delete(streamId);
                const msg = { ty: "stream-reset", connectionId, streamId, message };
                portToServer.postMessage(msg);
            },
        };
    });
}
/**
 * Returns a `Promise` that resolves when the instance shuts down. Since the function is also
 * an asynchronous function, the actual return type is `Promise<Promise<void>>`. That is, the
 * outer `Promise` yields once the instance starts, and the inner `Promise` yields once the
 * instance shuts down.
 */
export function startInstanceServer(config, initPortToClient) {
    return __awaiter(this, void 0, void 0, function* () {
        const { serverToClient: portToClient, wasmModule, maxLogLevel, cpuRateLimit, forbidTcp, forbidWs, forbidWss, forbidNonLocalWs, forbidWebRtc } = yield new Promise((resolve) => {
            initPortToClient.onmessage = (event) => resolve(event.data);
        });
        initPortToClient.close();
        const state = {
            instance: null,
            connections: new Map(),
            acceptedJsonRpcResponses: new Map(),
        };
        const eventsCallback = (event) => {
            switch (event.ty) {
                case "add-chain-result": {
                    if (event.success) {
                        state.acceptedJsonRpcResponses.set(event.chainId, 0);
                    }
                    break;
                }
                case "executor-shutdown":
                case "wasm-panic": {
                    if (state.onExecutorShutdownOrWasmPanic) {
                        const cb = state.onExecutorShutdownOrWasmPanic;
                        delete state.onExecutorShutdownOrWasmPanic;
                        cb();
                    }
                    break;
                }
                case "json-rpc-responses-non-empty": {
                    // Process this event asynchronously because we can't call into `instance`
                    // from within the events callback itself.
                    // TODO: do better than setTimeout?
                    setTimeout(() => {
                        const numAccepted = state.acceptedJsonRpcResponses.get(event.chainId);
                        if (numAccepted === undefined || numAccepted === 0)
                            return;
                        const response = state.instance.peekJsonRpcResponse(event.chainId);
                        if (response) {
                            state.acceptedJsonRpcResponses.set(event.chainId, numAccepted - 1);
                            const msg = { ty: "json-rpc-response", chainId: event.chainId, response };
                            portToClient.postMessage(msg);
                        }
                    }, 0);
                    return;
                }
                case "new-connection": {
                    state.connections.set(event.connectionId, new Set());
                    break;
                }
                case "connection-reset": {
                    state.connections.delete(event.connectionId);
                    break;
                }
                case "connection-stream-reset": {
                    state.connections.get(event.connectionId).delete(event.streamId);
                    break;
                }
            }
            const ev = event;
            portToClient.postMessage(ev);
        };
        // We create the `Promise` ahead of time in order to potentially catch potential `wasm-panic`
        // events as early as during initialization.
        const execFinishedPromise = new Promise((resolve) => state.onExecutorShutdownOrWasmPanic = resolve);
        state.instance = yield instance.startLocalInstance(Object.assign({ forbidTcp,
            forbidWs,
            forbidNonLocalWs,
            forbidWss,
            forbidWebRtc,
            cpuRateLimit,
            maxLogLevel }, config), wasmModule, eventsCallback);
        portToClient.onmessage = (messageEvent) => {
            const message = messageEvent.data;
            switch (message.ty) {
                case "add-chain": {
                    state.instance.addChain(message.chainSpec, message.databaseContent, message.potentialRelayChains, message.disableJsonRpc, message.jsonRpcMaxPendingRequests, message.jsonRpcMaxSubscriptions);
                    break;
                }
                case "remove-chain": {
                    state.acceptedJsonRpcResponses.delete(message.chainId);
                    state.instance.removeChain(message.chainId);
                    break;
                }
                case "request": {
                    state.instance.request(message.request, message.chainId); // TODO: return value unused
                    break;
                }
                case "accept-more-json-rpc-answers": {
                    const response = state.instance.peekJsonRpcResponse(message.chainId);
                    if (response) {
                        const msg = { ty: "json-rpc-response", chainId: message.chainId, response };
                        portToClient.postMessage(msg);
                    }
                    else {
                        const numAccepted = state.acceptedJsonRpcResponses.get(message.chainId);
                        state.acceptedJsonRpcResponses.set(message.chainId, numAccepted + 1);
                    }
                    break;
                }
                case "shutdown": {
                    state.instance.shutdownExecutor();
                    break;
                }
                case "connection-reset": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    state.instance.connectionReset(message.connectionId, message.message);
                    break;
                }
                case "connection-multistream-set-info": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    state.instance.connectionMultiStreamSetHandshakeInfo(message.connectionId, message.info);
                    break;
                }
                case "stream-message": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (message.streamId !== undefined && !state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    state.instance.streamMessage(message.connectionId, message.message, message.streamId);
                    break;
                }
                case "stream-opened": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    state.connections.get(message.connectionId).add(message.streamId);
                    state.instance.streamOpened(message.connectionId, message.streamId, message.direction);
                    break;
                }
                case "stream-writable-bytes": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (message.streamId !== undefined && !state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    state.instance.streamWritableBytes(message.connectionId, message.numExtra, message.streamId);
                    break;
                }
                case "stream-reset": {
                    // The connection might have been reset locally in the past.
                    if (!state.connections.has(message.connectionId))
                        return;
                    // The stream might have been reset locally in the past.
                    if (!state.connections.get(message.connectionId).has(message.streamId))
                        return;
                    state.connections.get(message.connectionId).delete(message.streamId);
                    state.instance.streamReset(message.connectionId, message.streamId, message.message);
                    break;
                }
            }
        };
        return execFinishedPromise;
    });
}
