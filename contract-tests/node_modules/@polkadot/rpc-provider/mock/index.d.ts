import type { Registry } from '@polkadot/types/types';
import type { ProviderInterface, ProviderInterfaceEmitCb, ProviderInterfaceEmitted } from '../types.js';
import type { MockStateSubscriptions } from './types.js';
/**
 * A mock provider mainly used for testing.
 * @return {ProviderInterface} The mock provider
 * @internal
 */
export declare class MockProvider implements ProviderInterface {
    private db;
    private emitter;
    private intervalId?;
    isUpdating: boolean;
    private registry;
    private prevNumber;
    private requests;
    subscriptions: MockStateSubscriptions;
    private subscriptionId;
    private subscriptionMap;
    constructor(registry: Registry);
    get hasSubscriptions(): boolean;
    clone(): MockProvider;
    connect(): Promise<void>;
    disconnect(): Promise<void>;
    get isClonable(): boolean;
    get isConnected(): boolean;
    on(type: ProviderInterfaceEmitted, sub: ProviderInterfaceEmitCb): () => void;
    send<T = any>(method: string, params: unknown[]): Promise<T>;
    subscribe(_type: string, method: string, ...params: unknown[]): Promise<number>;
    unsubscribe(_type: string, _method: string, id: number): Promise<boolean>;
    private init;
    private makeBlockHeader;
    private setStateBn;
    private updateSubs;
}
