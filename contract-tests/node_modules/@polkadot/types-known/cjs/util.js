"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getSpecExtensions = getSpecExtensions;
exports.getSpecTypes = getSpecTypes;
exports.getSpecHasher = getSpecHasher;
exports.getSpecRpc = getSpecRpc;
exports.getSpecRuntime = getSpecRuntime;
exports.getSpecAlias = getSpecAlias;
exports.getUpgradeVersion = getUpgradeVersion;
const util_1 = require("@polkadot/util");
const index_js_1 = require("./chain/index.js");
const index_js_2 = require("./spec/index.js");
const index_js_3 = require("./upgrades/index.js");
/**
 * @description Perform the callback function using the stringified spec/chain
 * @internal
 * */
function withNames(chainName, specName, fn) {
    return fn(chainName.toString(), specName.toString());
}
/**
 * @descriptionFflatten a VersionedType[] into a Record<string, string>
 * @internal
 * */
function filterVersions(versions = [], specVersion) {
    return versions
        .filter(({ minmax: [min, max] }) => (min === undefined || min === null || specVersion >= min) &&
        (max === undefined || max === null || specVersion <= max))
        .reduce((result, { types }) => ({ ...result, ...types }), {});
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable signed extensions (ready for registration)
 */
function getSpecExtensions({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.signedExtensions ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.signedExtensions ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable types (ready for registration)
 */
function getSpecTypes({ knownTypes }, chainName, specName, specVersion) {
    const _specVersion = (0, util_1.bnToBn)(specVersion).toNumber();
    return withNames(chainName, specName, (c, s) => ({
        // The order here is always, based on -
        //   - spec then chain
        //   - typesBundle takes higher precedence
        //   - types is the final catch-all override
        ...filterVersions(index_js_2.typesSpec[s], _specVersion),
        ...filterVersions(index_js_1.typesChain[c], _specVersion),
        ...filterVersions(knownTypes.typesBundle?.spec?.[s]?.types, _specVersion),
        ...filterVersions(knownTypes.typesBundle?.chain?.[c]?.types, _specVersion),
        ...(knownTypes.typesSpec?.[s] ?? {}),
        ...(knownTypes.typesChain?.[c] ?? {}),
        ...(knownTypes.types ?? {})
    }));
}
/**
 * @description Based on the chain or spec, return the hasher used
 */
function getSpecHasher({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => knownTypes.hasher ||
        knownTypes.typesBundle?.chain?.[c]?.hasher ||
        knownTypes.typesBundle?.spec?.[s]?.hasher ||
        null);
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable rpc definitions (ready for registration)
 */
function getSpecRpc({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.rpc ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.rpc ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable runtime definitions (ready for registration)
 */
function getSpecRuntime({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.runtime ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.runtime ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable alias definitions (ready for registration)
 */
function getSpecAlias({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        // as per versions, first spec, then chain then finally non-versioned
        ...(knownTypes.typesBundle?.spec?.[s]?.alias ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.alias ?? {}),
        ...(knownTypes.typesAlias ?? {})
    }));
}
/**
 * @description Returns a version record for known chains where upgrades are being tracked
 */
function getUpgradeVersion(genesisHash, blockNumber) {
    const known = index_js_3.upgrades.find((u) => genesisHash.eq(u.genesisHash));
    return known
        ? [
            known.versions.reduce((last, version) => {
                return blockNumber.gt(version.blockNumber)
                    ? version
                    : last;
            }, undefined),
            known.versions.find((version) => blockNumber.lte(version.blockNumber))
        ]
        : [undefined, undefined];
}
