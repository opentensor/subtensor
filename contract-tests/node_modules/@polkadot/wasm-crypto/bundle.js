import { bridge, initBridge } from './init.js';
export { packageInfo } from './packageInfo.js';
export { bridge };
/**
 * @internal
 * @description
 * This create an extenal interface function from the signature, all the while checking
 * the actual bridge wasm interface to ensure it has been initialized.
 *
 * This means that we can call it
 *
 *   withWasm(wasm: WasmCryptoInstance, a: number, b: string) => Uint8Array
 *
 * and in this case it will create an interface function with the signarure
 *
 *   (a: number, b: string) => Uint8Array
 */
function withWasm(fn) {
    return (...params) => {
        if (!bridge.wasm) {
            throw new Error('The WASM interface has not been initialized. Ensure that you wait for the initialization Promise with waitReady() from @polkadot/wasm-crypto (or cryptoWaitReady() from @polkadot/util-crypto) before attempting to use WASM-only interfaces.');
        }
        return fn(bridge.wasm, ...params);
    };
}
export const bip39Generate = /*#__PURE__*/ withWasm((wasm, words) => {
    wasm.ext_bip39_generate(8, words);
    return bridge.resultString();
});
export const bip39ToEntropy = /*#__PURE__*/ withWasm((wasm, phrase) => {
    wasm.ext_bip39_to_entropy(8, ...bridge.allocString(phrase));
    return bridge.resultU8a();
});
export const bip39ToMiniSecret = /*#__PURE__*/ withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_mini_secret(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
    return bridge.resultU8a();
});
export const bip39ToSeed = /*#__PURE__*/ withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_seed(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
    return bridge.resultU8a();
});
export const bip39Validate = /*#__PURE__*/ withWasm((wasm, phrase) => {
    const ret = wasm.ext_bip39_validate(...bridge.allocString(phrase));
    return ret !== 0;
});
export const ed25519KeypairFromSeed = /*#__PURE__*/ withWasm((wasm, seed) => {
    wasm.ext_ed_from_seed(8, ...bridge.allocU8a(seed));
    return bridge.resultU8a();
});
export const ed25519Sign = /*#__PURE__*/ withWasm((wasm, pubkey, seckey, message) => {
    wasm.ext_ed_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(seckey), ...bridge.allocU8a(message));
    return bridge.resultU8a();
});
export const ed25519Verify = /*#__PURE__*/ withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_ed_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
    return ret !== 0;
});
export const secp256k1FromSeed = /*#__PURE__*/ withWasm((wasm, seckey) => {
    wasm.ext_secp_from_seed(8, ...bridge.allocU8a(seckey));
    return bridge.resultU8a();
});
export const secp256k1Compress = /*#__PURE__*/ withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_compress(8, ...bridge.allocU8a(pubkey));
    return bridge.resultU8a();
});
export const secp256k1Expand = /*#__PURE__*/ withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_expand(8, ...bridge.allocU8a(pubkey));
    return bridge.resultU8a();
});
export const secp256k1Recover = /*#__PURE__*/ withWasm((wasm, msgHash, sig, recovery) => {
    wasm.ext_secp_recover(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(sig), recovery);
    return bridge.resultU8a();
});
export const secp256k1Sign = /*#__PURE__*/ withWasm((wasm, msgHash, seckey) => {
    wasm.ext_secp_sign(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(seckey));
    return bridge.resultU8a();
});
export const sr25519DeriveKeypairHard = /*#__PURE__*/ withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_hard(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
});
export const sr25519DeriveKeypairSoft = /*#__PURE__*/ withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_soft(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
});
export const sr25519DerivePublicSoft = /*#__PURE__*/ withWasm((wasm, pubkey, cc) => {
    wasm.ext_sr_derive_public_soft(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(cc));
    return bridge.resultU8a();
});
export const sr25519KeypairFromSeed = /*#__PURE__*/ withWasm((wasm, seed) => {
    wasm.ext_sr_from_seed(8, ...bridge.allocU8a(seed));
    return bridge.resultU8a();
});
export const sr25519Sign = /*#__PURE__*/ withWasm((wasm, pubkey, secret, message) => {
    wasm.ext_sr_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret), ...bridge.allocU8a(message));
    return bridge.resultU8a();
});
export const sr25519Verify = /*#__PURE__*/ withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_sr_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
    return ret !== 0;
});
export const sr25519Agree = /*#__PURE__*/ withWasm((wasm, pubkey, secret) => {
    wasm.ext_sr_agree(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret));
    return bridge.resultU8a();
});
export const vrfSign = /*#__PURE__*/ withWasm((wasm, secret, context, message, extra) => {
    wasm.ext_vrf_sign(8, ...bridge.allocU8a(secret), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra));
    return bridge.resultU8a();
});
export const vrfVerify = /*#__PURE__*/ withWasm((wasm, pubkey, context, message, extra, outAndProof) => {
    const ret = wasm.ext_vrf_verify(...bridge.allocU8a(pubkey), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra), ...bridge.allocU8a(outAndProof));
    return ret !== 0;
});
export const blake2b = /*#__PURE__*/ withWasm((wasm, data, key, size) => {
    wasm.ext_blake2b(8, ...bridge.allocU8a(data), ...bridge.allocU8a(key), size);
    return bridge.resultU8a();
});
export const hmacSha256 = /*#__PURE__*/ withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha256(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const hmacSha512 = /*#__PURE__*/ withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha512(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const keccak256 = /*#__PURE__*/ withWasm((wasm, data) => {
    wasm.ext_keccak256(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const keccak512 = /*#__PURE__*/ withWasm((wasm, data) => {
    wasm.ext_keccak512(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const pbkdf2 = /*#__PURE__*/ withWasm((wasm, data, salt, rounds) => {
    wasm.ext_pbkdf2(8, ...bridge.allocU8a(data), ...bridge.allocU8a(salt), rounds);
    return bridge.resultU8a();
});
export const scrypt = /*#__PURE__*/ withWasm((wasm, password, salt, log2n, r, p) => {
    wasm.ext_scrypt(8, ...bridge.allocU8a(password), ...bridge.allocU8a(salt), log2n, r, p);
    return bridge.resultU8a();
});
export const sha256 = /*#__PURE__*/ withWasm((wasm, data) => {
    wasm.ext_sha256(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const sha512 = /*#__PURE__*/ withWasm((wasm, data) => {
    wasm.ext_sha512(8, ...bridge.allocU8a(data));
    return bridge.resultU8a();
});
export const twox = /*#__PURE__*/ withWasm((wasm, data, rounds) => {
    wasm.ext_twox(8, ...bridge.allocU8a(data), rounds);
    return bridge.resultU8a();
});
export function isReady() {
    return !!bridge.wasm;
}
export async function waitReady() {
    try {
        const wasm = await initBridge();
        return !!wasm;
    }
    catch {
        return false;
    }
}
