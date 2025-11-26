"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HttpProvider = void 0;
const tslib_1 = require("tslib");
const util_1 = require("@polkadot/util");
const x_fetch_1 = require("@polkadot/x-fetch");
const index_js_1 = require("../coder/index.js");
const defaults_js_1 = tslib_1.__importDefault(require("../defaults.js"));
const lru_js_1 = require("../lru.js");
const ERROR_SUBSCRIBE = 'HTTP Provider does not have subscriptions, use WebSockets instead';
const l = (0, util_1.logger)('api-http');
/**
 * # @polkadot/rpc-provider
 *
 * @name HttpProvider
 *
 * @description The HTTP Provider allows sending requests using HTTP to a HTTP RPC server TCP port. It does not support subscriptions so you won't be able to listen to events such as new blocks or balance changes. It is usually preferable using the [[WsProvider]].
 *
 * @example
 * <BR>
 *
 * ```javascript
 * import Api from '@polkadot/api/promise';
 * import { HttpProvider } from '@polkadot/rpc-provider';
 *
 * const provider = new HttpProvider('http://127.0.0.1:9933');
 * const api = new Api(provider);
 * ```
 *
 * @see [[WsProvider]]
 */
class HttpProvider {
    #callCache;
    #cacheCapacity;
    #coder;
    #endpoint;
    #headers;
    #stats;
    #ttl;
    /**
     * @param {string} endpoint The endpoint url starting with http://
     * @param {Record<string, string>} headers The headers provided to the underlying Http Endpoint
     * @param {number} [cacheCapacity] Custom size of the HttpProvider LRUCache. Defaults to `DEFAULT_CAPACITY` (1024)
     * @param {number} [cacheTtl] Custom TTL of the HttpProvider LRUCache. Determines how long an object can live in the cache. Defaults to `DEFAULT_TTL` (30000)
     */
    constructor(endpoint = defaults_js_1.default.HTTP_URL, headers = {}, cacheCapacity, cacheTtl) {
        if (!/^(https|http):\/\//.test(endpoint)) {
            throw new Error(`Endpoint should start with 'http://' or 'https://', received '${endpoint}'`);
        }
        this.#coder = new index_js_1.RpcCoder();
        this.#endpoint = endpoint;
        this.#headers = headers;
        this.#cacheCapacity = cacheCapacity === 0 ? 0 : cacheCapacity || lru_js_1.DEFAULT_CAPACITY;
        const ttl = cacheTtl === undefined ? lru_js_1.DEFAULT_TTL : cacheTtl;
        this.#callCache = new lru_js_1.LRUCache(cacheCapacity === 0 ? 0 : cacheCapacity || lru_js_1.DEFAULT_CAPACITY, ttl);
        this.#ttl = cacheTtl;
        this.#stats = {
            active: { requests: 0, subscriptions: 0 },
            total: { bytesRecv: 0, bytesSent: 0, cached: 0, errors: 0, requests: 0, subscriptions: 0, timeout: 0 }
        };
    }
    /**
     * @summary `true` when this provider supports subscriptions
     */
    get hasSubscriptions() {
        return !!false;
    }
    /**
     * @description Returns a clone of the object
     */
    clone() {
        return new HttpProvider(this.#endpoint, this.#headers);
    }
    /**
     * @description Manually connect from the connection
     */
    async connect() {
        // noop
    }
    /**
     * @description Manually disconnect from the connection
     */
    async disconnect() {
        // noop
    }
    /**
     * @description Returns the connection stats
     */
    get stats() {
        return this.#stats;
    }
    /**
    * @description Returns the connection stats
    */
    get ttl() {
        return this.#ttl;
    }
    /**
     * @summary `true` when this provider supports clone()
     */
    get isClonable() {
        return !!true;
    }
    /**
     * @summary Whether the node is connected or not.
     * @return {boolean} true if connected
     */
    get isConnected() {
        return !!true;
    }
    /**
     * @summary Events are not supported with the HttpProvider, see [[WsProvider]].
     * @description HTTP Provider does not have 'on' emitters. WebSockets should be used instead.
     */
    on(_type, _sub) {
        l.error('HTTP Provider does not have \'on\' emitters, use WebSockets instead');
        return util_1.noop;
    }
    /**
     * @summary Send HTTP POST Request with Body to configured HTTP Endpoint.
     */
    async send(method, params, isCacheable) {
        this.#stats.total.requests++;
        const [, body] = this.#coder.encodeJson(method, params);
        if (this.#cacheCapacity === 0) {
            return this.#send(body);
        }
        const cacheKey = isCacheable ? `${method}::${(0, util_1.stringify)(params)}` : '';
        let resultPromise = isCacheable
            ? this.#callCache.get(cacheKey)
            : null;
        if (!resultPromise) {
            resultPromise = this.#send(body);
            if (isCacheable) {
                this.#callCache.set(cacheKey, resultPromise);
            }
        }
        else {
            this.#stats.total.cached++;
        }
        return resultPromise;
    }
    async #send(body) {
        this.#stats.active.requests++;
        this.#stats.total.bytesSent += body.length;
        try {
            const response = await (0, x_fetch_1.fetch)(this.#endpoint, {
                body,
                headers: {
                    Accept: 'application/json',
                    'Content-Length': `${body.length}`,
                    'Content-Type': 'application/json',
                    ...this.#headers
                },
                method: 'POST'
            });
            if (!response.ok) {
                throw new Error(`[${response.status}]: ${response.statusText}`);
            }
            const result = await response.text();
            this.#stats.total.bytesRecv += result.length;
            const decoded = this.#coder.decodeResponse(JSON.parse(result));
            this.#stats.active.requests--;
            return decoded;
        }
        catch (e) {
            this.#stats.active.requests--;
            this.#stats.total.errors++;
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            const { method, params } = JSON.parse(body);
            const rpcError = e;
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            const failedRequest = `\nFailed HTTP Request: ${JSON.stringify({ method, params })}`;
            // Provide HTTP Request alongside the error
            rpcError.message = `${rpcError.message}${failedRequest}`;
            throw rpcError;
        }
    }
    /**
     * @summary Subscriptions are not supported with the HttpProvider, see [[WsProvider]].
     */
    // eslint-disable-next-line @typescript-eslint/require-await
    async subscribe(_types, _method, _params, _cb) {
        l.error(ERROR_SUBSCRIBE);
        throw new Error(ERROR_SUBSCRIBE);
    }
    /**
     * @summary Subscriptions are not supported with the HttpProvider, see [[WsProvider]].
     */
    // eslint-disable-next-line @typescript-eslint/require-await
    async unsubscribe(_type, _method, _id) {
        l.error(ERROR_SUBSCRIBE);
        throw new Error(ERROR_SUBSCRIBE);
    }
}
exports.HttpProvider = HttpProvider;
