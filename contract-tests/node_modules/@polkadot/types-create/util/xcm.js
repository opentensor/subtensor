import { objectSpread } from '@polkadot/util';
export const XCM_MAPPINGS = ['AssetInstance', 'Fungibility', 'Junction', 'Junctions', 'MultiAsset', 'MultiAssetFilter', 'MultiLocation', 'Response', 'WildFungibility', 'WildMultiAsset', 'Xcm', 'XcmError'];
export function mapXcmTypes(version) {
    return XCM_MAPPINGS.reduce((all, key) => objectSpread(all, { [key]: `${key}${version}` }), {});
}
