"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519DeriveHard = void 0;
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const derive_js_1 = require("./derive.js");
exports.sr25519DeriveHard = (0, derive_js_1.createDeriveFn)(wasm_crypto_1.sr25519DeriveKeypairHard);
