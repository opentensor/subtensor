import { Keyring } from "@polkadot/keyring";
import { blake2AsHex } from "@polkadot/util-crypto";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { waitForTransactionWithRetry } from "./transactions.js";

export const ANNOUNCEMENT_DELAY = 10;
export const REANNOUNCEMENT_DELAY = 10;

/** Compute BLAKE2-256 hash of a keypair's public key (used for announcements). */
export function coldkeyHash(pair: KeyringPair): string {
    return blake2AsHex(pair.publicKey, 256);
}

// ── Sudo configuration ──────────────────────────────────────────────────

export async function sudoSetAnnouncementDelay(api: ApiPromise, delay: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetColdkeySwapAnnouncementDelay(delay);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_coldkey_swap_announcement_delay");
}

export async function sudoSetReannouncementDelay(api: ApiPromise, delay: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.adminUtils.sudoSetColdkeySwapReannouncementDelay(delay);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_coldkey_swap_reannouncement_delay");
}

// ── Transaction wrappers (throw on failure) ─────────────────────────────

export async function announceColdkeySwap(api: ApiPromise, signer: KeyringPair, newColdkeyHash: string): Promise<void> {
    const tx = api.tx.subtensorModule.announceColdkeySwap(newColdkeyHash);
    await waitForTransactionWithRetry(api, tx, signer, "announce_coldkey_swap");
}

export async function swapColdkeyAnnounced(api: ApiPromise, signer: KeyringPair, newAddress: string): Promise<void> {
    const tx = api.tx.subtensorModule.swapColdkeyAnnounced(newAddress);
    await waitForTransactionWithRetry(api, tx, signer, "swap_coldkey_announced");
}

export async function disputeColdkeySwap(api: ApiPromise, signer: KeyringPair): Promise<void> {
    const tx = api.tx.subtensorModule.disputeColdkeySwap();
    await waitForTransactionWithRetry(api, tx, signer, "dispute_coldkey_swap");
}

export async function sudoResetColdkeySwap(api: ApiPromise, coldkeyAddress: string): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.subtensorModule.resetColdkeySwap(coldkeyAddress);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(api, tx, alice, "sudo_reset_coldkey_swap");
}

export async function sudoSwapColdkey(
    api: ApiPromise,
    oldAddress: string,
    newAddress: string,
    swapCost: bigint = 0n,
): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.subtensorModule.swapColdkey(oldAddress, newAddress, swapCost);
    const tx = api.tx.sudo.sudo(internalCall);
    await waitForTransactionWithRetry(api, tx, alice, "sudo_swap_coldkey");
}

// ── Storage query helpers ───────────────────────────────────────────────

export async function getColdkeySwapAnnouncement(
    api: ApiPromise,
    address: string,
): Promise<{ when: number; hash: string } | null> {
    const result = await api.query.subtensorModule.coldkeySwapAnnouncements(address);
    if (result.isEmpty) return null;
    const json = result.toJSON() as [number, string] | null;
    if (!json) return null;
    return { when: json[0], hash: json[1] };
}

export async function getColdkeySwapDispute(
    api: ApiPromise,
    address: string,
): Promise<number | null> {
    const result = await api.query.subtensorModule.coldkeySwapDisputes(address);
    if (result.isEmpty) return null;
    return Number(result.toString());
}

/** Get the owner coldkey of a hotkey. */
export async function getHotkeyOwner(api: ApiPromise, hotkey: string): Promise<string> {
    return (await api.query.subtensorModule.owner(hotkey)).toString();
}

/** Get the list of hotkeys owned by a coldkey. */
export async function getOwnedHotkeys(api: ApiPromise, coldkey: string): Promise<string[]> {
    const result = await api.query.subtensorModule.ownedHotkeys(coldkey);
    return (result.toJSON() as string[]) ?? [];
}

/** Get the list of hotkeys a coldkey is staking to. */
export async function getStakingHotkeys(api: ApiPromise, coldkey: string): Promise<string[]> {
    const result = await api.query.subtensorModule.stakingHotkeys(coldkey);
    return (result.toJSON() as string[]) ?? [];
}

/** Get the owner coldkey of a subnet. */
export async function getSubnetOwner(api: ApiPromise, netuid: number): Promise<string> {
    return (await api.query.subtensorModule.subnetOwner(netuid)).toString();
}
