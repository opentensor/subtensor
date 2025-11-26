"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hasIdentity = void 0;
exports._identity = _identity;
exports.identity = identity;
exports.hasIdentityMulti = hasIdentityMulti;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const UNDEF_HEX = { toHex: () => undefined };
function dataAsString(data) {
    if (!data) {
        return data;
    }
    return data.isRaw
        ? (0, util_1.u8aToString)(data.asRaw.toU8a(true))
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
        return (0, rxjs_1.of)([identityOfOpt, undefined]);
    }
    else if (superOfOpt?.isSome) {
        const superOf = superOfOpt.unwrap();
        return (0, rxjs_1.combineLatest)([
            api.derive.accounts._identity(superOf[0]).pipe((0, rxjs_1.map)(([info]) => info)),
            (0, rxjs_1.of)(superOf)
        ]);
    }
    // nothing of value returned
    return (0, rxjs_1.of)([undefined, undefined]);
}
function _identity(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => accountId && api.query.identity?.identityOf
        ? (0, rxjs_1.combineLatest)([
            api.query.identity.identityOf(accountId),
            api.query.identity.superOf(accountId)
        ])
        : (0, rxjs_1.of)([undefined, undefined]));
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
function identity(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => api.derive.accounts._identity(accountId).pipe((0, rxjs_1.switchMap)(([identityOfOpt, superOfOpt]) => getParent(api, identityOfOpt, superOfOpt)), (0, rxjs_1.map)(([identityOfOpt, superOf]) => extractIdentity(identityOfOpt, superOf)), (0, rxjs_1.switchMap)((identity) => getSubIdentities(identity, api, accountId))));
}
function getSubIdentities(identity, api, accountId) {
    const targetAccount = identity.parent || accountId;
    if (!targetAccount || !api.query.identity) {
        // No valid accountId return the identity as-is
        return (0, rxjs_1.of)(identity);
    }
    return api.query.identity.subsOf(targetAccount).pipe((0, rxjs_1.map)((subsResponse) => {
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
exports.hasIdentity = (0, index_js_1.firstMemo)((api, accountId) => api.derive.accounts.hasIdentityMulti([accountId]));
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
function hasIdentityMulti(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIds) => api.query.identity?.identityOf
        ? (0, rxjs_1.combineLatest)([
            api.query.identity.identityOf.multi(accountIds),
            api.query.identity.superOf.multi(accountIds)
        ]).pipe((0, rxjs_1.map)(([identities, supers]) => identities.map((identityOfOpt, index) => {
            const superOfOpt = supers[index];
            const parentId = superOfOpt && superOfOpt.isSome
                ? superOfOpt.unwrap()[0].toString()
                : undefined;
            let display;
            if (identityOfOpt && identityOfOpt.isSome) {
                const value = dataAsString(identityCompat(identityOfOpt).info.display);
                if (value && !(0, util_1.isHex)(value)) {
                    display = value;
                }
            }
            return { display, hasIdentity: !!(display || parentId), parentId };
        })))
        : (0, rxjs_1.of)(accountIds.map(() => ({ hasIdentity: false }))));
}
