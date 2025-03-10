import { Address } from "viem"
import { encodeAddress } from "@polkadot/util-crypto";
import { ss58Address } from "@polkadot-labs/hdkd-helpers";
import { hexToU8a } from "@polkadot/util";
import { blake2AsU8a, decodeAddress } from "@polkadot/util-crypto";
import { Binary } from "polkadot-api";
import { SS58_PREFIX } from "./config"

export function toViemAddress(address: string): Address {
    let addressNoPrefix = address.replace("0x", "")
    return `0x${addressNoPrefix}`
}

export function convertH160ToSS58(ethAddress: string) {
    // get the public key
    const hash = convertH160ToPublicKey(ethAddress);

    // Convert the hash to SS58 format
    const ss58Address = encodeAddress(hash, SS58_PREFIX);
    return ss58Address;
}

export function convertPublicKeyToSs58(publickey: Uint8Array) {
    return ss58Address(publickey, SS58_PREFIX);
}

export function convertH160ToPublicKey(ethAddress: string) {
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
    const hash = blake2AsU8a(combined);
    return hash;
}

export function ss58ToEthAddress(ss58Address: string) {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = decodeAddress(ss58Address);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    const ethereumAddressBytes = publicKey.slice(0, 20);

    // Convert the 20 bytes into an Ethereum H160 address format (Hex string)
    const ethereumAddress = '0x' + Buffer.from(ethereumAddressBytes).toString('hex');

    return ethereumAddress;
}

export function ss58ToH160(ss58Address: string): Binary {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = decodeAddress(ss58Address);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    const ethereumAddressBytes = publicKey.slice(0, 20);


    return new Binary(ethereumAddressBytes);
}

export function ethAddressToH160(ethAddress: string): Binary {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = hexToU8a(ethAddress);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    // const ethereumAddressBytes = publicKey.slice(0, 20);


    return new Binary(publicKey);
}