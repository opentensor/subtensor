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
import { QueueFullError, AlreadyDestroyedError, AddChainError, JsonRpcDisabledError, CrashError } from '../public-types.js';
import * as instance from './local-instance.js';
import * as remote from './remote-instance.js';
// This function is similar to the `start` function found in `index.ts`, except with an extra
// parameter containing the platform-specific bindings.
// Contrary to the one within `index.js`, this function is not supposed to be directly used.
export function start(options, wasmModule, platformBindings) {
    const logCallback = options.logCallback || ((level, target, message) => {
        // The first parameter of the methods of `console` has some printf-like substitution
        // capabilities. We don't really need to use this, but not using it means that the logs might
        // not get printed correctly if they contain `%`.
        if (level <= 1) {
            console.error("[%s] %s", target, message);
        }
        else if (level == 2) {
            console.warn("[%s] %s", target, message);
        }
        else if (level == 3) {
            console.info("[%s] %s", target, message);
        }
        else if (level == 4) {
            console.debug("[%s] %s", target, message);
        }
        else {
            console.trace("[%s] %s", target, message);
        }
    });
    if (!(wasmModule instanceof Promise)) {
        wasmModule = Promise.resolve(wasmModule);
    }
    // Extract (to make sure the value doesn't change) and sanitize `cpuRateLimit`.
    let cpuRateLimit = options.cpuRateLimit || 1.0;
    if (isNaN(cpuRateLimit))
        cpuRateLimit = 1.0;
    if (cpuRateLimit > 1.0)
        cpuRateLimit = 1.0;
    if (cpuRateLimit < 0.0)
        cpuRateLimit = 0.0;
    // This object holds the state of everything.
    const state = {
        instance: { status: "not-created" },
        chainIds: new WeakMap(),
        connections: new Map(),
        addChainIdAllocations: [],
        addChainResults: new Map(),
        onExecutorShutdownOrWasmPanic: () => { },
        chains: new Map(),
    };
    // Callback called during the execution of the instance.
    const eventCallback = (event) => {
        switch (event.ty) {
            case "wasm-panic": {
                console.error("Smoldot has panicked" +
                    (event.currentTask ? (" while executing task `" + event.currentTask + "`") : "") +
                    ". This is a bug in smoldot. Please open an issue at " +
                    "https://github.com/smol-dot/smoldot/issues with the following message:\n" +
                    event.message);
                state.instance = {
                    status: "destroyed",
                    error: new CrashError(event.message),
                };
                state.connections.forEach((connec) => connec.reset());
                state.connections.clear();
                for (const addChainResult of state.addChainIdAllocations) {
                    addChainResult({ success: false, error: "Smoldot has crashed" });
                }
                state.addChainIdAllocations = [];
                state.addChainResults.forEach((addChainResult) => {
                    addChainResult({ success: false, error: "Smoldot has crashed" });
                });
                state.addChainResults.clear();
                for (const chain of Array.from(state.chains.values())) {
                    for (const callback of chain.jsonRpcResponsesPromises) {
                        callback();
                    }
                    chain.jsonRpcResponsesPromises = [];
                }
                state.chains.clear();
                const cb = state.onExecutorShutdownOrWasmPanic;
                state.onExecutorShutdownOrWasmPanic = () => { };
                cb();
                break;
            }
            case "executor-shutdown": {
                const cb = state.onExecutorShutdownOrWasmPanic;
                state.onExecutorShutdownOrWasmPanic = () => { };
                cb();
                break;
            }
            case "log": {
                logCallback(event.level, event.target, event.message);
                break;
            }
            case "add-chain-id-allocated": {
                const callback = state.addChainIdAllocations.shift();
                state.addChainResults.set(event.chainId, callback);
                break;
            }
            case "add-chain-result": {
                (state.addChainResults.get(event.chainId))(event);
                state.addChainResults.delete(event.chainId);
                break;
            }
            case "json-rpc-responses-non-empty": {
                // Notify every single promise found in `jsonRpcResponsesPromises`.
                const callbacks = state.chains.get(event.chainId).jsonRpcResponsesPromises;
                while (callbacks.length !== 0) {
                    (callbacks.shift())();
                }
                break;
            }
            case "new-connection": {
                const connectionId = event.connectionId;
                state.connections.set(connectionId, platformBindings.connect({
                    address: event.address,
                    onConnectionReset(message) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.connections.delete(connectionId);
                        state.instance.instance.connectionReset(connectionId, message);
                    },
                    onMessage(message, streamId) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.instance.instance.streamMessage(connectionId, message, streamId);
                    },
                    onStreamOpened(streamId, direction) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.instance.instance.streamOpened(connectionId, streamId, direction);
                    },
                    onMultistreamHandshakeInfo(info) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.instance.instance.connectionMultiStreamSetHandshakeInfo(connectionId, info);
                    },
                    onWritableBytes(numExtra, streamId) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.instance.instance.streamWritableBytes(connectionId, numExtra, streamId);
                    },
                    onStreamReset(streamId, message) {
                        if (state.instance.status !== "ready")
                            throw new Error();
                        state.instance.instance.streamReset(connectionId, streamId, message);
                    },
                }));
                break;
            }
            case "connection-reset": {
                const connection = state.connections.get(event.connectionId);
                connection.reset();
                state.connections.delete(event.connectionId);
                break;
            }
            case "connection-stream-open": {
                const connection = state.connections.get(event.connectionId);
                connection.openOutSubstream();
                break;
            }
            case "connection-stream-reset": {
                const connection = state.connections.get(event.connectionId);
                connection.reset(event.streamId);
                break;
            }
            case "stream-send": {
                const connection = state.connections.get(event.connectionId);
                connection.send(event.data, event.streamId);
                break;
            }
            case "stream-send-close": {
                const connection = state.connections.get(event.connectionId);
                connection.closeSend(event.streamId);
                break;
            }
        }
    };
    const portToWorker = options.portToWorker;
    if (!portToWorker) {
        // Start a local instance.
        state.instance = {
            status: "not-ready",
            whenReady: wasmModule
                .then((wasmModule) => {
                return instance.startLocalInstance({
                    forbidTcp: options.forbidTcp || false,
                    forbidWs: options.forbidWs || false,
                    forbidNonLocalWs: options.forbidNonLocalWs || false,
                    forbidWss: options.forbidWss || false,
                    forbidWebRtc: options.forbidWebRtc || false,
                    maxLogLevel: options.maxLogLevel || 3,
                    cpuRateLimit,
                    envVars: [],
                    performanceNow: platformBindings.performanceNow,
                    getRandomValues: platformBindings.getRandomValues,
                }, wasmModule.wasm, eventCallback);
            })
                .then((instance) => {
                // The Wasm instance might have been crashed before this callback is called.
                if (state.instance.status === "destroyed")
                    return;
                state.instance = {
                    status: "ready",
                    instance,
                };
            })
        };
    }
    else {
        // Connect to the remote instance.
        state.instance = {
            status: "not-ready",
            whenReady: remote.connectToInstanceServer({
                wasmModule: wasmModule.then((b) => b.wasm),
                forbidTcp: options.forbidTcp || false,
                forbidWs: options.forbidWs || false,
                forbidNonLocalWs: options.forbidNonLocalWs || false,
                forbidWss: options.forbidWss || false,
                forbidWebRtc: options.forbidWebRtc || false,
                maxLogLevel: options.maxLogLevel || 3,
                cpuRateLimit,
                portToServer: portToWorker,
                eventCallback
            }).then((instance) => {
                // The Wasm instance might have been crashed before this callback is called.
                if (state.instance.status === "destroyed")
                    return;
                state.instance = {
                    status: "ready",
                    instance,
                };
            })
        };
    }
    return {
        addChain: (options) => __awaiter(this, void 0, void 0, function* () {
            if (state.instance.status === "not-ready")
                yield state.instance.whenReady;
            if (state.instance.status === "destroyed")
                throw state.instance.error;
            if (state.instance.status === "not-created" || state.instance.status === "not-ready")
                throw new Error(); // Internal error, not supposed to ever happen.
            // Passing a JSON object for the chain spec is an easy mistake, so we provide a more
            // readable error.
            if (!(typeof options.chainSpec === 'string'))
                throw new Error("Chain specification must be a string");
            let potentialRelayChainsIds = [];
            if (!!options.potentialRelayChains) {
                for (const chain of options.potentialRelayChains) {
                    // The content of `options.potentialRelayChains` are supposed to be chains earlier
                    // returned by `addChain`.
                    const id = state.chainIds.get(chain);
                    if (id === undefined) // It is possible for `id` to be missing if it has earlier been removed.
                        continue;
                    potentialRelayChainsIds.push(id);
                }
            }
            // Sanitize `jsonRpcMaxPendingRequests`.
            let jsonRpcMaxPendingRequests = options.jsonRpcMaxPendingRequests === undefined ? Infinity : options.jsonRpcMaxPendingRequests;
            jsonRpcMaxPendingRequests = Math.floor(jsonRpcMaxPendingRequests);
            if (jsonRpcMaxPendingRequests <= 0 || isNaN(jsonRpcMaxPendingRequests)) {
                throw new AddChainError("Invalid value for `jsonRpcMaxPendingRequests`");
            }
            if (jsonRpcMaxPendingRequests > 0xffffffff) {
                jsonRpcMaxPendingRequests = 0xffffffff;
            }
            // Sanitize `jsonRpcMaxSubscriptions`.
            let jsonRpcMaxSubscriptions = options.jsonRpcMaxSubscriptions === undefined ? Infinity : options.jsonRpcMaxSubscriptions;
            jsonRpcMaxSubscriptions = Math.floor(jsonRpcMaxSubscriptions);
            if (jsonRpcMaxSubscriptions < 0 || isNaN(jsonRpcMaxSubscriptions)) {
                throw new AddChainError("Invalid value for `jsonRpcMaxSubscriptions`");
            }
            if (jsonRpcMaxSubscriptions > 0xffffffff) {
                jsonRpcMaxSubscriptions = 0xffffffff;
            }
            // Sanitize `databaseContent`.
            if (options.databaseContent !== undefined && typeof options.databaseContent !== 'string')
                throw new AddChainError("`databaseContent` is not a string");
            const promise = new Promise((resolve) => state.addChainIdAllocations.push(resolve));
            state.instance.instance.addChain(options.chainSpec, options.databaseContent || "", potentialRelayChainsIds, !!options.disableJsonRpc, jsonRpcMaxPendingRequests, jsonRpcMaxSubscriptions);
            const outcome = yield promise;
            if (!outcome.success)
                throw new AddChainError(outcome.error);
            const chainId = outcome.chainId;
            state.chains.set(chainId, {
                jsonRpcResponsesPromises: new Array()
            });
            const newChain = {
                sendJsonRpc: (request) => {
                    if (state.instance.status === "destroyed")
                        throw state.instance.error;
                    if (state.instance.status !== "ready")
                        throw new Error(); // Internal error. Never supposed to happen.
                    if (!state.chains.has(chainId))
                        throw new AlreadyDestroyedError();
                    if (options.disableJsonRpc)
                        throw new JsonRpcDisabledError();
                    const retVal = state.instance.instance.request(request, chainId);
                    switch (retVal) {
                        case 0: break;
                        case 1: throw new QueueFullError();
                        default: throw new Error("Internal error: unknown json_rpc_send error code: " + retVal);
                    }
                },
                nextJsonRpcResponse: () => __awaiter(this, void 0, void 0, function* () {
                    while (true) {
                        if (!state.chains.has(chainId))
                            throw new AlreadyDestroyedError();
                        if (options.disableJsonRpc)
                            return Promise.reject(new JsonRpcDisabledError());
                        if (state.instance.status === "destroyed")
                            throw state.instance.error;
                        if (state.instance.status !== "ready")
                            throw new Error(); // Internal error. Never supposed to happen.
                        // Try to pop a message from the queue.
                        const message = state.instance.instance.peekJsonRpcResponse(chainId);
                        if (message)
                            return message;
                        // If no message is available, wait for one to be.
                        yield new Promise((resolve) => {
                            state.chains.get(chainId).jsonRpcResponsesPromises.push(resolve);
                        });
                    }
                }),
                remove: () => {
                    if (state.instance.status === "destroyed")
                        throw state.instance.error;
                    if (state.instance.status !== "ready")
                        throw new Error(); // Internal error. Never supposed to happen.
                    if (!state.chains.has(chainId))
                        throw new AlreadyDestroyedError();
                    console.assert(state.chainIds.has(newChain));
                    state.chainIds.delete(newChain);
                    for (const callback of state.chains.get(chainId).jsonRpcResponsesPromises) {
                        callback();
                    }
                    state.chains.delete(chainId);
                    state.instance.instance.removeChain(chainId);
                },
            };
            state.chainIds.set(newChain, chainId);
            return newChain;
        }),
        terminate: () => __awaiter(this, void 0, void 0, function* () {
            if (state.instance.status === "not-ready")
                yield state.instance.whenReady;
            if (state.instance.status === "destroyed")
                throw state.instance.error;
            if (state.instance.status !== "ready")
                throw new Error(); // Internal error. Never supposed to happen.
            state.instance.instance.shutdownExecutor();
            // Wait for the `executor-shutdown` event to be generated.
            yield new Promise((resolve) => state.onExecutorShutdownOrWasmPanic = resolve);
            // In case the instance crashes while we were waiting, we don't want to overwrite
            // the error.
            if (state.instance.status === "ready")
                state.instance = { status: "destroyed", error: new AlreadyDestroyedError() };
            state.connections.forEach((connec) => connec.reset());
            state.connections.clear();
            for (const addChainResult of state.addChainIdAllocations) {
                addChainResult({ success: false, error: "Client.terminate() has been called" });
            }
            state.addChainIdAllocations = [];
            state.addChainResults.forEach((addChainResult) => {
                addChainResult({ success: false, error: "Client.terminate() has been called" });
            });
            state.addChainResults.clear();
            for (const chain of Array.from(state.chains.values())) {
                for (const callback of chain.jsonRpcResponsesPromises) {
                    callback();
                }
                chain.jsonRpcResponsesPromises = [];
            }
            state.chains.clear();
        })
    };
}
