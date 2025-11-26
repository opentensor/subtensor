import { stringToU8a } from '@polkadot/util';
import { bip39ToMiniSecret, isReady } from '@polkadot/wasm-crypto';
import { pbkdf2Encode } from '../pbkdf2/index.js';
import { mnemonicToEntropy } from './toEntropy.js';
import { mnemonicValidate } from './validate.js';
export function mnemonicToMiniSecret(mnemonic, password = '', wordlist, onlyJs) {
    if (!mnemonicValidate(mnemonic, wordlist, onlyJs)) {
        throw new Error('Invalid bip39 mnemonic specified');
    }
    else if (!wordlist && !onlyJs && isReady()) {
        return bip39ToMiniSecret(mnemonic, password);
    }
    const entropy = mnemonicToEntropy(mnemonic, wordlist);
    const salt = stringToU8a(`mnemonic${password}`);
    // return the first 32 bytes as the seed
    return pbkdf2Encode(entropy, salt).password.slice(0, 32);
}
