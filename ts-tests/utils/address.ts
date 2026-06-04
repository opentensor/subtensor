import { hexToU8a } from "@polkadot/util";
import { blake2AsU8a, encodeAddress } from "@polkadot/util-crypto";

const SS58_PREFIX = 42;

export function convertH160ToPublicKey(ethAddress: string) {
    const prefix = "evm:";
    const prefixBytes = new TextEncoder().encode(prefix);
    const addressBytes = hexToU8a(
        ethAddress.startsWith("0x") ? ethAddress : `0x${ethAddress}`,
    );
    const combined = new Uint8Array(prefixBytes.length + addressBytes.length);

    // Concatenate prefix and Ethereum address
    combined.set(prefixBytes);
    combined.set(addressBytes, prefixBytes.length);

    // Hash the combined data (the public key)
    const hash = blake2AsU8a(combined);
    return hash;
}

export function convertH160ToSS58(ethAddress: string) {
    // get the public key
    const hash = convertH160ToPublicKey(ethAddress);

    // Convert the hash to SS58 format
    const ss58Address = encodeAddress(hash, SS58_PREFIX);
    return ss58Address;
}
