"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.receivedHeartbeats = receivedHeartbeats;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
function mapResult([result, validators, heartbeats, numBlocks]) {
    validators.forEach((validator, index) => {
        const validatorId = validator.toString();
        const blockCount = numBlocks[index];
        const hasMessage = !heartbeats[index].isEmpty;
        const prev = result[validatorId];
        if (!prev || prev.hasMessage !== hasMessage || !prev.blockCount.eq(blockCount)) {
            result[validatorId] = {
                blockCount,
                hasMessage,
                isOnline: hasMessage || blockCount.gt(util_1.BN_ZERO)
            };
        }
    });
    return result;
}
/**
 * @name receivedHeartbeats
 * @description Return a boolean array indicating whether the passed accounts had received heartbeats in the current session.
 * @example
 * ```javascript
 * let unsub = await api.derive.imOnline.receivedHeartbeats((heartbeat) => {
 *   console.log(heartbeat);
 * });
 * ```
 */
function receivedHeartbeats(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.imOnline?.receivedHeartbeats
        ? api.derive.staking.overview().pipe((0, rxjs_1.switchMap)(({ currentIndex, validators }) => (0, rxjs_1.combineLatest)([
            (0, rxjs_1.of)({}),
            (0, rxjs_1.of)(validators),
            api.query.imOnline.receivedHeartbeats.multi(validators.map((_address, index) => [currentIndex, index])),
            api.query.imOnline.authoredBlocks.multi(validators.map((address) => [currentIndex, address]))
        ])), (0, rxjs_1.map)(mapResult))
        : (0, rxjs_1.of)({}));
}
