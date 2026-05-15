import { Keyring } from "@polkadot/keyring";
import { waitForTransactionWithRetry } from "./transactions.js";
import type { TypedApi } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";

export async function sudoSetStakeThreshold(api: TypedApi<typeof subtensor>, threshold: bigint): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const inner = api.tx.AdminUtils.sudo_set_stake_threshold({ min_stake: threshold });
    const tx = api.tx.Sudo.sudo({ call: inner.decodedCall });
    await waitForTransactionWithRetry(api, tx, alice, "sudo_set_stake_threshold");
}
