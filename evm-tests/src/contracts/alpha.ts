import { parseAbi } from "viem";

export const IALPHA_ADDRESS = "0x0000000000000000000000000000000000000806";

export const IAlphaABI = parseAbi([
  // View functions
  "function getAlphaPrice(uint16 netuid) view returns (uint256)",
  "function getTaoInPool(uint16 netuid) view returns (uint64)",
  "function getAlphaInPool(uint16 netuid) view returns (uint64)",
]);
