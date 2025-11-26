import { hasBigInt } from '@polkadot/util';
import { bip39ToSeed, isReady } from '@polkadot/wasm-crypto';
import { mnemonicToSeedSync } from './bip39.js';
import { mnemonicValidate } from './validate.js';
/**
 * @name mnemonicToLegacySeed
 * @summary Creates a valid Ethereum/Bitcoin-compatible seed from a mnemonic input
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate, mnemonicToLegacySeed, mnemonicValidate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * const isValidMnemonic = mnemonicValidate(mnemonic); // => boolean
 *
 * if (isValidMnemonic) {
 *   console.log(`Seed generated from mnemonic: ${mnemonicToLegacySeed(mnemonic)}`); => u8a
 * }
 * ```
 */
export function mnemonicToLegacySeed(mnemonic, password = '', onlyJs, byteLength = 32) {
    if (!mnemonicValidate(mnemonic)) {
        throw new Error('Invalid bip39 mnemonic specified');
    }
    else if (![32, 64].includes(byteLength)) {
        throw new Error(`Invalid seed length ${byteLength}, expected 32 or 64`);
    }
    return byteLength === 32
        ? !hasBigInt || (!onlyJs && isReady())
            ? bip39ToSeed(mnemonic, password)
            : mnemonicToSeedSync(mnemonic, password).subarray(0, 32)
        : mnemonicToSeedSync(mnemonic, password);
}
