/* @deprecated Use the `L1_CHAIN_ID()` method on the `L2AssetRouter` contract (deployed on `0x0000000000000000000000000000000000010003` address) */
export async function getL1ChainId(client) {
    const result = await client.request({ method: 'zks_L1ChainId' });
    return result;
}
//# sourceMappingURL=getL1ChainId.js.map