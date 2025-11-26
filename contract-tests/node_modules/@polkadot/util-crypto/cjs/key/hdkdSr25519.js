"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keyHdkdSr25519 = keyHdkdSr25519;
const deriveHard_js_1 = require("../sr25519/deriveHard.js");
const deriveSoft_js_1 = require("../sr25519/deriveSoft.js");
function keyHdkdSr25519(keypair, { chainCode, isSoft }) {
    return isSoft
        ? (0, deriveSoft_js_1.sr25519DeriveSoft)(keypair, chainCode)
        : (0, deriveHard_js_1.sr25519DeriveHard)(keypair, chainCode);
}
