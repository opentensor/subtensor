"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@polkadot/x-bigint/shim");
const crypto_js_1 = require("./crypto.js");
(0, crypto_js_1.cryptoWaitReady)().catch(() => {
    // shouldn't happen, logged and caught inside cryptoWaitReady
});
