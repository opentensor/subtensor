import { subtensor } from "@polkadot-api/descriptors";
import { Enum, TypedApi } from "polkadot-api";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getAliceSigner, getSignerFromKeypair, convertPublicKeyToSs58 } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";
import { log } from "./logger.js";

export async function addNewSubnetwork(
  api: TypedApi<typeof subtensor>,
  hotkey: KeyPair,
  coldkey: KeyPair,
): Promise<number> {
  const alice = getAliceSigner();
  const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue();

  // Disable register-network rate limiting for test setup via the new grouped target.
  const target = Enum("Group", 3);
  const limits = (await api.query.RateLimiting.Limits.getValue(target as never)) as any;
  const rateLimit =
    limits?.type === "Global" && limits.value?.type === "Exact"
      ? BigInt(limits.value.value)
      : BigInt(0);

  if (rateLimit !== BigInt(0)) {
    const internalCall = api.tx.RateLimiting.set_rate_limit({
      target: target as never,
      scope: undefined,
      limit: Enum("Exact", 0) as never,
    });
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall });
    await waitForTransactionWithRetry(api, tx, alice, "set_network_rate_limit");
  }

  const signer = getSignerFromKeypair(coldkey);
  const registerNetworkTx = api.tx.SubtensorModule.register_network({
    hotkey: convertPublicKeyToSs58(hotkey.publicKey),
  });
  await waitForTransactionWithRetry(api, registerNetworkTx, signer, "register_network");

  return totalNetworks;
}

export async function burnedRegister(
  api: TypedApi<typeof subtensor>,
  netuid: number,
  hotkeyAddress: string,
  coldkey: KeyPair,
): Promise<void> {
  const registered = await api.query.SubtensorModule.Uids.getValue(netuid, hotkeyAddress);
  if (registered !== undefined) {
    log.tx("burned_register", `skipped: hotkey already registered on netuid ${netuid}`);
    return;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.burned_register({ hotkey: hotkeyAddress, netuid: netuid });
  await waitForTransactionWithRetry(api, tx, signer, "burned_register");
}

export async function startCall(
  api: TypedApi<typeof subtensor>,
  netuid: number,
  coldkey: KeyPair,
): Promise<void> {
  const existingFirstEmission = await api.query.SubtensorModule.FirstEmissionBlockNumber.getValue(
    netuid,
  );
  if (existingFirstEmission !== undefined) {
    return;
  }

  const registerBlock = Number(
    await api.query.SubtensorModule.NetworkRegisteredAt.getValue(netuid),
  );
  let currentBlock = await api.query.System.Number.getValue();
  const duration = Number(await api.constants.SubtensorModule.InitialStartCallDelay);

  while (currentBlock - registerBlock <= duration) {
    await new Promise((resolve) => setTimeout(resolve, 2000));
    currentBlock = await api.query.System.Number.getValue();
  }

  await new Promise((resolve) => setTimeout(resolve, 2000));

  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.start_call({ netuid: netuid });
  try {
    await waitForTransactionWithRetry(api, tx, signer, "start_call");
  } catch (error) {
    if (
      error instanceof Error &&
      error.message.includes("FirstEmissionBlockNumberAlreadySet")
    ) {
      return;
    }
    throw error;
  }

  await new Promise((resolve) => setTimeout(resolve, 1000));
}
