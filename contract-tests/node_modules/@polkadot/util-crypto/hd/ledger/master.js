import { u8aConcat } from '@polkadot/util';
import { hmacShaAsU8a } from '../../hmac/index.js';
import { mnemonicToSeedSync } from '../../mnemonic/bip39.js';
const ED25519_CRYPTO = 'ed25519 seed';
export function ledgerMaster(mnemonic, password) {
    const seed = mnemonicToSeedSync(mnemonic, password);
    const chainCode = hmacShaAsU8a(ED25519_CRYPTO, new Uint8Array([1, ...seed]), 256);
    let priv;
    while (!priv || (priv[31] & 0b0010_0000)) {
        priv = hmacShaAsU8a(ED25519_CRYPTO, priv || seed, 512);
    }
    priv[0] &= 0b1111_1000;
    priv[31] &= 0b0111_1111;
    priv[31] |= 0b0100_0000;
    return u8aConcat(priv, chainCode);
}
