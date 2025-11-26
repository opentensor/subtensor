"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.upgrades = void 0;
const tslib_1 = require("tslib");
const networks_1 = require("@polkadot/networks");
const util_1 = require("@polkadot/util");
const allKnown = tslib_1.__importStar(require("./e2e/index.js"));
const NET_EXTRA = {
    westend: {
        genesisHash: ['0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e']
    }
};
/** @internal */
function mapRaw([network, versions]) {
    const chain = networks_1.selectableNetworks.find((n) => n.network === network) || NET_EXTRA[network];
    if (!chain) {
        throw new Error(`Unable to find info for chain ${network}`);
    }
    return {
        genesisHash: (0, util_1.hexToU8a)(chain.genesisHash[0]),
        network,
        versions: versions.map(([blockNumber, specVersion, apis]) => ({
            apis,
            blockNumber: new util_1.BN(blockNumber),
            specVersion: new util_1.BN(specVersion)
        }))
    };
}
exports.upgrades = Object.entries(allKnown).map(mapRaw);
