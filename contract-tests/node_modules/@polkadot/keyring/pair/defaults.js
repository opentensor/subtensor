/** public/secret section divider (generation 1-3, will change in 4, don't rely on value) */
export const PAIR_DIV = new Uint8Array([161, 35, 3, 33, 0]);
/** public/secret start block (generation 1-3, will change in 4, don't rely on value) */
export const PAIR_HDR = new Uint8Array([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32]);
/** length of a public key */
export const PUB_LENGTH = 32;
/** length of a salt */
export const SALT_LENGTH = 32;
/** length of a secret key */
export const SEC_LENGTH = 64;
/** length of a user-input seed */
export const SEED_LENGTH = 32;
