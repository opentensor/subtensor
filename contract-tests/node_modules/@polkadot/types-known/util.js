import { bnToBn } from '@polkadot/util';
import { typesChain } from './chain/index.js';
import { typesSpec } from './spec/index.js';
import { upgrades } from './upgrades/index.js';
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
export function getSpecExtensions({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.signedExtensions ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.signedExtensions ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable types (ready for registration)
 */
export function getSpecTypes({ knownTypes }, chainName, specName, specVersion) {
    const _specVersion = bnToBn(specVersion).toNumber();
    return withNames(chainName, specName, (c, s) => ({
        // The order here is always, based on -
        //   - spec then chain
        //   - typesBundle takes higher precedence
        //   - types is the final catch-all override
        ...filterVersions(typesSpec[s], _specVersion),
        ...filterVersions(typesChain[c], _specVersion),
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
export function getSpecHasher({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => knownTypes.hasher ||
        knownTypes.typesBundle?.chain?.[c]?.hasher ||
        knownTypes.typesBundle?.spec?.[s]?.hasher ||
        null);
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable rpc definitions (ready for registration)
 */
export function getSpecRpc({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.rpc ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.rpc ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable runtime definitions (ready for registration)
 */
export function getSpecRuntime({ knownTypes }, chainName, specName) {
    return withNames(chainName, specName, (c, s) => ({
        ...(knownTypes.typesBundle?.spec?.[s]?.runtime ?? {}),
        ...(knownTypes.typesBundle?.chain?.[c]?.runtime ?? {})
    }));
}
/**
 * @description Based on the chain and runtimeVersion, get the applicable alias definitions (ready for registration)
 */
export function getSpecAlias({ knownTypes }, chainName, specName) {
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
export function getUpgradeVersion(genesisHash, blockNumber) {
    const known = upgrades.find((u) => genesisHash.eq(u.genesisHash));
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
