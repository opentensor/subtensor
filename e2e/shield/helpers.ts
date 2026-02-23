import type { TypedApi, PolkadotSigner } from "polkadot-api";
import { Binary } from "polkadot-api";
import { hexToU8a } from "@polkadot/util";
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { randomBytes } from "@noble/ciphers/utils.js";
import { MlKem768 } from "mlkem";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import type { subtensor } from "@polkadot-api/descriptors";

export const getNextKey = async (
  api: TypedApi<typeof subtensor>,
): Promise<Uint8Array | undefined> => {
  // Query at "best" (not default "finalized") because keys rotate every block
  // and finalized lags ~2 blocks behind best with GRANDPA. Using finalized
  // would return a stale key whose hash won't match CurrentKey/NextKey at
  // block-building time, causing InvalidShieldedTxPubKeyHash rejection.
  const key = await api.query.MevShield.NextKey.getValue({ at: "best" });
  if (!key) return undefined;
  if (key instanceof Binary) return key.asBytes();
  return hexToU8a(key as string);
};

export const getCurrentKey = async (
  api: TypedApi<typeof subtensor>,
): Promise<Uint8Array | undefined> => {
  const key = await api.query.MevShield.CurrentKey.getValue({ at: "best" });
  if (!key) return undefined;
  if (key instanceof Binary) return key.asBytes();
  return hexToU8a(key as string);
};

export const encryptTransaction = async (
  plaintext: Uint8Array,
  publicKey: Uint8Array,
): Promise<Uint8Array> => {
  const keyHash = xxhashAsU8a(publicKey, 128);

  const mlKem = new MlKem768();
  const [kemCt, sharedSecret] = await mlKem.encap(publicKey);

  const nonce = randomBytes(24);
  const chacha = xchacha20poly1305(sharedSecret, nonce);
  const aeadCt = chacha.encrypt(plaintext);

  const kemLenBytes = new Uint8Array(2);
  new DataView(kemLenBytes.buffer).setUint16(0, kemCt.length, true);

  return new Uint8Array([...keyHash, ...kemLenBytes, ...kemCt, ...nonce, ...aeadCt]);
};

export const submitEncrypted = async (
  api: TypedApi<typeof subtensor>,
  signer: PolkadotSigner,
  innerTxBytes: Uint8Array,
  publicKey: Uint8Array,
  nonce?: number,
) => {
  const ciphertext = await encryptTransaction(innerTxBytes, publicKey);
  return submitEncryptedRaw(api, signer, ciphertext, nonce);
};

export const submitEncryptedRaw = async (
  api: TypedApi<typeof subtensor>,
  signer: PolkadotSigner,
  ciphertext: Uint8Array,
  nonce?: number,
) => {
  const tx = api.tx.MevShield.submit_encrypted({
    ciphertext: Binary.fromBytes(ciphertext),
  });
  return tx.signAndSubmit(signer, nonce !== undefined ? { nonce } : {});
};
