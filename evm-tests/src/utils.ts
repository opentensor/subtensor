import { defineChain, http, publicActions, createPublicClient } from "viem"
import { privateKeyToAccount, generatePrivateKey } from 'viem/accounts'
import { ethers } from "ethers"
import { ETH_LOCAL_URL } from "./config"

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