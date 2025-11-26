"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.selectableNetworks = exports.availableNetworks = exports.allNetworks = void 0;
const tslib_1 = require("tslib");
const ss58_registry_1 = tslib_1.__importDefault(require("@substrate/ss58-registry"));
const index_js_1 = require("./defaults/index.js");
const UNSORTED = [0, 2, 42];
const TESTNETS = ['testnet'];
function toExpanded(o) {
    const network = o.network || '';
    const nameParts = network.replace(/_/g, '-').split('-');
    const n = o;
    // ledger additions
    n.slip44 = index_js_1.knownLedger[network];
    n.hasLedgerSupport = !!n.slip44;
    // general items
    n.genesisHash = index_js_1.knownGenesis[network] || [];
    n.icon = index_js_1.knownIcon[network] || 'substrate';
    // filtering
    n.isTestnet = !!index_js_1.knownTestnet[network] || TESTNETS.includes(nameParts[nameParts.length - 1]);
    n.isIgnored = n.isTestnet || (!(o.standardAccount &&
        o.decimals?.length &&
        o.symbols?.length) &&
        o.prefix !== 42);
    return n;
}
function filterSelectable({ genesisHash, prefix }) {
    return !!genesisHash.length || prefix === 42;
}
function filterAvailable(n) {
    return !n.isIgnored && !!n.network;
}
function sortNetworks(a, b) {
    const isUnSortedA = UNSORTED.includes(a.prefix);
    const isUnSortedB = UNSORTED.includes(b.prefix);
    return isUnSortedA === isUnSortedB
        ? isUnSortedA
            ? 0
            : a.displayName.localeCompare(b.displayName)
        : isUnSortedA
            ? -1
            : 1;
}
exports.allNetworks = ss58_registry_1.default.map(toExpanded);
exports.availableNetworks = exports.allNetworks.filter(filterAvailable).sort(sortNetworks);
exports.selectableNetworks = exports.availableNetworks.filter(filterSelectable);
