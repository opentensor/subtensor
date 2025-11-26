"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MockProvider = void 0;
const tslib_1 = require("tslib");
const eventemitter3_1 = require("eventemitter3");
const testing_1 = require("@polkadot/keyring/testing");
const types_1 = require("@polkadot/types");
const jsonrpc_1 = tslib_1.__importDefault(require("@polkadot/types/interfaces/jsonrpc"));
const Header_004_json_1 = tslib_1.__importDefault(require("@polkadot/types-support/json/Header.004.json"));
const SignedBlock_004_immortal_json_1 = tslib_1.__importDefault(require("@polkadot/types-support/json/SignedBlock.004.immortal.json"));
const static_substrate_1 = tslib_1.__importDefault(require("@polkadot/types-support/metadata/static-substrate"));
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const INTERVAL = 1000;
const SUBSCRIPTIONS = Array.prototype.concat.apply([], Object.values(jsonrpc_1.default).map((section) => Object
    .values(section)
    .filter(({ isSubscription }) => isSubscription)
    .map(({ jsonrpc }) => jsonrpc)
    .concat('chain_subscribeNewHead')));
const keyring = (0, testing_1.createTestKeyring)({ type: 'ed25519' });
const l = (0, util_1.logger)('api-mock');
/**
 * A mock provider mainly used for testing.
 * @return {ProviderInterface} The mock provider
 * @internal
 */
class MockProvider {
    db = {};
    emitter = new eventemitter3_1.EventEmitter();
    intervalId;
    isUpdating = true;
    registry;
    prevNumber = new util_1.BN(-1);
    requests = {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        chain_getBlock: () => this.registry.createType('SignedBlock', SignedBlock_004_immortal_json_1.default.result).toJSON(),
        chain_getBlockHash: () => '0x1234000000000000000000000000000000000000000000000000000000000000',
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        chain_getFinalizedHead: () => this.registry.createType('Header', Header_004_json_1.default.result).hash,
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        chain_getHeader: () => this.registry.createType('Header', Header_004_json_1.default.result).toJSON(),
        rpc_methods: () => this.registry.createType('RpcMethods').toJSON(),
        state_getKeys: () => [],
        state_getKeysPaged: () => [],
        state_getMetadata: () => static_substrate_1.default,
        state_getRuntimeVersion: () => this.registry.createType('RuntimeVersion').toHex(),
        state_getStorage: (storage, [key]) => (0, util_1.u8aToHex)(storage[key]),
        system_chain: () => 'mockChain',
        system_health: () => ({}),
        system_name: () => 'mockClient',
        system_properties: () => ({ ss58Format: 42 }),
        system_upgradedToTripleRefCount: () => this.registry.createType('bool', true),
        system_version: () => '9.8.7',
        // eslint-disable-next-line @typescript-eslint/no-unsafe-return, sort-keys
        dev_echo: (_, params) => params
    };
    subscriptions = SUBSCRIPTIONS.reduce((subs, name) => {
        subs[name] = {
            callbacks: {},
            lastValue: null
        };
        return subs;
    }, {});
    subscriptionId = 0;
    subscriptionMap = {};
    constructor(registry) {
        this.registry = registry;
        this.init();
    }
    get hasSubscriptions() {
        return !!true;
    }
    clone() {
        throw new Error('Unimplemented');
    }
    async connect() {
        // noop
    }
    // eslint-disable-next-line @typescript-eslint/require-await
    async disconnect() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }
    get isClonable() {
        return !!false;
    }
    get isConnected() {
        return !!true;
    }
    on(type, sub) {
        this.emitter.on(type, sub);
        return () => {
            this.emitter.removeListener(type, sub);
        };
    }
    // eslint-disable-next-line @typescript-eslint/require-await
    async send(method, params) {
        l.debug(() => ['send', method, params]);
        if (!this.requests[method]) {
            throw new Error(`provider.send: Invalid method '${method}'`);
        }
        return this.requests[method](this.db, params);
    }
    // eslint-disable-next-line @typescript-eslint/require-await
    async subscribe(_type, method, ...params) {
        l.debug(() => ['subscribe', method, params]);
        if (!this.subscriptions[method]) {
            throw new Error(`provider.subscribe: Invalid method '${method}'`);
        }
        const callback = params.pop();
        const id = ++this.subscriptionId;
        this.subscriptions[method].callbacks[id] = callback;
        this.subscriptionMap[id] = method;
        if (this.subscriptions[method].lastValue !== null) {
            callback(null, this.subscriptions[method].lastValue);
        }
        return id;
    }
    // eslint-disable-next-line @typescript-eslint/require-await
    async unsubscribe(_type, _method, id) {
        const sub = this.subscriptionMap[id];
        l.debug(() => ['unsubscribe', id, sub]);
        if (!sub) {
            throw new Error(`Unable to find subscription for ${id}`);
        }
        delete this.subscriptionMap[id];
        delete this.subscriptions[sub].callbacks[id];
        return true;
    }
    init() {
        const emitEvents = ['connected', 'disconnected'];
        let emitIndex = 0;
        let newHead = this.makeBlockHeader();
        let counter = -1;
        const metadata = new types_1.Metadata(this.registry, static_substrate_1.default);
        this.registry.setMetadata(metadata);
        const query = (0, types_1.decorateStorage)(this.registry, metadata.asLatest, metadata.version);
        // Do something every 1 seconds
        this.intervalId = setInterval(() => {
            if (!this.isUpdating) {
                return;
            }
            // create a new header (next block)
            newHead = this.makeBlockHeader();
            // increment the balances and nonce for each account
            keyring.getPairs().forEach(({ publicKey }, index) => {
                this.setStateBn(query['system']['account'](publicKey), newHead.number.toBn().addn(index));
            });
            // set the timestamp for the current block
            this.setStateBn(query['timestamp']['now'](), Math.floor(Date.now() / 1000));
            this.updateSubs('chain_subscribeNewHead', newHead);
            // We emit connected/disconnected at intervals
            if (++counter % 2 === 1) {
                if (++emitIndex === emitEvents.length) {
                    emitIndex = 0;
                }
                this.emitter.emit(emitEvents[emitIndex]);
            }
        }, INTERVAL);
    }
    makeBlockHeader() {
        const blockNumber = this.prevNumber.addn(1);
        const header = this.registry.createType('Header', {
            digest: {
                logs: []
            },
            extrinsicsRoot: (0, util_crypto_1.randomAsU8a)(),
            number: blockNumber,
            parentHash: blockNumber.isZero()
                ? new Uint8Array(32)
                : (0, util_1.bnToU8a)(this.prevNumber, { bitLength: 256, isLe: false }),
            stateRoot: (0, util_1.bnToU8a)(blockNumber, { bitLength: 256, isLe: false })
        });
        this.prevNumber = blockNumber;
        return header;
    }
    setStateBn(key, value) {
        this.db[(0, util_1.u8aToHex)(key)] = (0, util_1.bnToU8a)(value, { bitLength: 64, isLe: true });
    }
    updateSubs(method, value) {
        this.subscriptions[method].lastValue = value;
        Object
            .values(this.subscriptions[method].callbacks)
            .forEach((cb) => {
            try {
                cb(null, value.toJSON());
            }
            catch (error) {
                l.error(`Error on '${method}' subscription`, error);
            }
        });
    }
}
exports.MockProvider = MockProvider;
