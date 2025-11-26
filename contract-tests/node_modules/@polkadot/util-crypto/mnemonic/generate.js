import { hasBigInt } from '@polkadot/util';
import { bip39Generate, isReady } from '@polkadot/wasm-crypto';
import { generateMnemonic } from './bip39.js';
/**
 * @name mnemonicGenerate
 * @summary Creates a valid mnemonic string using using [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).
 * @example
 * <BR>
 *
 * ```javascript
 * import { mnemonicGenerate } from '@polkadot/util-crypto';
 *
 * const mnemonic = mnemonicGenerate(); // => string
 * ```
 */
export function mnemonicGenerate(numWords = 12, wordlist, onlyJs) {
    return !hasBigInt || (!wordlist && !onlyJs && isReady())
        ? bip39Generate(numWords)
        : generateMnemonic(numWords, wordlist);
}
