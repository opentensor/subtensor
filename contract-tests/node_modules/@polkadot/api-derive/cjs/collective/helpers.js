"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getInstance = getInstance;
exports.withSection = withSection;
exports.callMethod = callMethod;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
function getInstance(api, section) {
    const instances = api.registry.getModuleInstances(api.runtimeVersion.specName, section);
    const name = instances?.length
        ? instances[0]
        : section;
    return api.query[name];
}
function withSection(section, fn) {
    return (instanceId, api) => (0, index_js_1.memo)(instanceId, fn(getInstance(api, section), api, instanceId));
}
function callMethod(method, empty) {
    return (section) => withSection(section, (query) => () => (0, util_1.isFunction)(query?.[method])
        ? query[method]()
        : (0, rxjs_1.of)(empty));
}
