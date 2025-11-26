import { createBlockNumberDerive } from './util.js';
/**
 * @name bestNumberFinalized
 * @returns A BlockNumber
 * @description Get the latest finalized block number.
 * @example
 * ```javascript
 * api.derive.chain.bestNumberFinalized((blockNumber) => {
 *   console.log(`the current finalized block is #${blockNumber}`);
 * });
 * ```
 */
export const bestNumberFinalized = /*#__PURE__*/ createBlockNumberDerive((api) => api.rpc.chain.subscribeFinalizedHeads());
