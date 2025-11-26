"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Decorate = void 0;
const rxjs_1 = require("rxjs");
const api_derive_1 = require("@polkadot/api-derive");
const rpc_core_1 = require("@polkadot/rpc-core");
const rpc_provider_1 = require("@polkadot/rpc-provider");
const types_1 = require("@polkadot/types");
const types_known_1 = require("@polkadot/types-known");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../submittable/index.js");
const augmentObject_js_1 = require("../util/augmentObject.js");
const decorate_js_1 = require("../util/decorate.js");
const validate_js_1 = require("../util/validate.js");
const Events_js_1 = require("./Events.js");
const find_js_1 = require("./find.js");
const PAGE_SIZE_K = 1000; // limit aligned with the 1k on the node (trie lookups are heavy)
const PAGE_SIZE_V = 250; // limited since the data may be > 16MB (e.g. misfiring elections)
const PAGE_SIZE_Q = 50; // queue of pending storage queries (mapped together, next tick)
const l = (0, util_1.logger)('api/init');
let instanceCounter = 0;
function getAtQueryFn(api, { method, section }) {
    return (0, util_1.assertReturn)(api.rx.query[section] && api.rx.query[section][method], () => `query.${section}.${method} is not available in this version of the metadata`);
}
class Decorate extends Events_js_1.Events {
    #instanceId;
    #runtimeLog = {};
    #registry;
    #storageGetQ = [];
    #storageSubQ = [];
    // HACK Use BN import so decorateDerive works... yes, wtf.
    __phantom = new util_1.BN(0);
    _type;
    _call = {};
    _consts = {};
    _derive;
    _errors = {};
    _events = {};
    _extrinsics;
    _extrinsicType = types_1.GenericExtrinsic.LATEST_EXTRINSIC_VERSION;
    _genesisHash;
    _isConnected;
    _isReady = false;
    _query = {};
    _queryMulti;
    _rpc;
    _rpcCore;
    _runtimeMap = {};
    _runtimeChain;
    _runtimeMetadata;
    _runtimeVersion;
    _rx = { call: {}, consts: {}, query: {}, tx: {} };
    _options;
    /**
     * This is the one and only method concrete children classes need to implement.
     * It's a higher-order function, which takes one argument
     * `method: Method extends (...args: any[]) => Observable<any>`
     * (and one optional `options`), and should return the user facing method.
     * For example:
     * - For ApiRx, `decorateMethod` should just be identity, because the input
     * function is already an Observable
     * - For ApiPromise, `decorateMethod` should return a function that takes all
     * the parameters from `method`, adds an optional `callback` argument, and
     * returns a Promise.
     *
     * We could easily imagine other user-facing interfaces, which are simply
     * implemented by transforming the Observable to Stream/Iterator/Kefir/Bacon
     * via `decorateMethod`.
     */
    _decorateMethod;
    /**
     * @description Create an instance of the class
     *
     * @param options Options object to create API instance or a Provider instance
     *
     * @example
     * <BR>
     *
     * ```javascript
     * import Api from '@polkadot/api/promise';
     *
     * const api = new Api().isReady();
     *
     * api.rpc.subscribeNewHeads((header) => {
     *   console.log(`new block #${header.number.toNumber()}`);
     * });
     * ```
     */
    constructor(options, type, decorateMethod) {
        super();
        this.#instanceId = `${++instanceCounter}`;
        this.#registry = options.source?.registry || options.registry || new types_1.TypeRegistry();
        this._rx.callAt = (blockHash, knownVersion) => (0, rxjs_1.from)(this.at(blockHash, knownVersion)).pipe((0, rxjs_1.map)((a) => a.rx.call));
        this._rx.queryAt = (blockHash, knownVersion) => (0, rxjs_1.from)(this.at(blockHash, knownVersion)).pipe((0, rxjs_1.map)((a) => a.rx.query));
        this._rx.registry = this.#registry;
        this._decorateMethod = decorateMethod;
        this._options = options;
        this._type = type;
        const provider = options.source
            ? options.source._rpcCore.provider.isClonable
                ? options.source._rpcCore.provider.clone()
                : options.source._rpcCore.provider
            : (options.provider || new rpc_provider_1.WsProvider());
        // The RPC interface decorates the known interfaces on init
        this._rpcCore = new rpc_core_1.RpcCore(this.#instanceId, this.#registry, {
            isPedantic: this._options.isPedantic,
            provider,
            rpcCacheCapacity: this._options.rpcCacheCapacity,
            ttl: this._options.provider?.ttl,
            userRpc: this._options.rpc
        });
        this._isConnected = new rxjs_1.BehaviorSubject(this._rpcCore.provider.isConnected);
        this._rx.hasSubscriptions = this._rpcCore.provider.hasSubscriptions;
    }
    /**
     * @description Return the current used registry
     */
    get registry() {
        return this.#registry;
    }
    /**
     * @description Creates an instance of a type as registered
     */
    createType(type, ...params) {
        return this.#registry.createType(type, ...params);
    }
    /**
     * @description Register additional user-defined of chain-specific types in the type registry
     */
    registerTypes(types) {
        types && this.#registry.register(types);
    }
    /**
     * @returns `true` if the API operates with subscriptions
     */
    get hasSubscriptions() {
        return this._rpcCore.provider.hasSubscriptions;
    }
    /**
     * @returns `true` if the API decorate multi-key queries
     */
    get supportMulti() {
        return this._rpcCore.provider.hasSubscriptions || !!this._rpcCore.state.queryStorageAt;
    }
    _emptyDecorated(registry, blockHash) {
        return {
            call: {},
            consts: {},
            errors: {},
            events: {},
            query: {},
            registry,
            rx: {
                call: {},
                query: {}
            },
            tx: (0, index_js_1.createSubmittable)(this._type, this._rx, this._decorateMethod, registry, blockHash)
        };
    }
    _createDecorated(registry, fromEmpty, decoratedApi, blockHash) {
        if (!decoratedApi) {
            decoratedApi = this._emptyDecorated(registry.registry, blockHash);
        }
        if (fromEmpty || !registry.decoratedMeta) {
            registry.decoratedMeta = (0, types_1.expandMetadata)(registry.registry, registry.metadata);
        }
        const runtime = this._decorateCalls(registry, this._decorateMethod, blockHash);
        const runtimeRx = this._decorateCalls(registry, this._rxDecorateMethod, blockHash);
        const storage = this._decorateStorage(registry.decoratedMeta, this._decorateMethod, blockHash);
        const storageRx = this._decorateStorage(registry.decoratedMeta, this._rxDecorateMethod, blockHash);
        (0, augmentObject_js_1.augmentObject)('consts', registry.decoratedMeta.consts, decoratedApi.consts, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('errors', registry.decoratedMeta.errors, decoratedApi.errors, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('events', registry.decoratedMeta.events, decoratedApi.events, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('query', storage, decoratedApi.query, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('query', storageRx, decoratedApi.rx.query, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('call', runtime, decoratedApi.call, fromEmpty);
        (0, augmentObject_js_1.augmentObject)('call', runtimeRx, decoratedApi.rx.call, fromEmpty);
        decoratedApi.findCall = (callIndex) => (0, find_js_1.findCall)(registry.registry, callIndex);
        decoratedApi.findError = (errorIndex) => (0, find_js_1.findError)(registry.registry, errorIndex);
        decoratedApi.queryMulti = blockHash
            ? this._decorateMultiAt(decoratedApi, this._decorateMethod, blockHash)
            : this._decorateMulti(this._decorateMethod);
        decoratedApi.runtimeVersion = registry.runtimeVersion;
        return {
            createdAt: blockHash,
            decoratedApi,
            decoratedMeta: registry.decoratedMeta
        };
    }
    _injectMetadata(registry, fromEmpty = false) {
        // clear the decoration, we are redoing it here
        if (fromEmpty || !registry.decoratedApi) {
            registry.decoratedApi = this._emptyDecorated(registry.registry);
        }
        const { decoratedApi, decoratedMeta } = this._createDecorated(registry, fromEmpty, registry.decoratedApi);
        this._call = decoratedApi.call;
        this._consts = decoratedApi.consts;
        this._errors = decoratedApi.errors;
        this._events = decoratedApi.events;
        this._query = decoratedApi.query;
        this._rx.call = decoratedApi.rx.call;
        this._rx.query = decoratedApi.rx.query;
        const tx = this._decorateExtrinsics(decoratedMeta, this._decorateMethod);
        const rxtx = this._decorateExtrinsics(decoratedMeta, this._rxDecorateMethod);
        if (fromEmpty || !this._extrinsics) {
            this._extrinsics = tx;
            this._rx.tx = rxtx;
        }
        else {
            (0, augmentObject_js_1.augmentObject)('tx', tx, this._extrinsics, false);
            (0, augmentObject_js_1.augmentObject)(null, rxtx, this._rx.tx, false);
        }
        (0, augmentObject_js_1.augmentObject)(null, decoratedMeta.consts, this._rx.consts, fromEmpty);
        this.emit('decorated');
    }
    /**
     * @deprecated
     * backwards compatible endpoint for metadata injection, may be removed in the future (However, it is still useful for testing injection)
     */
    injectMetadata(metadata, fromEmpty, registry) {
        this._injectMetadata({ counter: 0, metadata, registry: registry || this.#registry, runtimeVersion: this.#registry.createType('RuntimeVersionPartial') }, fromEmpty);
    }
    _decorateFunctionMeta(input, output) {
        output.meta = input.meta;
        output.method = input.method;
        output.section = input.section;
        output.toJSON = input.toJSON;
        if (input.callIndex) {
            output.callIndex = input.callIndex;
        }
        return output;
    }
    // Filter all RPC methods based on the results of the rpc_methods call. We do this in the following
    // manner to cater for both old and new:
    //   - when the number of entries are 0, only remove the ones with isOptional (account & contracts)
    //   - when non-zero, remove anything that is not in the array (we don't do this)
    _filterRpc(methods, additional) {
        // add any specific user-base RPCs
        if (Object.keys(additional).length !== 0) {
            this._rpcCore.addUserInterfaces(additional);
            // re-decorate, only adding any new additional interfaces
            this._decorateRpc(this._rpcCore, this._decorateMethod, this._rpc);
            this._decorateRpc(this._rpcCore, this._rxDecorateMethod, this._rx.rpc);
        }
        // extract the actual sections from the methods (this is useful when
        // we try and create mappings to runtime names via a hash mapping)
        const sectionMap = {};
        for (let i = 0, count = methods.length; i < count; i++) {
            const [section] = methods[i].split('_');
            sectionMap[section] = true;
        }
        // convert the actual section names into an easy name lookup
        const sections = Object.keys(sectionMap);
        for (let i = 0, count = sections.length; i < count; i++) {
            const nameA = (0, util_1.stringUpperFirst)(sections[i]);
            const nameB = `${nameA}Api`;
            this._runtimeMap[(0, util_crypto_1.blake2AsHex)(nameA, 64)] = nameA;
            this._runtimeMap[(0, util_crypto_1.blake2AsHex)(nameB, 64)] = nameB;
        }
        // finally we filter the actual methods to expose
        this._filterRpcMethods(methods);
    }
    _filterRpcMethods(exposed) {
        const hasResults = exposed.length !== 0;
        const allKnown = [...this._rpcCore.mapping.entries()];
        const allKeys = [];
        const count = allKnown.length;
        for (let i = 0; i < count; i++) {
            const [, { alias, endpoint, method, pubsub, section }] = allKnown[i];
            allKeys.push(`${section}_${method}`);
            if (pubsub) {
                allKeys.push(`${section}_${pubsub[1]}`);
                allKeys.push(`${section}_${pubsub[2]}`);
            }
            if (alias) {
                allKeys.push(...alias);
            }
            if (endpoint) {
                allKeys.push(endpoint);
            }
        }
        const unknown = exposed.filter((k) => !allKeys.includes(k) &&
            !k.includes('_unstable_'));
        if (unknown.length && !this._options.noInitWarn) {
            l.warn(`RPC methods not decorated: ${unknown.join(', ')}`);
        }
        // loop through all entries we have (populated in decorate) and filter as required
        // only remove when we have results and method missing, or with no results if optional
        for (let i = 0; i < count; i++) {
            const [k, { method, section }] = allKnown[i];
            if (hasResults && !exposed.includes(k) && k !== 'rpc_methods') {
                if (this._rpc[section]) {
                    delete this._rpc[section][method];
                    delete this._rx.rpc[section][method];
                }
            }
        }
    }
    _rpcSubmitter(decorateMethod) {
        const method = (method, ...params) => {
            return (0, rxjs_1.from)(this._rpcCore.provider.send(method, params));
        };
        return decorateMethod(method);
    }
    _decorateRpc(rpc, decorateMethod, input = this._rpcSubmitter(decorateMethod)) {
        const out = input;
        const decorateFn = (section, method) => {
            const source = rpc[section][method];
            const fn = decorateMethod(source, { methodName: method });
            fn.meta = source.meta;
            fn.raw = decorateMethod(source.raw, { methodName: method });
            return fn;
        };
        for (let s = 0, scount = rpc.sections.length; s < scount; s++) {
            const section = rpc.sections[s];
            if (!Object.prototype.hasOwnProperty.call(out, section)) {
                const methods = Object.keys(rpc[section]);
                const decorateInternal = (method) => decorateFn(section, method);
                for (let m = 0, mcount = methods.length; m < mcount; m++) {
                    const method = methods[m];
                    //  skip subscriptions where we have a non-subscribe interface
                    if (this.hasSubscriptions || !(method.startsWith('subscribe') || method.startsWith('unsubscribe'))) {
                        if (!Object.prototype.hasOwnProperty.call(out, section)) {
                            out[section] = {};
                        }
                        (0, util_1.lazyMethod)(out[section], method, decorateInternal);
                    }
                }
            }
        }
        return out;
    }
    // add all definition entries
    _addRuntimeDef(result, additional) {
        if (!additional) {
            return;
        }
        const entries = Object.entries(additional);
        for (let j = 0, ecount = entries.length; j < ecount; j++) {
            const [key, defs] = entries[j];
            if (result[key]) {
                // we have this one already, step through for new versions or
                // new methods and add those as applicable
                for (let k = 0, dcount = defs.length; k < dcount; k++) {
                    const def = defs[k];
                    const prev = result[key].find(({ version }) => def.version === version);
                    if (prev) {
                        // interleave the new methods with the old - last definition wins
                        (0, util_1.objectSpread)(prev.methods, def.methods);
                    }
                    else {
                        // we don't have this specific version, add it
                        result[key].push(def);
                    }
                }
            }
            else {
                // we don't have this runtime definition, add it as-is
                result[key] = defs;
            }
        }
    }
    // extract all runtime definitions
    _getRuntimeDefs(registry, specName, chain = '') {
        const result = {};
        const defValues = Object.values(types_1.typeDefinitions);
        // options > chain/spec > built-in, apply in reverse order with
        // methods overriding previous definitions (or interleave missing)
        for (let i = 0, count = defValues.length; i < count; i++) {
            this._addRuntimeDef(result, defValues[i].runtime);
        }
        this._addRuntimeDef(result, (0, types_known_1.getSpecRuntime)(registry, chain, specName));
        this._addRuntimeDef(result, this._options.runtime);
        return Object.entries(result);
    }
    // Helper for _getRuntimeDefsViaMetadata
    _getMethods(registry, methods) {
        const result = {};
        methods.forEach((m) => {
            const { docs, inputs, name, output } = m;
            result[name.toString()] = {
                description: docs.map((d) => d.toString()).join(),
                params: inputs.map(({ name, type }) => {
                    return { name: name.toString(), type: registry.lookup.getName(type) || registry.lookup.getTypeDef(type).type };
                }),
                type: registry.lookup.getName(output) || registry.lookup.getTypeDef(output).type
            };
        });
        return result;
    }
    // Maintains the same structure as `_getRuntimeDefs` in order to make conversion easier.
    _getRuntimeDefsViaMetadata(registry) {
        const result = {};
        const { apis } = registry.metadata;
        for (let i = 0, count = apis.length; i < count; i++) {
            const { methods, name } = apis[i];
            result[name.toString()] = [{
                    methods: this._getMethods(registry, methods),
                    // We set the version to 0 here since it will not be relevant when we are grabbing the runtime apis
                    // from the Metadata.
                    version: 0
                }];
        }
        return Object.entries(result);
    }
    // When the calls are available in the metadata, it will generate them based off of the metadata.
    // When they are not available it will use the hardcoded calls generated in the static types.
    _decorateCalls({ registry, runtimeVersion: { apis, specName, specVersion } }, decorateMethod, blockHash) {
        const result = {};
        const named = {};
        const hashes = {};
        const isApiInMetadata = registry.metadata.apis.length > 0;
        const sections = isApiInMetadata ? this._getRuntimeDefsViaMetadata(registry) : this._getRuntimeDefs(registry, specName, this._runtimeChain);
        const older = [];
        const implName = `${specName.toString()}/${specVersion.toString()}`;
        const hasLogged = this.#runtimeLog[implName] || false;
        this.#runtimeLog[implName] = true;
        if (isApiInMetadata) {
            for (let i = 0, scount = sections.length; i < scount; i++) {
                const [_section, secs] = sections[i];
                const sec = secs[0];
                const sectionHash = (0, util_crypto_1.blake2AsHex)(_section, 64);
                const section = (0, util_1.stringCamelCase)(_section);
                const methods = Object.entries(sec.methods);
                if (!named[section]) {
                    named[section] = {};
                }
                for (let m = 0, mcount = methods.length; m < mcount; m++) {
                    const [_method, def] = methods[m];
                    const method = (0, util_1.stringCamelCase)(_method);
                    named[section][method] = (0, util_1.objectSpread)({ method, name: `${_section}_${_method}`, section, sectionHash }, def);
                }
            }
        }
        else {
            for (let i = 0, scount = sections.length; i < scount; i++) {
                const [_section, secs] = sections[i];
                const sectionHash = (0, util_crypto_1.blake2AsHex)(_section, 64);
                const rtApi = apis.find(([a]) => a.eq(sectionHash));
                hashes[sectionHash] = true;
                if (rtApi) {
                    const all = secs.map(({ version }) => version).sort();
                    const sec = secs.find(({ version }) => rtApi[1].eq(version));
                    if (sec) {
                        const section = (0, util_1.stringCamelCase)(_section);
                        const methods = Object.entries(sec.methods);
                        if (methods.length) {
                            if (!named[section]) {
                                named[section] = {};
                            }
                            for (let m = 0, mcount = methods.length; m < mcount; m++) {
                                const [_method, def] = methods[m];
                                const method = (0, util_1.stringCamelCase)(_method);
                                named[section][method] = (0, util_1.objectSpread)({ method, name: `${_section}_${_method}`, section, sectionHash }, def);
                            }
                        }
                    }
                    else {
                        older.push(`${_section}/${rtApi[1].toString()} (${all.join('/')} known)`);
                    }
                }
            }
            // find the runtimes that we don't have hashes for
            const notFound = apis
                .map(([a, v]) => [a.toHex(), v.toString()])
                .filter(([a]) => !hashes[a])
                .map(([a, v]) => `${this._runtimeMap[a] || a}/${v}`);
            if (!this._options.noInitWarn && !hasLogged) {
                if (older.length) {
                    l.warn(`${implName}: Not decorating runtime apis without matching versions: ${older.join(', ')}`);
                }
                if (notFound.length) {
                    l.warn(`${implName}: Not decorating unknown runtime apis: ${notFound.join(', ')}`);
                }
            }
        }
        const stateCall = blockHash
            ? (name, bytes) => this._rpcCore.state.call(name, bytes, blockHash)
            : (name, bytes) => this._rpcCore.state.call(name, bytes);
        const lazySection = (section) => (0, util_1.lazyMethods)({}, Object.keys(named[section]), (method) => this._decorateCall(registry, named[section][method], stateCall, decorateMethod));
        const modules = Object.keys(named);
        for (let i = 0, count = modules.length; i < count; i++) {
            (0, util_1.lazyMethod)(result, modules[i], lazySection);
        }
        return result;
    }
    _decorateCall(registry, def, stateCall, decorateMethod) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const decorated = decorateMethod((...args) => {
            if (args.length !== def.params.length) {
                throw new Error(`${def.name}:: Expected ${def.params.length} arguments, found ${args.length}`);
            }
            const bytes = registry.createType('Raw', (0, util_1.u8aConcatStrict)(args.map((a, i) => registry.createTypeUnsafe(def.params[i].type, [a]).toU8a())));
            return stateCall(def.name, bytes).pipe((0, rxjs_1.map)((r) => registry.createTypeUnsafe(def.type, [r])));
        });
        decorated.meta = def;
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        return decorated;
    }
    // only be called if supportMulti is true
    _decorateMulti(decorateMethod) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        return decorateMethod((keys) => keys.length
            ? (this.hasSubscriptions
                ? this._rpcCore.state.subscribeStorage
                : this._rpcCore.state.queryStorageAt)(keys.map((args) => Array.isArray(args)
                ? args[0].creator.meta.type.isPlain
                    ? [args[0].creator]
                    : args[0].creator.meta.type.asMap.hashers.length === 1
                        ? [args[0].creator, args.slice(1)]
                        : [args[0].creator, ...args.slice(1)]
                : [args.creator]))
            : (0, rxjs_1.of)([]));
    }
    _decorateMultiAt(atApi, decorateMethod, blockHash) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        return decorateMethod((calls) => calls.length
            ? this._rpcCore.state.queryStorageAt(calls.map((args) => {
                if (Array.isArray(args)) {
                    const { creator } = getAtQueryFn(atApi, args[0].creator);
                    return creator.meta.type.isPlain
                        ? [creator]
                        : creator.meta.type.asMap.hashers.length === 1
                            ? [creator, args.slice(1)]
                            : [creator, ...args.slice(1)];
                }
                return [getAtQueryFn(atApi, args.creator).creator];
            }), blockHash)
            : (0, rxjs_1.of)([]));
    }
    _decorateExtrinsics({ tx }, decorateMethod) {
        const result = (0, index_js_1.createSubmittable)(this._type, this._rx, decorateMethod);
        const lazySection = (section) => (0, util_1.lazyMethods)({}, Object.keys(tx[section]), (method) => method.startsWith('$')
            ? tx[section][method]
            : this._decorateExtrinsicEntry(tx[section][method], result));
        const sections = Object.keys(tx);
        for (let i = 0, count = sections.length; i < count; i++) {
            (0, util_1.lazyMethod)(result, sections[i], lazySection);
        }
        return result;
    }
    _decorateExtrinsicEntry(method, creator) {
        const decorated = (...params) => creator(method(...params));
        // pass through the `.is`
        decorated.is = (other) => method.is(other);
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return
        return this._decorateFunctionMeta(method, decorated);
    }
    _decorateStorage({ query, registry }, decorateMethod, blockHash) {
        const result = {};
        const lazySection = (section) => (0, util_1.lazyMethods)({}, Object.keys(query[section]), (method) => blockHash
            ? this._decorateStorageEntryAt(registry, query[section][method], decorateMethod, blockHash)
            : this._decorateStorageEntry(query[section][method], decorateMethod));
        const sections = Object.keys(query);
        for (let i = 0, count = sections.length; i < count; i++) {
            (0, util_1.lazyMethod)(result, sections[i], lazySection);
        }
        return result;
    }
    _decorateStorageEntry(creator, decorateMethod) {
        const getArgs = (args, registry) => (0, validate_js_1.extractStorageArgs)(registry || this.#registry, creator, args);
        const getQueryAt = (blockHash) => (0, rxjs_1.from)(this.at(blockHash)).pipe((0, rxjs_1.map)((api) => getAtQueryFn(api, creator)));
        // Disable this where it occurs for each field we are decorating
        /* eslint-disable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment */
        const decorated = this._decorateStorageCall(creator, decorateMethod);
        decorated.creator = creator;
        // eslint-disable-next-line deprecation/deprecation
        decorated.at = decorateMethod((blockHash, ...args) => getQueryAt(blockHash).pipe((0, rxjs_1.switchMap)((q) => q(...args))));
        decorated.hash = decorateMethod((...args) => this._rpcCore.state.getStorageHash(getArgs(args)));
        decorated.is = (key) => key.section === creator.section &&
            key.method === creator.method;
        decorated.key = (...args) => (0, util_1.u8aToHex)((0, util_1.compactStripLength)(creator(...args))[1]);
        decorated.keyPrefix = (...args) => (0, util_1.u8aToHex)(creator.keyPrefix(...args));
        decorated.size = decorateMethod((...args) => this._rpcCore.state.getStorageSize(getArgs(args)));
        // eslint-disable-next-line deprecation/deprecation
        decorated.sizeAt = decorateMethod((blockHash, ...args) => getQueryAt(blockHash).pipe((0, rxjs_1.switchMap)((q) => this._rpcCore.state.getStorageSize(getArgs(args, q.creator.meta.registry), blockHash))));
        // .keys() & .entries() only available on map types
        if (creator.iterKey && creator.meta.type.isMap) {
            decorated.entries = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (...args) => this._retrieveMapEntries(creator, null, args)));
            // eslint-disable-next-line deprecation/deprecation
            decorated.entriesAt = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (blockHash, ...args) => getQueryAt(blockHash).pipe((0, rxjs_1.switchMap)((q) => this._retrieveMapEntries(q.creator, blockHash, args)))));
            decorated.entriesPaged = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (opts) => this._retrieveMapEntriesPaged(creator, undefined, opts)));
            decorated.keys = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (...args) => this._retrieveMapKeys(creator, null, args)));
            // eslint-disable-next-line deprecation/deprecation
            decorated.keysAt = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (blockHash, ...args) => getQueryAt(blockHash).pipe((0, rxjs_1.switchMap)((q) => this._retrieveMapKeys(q.creator, blockHash, args)))));
            decorated.keysPaged = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (opts) => this._retrieveMapKeysPaged(creator, undefined, opts)));
        }
        if (this.supportMulti && creator.meta.type.isMap) {
            // When using double map storage function, user need to pass double map key as an array
            decorated.multi = decorateMethod((args) => creator.meta.type.asMap.hashers.length === 1
                ? this._retrieveMulti(args.map((a) => [creator, [a]]))
                : this._retrieveMulti(args.map((a) => [creator, a])));
        }
        /* eslint-enable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment */
        return this._decorateFunctionMeta(creator, decorated);
    }
    _decorateStorageEntryAt(registry, creator, decorateMethod, blockHash) {
        const getArgs = (args) => (0, validate_js_1.extractStorageArgs)(registry, creator, args);
        // Disable this where it occurs for each field we are decorating
        /* eslint-disable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment */
        const decorated = decorateMethod((...args) => this._rpcCore.state.getStorage(getArgs(args), blockHash));
        decorated.creator = creator;
        decorated.hash = decorateMethod((...args) => this._rpcCore.state.getStorageHash(getArgs(args), blockHash));
        decorated.is = (key) => key.section === creator.section &&
            key.method === creator.method;
        decorated.key = (...args) => (0, util_1.u8aToHex)((0, util_1.compactStripLength)(creator(...args))[1]);
        decorated.keyPrefix = (...keys) => (0, util_1.u8aToHex)(creator.keyPrefix(...keys));
        decorated.size = decorateMethod((...args) => this._rpcCore.state.getStorageSize(getArgs(args), blockHash));
        // .keys() & .entries() only available on map types
        if (creator.iterKey && creator.meta.type.isMap) {
            decorated.entries = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (...args) => this._retrieveMapEntries(creator, blockHash, args)));
            decorated.entriesPaged = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (opts) => this._retrieveMapEntriesPaged(creator, blockHash, opts)));
            decorated.keys = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (...args) => this._retrieveMapKeys(creator, blockHash, args)));
            decorated.keysPaged = decorateMethod((0, rpc_core_1.memo)(this.#instanceId, (opts) => this._retrieveMapKeysPaged(creator, blockHash, opts)));
        }
        if (this.supportMulti && creator.meta.type.isMap) {
            // When using double map storage function, user need to pass double map key as an array
            decorated.multi = decorateMethod((args) => creator.meta.type.asMap.hashers.length === 1
                ? this._retrieveMulti(args.map((a) => [creator, [a]]), blockHash)
                : this._retrieveMulti(args.map((a) => [creator, a]), blockHash));
        }
        /* eslint-enable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment */
        return this._decorateFunctionMeta(creator, decorated);
    }
    _queueStorage(call, queue) {
        const query = queue === this.#storageSubQ
            ? this._rpcCore.state.subscribeStorage
            : this._rpcCore.state.queryStorageAt;
        let queueIdx = queue.length - 1;
        let valueIdx = 0;
        let valueObs;
        // if we don't have queue entries yet,
        // or the current queue has fired (see from below),
        // or the current queue has the max entries,
        // then we create a new queue
        if (queueIdx === -1 || !queue[queueIdx] || queue[queueIdx][1].length === PAGE_SIZE_Q) {
            queueIdx++;
            valueObs = (0, rxjs_1.from)(
            // we delay the execution until the next tick, this allows
            // any queries made in this timeframe to be added to the same
            // queue for a single query
            new Promise((resolve) => {
                (0, util_1.nextTick)(() => {
                    // get all the calls in this instance, resolve with it
                    // and then clear the queue so we don't add more
                    // (anything after this will be added to a new queue)
                    const calls = queue[queueIdx][1];
                    delete queue[queueIdx];
                    resolve(calls);
                });
            })).pipe((0, rxjs_1.switchMap)((calls) => query(calls)));
            queue.push([valueObs, [call]]);
        }
        else {
            valueObs = queue[queueIdx][0];
            valueIdx = queue[queueIdx][1].length;
            queue[queueIdx][1].push(call);
        }
        return valueObs.pipe(
        // return the single value at this index
        (0, rxjs_1.map)((values) => values[valueIdx]));
    }
    // Decorate the base storage call. In the case or rxjs or promise-without-callback (await)
    // we make a subscription, alternatively we push this through a single-shot query
    _decorateStorageCall(creator, decorateMethod) {
        const memoed = (0, rpc_core_1.memo)(this.#instanceId, (...args) => {
            const call = (0, validate_js_1.extractStorageArgs)(this.#registry, creator, args);
            if (!this.hasSubscriptions) {
                return this._rpcCore.state.getStorage(call);
            }
            return this._queueStorage(call, this.#storageSubQ);
        });
        return decorateMethod(memoed, {
            methodName: creator.method,
            overrideNoSub: (...args) => this._queueStorage((0, validate_js_1.extractStorageArgs)(this.#registry, creator, args), this.#storageGetQ)
        });
    }
    // retrieve a set of values for a specific set of keys - here we chunk the keys into PAGE_SIZE sizes
    _retrieveMulti(keys, blockHash) {
        if (!keys.length) {
            return (0, rxjs_1.of)([]);
        }
        const query = this.hasSubscriptions && !blockHash
            ? this._rpcCore.state.subscribeStorage
            : this._rpcCore.state.queryStorageAt;
        if (keys.length <= PAGE_SIZE_V) {
            return blockHash
                ? query(keys, blockHash)
                : query(keys);
        }
        return (0, rxjs_1.combineLatest)((0, util_1.arrayChunk)(keys, PAGE_SIZE_V).map((k) => blockHash
            ? query(k, blockHash)
            : query(k))).pipe((0, rxjs_1.map)(util_1.arrayFlatten));
    }
    _retrieveMapKeys({ iterKey, meta, method, section }, at, args) {
        if (!iterKey || !meta.type.isMap) {
            throw new Error('keys can only be retrieved on maps');
        }
        const headKey = iterKey(...args).toHex();
        const startSubject = new rxjs_1.BehaviorSubject(headKey);
        const query = at
            ? (startKey) => this._rpcCore.state.getKeysPaged(headKey, PAGE_SIZE_K, startKey, at)
            : (startKey) => this._rpcCore.state.getKeysPaged(headKey, PAGE_SIZE_K, startKey);
        const setMeta = (key) => key.setMeta(meta, section, method);
        return startSubject.pipe((0, rxjs_1.switchMap)(query), (0, rxjs_1.map)((keys) => keys.map(setMeta)), (0, rxjs_1.tap)((keys) => (0, util_1.nextTick)(() => {
            keys.length === PAGE_SIZE_K
                ? startSubject.next(keys[PAGE_SIZE_K - 1].toHex())
                : startSubject.complete();
        })), (0, rxjs_1.toArray)(), // toArray since we want to startSubject to be completed
        (0, rxjs_1.map)(util_1.arrayFlatten));
    }
    _retrieveMapKeysPaged({ iterKey, meta, method, section }, at, opts) {
        if (!iterKey || !meta.type.isMap) {
            throw new Error('keys can only be retrieved on maps');
        }
        const setMeta = (key) => key.setMeta(meta, section, method);
        const query = at
            ? (headKey) => this._rpcCore.state.getKeysPaged(headKey, opts.pageSize, opts.startKey || headKey, at)
            : (headKey) => this._rpcCore.state.getKeysPaged(headKey, opts.pageSize, opts.startKey || headKey);
        return query(iterKey(...opts.args).toHex()).pipe((0, rxjs_1.map)((keys) => keys.map(setMeta)));
    }
    _retrieveMapEntries(entry, at, args) {
        const query = at
            ? (keys) => this._rpcCore.state.queryStorageAt(keys, at)
            : (keys) => this._rpcCore.state.queryStorageAt(keys);
        return this._retrieveMapKeys(entry, at, args).pipe((0, rxjs_1.switchMap)((keys) => keys.length
            ? (0, rxjs_1.combineLatest)((0, util_1.arrayChunk)(keys, PAGE_SIZE_V).map(query)).pipe((0, rxjs_1.map)((valsArr) => (0, util_1.arrayFlatten)(valsArr).map((value, index) => [keys[index], value])))
            : (0, rxjs_1.of)([])));
    }
    _retrieveMapEntriesPaged(entry, at, opts) {
        const query = at
            ? (keys) => this._rpcCore.state.queryStorageAt(keys, at)
            : (keys) => this._rpcCore.state.queryStorageAt(keys);
        return this._retrieveMapKeysPaged(entry, at, opts).pipe((0, rxjs_1.switchMap)((keys) => keys.length
            ? query(keys).pipe((0, rxjs_1.map)((valsArr) => valsArr.map((value, index) => [keys[index], value])))
            : (0, rxjs_1.of)([])));
    }
    _decorateDeriveRx(decorateMethod) {
        const specName = this._runtimeVersion?.specName.toString();
        // Pull in derive from api-derive
        const available = (0, api_derive_1.getAvailableDerives)(this.#instanceId, this._rx, (0, util_1.objectSpread)({}, this._options.derives, this._options.typesBundle?.spec?.[specName || '']?.derives));
        return (0, decorate_js_1.decorateDeriveSections)(decorateMethod, available);
    }
    _decorateDerive(decorateMethod) {
        return (0, decorate_js_1.decorateDeriveSections)(decorateMethod, this._rx.derive);
    }
    /**
     * Put the `this.onCall` function of ApiRx here, because it is needed by
     * `api._rx`.
     */
    _rxDecorateMethod = (method) => {
        return method;
    };
}
exports.Decorate = Decorate;
