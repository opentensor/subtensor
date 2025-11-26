"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterEras = filterEras;
exports.erasHistoricApply = erasHistoricApply;
exports.erasHistoricApplyAccount = erasHistoricApplyAccount;
exports.singleEra = singleEra;
exports.combineEras = combineEras;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const ERA_CHUNK_SIZE = 14;
function chunkEras(eras, fn) {
    const chunked = (0, util_1.arrayChunk)(eras, ERA_CHUNK_SIZE);
    let index = 0;
    const subject = new rxjs_1.BehaviorSubject(chunked[index]);
    return subject.pipe((0, rxjs_1.switchMap)(fn), (0, rxjs_1.tap)(() => {
        (0, util_1.nextTick)(() => {
            index++;
            index === chunked.length
                ? subject.complete()
                : subject.next(chunked[index]);
        });
    }), (0, rxjs_1.toArray)(), (0, rxjs_1.map)(util_1.arrayFlatten));
}
function filterEras(eras, list) {
    return eras.filter((e) => !list.some(({ era }) => e.eq(era)));
}
function erasHistoricApply(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    (0, index_js_1.memo)(instanceId, (withActive = false) => api.derive.staking.erasHistoric(withActive).pipe((0, rxjs_1.switchMap)((e) => api.derive.staking[fn](e, withActive))));
}
function erasHistoricApplyAccount(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    (0, index_js_1.memo)(instanceId, (accountId, withActive = false, page) => api.derive.staking.erasHistoric(withActive).pipe((0, rxjs_1.switchMap)((e) => api.derive.staking[fn](accountId, e, withActive, page || 0))));
}
function singleEra(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    (0, index_js_1.memo)(instanceId, (era) => api.derive.staking[fn](era, true));
}
function combineEras(fn) {
    return (instanceId, api) => 
    // Cannot quite get the typing right, but it is right in the code
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    (0, index_js_1.memo)(instanceId, (eras, withActive) => !eras.length
        ? (0, rxjs_1.of)([])
        : chunkEras(eras, (eras) => (0, rxjs_1.combineLatest)(eras.map((e) => api.derive.staking[fn](e, withActive)))));
}
