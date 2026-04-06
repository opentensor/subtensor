/**
 * Polkadot.js (ApiPromise) compatible helpers for limit-orders dev tests.
 * The utils/ directory uses PAPI TypedApi which is incompatible with the
 * moonwall `dev` foundation that exposes context.polkadotJs().
 */
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { SignedOrder } from "utils";

export async function devForceSetBalance(
    polkadotJs: ApiPromise,
    context: any,
    address: string,
    amount: bigint
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.sudo
            .sudo(polkadotJs.tx.balances.forceSetBalance(address, amount))
            .signAsync(context.keyring.alice),
    ]);
}

export async function devAddStake(
    polkadotJs: ApiPromise,
    context: any,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    amount: bigint
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.subtensorModule
            .addStake(hotkey, netuid, amount)
            .signAsync(coldkey),
    ]);
}

export async function devAssociateHotKey(
    polkadotJs: ApiPromise,
    context: any,
    coldkey: KeyringPair,
    hotkey: string,
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.subtensorModule
            .tryAssociateHotkey(hotkey)
            .signAsync(coldkey),
    ]);
}

export async function devGetAlphaStake(
    polkadotJs: ApiPromise,
    hotkey: string,
    coldkey: string,
    netuid: number
): Promise<bigint> {
    const value = (await polkadotJs.query.subtensorModule.alphaV2(
        hotkey,
        coldkey,
        netuid
    ));

    const mantissa = value.mantissa;
    const exponent = value.exponent;

    let result: bigint;

    if (exponent >= 0n) {
        result = BigInt(mantissa) * BigInt(10) ** BigInt(exponent);
    } else {
        result = BigInt(mantissa) / BigInt(10) ** BigInt(-exponent);
    }

    return result;
}

export async function devGetBalance(
    polkadotJs: ApiPromise,
    address: string
): Promise<bigint> {
    const account = (await polkadotJs.query.system.account(address)) as any;
    return account.data.free.toBigInt();
}

export async function devSudoSetLockReductionInterval(
    polkadotJs: ApiPromise,
    context: any,
    alice: KeyringPair,
    interval: number): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.adminUtils
            .sudoSetLockReductionInterval(interval)
            .signAsync(alice),
    ]);
}

export async function devRegisterSubnet(
    polkadotJs: ApiPromise,
    context: any,
    alice: KeyringPair,
    hotkey: KeyringPair
): Promise<number> {
    await context.createBlock([
        await polkadotJs.tx.subtensorModule
            .registerNetwork(hotkey.address)
            .signAsync(alice),
    ]);
    const events = (await polkadotJs.query.system.events()) as any;
    const netuid = (events as any[])
        .filter((e: any) => e.event.method === "NetworkAdded")[0]
        .event.data[0].toNumber();
    return netuid;
}

export async function devEnableSubtoken(
    polkadotJs: ApiPromise,
    context: any,
    alice: KeyringPair,
    netuid: number
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.sudo
            .sudo(polkadotJs.tx.adminUtils.sudoSetSubtokenEnabled(netuid, true))
            .signAsync(alice),
    ]);
}

export async function devSeedPool(
    polkadotJs: ApiPromise,
    context: any,
    alice: KeyringPair,
    netuid: number,
    taoReserve: bigint,
    alphaIn: bigint
): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.sudo
            .sudo(polkadotJs.tx.adminUtils.sudoSetSubnetTao(netuid, taoReserve))
            .signAsync(alice),
    ]);
    await context.createBlock([
        await polkadotJs.tx.sudo
            .sudo(polkadotJs.tx.adminUtils.sudoSetSubnetAlphaIn(netuid, alphaIn))
            .signAsync(alice),
    ]);
}

export async function devExecuteOrders(
    polkadotJs: ApiPromise,
    context: any,
    alice: KeyringPair,
    orders: SignedOrder[]): Promise<void> {
    await context.createBlock([
        await polkadotJs.tx.limitOrders
            .executeOrders(orders)
            .signAsync(alice),
    ]);
}