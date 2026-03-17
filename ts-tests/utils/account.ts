import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import type { PolkadotSigner } from "polkadot-api";
import { getPolkadotSigner } from "polkadot-api/signer";
import { mnemonicGenerate } from "@polkadot/util-crypto";
import { Keyring } from "@polkadot/keyring";

export const getAccountNonce = async (api: ApiPromise, address: string): Promise<number> => {
    return (await api.query.system.account(address)).nonce.toNumber();
};

export function getSignerFromKeypair(keypair: KeyringPair): PolkadotSigner {
    return getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
}

export function generateKeyringPair(type: "sr25519" | "ed25519" = "sr25519"): KeyringPair {
    const keyring = new Keyring({ type });
    return keyring.addFromMnemonic(mnemonicGenerate());
}
