"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.all = all;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const VESTING_ID = '0x76657374696e6720';
function calcLocked(api, bestNumber, locks) {
    let lockedBalance = api.registry.createType('Balance');
    let lockedBreakdown = [];
    let vestingLocked = api.registry.createType('Balance');
    let allLocked = false;
    if (Array.isArray(locks)) {
        // only get the locks that are valid until passed the current block
        lockedBreakdown = locks.filter(({ until }) => !until || (bestNumber && until.gt(bestNumber)));
        allLocked = lockedBreakdown.some(({ amount }) => amount && amount.isMax());
        vestingLocked = api.registry.createType('Balance', lockedBreakdown.filter(({ id }) => id.eq(VESTING_ID)).reduce((result, { amount }) => result.iadd(amount), new util_1.BN(0)));
        // get the maximum of the locks according to https://github.com/paritytech/substrate/blob/master/srml/balances/src/lib.rs#L699
        const notAll = lockedBreakdown.filter(({ amount }) => amount && !amount.isMax());
        if (notAll.length) {
            lockedBalance = api.registry.createType('Balance', (0, util_1.bnMax)(...notAll.map(({ amount }) => amount)));
        }
    }
    return { allLocked, lockedBalance, lockedBreakdown, vestingLocked };
}
function calcShared(api, bestNumber, data, locks) {
    const { allLocked, lockedBalance, lockedBreakdown, vestingLocked } = calcLocked(api, bestNumber, locks);
    let transferable = null;
    if (data?.frameSystemAccountInfo?.frozen) {
        const { frameSystemAccountInfo, freeBalance, reservedBalance } = data;
        const noFrozenReserved = frameSystemAccountInfo.frozen.isZero() && reservedBalance.isZero();
        const ED = api.consts.balances.existentialDeposit;
        const maybeED = noFrozenReserved ? new util_1.BN(0) : ED;
        const frozenReserveDif = frameSystemAccountInfo.frozen.sub(reservedBalance);
        transferable = api.registry.createType('Balance', allLocked
            ? 0
            : (0, util_1.bnMax)(new util_1.BN(0), freeBalance.sub((0, util_1.bnMax)(maybeED, frozenReserveDif))));
    }
    return (0, util_1.objectSpread)({}, data, {
        availableBalance: api.registry.createType('Balance', allLocked ? 0 : (0, util_1.bnMax)(new util_1.BN(0), data?.freeBalance ? data.freeBalance.sub(lockedBalance) : new util_1.BN(0))),
        lockedBalance,
        lockedBreakdown,
        transferable,
        vestingLocked
    });
}
function calcVesting(bestNumber, shared, _vesting) {
    // Calculate the vesting balances,
    //  - offset = balance locked at startingBlock
    //  - perBlock is the unlock amount
    const vesting = _vesting || [];
    const isVesting = !shared.vestingLocked.isZero();
    const vestedBalances = vesting.map(({ locked, perBlock, startingBlock }) => bestNumber.gt(startingBlock)
        ? (0, util_1.bnMin)(locked, perBlock.mul(bestNumber.sub(startingBlock)))
        : util_1.BN_ZERO);
    const vestedBalance = vestedBalances.reduce((all, value) => all.iadd(value), new util_1.BN(0));
    const vestingTotal = vesting.reduce((all, { locked }) => all.iadd(locked), new util_1.BN(0));
    return {
        isVesting,
        vestedBalance,
        vestedClaimable: isVesting
            ? shared.vestingLocked.sub(vestingTotal.sub(vestedBalance))
            : util_1.BN_ZERO,
        vesting: vesting
            .map(({ locked, perBlock, startingBlock }, index) => ({
            endBlock: locked.div(perBlock).iadd(startingBlock),
            locked,
            perBlock,
            startingBlock,
            vested: vestedBalances[index]
        }))
            .filter(({ locked }) => !locked.isZero()),
        vestingTotal
    };
}
function calcBalances(api, result) {
    const [data, [vesting, allLocks, namedReserves], bestNumber] = result;
    const shared = calcShared(api, bestNumber, data, allLocks[0]);
    return (0, util_1.objectSpread)(shared, calcVesting(bestNumber, shared, vesting), {
        accountId: data.accountId,
        accountNonce: data.accountNonce,
        additional: allLocks
            .slice(1)
            .map((l, index) => calcShared(api, bestNumber, data.additional[index], l)),
        namedReserves
    });
}
function queryOld(api, accountId) {
    return (0, rxjs_1.combineLatest)([
        api.query.balances.locks(accountId),
        api.query.balances['vesting'](accountId)
    ]).pipe((0, rxjs_1.map)(([locks, optVesting]) => {
        let vestingNew = null;
        if (optVesting.isSome) {
            const { offset: locked, perBlock, startingBlock } = optVesting.unwrap();
            vestingNew = api.registry.createType('VestingInfo', { locked, perBlock, startingBlock });
        }
        return [
            vestingNew
                ? [vestingNew]
                : null,
            [locks],
            []
        ];
    }));
}
const isNonNullable = (nullable) => !!nullable;
function createCalls(calls) {
    return [
        calls.map((c) => !c),
        calls.filter(isNonNullable)
    ];
}
function queryCurrent(api, accountId, balanceInstances = ['balances']) {
    const [lockEmpty, lockQueries] = createCalls(balanceInstances.map((m) => api.derive[m]?.customLocks || api.query[m]?.locks));
    const [reserveEmpty, reserveQueries] = createCalls(balanceInstances.map((m) => api.query[m]?.reserves));
    return (0, rxjs_1.combineLatest)([
        api.query.vesting?.vesting
            ? api.query.vesting.vesting(accountId)
            : (0, rxjs_1.of)(api.registry.createType('Option<VestingInfo>')),
        lockQueries.length
            ? (0, rxjs_1.combineLatest)(lockQueries.map((c) => c(accountId)))
            : (0, rxjs_1.of)([]),
        reserveQueries.length
            ? (0, rxjs_1.combineLatest)(reserveQueries.map((c) => c(accountId)))
            : (0, rxjs_1.of)([])
    ]).pipe((0, rxjs_1.map)(([opt, locks, reserves]) => {
        let offsetLock = -1;
        let offsetReserve = -1;
        const vesting = opt.unwrapOr(null);
        return [
            vesting
                ? Array.isArray(vesting)
                    ? vesting
                    : [vesting]
                : null,
            lockEmpty.map((e) => e ? api.registry.createType('Vec<BalanceLock>') : locks[++offsetLock]),
            reserveEmpty.map((e) => e ? api.registry.createType('Vec<PalletBalancesReserveData>') : reserves[++offsetReserve])
        ];
    }));
}
/**
 * @name all
 * @description Retrieves the complete balance information for an account, including free balance, locked balance, reserved balance, and more.
 * @param {( AccountId | string )} address An accountsId in different formats.
 * @example
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.account(ALICE, (accountInfo) => {
 *   console.log(
 *     `${accountInfo.accountId} info:`,
 *     Object.keys(accountInfo).map((key) => `${key}: ${accountInfo[key]}`)
 *   );
 * });
 * ```
 */
function all(instanceId, api) {
    const balanceInstances = api.registry.getModuleInstances(api.runtimeVersion.specName, 'balances');
    return (0, index_js_1.memo)(instanceId, (address) => (0, rxjs_1.combineLatest)([
        api.derive.balances.account(address),
        (0, util_1.isFunction)(api.query.system?.account) || (0, util_1.isFunction)(api.query.balances?.account)
            ? queryCurrent(api, address, balanceInstances)
            : queryOld(api, address)
    ]).pipe((0, rxjs_1.switchMap)(([account, locks]) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(account),
        (0, rxjs_1.of)(locks),
        api.derive.chain.bestNumber()
    ])), (0, rxjs_1.map)((result) => calcBalances(api, result))));
}
