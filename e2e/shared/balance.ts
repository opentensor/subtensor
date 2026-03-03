import { subtensor, MultiAddress } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { getAliceSigner, convertH160ToSS58 } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";

export const TAO = BigInt(1000000000); // 10^9 RAO per TAO
export const ETH_PER_RAO = BigInt(1000000000); // 10^9 for ETH conversion

export function tao(value: number): bigint {
  return TAO * BigInt(value);
}

export async function getBalance(api: TypedApi<typeof subtensor>, ss58Address: string): Promise<bigint> {
  const account = await api.query.System.Account.getValue(ss58Address);
  return account.data.free;
}

export async function forceSetBalance(
  api: TypedApi<typeof subtensor>,
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

/**
 * Force set balance for an Ethereum H160 address.
 * Converts the ETH address to SS58 and sets the balance.
 */
export async function forceSetBalanceToEthAddress(
  api: TypedApi<typeof subtensor>,
  ethAddress: string,
  amount: bigint = tao(1e10)
): Promise<void> {
  const ss58Address = convertH160ToSS58(ethAddress);
  await forceSetBalance(api, ss58Address, amount);
}
