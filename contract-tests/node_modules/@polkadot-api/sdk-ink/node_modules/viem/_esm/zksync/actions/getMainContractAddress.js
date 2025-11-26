/* @deprecated Use `getBridgeHubContractAddress` and call `getZKChain(chainId)` instead. */
export async function getMainContractAddress(client) {
    const address = await client.request({ method: 'zks_getMainContract' });
    return address;
}
//# sourceMappingURL=getMainContractAddress.js.map