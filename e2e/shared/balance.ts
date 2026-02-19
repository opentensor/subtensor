import { devnet, MultiAddress } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { getAliceSigner } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";

export const TAO = BigInt(1000000000); // 10^9 RAO per TAO

export function tao(value: number): bigint {
  return TAO * BigInt(value);
}

export async function forceSetBalance(
  api: TypedApi<typeof devnet>,
  ss58Address: string,
  amount: bigint = tao(1e10)
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.Balances.force_set_balance({
    who: MultiAddress.Id(ss58Address),
    new_free: amount,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "force_set_balance");
}
