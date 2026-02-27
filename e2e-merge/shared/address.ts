import { sr25519CreateDerive } from "@polkadot-labs/hdkd";
import { DEV_PHRASE, entropyToMiniSecret, mnemonicToEntropy, KeyPair } from "@polkadot-labs/hdkd-helpers";
import { getPolkadotSigner } from "polkadot-api/signer";
import { PolkadotSigner } from "polkadot-api";
import { randomBytes } from "crypto";
import { ss58Address } from "@polkadot-labs/hdkd-helpers";

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
