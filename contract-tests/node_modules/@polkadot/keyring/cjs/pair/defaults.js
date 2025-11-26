"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SEED_LENGTH = exports.SEC_LENGTH = exports.SALT_LENGTH = exports.PUB_LENGTH = exports.PAIR_HDR = exports.PAIR_DIV = void 0;
/** public/secret section divider (generation 1-3, will change in 4, don't rely on value) */
exports.PAIR_DIV = new Uint8Array([161, 35, 3, 33, 0]);
/** public/secret start block (generation 1-3, will change in 4, don't rely on value) */
exports.PAIR_HDR = new Uint8Array([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32]);
/** length of a public key */
exports.PUB_LENGTH = 32;
/** length of a salt */
exports.SALT_LENGTH = 32;
/** length of a secret key */
exports.SEC_LENGTH = 64;
/** length of a user-input seed */
exports.SEED_LENGTH = 32;
