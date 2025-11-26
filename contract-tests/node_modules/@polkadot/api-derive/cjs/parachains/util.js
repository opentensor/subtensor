"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.didUpdateToBool = didUpdateToBool;
function didUpdateToBool(didUpdate, id) {
    return didUpdate.isSome
        ? didUpdate.unwrap().some((paraId) => paraId.eq(id))
        : false;
}
