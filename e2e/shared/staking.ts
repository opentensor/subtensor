import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getSignerFromKeypair, getAliceSigner } from "./address.js";
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

export async function unstakeAllAlpha(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.unstake_all_alpha({
    hotkey: hotkey,
  });
  await waitForTransactionWithRetry(api, tx, signer, "unstake_all_alpha");
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

export async function swapStake(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  originNetuid: number,
  destinationNetuid: number,
  amount: bigint
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.swap_stake({
    hotkey: hotkey,
    origin_netuid: originNetuid,
    destination_netuid: destinationNetuid,
    alpha_amount: amount,
  });
  await waitForTransactionWithRetry(api, tx, signer, "swap_stake");
}

export async function swapStakeLimit(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  hotkey: string,
  originNetuid: number,
  destinationNetuid: number,
  amount: bigint,
  limitPrice: bigint,
  allowPartial: boolean
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.swap_stake_limit({
    hotkey: hotkey,
    origin_netuid: originNetuid,
    destination_netuid: destinationNetuid,
    alpha_amount: amount,
    limit_price: limitPrice,
    allow_partial: allowPartial,
  });
  await waitForTransactionWithRetry(api, tx, signer, "swap_stake_limit");
}

export type RootClaimType = "Swap" | "Keep" | { type: "KeepSubnets"; subnets: number[] };

export async function getRootClaimType(
  api: TypedApi<typeof devnet>,
  coldkey: string
): Promise<RootClaimType> {
  const result = await api.query.SubtensorModule.RootClaimType.getValue(coldkey);
  if (result.type === "KeepSubnets") {
    return { type: "KeepSubnets", subnets: result.value.subnets as number[] };
  }
  return result.type as "Swap" | "Keep";
}

export async function setRootClaimType(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  claimType: RootClaimType
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  let newRootClaimType;
  if (typeof claimType === "string") {
    newRootClaimType = { type: claimType, value: undefined };
  } else {
    newRootClaimType = { type: "KeepSubnets", value: { subnets: claimType.subnets } };
  }
  const tx = api.tx.SubtensorModule.set_root_claim_type({
    new_root_claim_type: newRootClaimType,
  });
  await waitForTransactionWithRetry(api, tx, signer, "set_root_claim_type");
}

export async function claimRoot(
  api: TypedApi<typeof devnet>,
  coldkey: KeyPair,
  subnets: number[]
): Promise<void> {
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.claim_root({
    subnets: subnets,
  });
  await waitForTransactionWithRetry(api, tx, signer, "claim_root");
}

export async function getNumRootClaims(
  api: TypedApi<typeof devnet>
): Promise<bigint> {
  return await api.query.SubtensorModule.NumRootClaim.getValue();
}

export async function sudoSetNumRootClaims(
  api: TypedApi<typeof devnet>,
  newValue: bigint
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.SubtensorModule.sudo_set_num_root_claims({
    new_value: newValue,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_num_root_claims");
}

export async function getRootClaimThreshold(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.RootClaimableThreshold.getValue(netuid);
}

export async function sudoSetRootClaimThreshold(
  api: TypedApi<typeof devnet>,
  netuid: number,
  newValue: bigint
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.SubtensorModule.sudo_set_root_claim_threshold({
    netuid: netuid,
    new_value: newValue,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_root_claim_threshold");
}

export async function getTempo(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<number> {
  return await api.query.SubtensorModule.Tempo.getValue(netuid);
}

export async function sudoSetTempo(
  api: TypedApi<typeof devnet>,
  netuid: number,
  tempo: number
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_tempo({
    netuid: netuid,
    tempo: tempo,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_tempo");
}

export async function waitForBlocks(
  api: TypedApi<typeof devnet>,
  numBlocks: number
): Promise<void> {
  const startBlock = await api.query.System.Number.getValue();
  const targetBlock = startBlock + numBlocks;

  while (true) {
    const currentBlock = await api.query.System.Number.getValue();
    if (currentBlock >= targetBlock) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

export async function getRootClaimable(
  api: TypedApi<typeof devnet>,
  hotkey: string
): Promise<Map<number, bigint>> {
  const result = await api.query.SubtensorModule.RootClaimable.getValue(hotkey);
  const claimableMap = new Map<number, bigint>();
  for (const [netuid, amount] of result) {
    claimableMap.set(netuid, amount);
  }
  return claimableMap;
}

export async function getRootClaimed(
  api: TypedApi<typeof devnet>,
  netuid: number,
  hotkey: string,
  coldkey: string
): Promise<bigint> {
  return await api.query.SubtensorModule.RootClaimed.getValue(netuid, hotkey, coldkey);
}

export async function isSubtokenEnabled(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<boolean> {
  return await api.query.SubtensorModule.SubtokenEnabled.getValue(netuid);
}

export async function sudoSetSubtokenEnabled(
  api: TypedApi<typeof devnet>,
  netuid: number,
  enabled: boolean
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_subtoken_enabled({
    netuid: netuid,
    subtoken_enabled: enabled,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_subtoken_enabled");
}

export async function isNetworkAdded(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<boolean> {
  return await api.query.SubtensorModule.NetworksAdded.getValue(netuid);
}

export async function getAdminFreezeWindow(
  api: TypedApi<typeof devnet>
): Promise<number> {
  return await api.query.SubtensorModule.AdminFreezeWindow.getValue();
}

export async function sudoSetAdminFreezeWindow(
  api: TypedApi<typeof devnet>,
  window: number
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_admin_freeze_window({
    window: window,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_admin_freeze_window");
}

export async function sudoSetEmaPriceHalvingPeriod(
  api: TypedApi<typeof devnet>,
  netuid: number,
  emaPriceHalvingPeriod: number
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_ema_price_halving_period({
    netuid: netuid,
    ema_halving: BigInt(emaPriceHalvingPeriod),
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_ema_price_halving_period");
}

export async function sudoSetLockReductionInterval(
  api: TypedApi<typeof devnet>,
  interval: number
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_lock_reduction_interval({
    interval: BigInt(interval),
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_lock_reduction_interval");
}

export async function sudoSetSubnetMovingAlpha(
  api: TypedApi<typeof devnet>,
  alpha: bigint
): Promise<void> {
  const alice = getAliceSigner();
  const internalCall = api.tx.AdminUtils.sudo_set_subnet_moving_alpha({
    alpha: alpha,
  });
  const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
  await waitForTransactionWithRetry(api, tx, alice, "sudo_set_subnet_moving_alpha");
}

// Debug helpers for claim_root investigation
export async function getSubnetTAO(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.SubnetTAO.getValue(netuid);
}

export async function getSubnetMovingPrice(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.SubnetMovingPrice.getValue(netuid);
}

export async function getPendingRootAlphaDivs(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.PendingRootAlphaDivs.getValue(netuid);
}

export async function getTaoWeight(
  api: TypedApi<typeof devnet>
): Promise<bigint> {
  return await api.query.SubtensorModule.TaoWeight.getValue();
}

export async function getSubnetAlphaIn(
  api: TypedApi<typeof devnet>,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.SubnetAlphaIn.getValue(netuid);
}

export async function getTotalHotkeyAlpha(
  api: TypedApi<typeof devnet>,
  hotkey: string,
  netuid: number
): Promise<bigint> {
  return await api.query.SubtensorModule.TotalHotkeyAlpha.getValue(hotkey, netuid);
}
