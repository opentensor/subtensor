import { ethers } from "ethers";

// Local node RPC URL for ethers
export const ETH_LOCAL_URL = "http://localhost:9944";

// Conversion constants
const ETH_PER_RAO = BigInt(1000000000); // 10^9

/**
 * Convert RAO to ETH units (multiply by 10^9).
 * V1 staking contract uses ETH units (10^18), while V2 uses RAO (10^9).
 */
export function raoToEth(value: bigint): bigint {
  return ETH_PER_RAO * value;
}

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
