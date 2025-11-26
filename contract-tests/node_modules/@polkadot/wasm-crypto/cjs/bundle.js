"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.twox = exports.sha512 = exports.sha256 = exports.scrypt = exports.pbkdf2 = exports.keccak512 = exports.keccak256 = exports.hmacSha512 = exports.hmacSha256 = exports.blake2b = exports.vrfVerify = exports.vrfSign = exports.sr25519Agree = exports.sr25519Verify = exports.sr25519Sign = exports.sr25519KeypairFromSeed = exports.sr25519DerivePublicSoft = exports.sr25519DeriveKeypairSoft = exports.sr25519DeriveKeypairHard = exports.secp256k1Sign = exports.secp256k1Recover = exports.secp256k1Expand = exports.secp256k1Compress = exports.secp256k1FromSeed = exports.ed25519Verify = exports.ed25519Sign = exports.ed25519KeypairFromSeed = exports.bip39Validate = exports.bip39ToSeed = exports.bip39ToMiniSecret = exports.bip39ToEntropy = exports.bip39Generate = exports.bridge = exports.packageInfo = void 0;
exports.isReady = isReady;
exports.waitReady = waitReady;
const init_js_1 = require("./init.js");
Object.defineProperty(exports, "bridge", { enumerable: true, get: function () { return init_js_1.bridge; } });
var packageInfo_js_1 = require("./packageInfo.js");
Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
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
        if (!init_js_1.bridge.wasm) {
            throw new Error('The WASM interface has not been initialized. Ensure that you wait for the initialization Promise with waitReady() from @polkadot/wasm-crypto (or cryptoWaitReady() from @polkadot/util-crypto) before attempting to use WASM-only interfaces.');
        }
        return fn(init_js_1.bridge.wasm, ...params);
    };
}
exports.bip39Generate = withWasm((wasm, words) => {
    wasm.ext_bip39_generate(8, words);
    return init_js_1.bridge.resultString();
});
exports.bip39ToEntropy = withWasm((wasm, phrase) => {
    wasm.ext_bip39_to_entropy(8, ...init_js_1.bridge.allocString(phrase));
    return init_js_1.bridge.resultU8a();
});
exports.bip39ToMiniSecret = withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_mini_secret(8, ...init_js_1.bridge.allocString(phrase), ...init_js_1.bridge.allocString(password));
    return init_js_1.bridge.resultU8a();
});
exports.bip39ToSeed = withWasm((wasm, phrase, password) => {
    wasm.ext_bip39_to_seed(8, ...init_js_1.bridge.allocString(phrase), ...init_js_1.bridge.allocString(password));
    return init_js_1.bridge.resultU8a();
});
exports.bip39Validate = withWasm((wasm, phrase) => {
    const ret = wasm.ext_bip39_validate(...init_js_1.bridge.allocString(phrase));
    return ret !== 0;
});
exports.ed25519KeypairFromSeed = withWasm((wasm, seed) => {
    wasm.ext_ed_from_seed(8, ...init_js_1.bridge.allocU8a(seed));
    return init_js_1.bridge.resultU8a();
});
exports.ed25519Sign = withWasm((wasm, pubkey, seckey, message) => {
    wasm.ext_ed_sign(8, ...init_js_1.bridge.allocU8a(pubkey), ...init_js_1.bridge.allocU8a(seckey), ...init_js_1.bridge.allocU8a(message));
    return init_js_1.bridge.resultU8a();
});
exports.ed25519Verify = withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_ed_verify(...init_js_1.bridge.allocU8a(signature), ...init_js_1.bridge.allocU8a(message), ...init_js_1.bridge.allocU8a(pubkey));
    return ret !== 0;
});
exports.secp256k1FromSeed = withWasm((wasm, seckey) => {
    wasm.ext_secp_from_seed(8, ...init_js_1.bridge.allocU8a(seckey));
    return init_js_1.bridge.resultU8a();
});
exports.secp256k1Compress = withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_compress(8, ...init_js_1.bridge.allocU8a(pubkey));
    return init_js_1.bridge.resultU8a();
});
exports.secp256k1Expand = withWasm((wasm, pubkey) => {
    wasm.ext_secp_pub_expand(8, ...init_js_1.bridge.allocU8a(pubkey));
    return init_js_1.bridge.resultU8a();
});
exports.secp256k1Recover = withWasm((wasm, msgHash, sig, recovery) => {
    wasm.ext_secp_recover(8, ...init_js_1.bridge.allocU8a(msgHash), ...init_js_1.bridge.allocU8a(sig), recovery);
    return init_js_1.bridge.resultU8a();
});
exports.secp256k1Sign = withWasm((wasm, msgHash, seckey) => {
    wasm.ext_secp_sign(8, ...init_js_1.bridge.allocU8a(msgHash), ...init_js_1.bridge.allocU8a(seckey));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519DeriveKeypairHard = withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_hard(8, ...init_js_1.bridge.allocU8a(pair), ...init_js_1.bridge.allocU8a(cc));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519DeriveKeypairSoft = withWasm((wasm, pair, cc) => {
    wasm.ext_sr_derive_keypair_soft(8, ...init_js_1.bridge.allocU8a(pair), ...init_js_1.bridge.allocU8a(cc));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519DerivePublicSoft = withWasm((wasm, pubkey, cc) => {
    wasm.ext_sr_derive_public_soft(8, ...init_js_1.bridge.allocU8a(pubkey), ...init_js_1.bridge.allocU8a(cc));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519KeypairFromSeed = withWasm((wasm, seed) => {
    wasm.ext_sr_from_seed(8, ...init_js_1.bridge.allocU8a(seed));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519Sign = withWasm((wasm, pubkey, secret, message) => {
    wasm.ext_sr_sign(8, ...init_js_1.bridge.allocU8a(pubkey), ...init_js_1.bridge.allocU8a(secret), ...init_js_1.bridge.allocU8a(message));
    return init_js_1.bridge.resultU8a();
});
exports.sr25519Verify = withWasm((wasm, signature, message, pubkey) => {
    const ret = wasm.ext_sr_verify(...init_js_1.bridge.allocU8a(signature), ...init_js_1.bridge.allocU8a(message), ...init_js_1.bridge.allocU8a(pubkey));
    return ret !== 0;
});
exports.sr25519Agree = withWasm((wasm, pubkey, secret) => {
    wasm.ext_sr_agree(8, ...init_js_1.bridge.allocU8a(pubkey), ...init_js_1.bridge.allocU8a(secret));
    return init_js_1.bridge.resultU8a();
});
exports.vrfSign = withWasm((wasm, secret, context, message, extra) => {
    wasm.ext_vrf_sign(8, ...init_js_1.bridge.allocU8a(secret), ...init_js_1.bridge.allocU8a(context), ...init_js_1.bridge.allocU8a(message), ...init_js_1.bridge.allocU8a(extra));
    return init_js_1.bridge.resultU8a();
});
exports.vrfVerify = withWasm((wasm, pubkey, context, message, extra, outAndProof) => {
    const ret = wasm.ext_vrf_verify(...init_js_1.bridge.allocU8a(pubkey), ...init_js_1.bridge.allocU8a(context), ...init_js_1.bridge.allocU8a(message), ...init_js_1.bridge.allocU8a(extra), ...init_js_1.bridge.allocU8a(outAndProof));
    return ret !== 0;
});
exports.blake2b = withWasm((wasm, data, key, size) => {
    wasm.ext_blake2b(8, ...init_js_1.bridge.allocU8a(data), ...init_js_1.bridge.allocU8a(key), size);
    return init_js_1.bridge.resultU8a();
});
exports.hmacSha256 = withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha256(8, ...init_js_1.bridge.allocU8a(key), ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.hmacSha512 = withWasm((wasm, key, data) => {
    wasm.ext_hmac_sha512(8, ...init_js_1.bridge.allocU8a(key), ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.keccak256 = withWasm((wasm, data) => {
    wasm.ext_keccak256(8, ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.keccak512 = withWasm((wasm, data) => {
    wasm.ext_keccak512(8, ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.pbkdf2 = withWasm((wasm, data, salt, rounds) => {
    wasm.ext_pbkdf2(8, ...init_js_1.bridge.allocU8a(data), ...init_js_1.bridge.allocU8a(salt), rounds);
    return init_js_1.bridge.resultU8a();
});
exports.scrypt = withWasm((wasm, password, salt, log2n, r, p) => {
    wasm.ext_scrypt(8, ...init_js_1.bridge.allocU8a(password), ...init_js_1.bridge.allocU8a(salt), log2n, r, p);
    return init_js_1.bridge.resultU8a();
});
exports.sha256 = withWasm((wasm, data) => {
    wasm.ext_sha256(8, ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.sha512 = withWasm((wasm, data) => {
    wasm.ext_sha512(8, ...init_js_1.bridge.allocU8a(data));
    return init_js_1.bridge.resultU8a();
});
exports.twox = withWasm((wasm, data, rounds) => {
    wasm.ext_twox(8, ...init_js_1.bridge.allocU8a(data), rounds);
    return init_js_1.bridge.resultU8a();
});
function isReady() {
    return !!init_js_1.bridge.wasm;
}
async function waitReady() {
    try {
        const wasm = await (0, init_js_1.initBridge)();
        return !!wasm;
    }
    catch {
        return false;
    }
}
