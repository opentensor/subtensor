import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getSignerFromKeypair } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";

export async function addStake(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  netuid: number,
  amount: bigint
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.add_stake({
    hotkey: hotkey,
    netuid: netuid,
    amount_staked: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "add_stake");
}

export async function addStakeLimit(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  netuid: number,
  amount: bigint,
  limitPrice: bigint,
  allowPartial: boolean
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.add_stake_limit({
    hotkey: hotkey,
    netuid: netuid,
    amount_staked: amount,
    limit_price: limitPrice,
    allow_partial: allowPartial,
  });
  await waitForTransactionWithRetry(api, tx, signer, "add_stake_limit");
}

export async function removeStake(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  netuid: number,
  amount: bigint
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.remove_stake({
    hotkey: hotkey,
    netuid: netuid,
    amount_unstaked: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "remove_stake");
}

export async function removeStakeLimit(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  netuid: number,
  amount: bigint,
  limitPrice: bigint,
  allowPartial: boolean
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.remove_stake_limit({
    hotkey: hotkey,
    netuid: netuid,
    amount_unstaked: amount,
    limit_price: limitPrice,
    allow_partial: allowPartial,
  });
  await waitForTransactionWithRetry(api, tx, signer, "remove_stake_limit");
}

export async function removeStakeFullLimit(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  netuid: number,
  limitPrice: bigint | undefined
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.remove_stake_full_limit({
    hotkey: hotkey,
    netuid: netuid,
    limit_price: limitPrice,
  });
  await waitForTransactionWithRetry(api, tx, signer, "remove_stake_full_limit");
}

export async function unstakeAll(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.unstake_all({
    hotkey: hotkey,
  });
  await waitForTransactionWithRetry(api, tx, signer, "unstake_all");
}

export async function getStake(
  api: TypedApi<typeof devnet>,
  hotkey: string,
  coldkey: string,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.Alpha.getValue(hotkey, coldkey, netuid);
}

export async function transferStake(
  api: TypedApi<typeof devnet>,
  originColdkey: KeyPair,
  destinationColdkey: string,
  hotkey: string,
  originNetuid: number,
  destinationNetuid: number,
  amount: bigint
): Promise<void> {
  const signer = getSignerFromKeypair(originColdkey);
  const tx = api.tx.SubtensorModule.transfer_stake({
    destination_coldkey: destinationColdkey,
    hotkey: hotkey,
    origin_netuid: originNetuid,
    destination_netuid: destinationNetuid,
    alpha_amount: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "transfer_stake");
}

export async function moveStake(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  originHotkey: string,
  destinationHotkey: string,
  originNetuid: number,
  destinationNetuid: number,
  amount: bigint
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.move_stake({
    origin_hotkey: originHotkey,
    destination_hotkey: destinationHotkey,
    origin_netuid: originNetuid,
    destination_netuid: destinationNetuid,
    alpha_amount: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "move_stake");
}
