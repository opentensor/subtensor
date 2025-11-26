import { objectSpread } from '@polkadot/util';
import { polkadot } from './polkadot.js';
import { shell } from './shell.js';
import { statemint } from './statemint.js';
import { substrate } from './substrate.js';
export const allExtensions = objectSpread({}, substrate, polkadot, shell, statemint);
export const fallbackExtensions = [
    'CheckVersion',
    'CheckGenesis',
    'CheckEra',
    'CheckNonce',
    'CheckWeight',
    'ChargeTransactionPayment',
    'CheckBlockGasLimit'
];
export function findUnknownExtensions(extensions, userExtensions = {}) {
    const names = [...Object.keys(allExtensions), ...Object.keys(userExtensions)];
    return extensions.filter((k) => !names.includes(k));
}
export function expandExtensionTypes(extensions, type, userExtensions = {}) {
    return extensions
        // Always allow user extensions first - these should provide overrides
        .map((k) => userExtensions[k] || allExtensions[k])
        .filter((info) => !!info)
        .reduce((result, info) => objectSpread(result, info[type]), {});
}
