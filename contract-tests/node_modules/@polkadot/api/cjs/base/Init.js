"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Init = void 0;
const rxjs_1 = require("rxjs");
const types_1 = require("@polkadot/types");
const constants_1 = require("@polkadot/types/extrinsic/constants");
const types_known_1 = require("@polkadot/types-known");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const Decorate_js_1 = require("./Decorate.js");
const KEEPALIVE_INTERVAL = 10000;
const WITH_VERSION_SHORTCUT = false;
const SUPPORTED_METADATA_VERSIONS = [16, 15, 14];
const l = (0, util_1.logger)('api/init');
function textToString(t) {
    return t.toString();
}
class Init extends Decorate_js_1.Decorate {
    #atLast = null;
    #healthTimer = null;
    #registries = [];
    #updateSub = null;
    #waitingRegistries = {};
    constructor(options, type, decorateMethod) {
        super(options, type, decorateMethod);
        // all injected types added to the registry for overrides
        this.registry.setKnownTypes(options);
        // We only register the types (global) if this is not a cloned instance.
        // Do right up-front, so we get in the user types before we are actually
        // doing anything on-chain, this ensures we have the overrides in-place
        if (!options.source) {
            this.registerTypes(options.types);
        }
        else {
            this.#registries = options.source.#registries;
        }
        this._rpc = this._decorateRpc(this._rpcCore, this._decorateMethod);
        this._rx.rpc = this._decorateRpc(this._rpcCore, this._rxDecorateMethod);
        if (this.supportMulti) {
            this._queryMulti = this._decorateMulti(this._decorateMethod);
            this._rx.queryMulti = this._decorateMulti(this._rxDecorateMethod);
        }
        this._rx.signer = options.signer;
        this._rpcCore.setRegistrySwap((blockHash) => this.getBlockRegistry(blockHash));
        this._rpcCore.setResolveBlockHash((blockNumber) => (0, rxjs_1.firstValueFrom)(this._rpcCore.chain.getBlockHash(blockNumber)));
        if (this.hasSubscriptions) {
            this._rpcCore.provider.on('disconnected', () => this.#onProviderDisconnect());
            this._rpcCore.provider.on('error', (e) => this.#onProviderError(e));
            this._rpcCore.provider.on('connected', () => this.#onProviderConnect());
        }
        else if (!this._options.noInitWarn) {
            l.warn('Api will be available in a limited mode since the provider does not support subscriptions');
        }
        // If the provider was instantiated earlier, and has already emitted a
        // 'connected' event, then the `on('connected')` won't fire anymore. To
        // cater for this case, we call manually `this._onProviderConnect`.
        if (this._rpcCore.provider.isConnected) {
            this.#onProviderConnect().catch(util_1.noop);
        }
    }
    /**
     * @description Decorates a registry based on the runtime version
     */
    _initRegistry(registry, chain, version, metadata, chainProps) {
        registry.clearCache();
        registry.setChainProperties(chainProps || this.registry.getChainProperties());
        registry.setKnownTypes(this._options);
        registry.register((0, types_known_1.getSpecTypes)(registry, chain, version.specName, version.specVersion));
        registry.setHasher((0, types_known_1.getSpecHasher)(registry, chain, version.specName));
        // for bundled types, pull through the aliases defined
        if (registry.knownTypes.typesBundle) {
            registry.knownTypes.typesAlias = (0, types_known_1.getSpecAlias)(registry, chain, version.specName);
        }
        registry.setMetadata(metadata, undefined, (0, util_1.objectSpread)({}, (0, types_known_1.getSpecExtensions)(registry, chain, version.specName), this._options.signedExtensions), this._options.noInitWarn);
    }
    /**
     * @description Returns the default versioned registry
     */
    _getDefaultRegistry() {
        return (0, util_1.assertReturn)(this.#registries.find(({ isDefault }) => isDefault), 'Initialization error, cannot find the default registry');
    }
    /**
     * @description Returns a decorated API instance at a specific point in time
     */
    async at(blockHash, knownVersion) {
        const u8aHash = (0, util_1.u8aToU8a)(blockHash);
        const u8aHex = (0, util_1.u8aToHex)(u8aHash);
        const registry = await this.getBlockRegistry(u8aHash, knownVersion);
        if (!this.#atLast || this.#atLast[0] !== u8aHex) {
            // always create a new decoration - since we are pointing to a specific hash, this
            // means that all queries needs to use that hash (not a previous one already existing)
            this.#atLast = [u8aHex, this._createDecorated(registry, true, null, u8aHash).decoratedApi];
        }
        return this.#atLast[1];
    }
    async _createBlockRegistry(blockHash, header, version) {
        const registry = new types_1.TypeRegistry(blockHash);
        const metadata = await this._retrieveMetadata(version.apis, header.parentHash, registry);
        const runtimeChain = this._runtimeChain;
        if (!runtimeChain) {
            throw new Error('Invalid initializion order, runtimeChain is not available');
        }
        this._initRegistry(registry, runtimeChain, version, metadata);
        // add our new registry
        const result = { counter: 0, lastBlockHash: blockHash, metadata, registry, runtimeVersion: version };
        this.#registries.push(result);
        return result;
    }
    _cacheBlockRegistryProgress(key, creator) {
        // look for waiting resolves
        let waiting = this.#waitingRegistries[key];
        if ((0, util_1.isUndefined)(waiting)) {
            // nothing waiting, construct new
            waiting = this.#waitingRegistries[key] = new Promise((resolve, reject) => {
                creator()
                    .then((registry) => {
                    delete this.#waitingRegistries[key];
                    resolve(registry);
                })
                    .catch((error) => {
                    delete this.#waitingRegistries[key];
                    reject(error);
                });
            });
        }
        return waiting;
    }
    _getBlockRegistryViaVersion(blockHash, version) {
        if (version) {
            // check for pre-existing registries. We also check specName, e.g. it
            // could be changed like in Westmint with upgrade from shell -> westmint
            const existingViaVersion = this.#registries.find(({ runtimeVersion: { specName, specVersion } }) => specName.eq(version.specName) &&
                specVersion.eq(version.specVersion));
            if (existingViaVersion) {
                existingViaVersion.counter++;
                existingViaVersion.lastBlockHash = blockHash;
                return existingViaVersion;
            }
        }
        return null;
    }
    async _getBlockRegistryViaHash(blockHash) {
        // ensure we have everything required
        if (!this._genesisHash || !this._runtimeVersion) {
            throw new Error('Cannot retrieve data on an uninitialized chain');
        }
        // We have to assume that on the RPC layer the calls used here does not call back into
        // the registry swap, so getHeader & getRuntimeVersion should not be historic
        const header = this.registry.createType('HeaderPartial', this._genesisHash.eq(blockHash)
            ? { number: util_1.BN_ZERO, parentHash: this._genesisHash }
            : await (0, rxjs_1.firstValueFrom)(this._rpcCore.chain.getHeader.raw(blockHash)));
        if (header.parentHash.isEmpty) {
            l.warn(`Unable to retrieve header ${blockHash.toString()} and parent ${header.parentHash.toString()} from supplied hash`);
            throw new Error('Unable to retrieve header and parent from supplied hash');
        }
        // get the runtime version, either on-chain or via an known upgrade history
        const [firstVersion, lastVersion] = (0, types_known_1.getUpgradeVersion)(this._genesisHash, header.number);
        const version = this.registry.createType('RuntimeVersionPartial', WITH_VERSION_SHORTCUT && (firstVersion && (lastVersion ||
            firstVersion.specVersion.eq(this._runtimeVersion.specVersion)))
            ? { apis: firstVersion.apis, specName: this._runtimeVersion.specName, specVersion: firstVersion.specVersion }
            : await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getRuntimeVersion.raw(header.parentHash)));
        return (
        // try to find via version
        this._getBlockRegistryViaVersion(blockHash, version) ||
            // return new or in-flight result
            await this._cacheBlockRegistryProgress(version.toHex(), () => this._createBlockRegistry(blockHash, header, version)));
    }
    /**
     * @description Sets up a registry based on the block hash defined
     */
    async getBlockRegistry(blockHash, knownVersion) {
        return (
        // try to find via blockHash
        this.#registries.find(({ lastBlockHash }) => lastBlockHash && (0, util_1.u8aEq)(lastBlockHash, blockHash)) ||
            // try to find via version
            this._getBlockRegistryViaVersion(blockHash, knownVersion) ||
            // return new or in-flight result
            await this._cacheBlockRegistryProgress((0, util_1.u8aToHex)(blockHash), () => this._getBlockRegistryViaHash(blockHash)));
    }
    async _loadMeta() {
        // on re-connection to the same chain, we don't want to re-do everything from chain again
        if (this._isReady) {
            // on re-connection only re-subscribe to chain updates if we are not a clone
            if (!this._options.source) {
                this._subscribeUpdates();
            }
            return true;
        }
        this._unsubscribeUpdates();
        // only load from on-chain if we are not a clone (default path), alternatively
        // just use the values from the source instance provided
        [this._genesisHash, this._runtimeMetadata] = this._options.source?._isReady
            ? await this._metaFromSource(this._options.source)
            : await this._metaFromChain(this._options.metadata);
        return this._initFromMeta(this._runtimeMetadata);
    }
    // eslint-disable-next-line @typescript-eslint/require-await
    async _metaFromSource(source) {
        this._extrinsicType = source.extrinsicVersion;
        this._runtimeChain = source.runtimeChain;
        this._runtimeVersion = source.runtimeVersion;
        // manually build a list of all available methods in this RPC, we are
        // going to filter on it to align the cloned RPC without making a call
        const sections = Object.keys(source.rpc);
        const rpcs = [];
        for (let s = 0, scount = sections.length; s < scount; s++) {
            const section = sections[s];
            const methods = Object.keys(source.rpc[section]);
            for (let m = 0, mcount = methods.length; m < mcount; m++) {
                rpcs.push(`${section}_${methods[m]}`);
            }
        }
        this._filterRpc(rpcs, (0, types_known_1.getSpecRpc)(this.registry, source.runtimeChain, source.runtimeVersion.specName));
        return [source.genesisHash, source.runtimeMetadata];
    }
    // subscribe to metadata updates, inject the types on changes
    _subscribeUpdates() {
        if (this.#updateSub || !this.hasSubscriptions) {
            return;
        }
        this.#updateSub = this._rpcCore.state.subscribeRuntimeVersion().pipe((0, rxjs_1.switchMap)((version) => 
        // only retrieve the metadata when the on-chain version has been changed
        this._runtimeVersion?.specVersion.eq(version.specVersion)
            ? (0, rxjs_1.of)(false)
            : this._rpcCore.state.getMetadata().pipe((0, rxjs_1.map)((metadata) => {
                l.log(`Runtime version updated to spec=${version.specVersion.toString()}, tx=${version.transactionVersion.toString()}`);
                this._runtimeMetadata = metadata;
                this._runtimeVersion = version;
                this._rx.runtimeVersion = version;
                // update the default registry version
                const thisRegistry = this._getDefaultRegistry();
                const runtimeChain = this._runtimeChain;
                if (!runtimeChain) {
                    throw new Error('Invalid initializion order, runtimeChain is not available');
                }
                // setup the data as per the current versions
                thisRegistry.metadata = metadata;
                thisRegistry.runtimeVersion = version;
                this._initRegistry(this.registry, runtimeChain, version, metadata);
                this._injectMetadata(thisRegistry, true);
                return true;
            })))).subscribe();
    }
    async _metaFromChain(optMetadata) {
        const [genesisHash, runtimeVersion, chain, chainProps, rpcMethods] = await Promise.all([
            (0, rxjs_1.firstValueFrom)(this._rpcCore.chain.getBlockHash(0)),
            (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getRuntimeVersion()),
            (0, rxjs_1.firstValueFrom)(this._rpcCore.system.chain()),
            (0, rxjs_1.firstValueFrom)(this._rpcCore.system.properties()),
            (0, rxjs_1.firstValueFrom)(this._rpcCore.rpc.methods())
        ]);
        // set our chain version & genesisHash as returned
        this._runtimeChain = chain;
        this._runtimeVersion = runtimeVersion;
        this._rx.runtimeVersion = runtimeVersion;
        // retrieve metadata, either from chain  or as pass-in via options
        const metadataKey = `${genesisHash.toHex() || '0x'}-${runtimeVersion.specVersion.toString()}`;
        const metadata = optMetadata?.[metadataKey]
            ? new types_1.Metadata(this.registry, optMetadata[metadataKey])
            : await this._retrieveMetadata(runtimeVersion.apis);
        // initializes the registry & RPC
        this._initRegistry(this.registry, chain, runtimeVersion, metadata, chainProps);
        this._filterRpc(rpcMethods.methods.map(textToString), (0, types_known_1.getSpecRpc)(this.registry, chain, runtimeVersion.specName));
        this._subscribeUpdates();
        // setup the initial registry, when we have none
        if (!this.#registries.length) {
            this.#registries.push({ counter: 0, isDefault: true, metadata, registry: this.registry, runtimeVersion });
        }
        // get unique types & validate
        metadata.getUniqTypes(this._options.throwOnUnknown || false);
        return [genesisHash, metadata];
    }
    _initFromMeta(metadata) {
        const runtimeVersion = this._runtimeVersion;
        if (!runtimeVersion) {
            throw new Error('Invalid initializion order, runtimeVersion is not available');
        }
        // ExtrinsicV5 is not fully supported yet, for that reason we default to version 4
        this._extrinsicType = metadata.asLatest.extrinsic.versions.at(0) || constants_1.LATEST_EXTRINSIC_VERSION;
        this._rx.extrinsicType = this._extrinsicType;
        this._rx.genesisHash = this._genesisHash;
        this._rx.runtimeVersion = runtimeVersion;
        // inject metadata and adjust the types as detected
        this._injectMetadata(this._getDefaultRegistry(), true);
        // derive is last, since it uses the decorated rx
        this._rx.derive = this._decorateDeriveRx(this._rxDecorateMethod);
        this._derive = this._decorateDerive(this._decorateMethod);
        return true;
    }
    /**
     * @internal
     *
     * Tries to use runtime api calls to retrieve metadata. This ensures the api initializes with the latest metadata.
     * If the runtime call is not there it will use the rpc method.
     */
    async _retrieveMetadata(apis, at, registry) {
        let metadataVersion = null;
        const metadataApi = apis.find(([a]) => a.eq((0, util_crypto_1.blake2AsHex)('Metadata', 64)));
        const typeRegistry = registry || this.registry;
        // This chain does not have support for the metadataApi, or does not have the required version.
        if (!metadataApi || metadataApi[1].toNumber() < 2) {
            l.warn('MetadataApi not available, rpc::state::get_metadata will be used.');
            return at
                ? new types_1.Metadata(typeRegistry, await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getMetadata.raw(at)))
                : await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getMetadata());
        }
        try {
            const metadataVersionsAsBytes = at
                ? await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.call.raw('Metadata_metadata_versions', '0x', at))
                : await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.call('Metadata_metadata_versions', '0x'));
            const versions = typeRegistry.createType('Vec<u32>', metadataVersionsAsBytes);
            // For unstable versions of the metadata the last value is set to u32 MAX in the runtime. This ensures only supported stable versions are used.
            metadataVersion = versions.filter((ver) => SUPPORTED_METADATA_VERSIONS.includes(ver.toNumber())).reduce((largest, current) => current.gt(largest) ? current : largest);
        }
        catch (e) {
            l.debug(e.message);
            l.warn('error with state_call::Metadata_metadata_versions, rpc::state::get_metadata will be used');
        }
        // When the metadata version does not align with the latest supported versions we ensure not to call the metadata runtime call.
        // I noticed on some previous runtimes that have support for `Metadata_metadata_at_version` that very irregular versions were being returned.
        // This was evident with runtime 1000000 - it return a very large number. This ensures we always stick within what is supported.
        if (metadataVersion && !SUPPORTED_METADATA_VERSIONS.includes(metadataVersion.toNumber())) {
            metadataVersion = null;
        }
        if (metadataVersion) {
            try {
                const metadataBytes = at
                    ? await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.call.raw('Metadata_metadata_at_version', (0, util_1.u8aToHex)(metadataVersion.toU8a()), at))
                    : await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.call('Metadata_metadata_at_version', (0, util_1.u8aToHex)(metadataVersion.toU8a())));
                // When the metadata is called with `at` it is required to use `.raw`. Therefore since the length prefix is not present the
                // need to create a `Raw` type is necessary before creating the `OpaqueMetadata` type or else there will be a magic number
                // mismatch
                const rawMeta = at
                    ? typeRegistry.createType('Raw', metadataBytes).toU8a()
                    : metadataBytes;
                const opaqueMetadata = typeRegistry.createType('Option<OpaqueMetadata>', rawMeta).unwrapOr(null);
                if (opaqueMetadata) {
                    return new types_1.Metadata(typeRegistry, opaqueMetadata.toHex());
                }
            }
            catch (e) {
                l.debug(e.message);
                l.warn('error with state_call::Metadata_metadata_at_version, rpc::state::get_metadata will be used');
            }
        }
        return at
            ? new types_1.Metadata(typeRegistry, await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getMetadata.raw(at)))
            : await (0, rxjs_1.firstValueFrom)(this._rpcCore.state.getMetadata());
    }
    _subscribeHealth() {
        this._unsubscribeHealth();
        // Only enable the health keepalive on WS, not needed on HTTP
        this.#healthTimer = this.hasSubscriptions
            ? setInterval(() => {
                (0, rxjs_1.firstValueFrom)(this._rpcCore.system.health.raw()).catch(util_1.noop);
            }, KEEPALIVE_INTERVAL)
            : null;
    }
    _unsubscribeHealth() {
        if (this.#healthTimer) {
            clearInterval(this.#healthTimer);
            this.#healthTimer = null;
        }
    }
    _unsubscribeUpdates() {
        if (this.#updateSub) {
            this.#updateSub.unsubscribe();
            this.#updateSub = null;
        }
    }
    _unsubscribe() {
        this._unsubscribeHealth();
        this._unsubscribeUpdates();
    }
    async #onProviderConnect() {
        this._isConnected.next(true);
        this.emit('connected');
        try {
            const cryptoReady = this._options.initWasm === false
                ? true
                : await (0, util_crypto_1.cryptoWaitReady)();
            const hasMeta = await this._loadMeta();
            this._subscribeHealth();
            if (hasMeta && !this._isReady && cryptoReady) {
                this._isReady = true;
                this.emit('ready', this);
            }
        }
        catch (_error) {
            const error = new Error(`FATAL: Unable to initialize the API: ${_error.message}`);
            l.error(error);
            this.emit('error', error);
        }
    }
    #onProviderDisconnect() {
        this._isConnected.next(false);
        this._unsubscribe();
        this.emit('disconnected');
    }
    #onProviderError(error) {
        this.emit('error', error);
    }
}
exports.Init = Init;
