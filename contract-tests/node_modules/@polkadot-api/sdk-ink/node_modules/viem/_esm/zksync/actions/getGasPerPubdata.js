import { hexToBigInt } from '../../utils/encoding/fromHex.js';
export async function getGasPerPubdata(client) {
    const result = await client.request({
        method: 'zks_gasPerPubdata',
    }, {
        dedupe: true,
    });
    return hexToBigInt(result);
}
//# sourceMappingURL=getGasPerPubdata.js.map