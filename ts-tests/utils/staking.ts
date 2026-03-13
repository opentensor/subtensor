import { waitForTransactionWithRetry } from "./transactions.js";
import { Keyring } from "@polkadot/keyring";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

// U64F64 is a 128-bit fixed-point type with 64 fractional bits.
// Raw storage values must be divided by 2^64 to get the actual value.
const U64F64_FRACTIONAL_BITS = 64n;
const U64F64_MULTIPLIER = 1n << U64F64_FRACTIONAL_BITS; // 2^64

/**
 * Convert a raw U64F64 storage value to its integer part (truncated).
 */
export function u64f64ToInt(raw: bigint): bigint {
    return raw >> U64F64_FRACTIONAL_BITS;
}

/**
 * Convert an integer to U64F64 raw format for use in extrinsics.
 */
export function intToU64f64(value: bigint): bigint {
    return value << U64F64_FRACTIONAL_BITS;
}

/**
 * Convert a raw U64F64 storage value to a decimal number for display.
 */
export function u64f64ToNumber(raw: bigint): number {
    return Number(raw) / Number(U64F64_MULTIPLIER);
}

export async function addStake(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    amount: bigint
): Promise<void> {
    const tx = api.tx.subtensorModule.addStake(hotkey, netuid, amount);
    await waitForTransactionWithRetry(tx, coldkey, "add_stake");
}

export async function addStakeLimit(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    amount: bigint,
    limitPrice: bigint,
    allowPartial: boolean
): Promise<void> {
    const tx = api.tx.subtensorModule.addStakeLimit(hotkey, netuid, amount, limitPrice, allowPartial);
    await waitForTransactionWithRetry(tx, coldkey, "add_stake_limit");
}

export async function removeStake(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    amount: bigint
): Promise<void> {
    const tx = api.tx.subtensorModule.removeStake(hotkey, netuid, amount);
    await waitForTransactionWithRetry(tx, coldkey, "remove_stake");
}

export async function removeStakeLimit(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    amount: bigint,
    limitPrice: bigint,
    allowPartial: boolean
): Promise<void> {
    const tx = api.tx.subtensorModule.removeStakeLimit(hotkey, netuid, amount, limitPrice, allowPartial);
    await waitForTransactionWithRetry(tx, coldkey, "remove_stake_limit");
}

export async function removeStakeFullLimit(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    netuid: number,
    limitPrice: bigint | undefined
): Promise<void> {
    const tx = api.tx.subtensorModule.removeStakeFullLimit(hotkey, netuid, limitPrice);
    await waitForTransactionWithRetry(tx, coldkey, "remove_stake_full_limit");
}

export async function unstakeAll(api: ApiPromise, coldkey: KeyringPair, hotkey: string): Promise<void> {
    const tx = api.tx.subtensorModule.unstakeAll(hotkey);
    await waitForTransactionWithRetry(tx, coldkey, "unstake_all");
}

export async function unstakeAllAlpha(api: ApiPromise, coldkey: KeyringPair, hotkey: string): Promise<void> {
    const tx = api.tx.subtensorModule.unstakeAllAlpha(hotkey);
    await waitForTransactionWithRetry(tx, coldkey, "unstake_all_alpha");
}

/**
 * Get stake shares (Alpha) for a hotkey/coldkey/netuid triplet.
 * Returns the integer part of the U64F64 value.
 */
export async function getStake(api: ApiPromise, hotkey: string, coldkey: string, netuid: number): Promise<bigint> {
    const obj = (await api.query.subtensorModule.alpha(hotkey, coldkey, netuid)).toJSON() as { bits: number };
    return BigInt(obj.bits);
}

/**
 * Get raw stake shares (Alpha) in U64F64 format.
 * Use this when you need the raw value for extrinsics like transfer_stake.
 */
export async function getStakeRaw(api: ApiPromise, hotkey: string, coldkey: string, netuid: number): Promise<bigint> {
    const obj = (await api.query.subtensorModule.alpha(hotkey, coldkey, netuid)).toJSON() as { bits: number };
    return BigInt(obj.bits);
}

export async function transferStake(
    api: ApiPromise,
    originColdkey: KeyringPair,
    destinationColdkey: string,
    hotkey: string,
    originNetuid: number,
    destinationNetuid: number,
    amount: bigint
): Promise<void> {
    const tx = api.tx.subtensorModule.transferStake(
        destinationColdkey,
        hotkey,
        originNetuid,
        destinationNetuid,
        amount
    );
    await waitForTransactionWithRetry(tx, originColdkey, "transfer_stake");
}

export async function moveStake(
    api: ApiPromise,
    coldkey: KeyringPair,
    originHotkey: string,
    destinationHotkey: string,
    originNetuid: number,
    destinationNetuid: number,
    amount: bigint
): Promise<void> {
    const tx = api.tx.subtensorModule.moveStake(
        originHotkey,
        destinationHotkey,
        originNetuid,
        destinationNetuid,
        amount
    );
    await waitForTransactionWithRetry(tx, coldkey, "move_stake");
}

export async function swapStake(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    originNetuid: number,
    destinationNetuid: number,
    amount: bigint
): Promise<void> {
    const tx = api.tx.subtensorModule.swapStake(hotkey, originNetuid, destinationNetuid, amount);
    await waitForTransactionWithRetry(tx, coldkey, "swap_stake");
}

export async function swapStakeLimit(
    api: ApiPromise,
    coldkey: KeyringPair,
    hotkey: string,
    originNetuid: number,
    destinationNetuid: number,
    amount: bigint,
    limitPrice: bigint,
    allowPartial: boolean
): Promise<void> {
    const tx = api.tx.subtensorModule.swapStakeLimit(
        hotkey,
        originNetuid,
        destinationNetuid,
        amount,
        limitPrice,
        allowPartial
    );
    await waitForTransactionWithRetry(tx, coldkey, "swap_stake_limit");
}

