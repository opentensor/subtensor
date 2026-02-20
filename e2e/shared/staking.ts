import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getSignerFromKeypair } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";

export async function addStake(
  api: TypedApi<typeof devnet>,
  netuid: number,
  hotkeyAddress: string,
  amount: bigint,
  coldkey: KeyPair
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.add_stake({
    netuid: netuid,
    hotkey: hotkeyAddress,
    amount_staked: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "add_stake");
}

export async function addStakeLimit(
  api: TypedApi<typeof devnet>,
  netuid: number,
  hotkeyAddress: string,
  amount: bigint,
  limitPrice: bigint,
  allowPartial: boolean,
  coldkey: KeyPair
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.add_stake_limit({
    netuid: netuid,
    hotkey: hotkeyAddress,
    amount_staked: amount,
    limit_price: limitPrice,
    allow_partial: allowPartial,
  });
  await waitForTransactionWithRetry(api, tx, signer, "add_stake_limit");
}

export async function getStake(
  api: TypedApi<typeof devnet>,
  hotkeyAddress: string,
  coldkeyAddress: string,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.Alpha.getValue(hotkeyAddress, coldkeyAddress, netuid);
}
