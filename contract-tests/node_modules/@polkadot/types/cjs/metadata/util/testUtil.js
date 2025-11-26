"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decodeLatestMeta = decodeLatestMeta;
exports.toLatest = toLatest;
exports.defaultValues = defaultValues;
exports.testMeta = testMeta;
const tslib_1 = require("tslib");
const node_fs_1 = tslib_1.__importDefault(require("node:fs"));
const util_1 = require("@polkadot/util");
const index_js_1 = require("../../create/index.js");
const index_js_2 = require("../../util/index.js");
const Metadata_js_1 = require("../Metadata.js");
const getUniqTypes_js_1 = require("./getUniqTypes.js");
function getJsonName(version, type, sub) {
    return new URL(`../../../../types-support/src/metadata/v${version}/${type}-${sub}.json`, import.meta.url);
}
function writeJson(json, version, type, sub) {
    node_fs_1.default.writeFileSync(getJsonName(version, type, sub), (0, util_1.stringify)(json, 2), { flag: 'w' });
}
function readJson(version, type, sub) {
    return JSON.parse(node_fs_1.default.readFileSync(getJsonName(version, type, sub), 'utf-8'));
}
function handleMetadata(registry, version, data) {
    let metadata;
    if (version > 15) {
        const opaqueMetadata = registry.createType('Option<OpaqueMetadata>', registry.createType('Raw', data).toU8a()).unwrap();
        metadata = new Metadata_js_1.Metadata(registry, opaqueMetadata.toHex());
    }
    else {
        try {
            metadata = new Metadata_js_1.Metadata(registry, data);
        }
        catch {
            const opaqueMetadata = registry.createType('Option<OpaqueMetadata>', registry.createType('Raw', data).toU8a()).unwrap();
            metadata = new Metadata_js_1.Metadata(registry, opaqueMetadata.toHex());
        }
    }
    return metadata;
}
/** @internal */
function decodeLatestMeta(registry, type, version, { data }) {
    const metadata = handleMetadata(registry, version, data);
    registry.setMetadata(metadata);
    it('decodes latest substrate properly', () => {
        const json = metadata.toJSON();
        delete json.metadata[`v${metadata.version}`].lookup;
        expect(metadata.version).toBe(version);
        try {
            expect(json).toEqual(readJson(version, type, 'json'));
        }
        catch (error) {
            if (process.env['GITHUB_REPOSITORY']) {
                throw error;
            }
            console.error(error);
            writeJson(json, version, type, 'json');
        }
    });
    if (version >= 14) {
        it('decodes latest types correctly', () => {
            const json = metadata.asLatest.lookup.types.toJSON();
            try {
                expect(json).toEqual(readJson(version, type, 'types'));
            }
            catch (error) {
                if (process.env['GITHUB_REPOSITORY']) {
                    throw error;
                }
                console.error(error);
                writeJson(json, version, type, 'types');
            }
        });
    }
}
/** @internal */
function toLatest(registry, version, { data }, withThrow = true) {
    it(`converts v${version} to latest`, () => {
        const metadata = handleMetadata(registry, version, data);
        registry.setMetadata(metadata);
        const latest = metadata.asLatest;
        if (metadata.version < 14) {
            (0, getUniqTypes_js_1.getUniqTypes)(registry, latest, withThrow);
        }
    });
}
/** @internal */
function defaultValues(registry, { data, fails = [] }, withThrow = true, withFallbackCheck = false, version) {
    describe('storage with default values', () => {
        const metadata = handleMetadata(registry, version, data);
        const { pallets } = metadata.asLatest;
        pallets.filter(({ storage }) => storage.isSome).forEach(({ name, storage }) => {
            const sectionName = (0, util_1.stringCamelCase)(name);
            storage.unwrap().items.forEach(({ fallback, modifier, name, type }) => {
                const inner = (0, index_js_2.unwrapStorageType)(registry, type, modifier.isOptional);
                const location = `${sectionName}.${(0, util_1.stringCamelCase)(name)}: ${inner}`;
                it(location, () => {
                    expect(() => {
                        try {
                            const instance = registry.createTypeUnsafe(registry.createLookupType((0, index_js_2.unwrapStorageSi)(type)), [(0, util_1.hexToU8a)(fallback.toHex())], { isOptional: modifier.isOptional });
                            if (withFallbackCheck) {
                                const [hexType, hexOrig] = [(0, util_1.u8aToHex)(instance.toU8a()), (0, util_1.u8aToHex)(fallback.toU8a(true))];
                                if (hexType !== hexOrig) {
                                    throw new Error(`Fallback does not match (${((hexOrig.length - 2) / 2) - ((hexType.length - 2) / 2)} bytes missing): ${hexType} !== ${hexOrig}`);
                                }
                            }
                        }
                        catch (error) {
                            const message = `${location}:: ${error.message}`;
                            if (withThrow && !fails.some((f) => location.includes(f))) {
                                throw new Error(message);
                            }
                            else {
                                console.warn(message);
                            }
                        }
                    }).not.toThrow();
                });
            });
        });
    });
}
function serialize(registry, { data }, version) {
    const metadata = handleMetadata(registry, version, data);
    if (version < 15) {
        it('serializes to hex in the same form as retrieved', () => {
            expect(metadata.toHex()).toEqual(data);
        });
        // NOTE Assuming the first passes this is actually something that doesn't test
        // anything new. If the first line in this function passed and the above values
        // are equivalent, this would be as well.
        it.skip('can construct from a re-serialized form', () => {
            expect(() => new Metadata_js_1.Metadata(registry, metadata.toHex())).not.toThrow();
        });
        // as used in the extension
        it('can construct from asCallsOnly.toHex()', () => {
            expect(() => new Metadata_js_1.Metadata(registry, metadata.asCallsOnly.toHex())).not.toThrow();
        });
    }
}
function testMeta(version, matchers, withFallback = true) {
    describe(`MetadataV${version}`, () => {
        for (const [type, matcher] of Object.entries(matchers)) {
            const registry = new index_js_1.TypeRegistry();
            describe(type, () => {
                serialize(registry, matcher, version);
                decodeLatestMeta(registry, type, version, matcher);
                toLatest(registry, version, matcher);
                defaultValues(registry, matcher, true, withFallback, version);
            });
        }
    });
}
