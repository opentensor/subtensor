import { ed25519PairFromSeed } from '../../ed25519/index.js';
import { mnemonicValidate } from '../../mnemonic/index.js';
import { HARDENED, hdValidatePath } from '../validatePath.js';
import { ledgerDerivePrivate } from './derivePrivate.js';
import { ledgerMaster } from './master.js';
export function hdLedger(_mnemonic, path) {
    const words = _mnemonic
        .split(' ')
        .map((s) => s.trim())
        .filter((s) => s);
    if (![12, 24, 25].includes(words.length)) {
        throw new Error('Expected a mnemonic with 24 words (or 25 including a password)');
    }
    const [mnemonic, password] = words.length === 25
        ? [words.slice(0, 24).join(' '), words[24]]
        : [words.join(' '), ''];
    if (!mnemonicValidate(mnemonic)) {
        throw new Error('Invalid mnemonic passed to ledger derivation');
    }
    else if (!hdValidatePath(path)) {
        throw new Error('Invalid derivation path');
    }
    const parts = path.split('/').slice(1);
    let seed = ledgerMaster(mnemonic, password);
    for (const p of parts) {
        const n = parseInt(p.replace(/'$/, ''), 10);
        seed = ledgerDerivePrivate(seed, (n < HARDENED) ? (n + HARDENED) : n);
    }
    return ed25519PairFromSeed(seed.slice(0, 32));
}
