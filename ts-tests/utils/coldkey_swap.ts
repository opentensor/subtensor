import { Keyring } from "@polkadot/keyring";
import { blake2AsHex } from "@polkadot/util-crypto";
import type { KeyringPair } from "@moonwall/util";
import { waitForTransactionWithRetry } from "./transactions.js";
import type { TypedApi } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";
import { FixedSizeBinary } from "polkadot-api";

export const ANNOUNCEMENT_DELAY = 20;
export const REANNOUNCEMENT_DELAY = 20;

/** Compute BLAKE2-256 hash of a keypair's public key as a FixedSizeBinary (used for announcement storage / events). */
export function coldkeyHashBinary(pair: KeyringPair): FixedSizeBinary<32> {
    return FixedSizeBinary.fromHex(blake2AsHex(pair.publicKey, 256));
}

/** Compute BLAKE2-256 hash of a keypair's public key as hex string. */
export function coldkeyHash(pair: KeyringPair): string {
    return blake2AsHex(pair.publicKey, 256);
}

// ── Sudo configuration ──────────────────────────────────────────────────

export async function sudoSetAnnouncementDelay(api: TypedApi<typeof subtensor>, delay: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.AdminUtils.sudo_set_coldkey_swap_announcement_delay({
        duration: delay,
    });
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_coldkey_swap_announcement_delay");
}

export async function sudoSetReannouncementDelay(api: TypedApi<typeof subtensor>, delay: number): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalCall = api.tx.AdminUtils.sudo_set_coldkey_swap_reannouncement_delay({
        duration: delay,
    });
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_coldkey_swap_reannouncement_delay");
}

// ── Transaction wrappers (throw on failure) ─────────────────────────────

export async function announceColdkeySwap(
    api: TypedApi<typeof subtensor>,
    signer: KeyringPair,
    newColdkey: KeyringPair | string
): Promise<void> {
    const newAddress = typeof newColdkey === "string" ? newColdkey : newColdkey.address;
    const tx = api.tx.SubtensorModule.announce_coldkey_swap({
        new_coldkey: newAddress,
    });
    await waitForTransactionWithRetry(api, tx, signer, "announce_coldkey_swap");
}

export async function swapColdkeyAnnounced(
    api: TypedApi<typeof subtensor>,
    signer: KeyringPair,
    newAddress: string
): Promise<void> {
    const tx = api.tx.SubtensorModule.swap_coldkey_announced({
        new_coldkey: newAddress,
    });
    await waitForTransactionWithRetry(api, tx, signer, "swap_coldkey_announced");
}

export async function disputeColdkeySwap(api: TypedApi<typeof subtensor>, signer: KeyringPair): Promise<void> {
    const tx = api.tx.SubtensorModule.dispute_coldkey_swap();
    await waitForTransactionWithRetry(api, tx, signer, "dispute_coldkey_swap");
}

export async function clearColdkeySwapAnnouncement(
    api: TypedApi<typeof subtensor>,
    signer: KeyringPair
): Promise<void> {
    const tx = api.tx.SubtensorModule.clear_coldkey_swap_announcement();
    await waitForTransactionWithRetry(api, tx, signer, "clear_coldkey_swap_announcement");
}
