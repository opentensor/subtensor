"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TypeRegistry = void 0;
const tslib_1 = require("tslib");
const types_codec_1 = require("@polkadot/types-codec");
const types_create_1 = require("@polkadot/types-create");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../extrinsic/signedExtensions/index.js");
const Event_js_1 = require("../generic/Event.js");
const baseTypes = tslib_1.__importStar(require("../index.types.js"));
const definitions = tslib_1.__importStar(require("../interfaces/definitions.js"));
const index_js_2 = require("../metadata/decorate/extrinsics/index.js");
const index_js_3 = require("../metadata/decorate/index.js");
const Metadata_js_1 = require("../metadata/Metadata.js");
const index_js_4 = require("../metadata/PortableRegistry/index.js");
const lazy_js_1 = require("./lazy.js");
const DEFAULT_FIRST_CALL_IDX = new Uint8Array(2);
const l = (0, util_1.logger)('registry');
function sortDecimalStrings(a, b) {
    return parseInt(a, 10) - parseInt(b, 10);
}
function valueToString(v) {
    return v.toString();
}
function getFieldArgs(lookup, fields) {
    const count = fields.length;
    const args = new Array(count);
    for (let i = 0; i < count; i++) {
        args[i] = lookup.getTypeDef(fields[i].type).type;
    }
    return args;
}
function clearRecord(record) {
    const keys = Object.keys(record);
    for (let i = 0, count = keys.length; i < count; i++) {
        delete record[keys[i]];
    }
}
function getVariantStringIdx({ index }) {
    return index.toString();
}
function injectErrors(_, { lookup, pallets }, version, result) {
    clearRecord(result);
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { errors, index, name } = pallets[i];
        if (errors.isSome) {
            const sectionName = (0, util_1.stringCamelCase)(name);
            (0, util_1.lazyMethod)(result, version >= 12 ? index.toNumber() : i, () => (0, lazy_js_1.lazyVariants)(lookup, errors.unwrap(), getVariantStringIdx, ({ docs, fields, index, name }) => ({
                args: getFieldArgs(lookup, fields),
                docs: docs.map(valueToString),
                fields,
                index: index.toNumber(),
                method: name.toString(),
                name: name.toString(),
                section: sectionName
            })));
        }
    }
}
function injectEvents(registry, { lookup, pallets }, version, result) {
    const filtered = pallets.filter(index_js_3.filterEventsSome);
    clearRecord(result);
    for (let i = 0, count = filtered.length; i < count; i++) {
        const { events, index, name } = filtered[i];
        (0, util_1.lazyMethod)(result, version >= 12 ? index.toNumber() : i, () => (0, lazy_js_1.lazyVariants)(lookup, events.unwrap(), getVariantStringIdx, (variant) => {
            const meta = registry.createType('EventMetadataLatest', (0, util_1.objectSpread)({}, variant, { args: getFieldArgs(lookup, variant.fields) }));
            return class extends Event_js_1.GenericEventData {
                constructor(registry, value) {
                    super(registry, value, meta, (0, util_1.stringCamelCase)(name), variant.name.toString());
                }
            };
        }));
    }
}
function injectExtrinsics(registry, { lookup, pallets }, version, result, mapping) {
    const filtered = pallets.filter(index_js_3.filterCallsSome);
    clearRecord(result);
    clearRecord(mapping);
    for (let i = 0, count = filtered.length; i < count; i++) {
        const { calls, index, name } = filtered[i];
        const sectionIndex = version >= 12 ? index.toNumber() : i;
        const sectionName = (0, util_1.stringCamelCase)(name);
        const allCalls = calls.unwrap();
        (0, util_1.lazyMethod)(result, sectionIndex, () => (0, lazy_js_1.lazyVariants)(lookup, allCalls, getVariantStringIdx, (variant) => (0, index_js_2.createCallFunction)(registry, lookup, variant, sectionName, sectionIndex)));
        const { path } = registry.lookup.getSiType(allCalls.type);
        // frame_system::pallet::Call / pallet_balances::pallet::Call / polkadot_runtime_parachains::configuration::pallet::Call /
        const palletIdx = path.findIndex((v) => v.eq('pallet'));
        if (palletIdx !== -1) {
            const name = (0, util_1.stringCamelCase)(path
                .slice(0, palletIdx)
                .map((p, i) => i === 0
                // frame_system || pallet_balances
                ? p.replace(/^(frame|pallet)_/, '')
                : p)
                .join(' '));
            if (!mapping[name]) {
                mapping[name] = [sectionName];
            }
            else {
                mapping[name].push(sectionName);
            }
        }
    }
}
function extractProperties(registry, metadata) {
    const original = registry.getChainProperties();
    const constants = (0, index_js_3.decorateConstants)(registry, metadata.asLatest, metadata.version);
    const ss58Format = constants['system'] && (constants['system']['sS58Prefix'] || constants['system']['ss58Prefix']);
    if (!ss58Format) {
        return original;
    }
    const { isEthereum, tokenDecimals, tokenSymbol } = original || {};
    return registry.createTypeUnsafe('ChainProperties', [{ isEthereum, ss58Format, tokenDecimals, tokenSymbol }]);
}
class TypeRegistry {
    #chainProperties;
    #classes = new Map();
    #definitions = new Map();
    #firstCallIndex = null;
    #hasher = util_crypto_1.blake2AsU8a;
    #knownTypes = {};
    #lookup;
    #metadata;
    #metadataVersion = 0;
    #signedExtensions = index_js_1.fallbackExtensions;
    #unknownTypes = new Map();
    #userExtensions;
    #knownDefaults;
    #knownDefaultsEntries;
    #knownDefinitions;
    #metadataCalls = {};
    #metadataErrors = {};
    #metadataEvents = {};
    #moduleMap = {};
    createdAtHash;
    constructor(createdAtHash) {
        this.#knownDefaults = new Map(Object.entries({ Json: types_codec_1.Json, Metadata: Metadata_js_1.Metadata, PortableRegistry: index_js_4.PortableRegistry, Raw: types_codec_1.Raw, ...baseTypes }));
        this.#knownDefaultsEntries = Array.from(this.#knownDefaults.entries());
        this.#knownDefinitions = definitions;
        const allKnown = Object.values(this.#knownDefinitions);
        for (let i = 0, count = allKnown.length; i < count; i++) {
            this.register(allKnown[i].types);
        }
        if (createdAtHash) {
            this.createdAtHash = this.createType('BlockHash', createdAtHash);
        }
    }
    get chainDecimals() {
        if (this.#chainProperties?.tokenDecimals.isSome) {
            const allDecimals = this.#chainProperties.tokenDecimals.unwrap();
            if (allDecimals.length) {
                return allDecimals.map((b) => b.toNumber());
            }
        }
        return [12];
    }
    get chainIsEthereum() {
        return this.#chainProperties?.isEthereum.isTrue || false;
    }
    get chainSS58() {
        return this.#chainProperties?.ss58Format.isSome
            ? this.#chainProperties.ss58Format.unwrap().toNumber()
            : undefined;
    }
    get chainTokens() {
        if (this.#chainProperties?.tokenSymbol.isSome) {
            const allTokens = this.#chainProperties.tokenSymbol.unwrap();
            if (allTokens.length) {
                return allTokens.map(valueToString);
            }
        }
        return [util_1.formatBalance.getDefaults().unit];
    }
    get firstCallIndex() {
        return this.#firstCallIndex || DEFAULT_FIRST_CALL_IDX;
    }
    /**
     * @description Returns true if the type is in a Compat format
     */
    isLookupType(value) {
        return /Lookup\d+$/.test(value);
    }
    /**
     * @description Creates a lookup string from the supplied id
     */
    createLookupType(lookupId) {
        return `Lookup${typeof lookupId === 'number' ? lookupId : lookupId.toNumber()}`;
    }
    get knownTypes() {
        return this.#knownTypes;
    }
    get lookup() {
        return (0, util_1.assertReturn)(this.#lookup, 'PortableRegistry has not been set on this registry');
    }
    get metadata() {
        return (0, util_1.assertReturn)(this.#metadata, 'Metadata has not been set on this registry');
    }
    get unknownTypes() {
        return [...this.#unknownTypes.keys()];
    }
    get signedExtensions() {
        return this.#signedExtensions;
    }
    clearCache() {
        this.#classes = new Map();
    }
    /**
     * @describe Creates an instance of the class
     */
    createClass(type) {
        return (0, types_create_1.createClassUnsafe)(this, type);
    }
    /**
     * @describe Creates an instance of the class
     */
    createClassUnsafe(type) {
        return (0, types_create_1.createClassUnsafe)(this, type);
    }
    /**
     * @description Creates an instance of a type as registered
     */
    createType(type, ...params) {
        return (0, types_create_1.createTypeUnsafe)(this, type, params);
    }
    /**
     * @description Creates an instance of a type as registered
     */
    createTypeUnsafe(type, params, options) {
        return (0, types_create_1.createTypeUnsafe)(this, type, params, options);
    }
    // find a specific call
    findMetaCall(callIndex) {
        const [section, method] = [callIndex[0], callIndex[1]];
        return (0, util_1.assertReturn)(this.#metadataCalls[`${section}`] && this.#metadataCalls[`${section}`][`${method}`], () => `findMetaCall: Unable to find Call with index [${section}, ${method}]/[${callIndex.toString()}]`);
    }
    // finds an error
    findMetaError(errorIndex) {
        const [section, method] = (0, util_1.isU8a)(errorIndex)
            ? [errorIndex[0], errorIndex[1]]
            : [
                errorIndex.index.toNumber(),
                (0, util_1.isU8a)(errorIndex.error)
                    ? errorIndex.error[0]
                    : errorIndex.error.toNumber()
            ];
        return (0, util_1.assertReturn)(this.#metadataErrors[`${section}`] && this.#metadataErrors[`${section}`][`${method}`], () => `findMetaError: Unable to find Error with index [${section}, ${method}]/[${errorIndex.toString()}]`);
    }
    findMetaEvent(eventIndex) {
        const [section, method] = [eventIndex[0], eventIndex[1]];
        return (0, util_1.assertReturn)(this.#metadataEvents[`${section}`] && this.#metadataEvents[`${section}`][`${method}`], () => `findMetaEvent: Unable to find Event with index [${section}, ${method}]/[${eventIndex.toString()}]`);
    }
    get(name, withUnknown, knownTypeDef) {
        return this.getUnsafe(name, withUnknown, knownTypeDef);
    }
    getUnsafe(name, withUnknown, knownTypeDef) {
        let Type = this.#classes.get(name) || this.#knownDefaults.get(name);
        // we have not already created the type, attempt it
        if (!Type) {
            const definition = this.#definitions.get(name);
            let BaseType;
            // we have a definition, so create the class now (lazily)
            if (definition) {
                BaseType = (0, types_create_1.createClassUnsafe)(this, definition);
            }
            else if (knownTypeDef) {
                BaseType = (0, types_create_1.constructTypeClass)(this, knownTypeDef);
            }
            else if (withUnknown) {
                l.warn(`Unable to resolve type ${name}, it will fail on construction`);
                this.#unknownTypes.set(name, true);
                BaseType = types_codec_1.DoNotConstruct.with(name);
            }
            if (BaseType) {
                // NOTE If we didn't extend here, we would have strange artifacts. An example is
                // Balance, with this, new Balance() instanceof u128 is true, but Balance !== u128
                // Additionally, we now pass through the registry, which is a link to ourselves
                Type = class extends BaseType {
                };
                this.#classes.set(name, Type);
                // In the case of lookups, we also want to store the actual class against
                // the lookup name, instad of having to traverse again
                if (knownTypeDef && (0, util_1.isNumber)(knownTypeDef.lookupIndex)) {
                    this.#classes.set(this.createLookupType(knownTypeDef.lookupIndex), Type);
                }
            }
        }
        return Type;
    }
    getChainProperties() {
        return this.#chainProperties;
    }
    getClassName(Type) {
        // we cannot rely on export order (anymore, since babel/core 7.15.8), so in the case of
        // items such as u32 & U32, we get the lowercase versions here... not quite as optimal
        // (previously this used to be a simple find & return)
        const names = [];
        for (const [name, Clazz] of this.#knownDefaultsEntries) {
            if (Type === Clazz) {
                names.push(name);
            }
        }
        for (const [name, Clazz] of this.#classes.entries()) {
            if (Type === Clazz) {
                names.push(name);
            }
        }
        return names.length
            // both sort and reverse are done in-place
            // ['U32', 'u32'] -> ['u32', 'U32']
            ? names.sort().reverse()[0]
            : undefined;
    }
    getDefinition(typeName) {
        return this.#definitions.get(typeName);
    }
    getModuleInstances(specName, moduleName) {
        return this.#knownTypes?.typesBundle?.spec?.[specName.toString()]?.instances?.[moduleName] || this.#moduleMap[moduleName];
    }
    getOrThrow(name) {
        const Clazz = this.get(name);
        if (!Clazz) {
            throw new Error(`type ${name} not found`);
        }
        return Clazz;
    }
    getOrUnknown(name) {
        return this.get(name, true);
    }
    // Only used in extrinsic version 5
    getTransactionExtensionVersion() {
        return 0;
    }
    getSignedExtensionExtra() {
        return (0, index_js_1.expandExtensionTypes)(this.#signedExtensions, 'payload', this.#userExtensions);
    }
    getSignedExtensionTypes() {
        return (0, index_js_1.expandExtensionTypes)(this.#signedExtensions, 'extrinsic', this.#userExtensions);
    }
    hasClass(name) {
        return this.#classes.has(name) || !!this.#knownDefaults.has(name);
    }
    hasDef(name) {
        return this.#definitions.has(name);
    }
    hasType(name) {
        return !this.#unknownTypes.get(name) && (this.hasClass(name) || this.hasDef(name));
    }
    hash(data) {
        return this.createType('CodecHash', this.#hasher(data));
    }
    // eslint-disable-next-line no-dupe-class-members
    register(arg1, arg2) {
        // NOTE Constructors appear as functions here
        if ((0, util_1.isFunction)(arg1)) {
            this.#classes.set(arg1.name, arg1);
        }
        else if ((0, util_1.isString)(arg1)) {
            if (!(0, util_1.isFunction)(arg2)) {
                throw new Error(`Expected class definition passed to '${arg1}' registration`);
            }
            else if (arg1 === arg2.toString()) {
                throw new Error(`Unable to register circular ${arg1} === ${arg1}`);
            }
            this.#classes.set(arg1, arg2);
        }
        else {
            this.#registerObject(arg1);
        }
    }
    #registerObject = (obj) => {
        const entries = Object.entries(obj);
        for (let e = 0, count = entries.length; e < count; e++) {
            const [name, type] = entries[e];
            if ((0, util_1.isFunction)(type)) {
                // This _looks_ a bit funny, but `typeof Clazz === 'function'
                this.#classes.set(name, type);
            }
            else {
                const def = (0, util_1.isString)(type)
                    ? type
                    : (0, util_1.stringify)(type);
                if (name === def) {
                    throw new Error(`Unable to register circular ${name} === ${def}`);
                }
                // we already have this type, remove the classes registered for it
                if (this.#classes.has(name)) {
                    this.#classes.delete(name);
                }
                this.#definitions.set(name, def);
            }
        }
    };
    // sets the chain properties
    setChainProperties(properties) {
        if (properties) {
            this.#chainProperties = properties;
        }
    }
    setHasher(hasher) {
        this.#hasher = hasher || util_crypto_1.blake2AsU8a;
    }
    setKnownTypes(knownTypes) {
        this.#knownTypes = knownTypes;
    }
    setLookup(lookup) {
        this.#lookup = lookup;
        // register all applicable types found
        lookup.register();
    }
    // register alias types alongside the portable/lookup setup
    // (we don't combine this into setLookup since that would/could
    // affect stand-along lookups, such as ABIs which don't have
    // actual on-chain metadata)
    #registerLookup = (lookup) => {
        // attach the lookup before we register any types
        this.setLookup(lookup);
        // we detect based on runtime configuration
        let Weight = null;
        if (this.hasType('SpWeightsWeightV2Weight')) {
            // detection for WeightV2 type based on latest naming
            const weightv2 = this.createType('SpWeightsWeightV2Weight');
            Weight = weightv2.refTime && weightv2.proofSize
                // with both refTime & proofSize we use as-is (WeightV2)
                ? 'SpWeightsWeightV2Weight'
                // fallback to WeightV1 (WeightV1.5 is a struct, single field)
                : 'WeightV1';
        }
        else if (!(0, util_1.isBn)(this.createType('Weight'))) {
            // where we have an already-supplied BN override, we don't clobber
            // it with our detected value (This protects against pre-defines
            // where Weight may be aliassed to WeightV0, e.g. in early Kusama chains)
            Weight = 'WeightV1';
        }
        if (Weight) {
            // we have detected a version, adjust the definition
            this.register({ Weight });
        }
    };
    // sets the metadata
    setMetadata(metadata, signedExtensions, userExtensions, noInitWarn) {
        this.#metadata = metadata.asLatest;
        this.#metadataVersion = metadata.version;
        this.#firstCallIndex = null;
        // attach the lookup at this point and register relevant types (before injecting)
        this.#registerLookup(this.#metadata.lookup);
        injectExtrinsics(this, this.#metadata, this.#metadataVersion, this.#metadataCalls, this.#moduleMap);
        injectErrors(this, this.#metadata, this.#metadataVersion, this.#metadataErrors);
        injectEvents(this, this.#metadata, this.#metadataVersion, this.#metadataEvents);
        // set the default call index (the lowest section, the lowest method)
        // in most chains this should be 0,0
        const [defSection] = Object
            .keys(this.#metadataCalls)
            .sort(sortDecimalStrings);
        if (defSection) {
            const [defMethod] = Object
                .keys(this.#metadataCalls[defSection])
                .sort(sortDecimalStrings);
            if (defMethod) {
                this.#firstCallIndex = new Uint8Array([parseInt(defSection, 10), parseInt(defMethod, 10)]);
            }
        }
        // setup the available extensions
        this.setSignedExtensions(signedExtensions || (this.#metadata.extrinsic.versions.length > 0 && this.#metadata.extrinsic.versions.every((value) => value > 0)
            // FIXME Use the extension and their injected types
            ? this.#metadata.extrinsic.transactionExtensions.map(({ identifier }) => identifier.toString())
            : index_js_1.fallbackExtensions), userExtensions, noInitWarn);
        // setup the chain properties with format overrides
        this.setChainProperties(extractProperties(this, metadata));
    }
    // sets the available signed extensions
    setSignedExtensions(signedExtensions = index_js_1.fallbackExtensions, userExtensions, noInitWarn) {
        this.#signedExtensions = signedExtensions;
        this.#userExtensions = userExtensions;
        if (!noInitWarn) {
            const unknown = (0, index_js_1.findUnknownExtensions)(this.#signedExtensions, this.#userExtensions);
            if (unknown.length) {
                l.warn(`Unknown signed extensions ${unknown.join(', ')} found, treating them as no-effect`);
            }
        }
    }
}
exports.TypeRegistry = TypeRegistry;
