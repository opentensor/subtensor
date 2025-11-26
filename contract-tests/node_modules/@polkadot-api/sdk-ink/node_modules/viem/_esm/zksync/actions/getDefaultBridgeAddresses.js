export async function getDefaultBridgeAddresses(client) {
    const addresses = await client.request({ method: 'zks_getBridgeContracts' });
    return {
        erc20L1: addresses.l1Erc20DefaultBridge,
        sharedL1: addresses.l1SharedDefaultBridge,
        sharedL2: addresses.l2SharedDefaultBridge,
        l1Nullifier: addresses.l1Nullifier,
        l1NativeTokenVault: addresses.l1NativeTokenVault,
    };
}
//# sourceMappingURL=getDefaultBridgeAddresses.js.map