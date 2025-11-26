"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.XCM_MAPPINGS = void 0;
exports.mapXcmTypes = mapXcmTypes;
const util_1 = require("@polkadot/util");
exports.XCM_MAPPINGS = ['AssetInstance', 'Fungibility', 'Junction', 'Junctions', 'MultiAsset', 'MultiAssetFilter', 'MultiLocation', 'Response', 'WildFungibility', 'WildMultiAsset', 'Xcm', 'XcmError'];
function mapXcmTypes(version) {
    return exports.XCM_MAPPINGS.reduce((all, key) => (0, util_1.objectSpread)(all, { [key]: `${key}${version}` }), {});
}
