import { DedotClient } from "dedot";
import { hexToU8a } from "@polkadot/util";
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { randomBytes } from "@noble/ciphers/utils.js";
import { MlKem768 } from "mlkem";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { NodeSubtensorApi } from "../node-subtensor/index.js";

export { connectClient, createKeyring, getAccountNonce, getBalance } from "e2e-shared/client.js";

export const getNextKey = async (
  client: DedotClient<NodeSubtensorApi>,
): Promise<Uint8Array | undefined> => {
  const key = await client.query.mevShield.nextKey();
  if (!key) return undefined;
  return hexToU8a(key);
};

export const getCurrentKey = async (
  client: DedotClient<NodeSubtensorApi>,
): Promise<Uint8Array | undefined> => {
  const key = await client.query.mevShield.currentKey();
  if (!key) return undefined;
  return hexToU8a(key);
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
  client: DedotClient<NodeSubtensorApi>,
  signer: KeyringPair,
  innerTxBytes: Uint8Array,
  publicKey: Uint8Array,
  nonce?: number,
) => {
  const ciphertext = await encryptTransaction(innerTxBytes, publicKey);
  return submitEncryptedRaw(client, signer, ciphertext, nonce);
};

export const submitEncryptedRaw = async (
  client: DedotClient<NodeSubtensorApi>,
  signer: KeyringPair,
  ciphertext: Uint8Array,
  nonce?: number,
) => {
  const tx = client.tx.mevShield.submitEncrypted(ciphertext);
  const signed = await tx.sign(signer, nonce !== undefined ? { nonce } : {});

  return signed.send().untilFinalized();
};
