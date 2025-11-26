"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decorateStorage = decorateStorage;
const util_1 = require("@polkadot/util");
const util_js_1 = require("../util.js");
const createFunction_js_1 = require("./createFunction.js");
const getStorage_js_1 = require("./getStorage.js");
const util_js_2 = require("./util.js");
const VERSION_NAME = 'palletVersion';
const VERSION_KEY = ':__STORAGE_VERSION__:';
const VERSION_DOCS = { docs: 'Returns the current pallet version from storage', type: 'u16' };
/** @internal */
function decorateStorage(registry, { pallets }, _metaVersion) {
    const result = (0, getStorage_js_1.getStorage)(registry);
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { name, storage } = pallets[i];
        if (storage.isSome) {
            const section = (0, util_1.stringCamelCase)(name);
            const { items, prefix: _prefix } = storage.unwrap();
            const prefix = _prefix.toString();
            (0, util_1.lazyMethod)(result, section, () => (0, util_1.lazyMethods)({
                palletVersion: (0, util_js_2.createRuntimeFunction)({ method: VERSION_NAME, prefix, section }, (0, createFunction_js_1.createKeyRaw)(registry, { method: VERSION_KEY, prefix: name.toString() }, createFunction_js_1.NO_RAW_ARGS), VERSION_DOCS)(registry)
            }, items, (meta) => (0, createFunction_js_1.createFunction)(registry, { meta, method: meta.name.toString(), prefix, section }, {}), util_js_1.objectNameToCamel));
        }
    }
    return result;
}
