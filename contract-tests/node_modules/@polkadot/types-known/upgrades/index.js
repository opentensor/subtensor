import { selectableNetworks } from '@polkadot/networks';
import { BN, hexToU8a } from '@polkadot/util';
import * as allKnown from './e2e/index.js';
const NET_EXTRA = {
    westend: {
        genesisHash: ['0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e']
    }
};
/** @internal */
function mapRaw([network, versions]) {
    const chain = selectableNetworks.find((n) => n.network === network) || NET_EXTRA[network];
    if (!chain) {
        throw new Error(`Unable to find info for chain ${network}`);
    }
    return {
        genesisHash: hexToU8a(chain.genesisHash[0]),
        network,
        versions: versions.map(([blockNumber, specVersion, apis]) => ({
            apis,
            blockNumber: new BN(blockNumber),
            specVersion: new BN(specVersion)
        }))
    };
}
export const upgrades = Object.entries(allKnown).map(mapRaw);
