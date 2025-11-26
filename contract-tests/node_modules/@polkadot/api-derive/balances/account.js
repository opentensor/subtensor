import { combineLatest, map, of, switchMap } from 'rxjs';
import { isFunction, objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
function zeroBalance(api) {
    return api.registry.createType('Balance');
}
function getBalance(api, [freeBalance, reservedBalance, frozenFeeOrFrozen, frozenMiscOrFlags], accType) {
    const votingBalance = api.registry.createType('Balance', freeBalance.toBn());
    if (accType.isFrameAccountData) {
        return {
            frameSystemAccountInfo: {
                flags: frozenMiscOrFlags,
                frozen: frozenFeeOrFrozen
            },
            freeBalance,
            frozenFee: api.registry.createType('Balance', 0),
            frozenMisc: api.registry.createType('Balance', 0),
            reservedBalance,
            votingBalance
        };
    }
    return {
        freeBalance,
        frozenFee: frozenFeeOrFrozen,
        frozenMisc: frozenMiscOrFlags,
        reservedBalance,
        votingBalance
    };
}
function calcBalances(api, [accountId, [accountNonce, [primary, ...additional], accType]]) {
    return objectSpread({
        accountId,
        accountNonce,
        additional: additional.map((b) => getBalance(api, b, accType))
    }, getBalance(api, primary, accType));
}
function queryBalancesFree(api, accountId) {
    return combineLatest([
        api.query.balances['freeBalance'](accountId),
        api.query.balances['reservedBalance'](accountId),
        api.query.system['accountNonce'](accountId)
    ]).pipe(map(([freeBalance, reservedBalance, accountNonce]) => [
        accountNonce,
        [[freeBalance, reservedBalance, zeroBalance(api), zeroBalance(api)]],
        { isFrameAccountData: false }
    ]));
}
function queryNonceOnly(api, accountId) {
    const fill = (nonce) => [
        nonce,
        [[zeroBalance(api), zeroBalance(api), zeroBalance(api), zeroBalance(api)]],
        { isFrameAccountData: false }
    ];
    return isFunction(api.query.system.account)
        ? api.query.system.account(accountId).pipe(map(({ nonce }) => fill(nonce)))
        : isFunction(api.query.system['accountNonce'])
            ? api.query.system['accountNonce'](accountId).pipe(map((nonce) => fill(nonce)))
            : of(fill(api.registry.createType('Index')));
}
function queryBalancesAccount(api, accountId, modules = ['balances']) {
    const balances = modules
        .map((m) => api.derive[m]?.customAccount || api.query[m]?.account)
        .filter((q) => isFunction(q));
    const extract = (nonce, data) => [
        nonce,
        data.map(({ feeFrozen, free, miscFrozen, reserved }) => [free, reserved, feeFrozen, miscFrozen]),
        { isFrameAccountData: false }
    ];
    // NOTE this is for the first case where we do have instances specified
    return balances.length
        ? isFunction(api.query.system.account)
            ? combineLatest([
                api.query.system.account(accountId),
                ...balances.map((c) => c(accountId))
            ]).pipe(map(([{ nonce }, ...balances]) => extract(nonce, balances)))
            : combineLatest([
                api.query.system['accountNonce'](accountId),
                ...balances.map((c) => c(accountId))
            ]).pipe(map(([nonce, ...balances]) => extract(nonce, balances)))
        : queryNonceOnly(api, accountId);
}
function querySystemAccount(api, accountId) {
    // AccountInfo is current, support old, eg. Edgeware
    return api.query.system.account(accountId).pipe(map((infoOrTuple) => {
        const data = infoOrTuple.nonce
            ? infoOrTuple.data
            : infoOrTuple[1];
        const nonce = infoOrTuple.nonce || infoOrTuple[0];
        if (!data || data.isEmpty) {
            return [
                nonce,
                [[zeroBalance(api), zeroBalance(api), zeroBalance(api), zeroBalance(api)]],
                { isFrameAccountData: false }
            ];
        }
        const isFrameType = !!infoOrTuple.data.frozen;
        if (isFrameType) {
            const { flags, free, frozen, reserved } = data;
            return [
                nonce,
                [[free, reserved, frozen, flags]],
                { isFrameAccountData: true }
            ];
        }
        else {
            const { feeFrozen, free, miscFrozen, reserved } = data;
            return [
                nonce,
                [[free, reserved, feeFrozen, miscFrozen]],
                { isFrameAccountData: false }
            ];
        }
    }));
}
/**
 * @name account
 * @description Retrieves the essential balance details for an account, such as free balance and account nonce.
 * @param {( AccountIndex | AccountId | Address | string )} address An accountsId in different formats.
 * @example
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.all(ALICE, ({ accountId, lockedBalance }) => {
 *   console.log(`The account ${accountId} has a locked balance ${lockedBalance} units.`);
 * });
 * ```
 */
export function account(instanceId, api) {
    const balanceInstances = api.registry.getModuleInstances(api.runtimeVersion.specName, 'balances');
    const nonDefaultBalances = balanceInstances && balanceInstances[0] !== 'balances';
    return memo(instanceId, (address) => api.derive.accounts.accountId(address).pipe(switchMap((accountId) => (accountId
        ? combineLatest([
            of(accountId),
            nonDefaultBalances
                ? queryBalancesAccount(api, accountId, balanceInstances)
                : isFunction(api.query.system?.account)
                    ? querySystemAccount(api, accountId)
                    : isFunction(api.query.balances?.account)
                        ? queryBalancesAccount(api, accountId)
                        : isFunction(api.query.balances?.['freeBalance'])
                            ? queryBalancesFree(api, accountId)
                            : queryNonceOnly(api, accountId)
        ])
        : of([api.registry.createType('AccountId'), [
                api.registry.createType('Index'),
                [[zeroBalance(api), zeroBalance(api), zeroBalance(api), zeroBalance(api)]],
                { isFrameAccountData: false }
            ]]))), map((result) => calcBalances(api, result))));
}
