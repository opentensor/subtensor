import { ethers } from "ethers";

// Local node RPC URL for ethers
export const ETH_LOCAL_URL = "http://localhost:9944";

/**
 * Generate a random Ethereum wallet connected to the local node.
 */
export function generateRandomEthersWallet(): ethers.Wallet {
  const account = ethers.Wallet.createRandom();
  const provider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);
  return new ethers.Wallet(account.privateKey, provider);
}

/**
 * Create an ethers provider connected to the local node.
 */
export function getEthersProvider(): ethers.JsonRpcProvider {
  return new ethers.JsonRpcProvider(ETH_LOCAL_URL);
}
