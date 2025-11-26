import { combineLatest, map, of, switchMap } from 'rxjs';
import { u8aToString } from '@polkadot/util';
import { memo } from '../util/index.js';
function retrieveNick(api, accountId) {
    return (accountId && api.query['nicks']?.['nameOf']
        ? api.query['nicks']['nameOf'](accountId)
        : of(undefined)).pipe(map((nameOf) => nameOf?.isSome
        ? u8aToString(nameOf.unwrap()[0]).substring(0, api.consts['nicks']['maxLength'].toNumber())
        : undefined));
}
/**
 * @name info
 * @description Returns aux. info with regards to an account, current that includes the accountId, accountIndex, identity and nickname
 * @param {(AccountIndex | AccountId | Address | Uint8Array | string | null)} address An accounts in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const info = await api.derive.accounts.info(ALICE);
 * console.log(
 *   "Account Info: ",
 *   Object.keys(info).map((key) => `${key}: ${info[key]}`)
 * );
 * ```
 */
export function info(instanceId, api) {
    return memo(instanceId, (address) => api.derive.accounts.idAndIndex(address).pipe(switchMap(([accountId, accountIndex]) => combineLatest([
        of({ accountId, accountIndex }),
        api.derive.accounts.identity(accountId),
        retrieveNick(api, accountId)
    ])), map(([{ accountId, accountIndex }, identity, nickname]) => ({
        accountId, accountIndex, identity, nickname
    }))));
}
