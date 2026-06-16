import { MultiAddress, subtensor } from "@polkadot-api/descriptors";
import { Keyring } from "@polkadot/keyring";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { TypedApi } from "polkadot-api";
import { Binary } from "polkadot-api";
import { convertPublicKeyToSs58 } from "./address.ts";
import { getBalance } from "./balance.ts";
import { sudoSetAdminFreezeWindow } from "./staking.ts";
import { sendTransaction, waitForTransactionWithRetry } from "./transactions.ts";

export const BITTENSOR_WASM_PATH = "./ink/bittensor.wasm";

export async function getTransferCallCode(
    api: TypedApi<typeof subtensor>,
    receiver: KeyringPair,
    transferAmount: number
): Promise<number[]> {
    const unsignedTx = api.tx.Balances.transfer_keep_alive({
        dest: MultiAddress.Id(convertPublicKeyToSs58(receiver.publicKey)),
        value: BigInt(transferAmount),
    });
    const encodedCallDataBytes = await unsignedTx.getEncodedData();
    return [...encodedCallDataBytes.asBytes()];
}

export async function getProxies(api: TypedApi<typeof subtensor>, address: string): Promise<string[]> {
    const entries = await api.query.Proxy.Proxies.getEntries();
    const result: string[] = [];
    for (const entry of entries) {
        const proxyAddress = entry.keyArgs[0];
        const values = entry.value;
        const proxies = values[0];
        for (const proxy of proxies) {
            if (proxy.delegate === address) {
                result.push(proxyAddress);
            }
        }
    }
    return result;
}

export async function setAdminFreezeWindow(api: TypedApi<typeof subtensor>): Promise<void> {
    await sudoSetAdminFreezeWindow(api, 0);
    const window = await api.query.SubtensorModule.AdminFreezeWindow.getValue();
    if (window !== 0) {
        throw new Error(`Expected AdminFreezeWindow=0, got ${window}`);
    }
}

export async function setTargetRegistrationsPerInterval(
    api: TypedApi<typeof subtensor>,
    netuid: number
): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalTx = api.tx.AdminUtils.sudo_set_target_registrations_per_interval({
        netuid,
        target_registrations_per_interval: 1000,
    });
    const tx = api.tx.Sudo.sudo({ call: internalTx.decodedCall });
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_target_registrations_per_interval");

    const target = await api.query.SubtensorModule.TargetRegistrationsPerInterval.getValue(netuid);
    if (target !== 1000) {
        throw new Error(`Expected TargetRegistrationsPerInterval=1000 for netuid ${netuid}, got ${target}`);
    }
}

export async function sendWasmContractExtrinsic(
    api: TypedApi<typeof subtensor>,
    coldkey: KeyringPair,
    contractAddress: string,
    data: { asBytes(): Uint8Array }
): Promise<void> {
    const tx = api.tx.Contracts.call({
        value: BigInt(0),
        dest: MultiAddress.Id(contractAddress),
        data: Binary.fromBytes(data.asBytes()),
        gas_limit: {
            ref_time: BigInt(10_000_000_000),
            proof_size: BigInt(10_000_000),
        },
        storage_deposit_limit: BigInt(1_000_000_000),
    });
    await waitForTransactionWithRetry(api, tx, coldkey, "contracts_call", 5);
}

/** Submit a contract call without failing when the contract reverts (expected for atomic-failure tests). */
export async function sendWasmContractExtrinsicAllowFailure(
    api: TypedApi<typeof subtensor>,
    coldkey: KeyringPair,
    contractAddress: string,
    data: { asBytes(): Uint8Array }
): Promise<void> {
    const tx = api.tx.Contracts.call({
        value: BigInt(0),
        dest: MultiAddress.Id(contractAddress),
        data: Binary.fromBytes(data.asBytes()),
        gas_limit: {
            ref_time: BigInt(10_000_000_000),
            proof_size: BigInt(10_000_000),
        },
        storage_deposit_limit: BigInt(1_000_000_000),
    });
    await sendTransaction(tx, coldkey);
}

export async function getStakeInfoForHotkeyColdkeyNetuid(
    api: TypedApi<typeof subtensor>,
    hotkey: string,
    coldkey: string,
    netuid: number
): Promise<bigint | undefined> {
    return (await api.apis.StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid(hotkey, coldkey, netuid))
        ?.stake;
}

export async function instantiateWasmContract(
    api: TypedApi<typeof subtensor>,
    coldkey: KeyringPair,
    wasmBytecode: Uint8Array,
    constructorData: { asBytes(): Uint8Array }
): Promise<string> {
    const tx = api.tx.Contracts.instantiate_with_code({
        code: Binary.fromBytes(wasmBytecode),
        storage_deposit_limit: BigInt(10_000_000),
        value: BigInt(0),
        gas_limit: {
            ref_time: BigInt(1_000_000_000),
            proof_size: BigInt(1_000_000),
        },
        data: Binary.fromBytes(constructorData.asBytes()),
        salt: Binary.fromHex("0x"),
    });

    const result = await sendTransaction(tx, coldkey);
    if (!result.success) {
        throw new Error(`instantiate_with_code failed: ${result.errorMessage ?? "unknown error"}`);
    }

    const instantiatedEvents = await api.event.Contracts.Instantiated.filter(result.events);
    if (instantiatedEvents.length === 0) {
        throw new Error("No Contracts.Instantiated events found after instantiate_with_code");
    }

    return instantiatedEvents[0].contract;
}

export { convertPublicKeyToSs58, getBalance };
