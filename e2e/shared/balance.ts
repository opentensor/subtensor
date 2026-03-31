import { subtensor, MultiAddress } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { getAliceSigner } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";

export const TAO = BigInt(1000000000); // 10^9 RAO per TAO

export function tao(value: number): bigint {
  return TAO * BigInt(value);
}

export async function getBalance(
  api: TypedApi<typeof subtensor>,
  ss58Address: string,
): Promise<bigint> {
  const account = await api.query.System.Account.getValue(ss58Address);
  return account.data.free;
}

export async function forceSetBalance(
  api: TypedApi<typeof subtensor>,
  ss58Address: string,
  amount: bigint = tao(1e10),
): Promise<void> {
  await forceSetBalances(api, [ss58Address], amount);
}

export async function forceSetBalances(
  api: TypedApi<typeof subtensor>,
  ss58Addresses: string[],
  amount: bigint = tao(1e10),
): Promise<void> {
  const alice = getAliceSigner();
  const calls = ss58Addresses.map((ss58Address) =>
    api.tx.Balances.force_set_balance({
      who: MultiAddress.Id(ss58Address),
      new_free: amount,
    }).decodedCall,
  );
  const batch = api.tx.Utility.force_batch({ calls });
  const tx = api.tx.Sudo.sudo({ call: batch.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "force_set_balance");
}
