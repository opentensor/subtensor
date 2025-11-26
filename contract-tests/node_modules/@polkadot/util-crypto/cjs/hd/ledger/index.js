"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hdLedger = hdLedger;
const index_js_1 = require("../../ed25519/index.js");
const index_js_2 = require("../../mnemonic/index.js");
const validatePath_js_1 = require("../validatePath.js");
const derivePrivate_js_1 = require("./derivePrivate.js");
const master_js_1 = require("./master.js");
function hdLedger(_mnemonic, path) {
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
    if (!(0, index_js_2.mnemonicValidate)(mnemonic)) {
        throw new Error('Invalid mnemonic passed to ledger derivation');
    }
    else if (!(0, validatePath_js_1.hdValidatePath)(path)) {
        throw new Error('Invalid derivation path');
    }
    const parts = path.split('/').slice(1);
    let seed = (0, master_js_1.ledgerMaster)(mnemonic, password);
    for (const p of parts) {
        const n = parseInt(p.replace(/'$/, ''), 10);
        seed = (0, derivePrivate_js_1.ledgerDerivePrivate)(seed, (n < validatePath_js_1.HARDENED) ? (n + validatePath_js_1.HARDENED) : n);
    }
    return (0, index_js_1.ed25519PairFromSeed)(seed.slice(0, 32));
}
