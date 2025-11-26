import type { ProviderInterface, ProviderInterfaceCallback, ProviderInterfaceEmitCb, ProviderInterfaceEmitted, ProviderStats } from '../types.js';
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
export declare class HttpProvider implements ProviderInterface {
    #private;
    /**
     * @param {string} endpoint The endpoint url starting with http://
     * @param {Record<string, string>} headers The headers provided to the underlying Http Endpoint
     * @param {number} [cacheCapacity] Custom size of the HttpProvider LRUCache. Defaults to `DEFAULT_CAPACITY` (1024)
     * @param {number} [cacheTtl] Custom TTL of the HttpProvider LRUCache. Determines how long an object can live in the cache. Defaults to `DEFAULT_TTL` (30000)
     */
    constructor(endpoint?: string, headers?: Record<string, string>, cacheCapacity?: number, cacheTtl?: number | null);
    /**
     * @summary `true` when this provider supports subscriptions
     */
    get hasSubscriptions(): boolean;
    /**
     * @description Returns a clone of the object
     */
    clone(): HttpProvider;
    /**
     * @description Manually connect from the connection
     */
    connect(): Promise<void>;
    /**
     * @description Manually disconnect from the connection
     */
    disconnect(): Promise<void>;
    /**
     * @description Returns the connection stats
     */
    get stats(): ProviderStats;
    /**
    * @description Returns the connection stats
    */
    get ttl(): number | null | undefined;
    /**
     * @summary `true` when this provider supports clone()
     */
    get isClonable(): boolean;
    /**
     * @summary Whether the node is connected or not.
     * @return {boolean} true if connected
     */
    get isConnected(): boolean;
    /**
     * @summary Events are not supported with the HttpProvider, see [[WsProvider]].
     * @description HTTP Provider does not have 'on' emitters. WebSockets should be used instead.
     */
    on(_type: ProviderInterfaceEmitted, _sub: ProviderInterfaceEmitCb): () => void;
    /**
     * @summary Send HTTP POST Request with Body to configured HTTP Endpoint.
     */
    send<T>(method: string, params: unknown[], isCacheable?: boolean): Promise<T>;
    /**
     * @summary Subscriptions are not supported with the HttpProvider, see [[WsProvider]].
     */
    subscribe(_types: string, _method: string, _params: unknown[], _cb: ProviderInterfaceCallback): Promise<number>;
    /**
     * @summary Subscriptions are not supported with the HttpProvider, see [[WsProvider]].
     */
    unsubscribe(_type: string, _method: string, _id: number): Promise<boolean>;
}
