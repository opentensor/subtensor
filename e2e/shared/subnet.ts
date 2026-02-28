import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getAliceSigner, getSignerFromKeypair, convertPublicKeyToSs58 } from "./address.js";
import { waitForTransactionWithRetry } from "./transactions.js";
import { log } from "./logger.js";

export async function addNewSubnetwork(
  api: TypedApi<typeof devnet>,
  hotkey: KeyPair,
  coldkey: KeyPair
): Promise<number> {
  const alice = getAliceSigner();
  const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue();

  // Disable network rate limit for testing
  const rateLimit = await api.query.SubtensorModule.NetworkRateLimit.getValue();
  if (rateLimit !== BigInt(0)) {
    const internalCall = api.tx.AdminUtils.sudo_set_network_rate_limit({ rate_limit: BigInt(0) });
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
  api: TypedApi<typeof devnet>,
  netuid: number,
  hotkeyAddress: string,
  coldkey: KeyPair
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
  api: TypedApi<typeof devnet>,
  netuid: number,
  coldkey: KeyPair
): Promise<void> {
  const registerBlock = Number(await api.query.SubtensorModule.NetworkRegisteredAt.getValue(netuid));
  let currentBlock = await api.query.System.Number.getValue();
  const duration = Number(await api.constants.SubtensorModule.InitialStartCallDelay);

  while (currentBlock - registerBlock <= duration) {
    await new Promise((resolve) => setTimeout(resolve, 2000));
    currentBlock = await api.query.System.Number.getValue();
  }

  await new Promise((resolve) => setTimeout(resolve, 2000));

  const signer = getSignerFromKeypair(coldkey);
  const tx = api.tx.SubtensorModule.start_call({ netuid: netuid });
  await waitForTransactionWithRetry(api, tx, signer, "start_call");

  await new Promise((resolve) => setTimeout(resolve, 1000));
}
