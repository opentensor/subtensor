import { waitForTransactionWithRetry } from "./transactions.js";
import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";
import type { ApiPromise } from "@polkadot/api";

export async function addNewSubnetwork(api: ApiPromise, hotkey: KeyringPair, coldkey: KeyringPair): Promise<number> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const totalNetworks = (await api.query.subtensorModule.totalNetworks()).toString();

    // Disable network rate limit for testing
    const rateLimit = BigInt((await api.query.subtensorModule.networkRateLimit()).toString());
    if (rateLimit !== BigInt(0)) {
        const internalTx = api.tx.adminUtils.sudoSetNetworkRateLimit(BigInt(0));
        const tx = api.tx.sudo.sudo(internalTx);
        await waitForTransactionWithRetry(api, tx, alice, "set_network_rate_limit");
    }

    const registerNetworkTx = api.tx.subtensorModule.registerNetwork(hotkey.address);
    await waitForTransactionWithRetry(api, registerNetworkTx, coldkey, "register_network");

    return Number(totalNetworks);
}

export async function burnedRegister(
    api: ApiPromise,
    netuid: number,
    hotkeyAddress: string,
    coldkey: KeyringPair
): Promise<void> {
    const registered = (await api.query.subtensorModule.uids(netuid, hotkeyAddress)).toJSON();
    if (registered !== null) {
        log.tx("burned_register", `skipped: hotkey already registered on netuid ${netuid}`);
        return;
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
    const tx = api.tx.subtensorModule.burnedRegister(netuid, hotkeyAddress);
    await waitForTransactionWithRetry(api, tx, coldkey, "burned_register");
}

export async function startCall(api: ApiPromise, netuid: number, coldkey: KeyringPair): Promise<void> {
    const registerBlock = Number(await api.query.subtensorModule.networkRegisteredAt(netuid).toString());
    let currentBlock = Number(await api.query.system.number().toString());
    const duration = Number(api.consts.subtensorModule.initialStartCallDelay.toString());

    while (currentBlock - registerBlock <= duration) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        currentBlock = Number((await api.query.system.number()).toString());
    }

    await new Promise((resolve) => setTimeout(resolve, 2000));

    const tx = api.tx.subtensorModule.startCall(netuid);
    await waitForTransactionWithRetry(api, tx, coldkey, "start_call");

    await new Promise((resolve) => setTimeout(resolve, 1000));
}
