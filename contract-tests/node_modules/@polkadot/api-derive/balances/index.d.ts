import { all } from './all.js';
export * from './account.js';
export * from './votingBalances.js';
/**
 * @name votingBalance
 * @param {( AccountId | string )} address An accounts Id in different formats.
 * @returns An object containing the results of various balance queries
 * @example
 * <BR>
 *
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.votingBalance(ALICE, ({ accountId, lockedBalance }) => {
 *   console.log(`The account ${accountId} has a locked balance ${lockedBalance} units.`);
 * });
 * ```
 */
declare const votingBalance: typeof all;
export { all, votingBalance };
