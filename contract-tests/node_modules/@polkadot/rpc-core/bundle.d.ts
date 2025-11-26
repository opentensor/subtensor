import type { ProviderInterface } from '@polkadot/rpc-provider/types';
import type { AnyNumber, DefinitionRpc, DefinitionRpcExt, DefinitionRpcSub, Registry } from '@polkadot/types/types';
import type { RpcCoreStats } from './types/index.js';
export { packageInfo } from './packageInfo.js';
export * from './util/index.js';
interface Options {
    isPedantic?: boolean;
    provider: ProviderInterface;
    /**
     * Custom size of the rpc LRUCache capacity. Defaults to `RPC_CORE_DEFAULT_CAPACITY` (1024 * 10 * 10)
     */
    rpcCacheCapacity?: number;
    ttl?: number | null;
    userRpc?: Record<string, Record<string, DefinitionRpc | DefinitionRpcSub>>;
}
/**
 * @name Rpc
 * @summary The API may use a HTTP or WebSockets provider.
 * @description It allows for querying a Polkadot Client Node.
 * WebSockets provider is recommended since HTTP provider only supports basic querying.
 *
 * ```mermaid
 * graph LR;
 *   A[Api] --> |WebSockets| B[WsProvider];
 *   B --> |endpoint| C[ws://127.0.0.1:9944]
 * ```
 *
 * @example
 * <BR>
 *
 * ```javascript
 * import Rpc from '@polkadot/rpc-core';
 * import { WsProvider } from '@polkadot/rpc-provider/ws';
 *
 * const provider = new WsProvider('ws://127.0.0.1:9944');
 * const rpc = new Rpc(provider);
 * ```
 */
export declare class RpcCore {
    #private;
    readonly mapping: Map<string, DefinitionRpcExt>;
    readonly provider: ProviderInterface;
    readonly sections: string[];
    /**
     * @constructor
     * Default constructor for the core RPC handler
     * @param {Registry} registry Type Registry
     * @param {ProviderInterface} options.provider An API provider using any of the supported providers (HTTP, SC or WebSocket)
     * @param {number} [options.rpcCacheCapacity] Custom size of the rpc LRUCache capacity. Defaults to `RPC_CORE_DEFAULT_CAPACITY` (1024 * 10 * 10)
     */
    constructor(instanceId: string, registry: Registry, { isPedantic, provider, rpcCacheCapacity, ttl, userRpc }: Options);
    /**
     * @description Returns the connected status of a provider
     */
    get isConnected(): boolean;
    /**
     * @description Manually connect from the attached provider
     */
    connect(): Promise<void>;
    /**
     * @description Manually disconnect from the attached provider
     */
    disconnect(): Promise<void>;
    /**
     * @description Returns the underlying core stats, including those from teh provider
     */
    get stats(): RpcCoreStats | undefined;
    /**
     * @description Sets a registry swap (typically from Api)
     */
    setRegistrySwap(registrySwap: (blockHash: Uint8Array) => Promise<{
        registry: Registry;
    }>): void;
    /**
     * @description Sets a function to resolve block hash from block number
     */
    setResolveBlockHash(resolveBlockHash: (blockNumber: AnyNumber) => Promise<Uint8Array>): void;
    addUserInterfaces(userRpc: Record<string, Record<string, DefinitionRpc | DefinitionRpcSub>>): void;
    private _memomize;
    private _formatResult;
    private _createMethodSend;
    private _createSubscriber;
    private _createMethodSubscribe;
    private _formatParams;
    private _formatOutput;
    private _formatStorageData;
    private _formatStorageSet;
    private _formatStorageSetEntry;
    private _setToCache;
    private _newType;
}
