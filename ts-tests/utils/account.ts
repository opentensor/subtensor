import type { KeyringPair } from "@moonwall/util";
import type { PolkadotSigner, TypedApi } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";
import { getPolkadotSigner } from "polkadot-api/signer";
import { mnemonicGenerate } from "@polkadot/util-crypto";
import { Keyring } from "@polkadot/keyring";

export const getAccountNonce = async (api: TypedApi<typeof subtensor>, address: string): Promise<number> => {
    const account = await api.query.System.Account.getValue(address, { at: "best" });
    return account.nonce;
};

export function getSignerFromKeypair(keypair: KeyringPair): PolkadotSigner {
    return getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
}

export function generateKeyringPair(type: "sr25519" | "ed25519" = "sr25519"): KeyringPair {
    const keyring = new Keyring({ type });
    return keyring.addFromMnemonic(mnemonicGenerate());
}
