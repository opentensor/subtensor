"use strict";
// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
Object.defineProperty(exports, "__esModule", { value: true });
exports.start = exports.JsonRpcDisabledError = exports.QueueFullError = exports.CrashError = exports.AlreadyDestroyedError = exports.AddChainError = void 0;
const no_auto_bytecode_nodejs_js_1 = require("./no-auto-bytecode-nodejs.js");
const bytecode_nodejs_js_1 = require("./bytecode-nodejs.js");
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
 * @param options Configuration of the client. Defaults to `{}`.
 */
function start(options) {
    options = options || {};
    return (0, no_auto_bytecode_nodejs_js_1.startWithBytecode)(Object.assign({ bytecode: (0, bytecode_nodejs_js_1.compileBytecode)() }, options));
}
exports.start = start;
