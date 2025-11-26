"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseAsn1Signature = parseAsn1Signature;
exports.parseCredentialPublicKey = parseCredentialPublicKey;
const p256_1 = require("@noble/curves/p256");
const Hex = require("../Hex.js");
const PublicKey = require("../PublicKey.js");
const WebAuthnP256_js_1 = require("../WebAuthnP256.js");
function parseAsn1Signature(bytes) {
    const r_start = bytes[4] === 0 ? 5 : 4;
    const r_end = r_start + 32;
    const s_start = bytes[r_end + 2] === 0 ? r_end + 3 : r_end + 2;
    const r = BigInt(Hex.fromBytes(bytes.slice(r_start, r_end)));
    const s = BigInt(Hex.fromBytes(bytes.slice(s_start)));
    return {
        r,
        s: s > p256_1.p256.CURVE.n / 2n ? p256_1.p256.CURVE.n - s : s,
    };
}
async function parseCredentialPublicKey(response) {
    try {
        const publicKeyBuffer = response.getPublicKey();
        if (!publicKeyBuffer)
            throw new WebAuthnP256_js_1.CredentialCreationFailedError();
        const publicKeyBytes = new Uint8Array(publicKeyBuffer);
        const cryptoKey = await crypto.subtle.importKey('spki', new Uint8Array(publicKeyBytes), {
            name: 'ECDSA',
            namedCurve: 'P-256',
            hash: 'SHA-256',
        }, true, ['verify']);
        const publicKey = new Uint8Array(await crypto.subtle.exportKey('raw', cryptoKey));
        return PublicKey.from(publicKey);
    }
    catch (error) {
        if (error.message !== 'Permission denied to access object')
            throw error;
        const data = new Uint8Array(response.attestationObject);
        const coordinateLength = 0x20;
        const cborPrefix = 0x58;
        const findStart = (key) => {
            const coordinate = new Uint8Array([key, cborPrefix, coordinateLength]);
            for (let i = 0; i < data.length - coordinate.length; i++)
                if (coordinate.every((byte, j) => data[i + j] === byte))
                    return i + coordinate.length;
            throw new WebAuthnP256_js_1.CredentialCreationFailedError();
        };
        const xStart = findStart(0x21);
        const yStart = findStart(0x22);
        return PublicKey.from(new Uint8Array([
            0x04,
            ...data.slice(xStart, xStart + coordinateLength),
            ...data.slice(yStart, yStart + coordinateLength),
        ]));
    }
}
//# sourceMappingURL=webauthn.js.map