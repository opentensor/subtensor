import { createBlockNumberDerive } from './util.js';
/**
 * @name bestNumber
 * @descrive Retrieves the latest block number.
 * @example
 * ```javascript
 * api.derive.chain.bestNumber((blockNumber) => {
 *   console.log(`the current best block is #${blockNumber}`);
 * });
 * ```
 */
export const bestNumber = /*#__PURE__*/ createBlockNumberDerive((api) => api.rpc.chain.subscribeNewHeads());
