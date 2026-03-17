import { waitForTransactionWithRetry } from "./transactions.js";
import { Keyring } from "@polkadot/keyring";
import type { ApiPromise } from "@polkadot/api";

export const TAO = BigInt(1000000000); // 10^9 RAO per TAO

export function tao(value: number): bigint {
    return TAO * BigInt(value);
}

export async function getBalance(api: ApiPromise, ss58Address: string): Promise<bigint> {
    const account = (await api.query.system.account(ss58Address)) as any; // TODO: fix any

    return account.data.free.toBigInt();
}

export async function forceSetBalance(api: ApiPromise, address: string, amount: bigint = tao(1e10)): Promise<void> {
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");
    const internalTx = api.tx.balances.forceSetBalance(address, amount);
    const tx = api.tx.sudo.sudo(internalTx);
    await waitForTransactionWithRetry(api, tx, alice, "force_set_balance");
}
