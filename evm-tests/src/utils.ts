import { defineChain, http, publicActions, createPublicClient } from "viem"
import { privateKeyToAccount, generatePrivateKey } from 'viem/accounts'
import { ethers } from "ethers"
import { ETH_LOCAL_URL } from "./config"
import { FixedSizeBinary } from "polkadot-api";

export type ClientUrlType = 'http://localhost:9944';

export const chain = (id: number, url: string) => defineChain({
    id: id,
    name: 'bittensor',
    network: 'bittensor',
    nativeCurrency: {
        name: 'tao',
        symbol: 'TAO',
        decimals: 9,
    },
    rpcUrls: {
        default: {
            http: [url],
        },
    },
    testnet: true,
})


export async function getPublicClient(url: ClientUrlType) {
    const wallet = createPublicClient({
        chain: chain(42, url),
        transport: http(),

    })

    return wallet.extend(publicActions)
}

/**
 * Generates a random Ethereum wallet
 * @returns wallet keyring
 */
export function generateRandomEthWallet() {
    let privateKey = generatePrivateKey().toString();
    privateKey = privateKey.replace('0x', '');

    const account = privateKeyToAccount(`0x${privateKey}`)
    return account
}


export function generateRandomEthersWallet() {
    const account = ethers.Wallet.createRandom();
    const provider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);

    const wallet = new ethers.Wallet(account.privateKey, provider);
    return wallet;
}

export function convertToFixedSizeBinary<T extends number>(hexString: string, size: T): FixedSizeBinary<T> {
    // Convert hex string to a byte array
    const byteArray = hexStringToUint8Array(hexString);

    // Ensure the byte array is exactly the specified size
    if (byteArray.length !== size) {
        throw new Error(`The provided string "${hexString}" does not convert to exactly ${size} bytes.`);
    }

    return new FixedSizeBinary(byteArray);
}

export function hexStringToUint8Array(hexString: string): Uint8Array {
    if (hexString.startsWith('0x')) hexString = hexString.slice(2);
    if (hexString.length % 2 !== 0) hexString = '0' + hexString;
    const bytes = new Uint8Array(hexString.length / 2);
    for (let i = 0; i < bytes.length; i++) {
        bytes[i] = parseInt(hexString.substring(i * 2, i * 2 + 2), 16);
    }
    return bytes;
}
