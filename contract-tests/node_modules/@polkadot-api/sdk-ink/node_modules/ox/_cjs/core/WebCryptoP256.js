"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createKeyPair = createKeyPair;
exports.createKeyPairECDH = createKeyPairECDH;
exports.getSharedSecret = getSharedSecret;
exports.sign = sign;
exports.verify = verify;
const p256_1 = require("@noble/curves/p256");
const Bytes = require("./Bytes.js");
const Hex = require("./Hex.js");
const PublicKey = require("./PublicKey.js");
async function createKeyPair(options = {}) {
    const { extractable = false } = options;
    const keypair = await globalThis.crypto.subtle.generateKey({
        name: 'ECDSA',
        namedCurve: 'P-256',
    }, extractable, ['sign', 'verify']);
    const publicKey_raw = await globalThis.crypto.subtle.exportKey('raw', keypair.publicKey);
    const publicKey = PublicKey.from(new Uint8Array(publicKey_raw));
    return {
        privateKey: keypair.privateKey,
        publicKey,
    };
}
async function createKeyPairECDH(options = {}) {
    const { extractable = false } = options;
    const keypair = await globalThis.crypto.subtle.generateKey({
        name: 'ECDH',
        namedCurve: 'P-256',
    }, extractable, ['deriveKey', 'deriveBits']);
    const publicKey_raw = await globalThis.crypto.subtle.exportKey('raw', keypair.publicKey);
    const publicKey = PublicKey.from(new Uint8Array(publicKey_raw));
    return {
        privateKey: keypair.privateKey,
        publicKey,
    };
}
async function getSharedSecret(options) {
    const { as = 'Hex', privateKey, publicKey } = options;
    if (privateKey.algorithm.name === 'ECDSA') {
        throw new Error('privateKey is not compatible with ECDH. please use `createKeyPairECDH` to create an ECDH key.');
    }
    const publicKeyCrypto = await globalThis.crypto.subtle.importKey('raw', PublicKey.toBytes(publicKey), { name: 'ECDH', namedCurve: 'P-256' }, false, []);
    const sharedSecretBuffer = await globalThis.crypto.subtle.deriveBits({
        name: 'ECDH',
        public: publicKeyCrypto,
    }, privateKey, 256);
    const sharedSecret = new Uint8Array(sharedSecretBuffer);
    if (as === 'Hex')
        return Hex.fromBytes(sharedSecret);
    return sharedSecret;
}
async function sign(options) {
    const { payload, privateKey } = options;
    const signature = await globalThis.crypto.subtle.sign({
        name: 'ECDSA',
        hash: 'SHA-256',
    }, privateKey, Bytes.from(payload));
    const signature_bytes = Bytes.fromArray(new Uint8Array(signature));
    const r = Bytes.toBigInt(Bytes.slice(signature_bytes, 0, 32));
    let s = Bytes.toBigInt(Bytes.slice(signature_bytes, 32, 64));
    if (s > p256_1.p256.CURVE.n / 2n)
        s = p256_1.p256.CURVE.n - s;
    return { r, s };
}
async function verify(options) {
    const { payload, signature } = options;
    const publicKey = await globalThis.crypto.subtle.importKey('raw', PublicKey.toBytes(options.publicKey), { name: 'ECDSA', namedCurve: 'P-256' }, true, ['verify']);
    return await globalThis.crypto.subtle.verify({
        name: 'ECDSA',
        hash: 'SHA-256',
    }, publicKey, Bytes.concat(Bytes.fromNumber(signature.r), Bytes.fromNumber(signature.s)), Bytes.from(payload));
}
//# sourceMappingURL=WebCryptoP256.js.map