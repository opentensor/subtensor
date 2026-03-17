import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { randomBytes } from "ethers";
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { MlKem768 } from "mlkem";
import { type TypedApi, Binary } from "polkadot-api";
import type { subtensor } from "@polkadot-api/descriptors";
import { getSignerFromKeypair } from "./account.ts";

export const getNextKey = async (api: ApiPromise): Promise<Uint8Array | undefined> => {
    const bestHeader = await api.rpc.chain.getHeader();
    const bestHash = bestHeader.hash;

    // Query at best block hash directly
    const key = await api.query.mevShield.nextKey.at(bestHash);
    if (key.isEmpty) return undefined;

    // BoundedVec<u8> decodes as Bytes/Vec<u8>
    const bytes = key.toU8a(true);
    return bytes.length > 0 ? bytes : undefined;
};

export const getCurrentKey = async (api: ApiPromise): Promise<Uint8Array | undefined> => {
    const bestHash = await api.rpc.chain.getBlockHash((await api.rpc.chain.getHeader()).number.toNumber());
    const key = await api.query.mevShield.currentKey.at(bestHash);
    if (key.isNone) return undefined;
    return key.unwrap().toU8a(true);
};

export const encryptTransaction = async (plaintext: Uint8Array, publicKey: Uint8Array): Promise<Uint8Array> => {
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
    signer: KeyringPair,
    innerTxBytes: Uint8Array,
    publicKey: Uint8Array,
    nonce?: number
) => {
    const ciphertext = await encryptTransaction(innerTxBytes, publicKey);
    return submitEncryptedRaw(api, signer, ciphertext, nonce);
};

export const submitEncryptedRaw = async (
    api: TypedApi<typeof subtensor>,
    signer: KeyringPair,
    ciphertext: Uint8Array,
    nonce?: number
) => {
    const tx = api.tx.MevShield.submit_encrypted({
        ciphertext: Binary.fromBytes(ciphertext),
    });
    return tx.signAndSubmit(getSignerFromKeypair(signer), {
        ...(nonce !== undefined ? { nonce } : {}),
        mortality: { mortal: true, period: 8 },
    });
};
