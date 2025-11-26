import { combineLatest, map, of, switchMap } from 'rxjs';
import { isHex, u8aToString } from '@polkadot/util';
import { firstMemo, memo } from '../util/index.js';
const UNDEF_HEX = { toHex: () => undefined };
function dataAsString(data) {
    if (!data) {
        return data;
    }
    return data.isRaw
        ? u8aToString(data.asRaw.toU8a(true))
        : data.isNone
            ? undefined
            : data.toHex();
}
function extractOther(additional) {
    return additional.reduce((other, [_key, _value]) => {
        const key = dataAsString(_key);
        const value = dataAsString(_value);
        if (key && value) {
            other[key] = value;
        }
        return other;
    }, {});
}
function identityCompat(identityOfOpt) {
    const identity = identityOfOpt.unwrap();
    return Array.isArray(identity)
        ? identity[0]
        : identity;
}
function extractIdentity(identityOfOpt, superOf) {
    if (!identityOfOpt?.isSome) {
        return { judgements: [] };
    }
    const { info, judgements } = identityCompat(identityOfOpt);
    const topDisplay = dataAsString(info.display);
    return {
        discord: dataAsString(info.discord),
        display: (superOf && dataAsString(superOf[1])) || topDisplay,
        displayParent: superOf && topDisplay,
        email: dataAsString(info.email),
        github: dataAsString(info.github),
        image: dataAsString(info.image),
        judgements,
        legal: dataAsString(info.legal),
        matrix: dataAsString(info.matrix),
        other: info.additional ? extractOther(info.additional) : {},
        parent: superOf?.[0],
        pgp: info.pgpFingerprint.unwrapOr(UNDEF_HEX).toHex(),
        riot: dataAsString(info.riot),
        twitter: dataAsString(info.twitter),
        web: dataAsString(info.web)
    };
}
function getParent(api, identityOfOpt, superOfOpt) {
    if (identityOfOpt?.isSome) {
        // this identity has something set
        return of([identityOfOpt, undefined]);
    }
    else if (superOfOpt?.isSome) {
        const superOf = superOfOpt.unwrap();
        return combineLatest([
            api.derive.accounts._identity(superOf[0]).pipe(map(([info]) => info)),
            of(superOf)
        ]);
    }
    // nothing of value returned
    return of([undefined, undefined]);
}
export function _identity(instanceId, api) {
    return memo(instanceId, (accountId) => accountId && api.query.identity?.identityOf
        ? combineLatest([
            api.query.identity.identityOf(accountId),
            api.query.identity.superOf(accountId)
        ])
        : of([undefined, undefined]));
}
/**
 * @name identity
 * @description Retrieves the on chain identity information for a given account.
 * @param {(AccountId | Uint8Array | string)} accoutId The account identifier to query the identity for.
 * @example
 * ```javascript
 * const ALICE = "13xAUH";
 *
 * api.derive.accounts.identity(ALICE, (identity) => {
 *   console.log(
 *     "Account Identity:",
 *     Object.keys(identity).map((key) => `${key}: ${identity[key]}`)
 *   );
 * });
 * ```
 */
export function identity(instanceId, api) {
    return memo(instanceId, (accountId) => api.derive.accounts._identity(accountId).pipe(switchMap(([identityOfOpt, superOfOpt]) => getParent(api, identityOfOpt, superOfOpt)), map(([identityOfOpt, superOf]) => extractIdentity(identityOfOpt, superOf)), switchMap((identity) => getSubIdentities(identity, api, accountId))));
}
function getSubIdentities(identity, api, accountId) {
    const targetAccount = identity.parent || accountId;
    if (!targetAccount || !api.query.identity) {
        // No valid accountId return the identity as-is
        return of(identity);
    }
    return api.query.identity.subsOf(targetAccount).pipe(map((subsResponse) => {
        const subs = subsResponse[1];
        return {
            ...identity,
            subs
        };
    }));
}
/**
 * @name hasIdentity
 * @description Checks if a specific account has an identity registered on chain.
 * @param {(AccountId | Uint8Array | string)} accoutId The account identifier to query.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * console.log(await api.derive.accounts.hasIdentity(ALICE));
 * ```
 */
export const hasIdentity = /*#__PURE__*/ firstMemo((api, accountId) => api.derive.accounts.hasIdentityMulti([accountId]));
/**
 * @name hasIdentityMulti
 * @description Checks whether multiple accounts have on chain identities registered.
 * @param {(AccountId | Uint8Array | string)[]} accountIds Array of account identifiers to query.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const BOB = "16WW";
 * console.log(await api.derive.accounts.hasIdentityMulti([ALICE, BOB]));
 * ```
 */
export function hasIdentityMulti(instanceId, api) {
    return memo(instanceId, (accountIds) => api.query.identity?.identityOf
        ? combineLatest([
            api.query.identity.identityOf.multi(accountIds),
            api.query.identity.superOf.multi(accountIds)
        ]).pipe(map(([identities, supers]) => identities.map((identityOfOpt, index) => {
            const superOfOpt = supers[index];
            const parentId = superOfOpt && superOfOpt.isSome
                ? superOfOpt.unwrap()[0].toString()
                : undefined;
            let display;
            if (identityOfOpt && identityOfOpt.isSome) {
                const value = dataAsString(identityCompat(identityOfOpt).info.display);
                if (value && !isHex(value)) {
                    display = value;
                }
            }
            return { display, hasIdentity: !!(display || parentId), parentId };
        })))
        : of(accountIds.map(() => ({ hasIdentity: false }))));
}
