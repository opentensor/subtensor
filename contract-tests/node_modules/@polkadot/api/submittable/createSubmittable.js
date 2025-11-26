import { createClass } from './createClass.js';
export function createSubmittable(apiType, api, decorateMethod, registry, blockHash) {
    const Submittable = createClass({ api, apiType, blockHash, decorateMethod });
    return (extrinsic) => new Submittable(registry || api.registry, extrinsic);
}
