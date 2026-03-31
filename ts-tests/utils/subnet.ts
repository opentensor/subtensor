import { waitForSudoTransactionWithRetry, waitForTransactionWithRetry } from "./transactions.js";
import { log } from "./logger.js";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";
import type { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import { Enum } from "polkadot-api";

export async function addNewSubnetwork(
    api: TypedApi<typeof subtensor>,
    hotkey: KeyringPair,
    coldkey: KeyringPair
): Promise<number> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue();

    const target = Enum("Group", 3);
    const limits = (await api.query.RateLimiting.Limits.getValue(target as never)) as any;
    const rateLimit =
        limits?.type === "Global" && limits.value?.type === "Exact" ? BigInt(limits.value.value) : BigInt(0);

    if (rateLimit !== BigInt(0)) {
        const internalCall = api.tx.RateLimiting.set_rate_limit({
            target: target as never,
            scope: undefined,
            limit: Enum("Exact", 0) as never,
        });
        const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
        await waitForSudoTransactionWithRetry(api, tx, alice, "set_network_rate_limit");
    }

    const registerNetworkTx = api.tx.SubtensorModule.register_network({
        hotkey: hotkey.address,
    });
    await waitForTransactionWithRetry(api, registerNetworkTx, coldkey, "register_network");

    return totalNetworks;
}

export async function burnedRegister(
    api: TypedApi<typeof subtensor>,
    netuid: number,
    hotkeyAddress: string,
    coldkey: KeyringPair
): Promise<void> {
    const registered = await api.query.SubtensorModule.Uids.getValue(netuid, hotkeyAddress);
    if (registered !== undefined) {
        log.tx("burned_register", `skipped: hotkey already registered on netuid ${netuid}`);
        return;
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
    const tx = api.tx.SubtensorModule.burned_register({ hotkey: hotkeyAddress, netuid: netuid });
    await waitForTransactionWithRetry(api, tx, coldkey, "burned_register");
}

export async function startCall(api: TypedApi<typeof subtensor>, netuid: number, coldkey: KeyringPair): Promise<void> {
    const existingFirstEmission = await api.query.SubtensorModule.FirstEmissionBlockNumber.getValue(netuid);
    if (existingFirstEmission !== undefined) {
        return;
    }

    const registerBlock = Number(await api.query.SubtensorModule.NetworkRegisteredAt.getValue(netuid));
    let currentBlock = await api.query.System.Number.getValue();
    const duration = Number(await api.constants.SubtensorModule.InitialStartCallDelay);

    while (currentBlock - registerBlock <= duration) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        currentBlock = await api.query.System.Number.getValue();
    }

    await new Promise((resolve) => setTimeout(resolve, 2000));

    const tx = api.tx.SubtensorModule.start_call({ netuid: netuid });
    try {
        await waitForTransactionWithRetry(api, tx, coldkey, "start_call");
    } catch (error) {
        if (error instanceof Error && error.message.includes("FirstEmissionBlockNumberAlreadySet")) {
            return;
        }
        throw error;
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
}
