"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createSubmittable = createSubmittable;
const createClass_js_1 = require("./createClass.js");
function createSubmittable(apiType, api, decorateMethod, registry, blockHash) {
    const Submittable = (0, createClass_js_1.createClass)({ api, apiType, blockHash, decorateMethod });
    return (extrinsic) => new Submittable(registry || api.registry, extrinsic);
}
