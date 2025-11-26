"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getHasher = getHasher;
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const DEFAULT_FN = (data) => (0, util_crypto_1.xxhashAsU8a)(data, 128);
const HASHERS = {
    Blake2_128: (data) => // eslint-disable-line camelcase
     (0, util_crypto_1.blake2AsU8a)(data, 128),
    Blake2_128Concat: (data) => // eslint-disable-line camelcase
     (0, util_1.u8aConcat)((0, util_crypto_1.blake2AsU8a)(data, 128), (0, util_1.u8aToU8a)(data)),
    Blake2_256: (data) => // eslint-disable-line camelcase
     (0, util_crypto_1.blake2AsU8a)(data, 256),
    Identity: (data) => (0, util_1.u8aToU8a)(data),
    Twox128: (data) => (0, util_crypto_1.xxhashAsU8a)(data, 128),
    Twox256: (data) => (0, util_crypto_1.xxhashAsU8a)(data, 256),
    Twox64Concat: (data) => (0, util_1.u8aConcat)((0, util_crypto_1.xxhashAsU8a)(data, 64), (0, util_1.u8aToU8a)(data))
};
/** @internal */
function getHasher(hasher) {
    return HASHERS[hasher.type] || DEFAULT_FN;
}
