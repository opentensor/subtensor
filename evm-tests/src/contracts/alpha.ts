import { parseAbi } from "viem";

export const IALPHA_ADDRESS = "0x0000000000000000000000000000000000000806";

export const IAlphaABI = parseAbi([
  // View functions
  "function getAlphaPrice(uint16 netuid) view returns (uint256)",
  "function getMovingAlphaPrice(uint16 netuid) view returns (uint256)",
  "function getTaoInPool(uint16 netuid) view returns (uint64)",
  "function getAlphaInPool(uint16 netuid) view returns (uint64)",
  "function getAlphaOutPool(uint16 netuid) view returns (uint64)",
  "function getAlphaIssuance(uint16 netuid) view returns (uint64)",
  "function getTaoWeight() view returns (uint256)",
  "function simSwapTaoForAlpha(uint16 netuid, uint64 tao) view returns (uint256)",
  "function simSwapAlphaForTao(uint16 netuid, uint64 alpha) view returns (uint256)",
  "function getSubnetMechanism(uint16 netuid) view returns (uint16)",
  "function getMinimumPoolLiquidity() view returns (uint256)",
  "function getRootNetuid() view returns (uint16)",
  "function getEMAPriceHalvingBlocks(uint16 netuid) view returns (uint64)",
  "function getSubnetVolume(uint16 netuid) view returns (uint256)",
]);
