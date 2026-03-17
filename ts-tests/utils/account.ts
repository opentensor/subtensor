import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import type { PolkadotSigner } from "polkadot-api";
import { getPolkadotSigner } from "polkadot-api/signer";

export const getAccountNonce = async (api: ApiPromise, address: string): Promise<number> => {
    return (await api.query.system.account(address)).nonce.toNumber();
};

export function getSignerFromKeypair(keypair: KeyringPair): PolkadotSigner {
    return getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign);
}
