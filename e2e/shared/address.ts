import { sr25519CreateDerive } from "@polkadot-labs/hdkd";
import { DEV_PHRASE, entropyToMiniSecret, mnemonicToEntropy, KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getPolkadotSigner } from "polkadot-api/signer";
import { PolkadotSigner } from "polkadot-api";
import { randomBytes } from "crypto";
import { ss58Address } from "@polkadot-labs/hdkd-helpers";
import { encodeAddress, blake2AsU8a } from "@polkadot/util-crypto";
import { hexToU8a } from "@polkadot/util";

export const SS58_PREFIX = 42;

// ─── KEYPAIR UTILITIES ───────────────────────────────────────────────────────

export function getKeypairFromPath(path: string): KeyPair {
  const entropy = mnemonicToEntropy(DEV_PHRASE);
  const miniSecret = entropyToMiniSecret(entropy);
  const derive = sr25519CreateDerive(miniSecret);
  return derive(path);
}

export const getAlice = () => getKeypairFromPath("//Alice");

export function getRandomSubstrateKeypair(): KeyPair {
  const seed = randomBytes(32);
  const miniSecret = entropyToMiniSecret(seed);
  const derive = sr25519CreateDerive(miniSecret);
  return derive("");
}

// ─── SIGNER UTILITIES ────────────────────────────────────────────────────────

export function getSignerFromKeypair(keypair: KeyPair): PolkadotSigner {
  return getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
}

export function getSignerFromPath(path: string): PolkadotSigner {
  return getSignerFromKeypair(getKeypairFromPath(path));
}

export const getAliceSigner = () => getSignerFromPath("//Alice");

// ─── ADDRESS UTILITIES ───────────────────────────────────────────────────────

export function convertPublicKeyToSs58(publicKey: Uint8Array): string {
  return ss58Address(publicKey, SS58_PREFIX);
}

// ─── EVM ADDRESS UTILITIES ──────────────────────────────────────────────────

/**
 * Convert an Ethereum H160 address to a Substrate public key.
 * Uses the "evm:" prefix and blake2 hash.
 */
export function convertH160ToPublicKey(ethAddress: string): Uint8Array {
  const prefix = "evm:";
  const prefixBytes = new TextEncoder().encode(prefix);
  const addressBytes = hexToU8a(
    ethAddress.startsWith("0x") ? ethAddress : `0x${ethAddress}`
  );
  const combined = new Uint8Array(prefixBytes.length + addressBytes.length);

  // Concatenate prefix and Ethereum address
  combined.set(prefixBytes);
  combined.set(addressBytes, prefixBytes.length);

  // Hash the combined data (the public key)
  return blake2AsU8a(combined);
}

/**
 * Convert an Ethereum H160 address to SS58 format.
 */
export function convertH160ToSS58(ethAddress: string): string {
  const publicKey = convertH160ToPublicKey(ethAddress);
  return encodeAddress(publicKey, SS58_PREFIX);
}
