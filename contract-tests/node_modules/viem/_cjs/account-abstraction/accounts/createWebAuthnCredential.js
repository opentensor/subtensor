"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createWebAuthnCredential = createWebAuthnCredential;
const PublicKey = require("ox/PublicKey");
const WebAuthnP256 = require("ox/WebAuthnP256");
async function createWebAuthnCredential(parameters) {
    const credential = await WebAuthnP256.createCredential(parameters);
    return {
        id: credential.id,
        publicKey: PublicKey.toHex(credential.publicKey, { includePrefix: false }),
        raw: credential.raw,
    };
}
//# sourceMappingURL=createWebAuthnCredential.js.map