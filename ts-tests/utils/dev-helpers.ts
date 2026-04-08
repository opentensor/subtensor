/**
 * Polkadot.js (ApiPromise) compatible helpers for dev tests.
 * Uses ApiPromise, not PAPI TypedApi — keep them separate.
 */
import type { ApiPromise } from "@polkadot/api";
import { tao } from "./balance.ts";
import { DevModeContext } from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";

export async function devForceSetBalance(
    polkadotJs: ApiPromise,
    context: any,
    address: string,
    amount: bigint = tao(1e10)
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.sudo
            .sudo(polkadotJs.tx.balances.forceSetBalance(address, amount))
            .signAsync(context.keyring.alice),
    ]);
}

export async function devTryAssociateHotkey(
    api: ApiPromise,
    context: any,
    coldkey: KeyringPair,
    hotkey: string
): Promise<void> {
    await context.createBlock([await api.tx.subtensorModule.tryAssociateHotkey(hotkey).signAsync(coldkey)]);
}

export async function devSetWeightsTx(
    api: ApiPromise,
    context: DevModeContext,
    coldkey: KeyringPair,
    netuid: number,
    uids: number[],
    values: number[],
    versionKey: bigint
): Promise<void> {
    await context.createBlock([
        await api.tx.subtensorModule.setWeights(netuid, uids, values, versionKey).signAsync(coldkey),
    ]);
}
