"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lazyDeriveSection = void 0;
exports.getAvailableDerives = getAvailableDerives;
const tslib_1 = require("tslib");
const index_js_1 = require("./util/index.js");
Object.defineProperty(exports, "lazyDeriveSection", { enumerable: true, get: function () { return index_js_1.lazyDeriveSection; } });
const derive_js_1 = require("./derive.js");
tslib_1.__exportStar(require("./derive.js"), exports);
tslib_1.__exportStar(require("./type/index.js"), exports);
const checks = {
    allianceMotion: {
        instances: ['allianceMotion'],
        methods: []
    },
    bagsList: {
        instances: ['voterBagsList', 'voterList', 'bagsList'],
        methods: [],
        withDetect: true
    },
    contracts: {
        instances: ['contracts'],
        methods: []
    },
    council: {
        instances: ['council'],
        methods: [],
        withDetect: true
    },
    crowdloan: {
        instances: ['crowdloan'],
        methods: []
    },
    democracy: {
        instances: ['democracy'],
        methods: []
    },
    elections: {
        instances: ['phragmenElection', 'electionsPhragmen', 'elections', 'council'],
        methods: [],
        withDetect: true
    },
    imOnline: {
        instances: ['imOnline'],
        methods: []
    },
    membership: {
        instances: ['membership'],
        methods: []
    },
    parachains: {
        instances: ['parachains', 'registrar'],
        methods: []
    },
    session: {
        instances: ['session'],
        methods: []
    },
    society: {
        instances: ['society'],
        methods: []
    },
    staking: {
        instances: ['staking'],
        methods: ['erasRewardPoints']
    },
    technicalCommittee: {
        instances: ['technicalCommittee'],
        methods: [],
        withDetect: true
    },
    treasury: {
        instances: ['treasury'],
        methods: []
    }
};
function getModuleInstances(api, specName, moduleName) {
    return api.registry.getModuleInstances(specName, moduleName) || [];
}
/**
 * Returns an object that will inject `api` into all the functions inside
 * `allSections`, and keep the object architecture of `allSections`.
 */
/** @internal */
function injectFunctions(instanceId, api, derives) {
    const result = {};
    const names = Object.keys(derives);
    const keys = Object.keys(api.query);
    const specName = api.runtimeVersion.specName;
    const filterKeys = (q) => keys.includes(q);
    const filterInstances = (q) => getModuleInstances(api, specName, q).some(filterKeys);
    const filterMethods = (all) => (m) => all.some((q) => keys.includes(q) && api.query[q][m]);
    const getKeys = (s) => Object.keys(derives[s]);
    const creator = (s, m) => derives[s][m](instanceId, api);
    const isIncluded = (c) => (!checks[c] || ((checks[c].instances.some(filterKeys) && (!checks[c].methods.length ||
        checks[c].methods.every(filterMethods(checks[c].instances)))) ||
        (checks[c].withDetect &&
            checks[c].instances.some(filterInstances))));
    for (let i = 0, count = names.length; i < count; i++) {
        const name = names[i];
        isIncluded(name) &&
            (0, index_js_1.lazyDeriveSection)(result, name, getKeys, creator);
    }
    return result;
}
/** @internal */
function getAvailableDerives(instanceId, api, custom = {}) {
    return {
        ...injectFunctions(instanceId, api, derive_js_1.derive),
        ...injectFunctions(instanceId, api, custom)
    };
}