export type RootClaimType = "Swap" | "Keep" | { type: "KeepSubnets"; subnets: number[] };

export async function getRootClaimType(api: ApiPromise, coldkey: string): Promise<RootClaimType> {
    const result = (await api.query.subtensorModule.tootClaimType(coldkey)).toJSON() as any; // TODO: Fix any
    if (result.type === "KeepSubnets") {
        return { type: "KeepSubnets", subnets: result.value.subnets as number[] };
    }
    return result.type as "Swap" | "Keep";
}

export async function setRootClaimType(api: ApiPromise, coldkey: KeyringPair, claimType: RootClaimType): Promise<void> {
    let newRootClaimType: any;
    if (typeof claimType === "string") {
        newRootClaimType = { type: claimType, value: undefined };
    } else {
        newRootClaimType = { type: "KeepSubnets", value: { subnets: claimType.subnets } };
    }
    const tx = api.tx.subtensorModule.setRootClaimType(newRootClaimType);
    await waitForTransactionWithRetry(tx, coldkey, "set_root_claim_type");
}

export async function claimRoot(api: ApiPromise, coldkey: KeyringPair, subnets: number[]): Promise<void> {
    const tx = api.tx.subtensorModule.claimRoot(subnets);
    await waitForTransactionWithRetry(tx, coldkey, "claim_root");
}

export async function getNumRootClaims(api: ApiPromise): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.NumRootClaim()).toString());
}

export async function sudoSetNumRootClaims(api: ApiPromise, newValue: bigint): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.subtensorModule.sudoSetNumRootClaims(newValue);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_num_root_claims");
}

export async function getRootClaimThreshold(api: ApiPromise, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.RootClaimableThreshold(netuid)).toString());
}

export async function sudoSetRootClaimThreshold(api: ApiPromise, netuid: number, newValue: bigint): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.subtensorModule.sudoSetRootClaimThreshold(netuid, newValue);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_root_claim_threshold");
}

export async function getTempo(api: ApiPromise, netuid: number): Promise<number> {
    return Number((await api.query.subtensorModule.tempo(netuid)).toString());
}

export async function sudoSetTempo(api: ApiPromise, netuid: number, tempo: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetTempo(netuid, tempo);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_tempo");
}

export async function waitForBlocks(api: ApiPromise, numBlocks: number): Promise<void> {
    const startBlock = await api.query.system.number();
    const targetBlock = startBlock + numBlocks;

    while (true) {
        const currentBlock = await api.query.system.number();
        if (currentBlock >= targetBlock) {
            break;
        }
        await new Promise((resolve) => setTimeout(resolve, 1000));
    }
}

export async function getRootClaimable(api: ApiPromise, hotkey: string): Promise<Map<number, bigint>> {
    const result = (await api.query.subtensorModule.rootClaimable(hotkey)).toJSON() as any; // TODO: Fix any
    const claimableMap = new Map<number, bigint>();
    for (const [netuid, amount] of result) {
        claimableMap.set(netuid, amount);
    }
    return claimableMap;
}

export async function getRootClaimed(
    api: ApiPromise,
    netuid: number,
    hotkey: string,
    coldkey: string
): Promise<bigint> {
    return BigInt((await api.query.subtensorModule.rootClaimed(netuid, hotkey, coldkey)).toString());
}

export async function isSubtokenEnabled(api: ApiPromise, netuid: number): Promise<boolean> {
    return await api.query.SubtensorModule.SubtokenEnabled(netuid);
}

export async function sudoSetSubtokenEnabled(api: ApiPromise, netuid: number, enabled: boolean): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetSubtokenEnabled(netuid, enabled);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_subtoken_enabled");
}

export async function isNetworkAdded(api: ApiPromise, netuid: number): Promise<boolean> {
    return await api.query.subtensorModule.networksAdded(netuid);
}

export async function getAdminFreezeWindow(api: ApiPromise): Promise<number> {
    return Number((await api.query.subtensorModule.adminFreezeWindow()).toString());
}

export async function sudoSetAdminFreezeWindow(api: ApiPromise, window: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetAdminFreezeWindow(window);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_admin_freeze_window");
}

export async function sudoSetEmaPriceHalvingPeriod(
    api: ApiPromise,
    netuid: number,
    emaPriceHalvingPeriod: number
): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetEmaPriceHalvingPeriod(netuid, BigInt(emaPriceHalvingPeriod));
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_ema_price_halving_period");
}

export async function sudoSetLockReductionInterval(api: ApiPromise, interval: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetLockReductionInterval(BigInt(interval));
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_lock_reduction_interval");
}

export async function sudoSetSubnetMovingAlpha(api: ApiPromise, alpha: bigint): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetSubnetMovingAlpha(alpha);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(tx, alice, "sudo_set_subnet_moving_alpha");
}

// Debug helpers for claim_root investigation
export async function getSubnetTAO(api: ApiPromise, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.SubnetTAO(netuid)).toString());
}

export async function getSubnetMovingPrice(api: ApiPromise, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.SubnetMovingPrice(netuid)).toString());
}

export async function getPendingRootAlphaDivs(api: ApiPromise, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.PendingRootAlphaDivs(netuid)).toString());
}

export async function getTaoWeight(api: ApiPromise): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.TaoWeight()).toString());
}

export async function getSubnetAlphaIn(api: ApiPromise, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.SubnetAlphaIn(netuid)).toString());
}

export async function getTotalHotkeyAlpha(api: ApiPromise, hotkey: string, netuid: number): Promise<bigint> {
    return BigInt((await api.query.SubtensorModule.TotalHotkeyAlpha(hotkey, netuid)).toString());
}
