import type { Address } from 'abitype';
import type { UrlRequiredErrorType } from '../../errors/transport.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Hash, LogTopic } from '../../types/misc.js';
import type { RpcResponse } from '../../types/rpc.js';
import { type GetIpcRpcClientOptions, type IpcRpcClient } from '../../utils/rpc/ipc.js';
import { type CreateTransportErrorType, type Transport, type TransportConfig } from './createTransport.js';
type IpcTransportSubscribeParameters = {
    onData: (data: RpcResponse) => void;
    onError?: ((error: any) => void) | undefined;
};
type IpcTransportSubscribeReturnType = {
    subscriptionId: Hash;
    unsubscribe: () => Promise<RpcResponse<boolean>>;
};
type IpcTransportSubscribe = {
    subscribe(args: IpcTransportSubscribeParameters & ({
        params: ['newHeads'];
    } | {
        params: ['newPendingTransactions'];
    } | {
        params: [
            'logs',
            {
                address?: Address | Address[];
                topics?: LogTopic[];
            }
        ];
    } | {
        params: ['syncing'];
    })): Promise<IpcTransportSubscribeReturnType>;
};
export type IpcTransportConfig = {
    /** The key of the Ipc transport. */
    key?: TransportConfig['key'] | undefined;
    /** Methods to include or exclude from executing RPC requests. */
    methods?: TransportConfig['methods'] | undefined;
    /** The name of the Ipc transport. */
    name?: TransportConfig['name'] | undefined;
    /**
     * Whether or not to attempt to reconnect on socket failure.
     * @default true
     */
    reconnect?: GetIpcRpcClientOptions['reconnect'] | undefined;
    /** The max number of times to retry. */
    retryCount?: TransportConfig['retryCount'] | undefined;
    /** The base delay (in ms) between retries. */
    retryDelay?: TransportConfig['retryDelay'] | undefined;
    /** The timeout (in ms) for async Ipc requests. Default: 10_000 */
    timeout?: TransportConfig['timeout'] | undefined;
};
export type IpcTransport = Transport<'ipc', {
    getRpcClient(): Promise<IpcRpcClient>;
    subscribe: IpcTransportSubscribe['subscribe'];
}>;
export type IpcTransportErrorType = CreateTransportErrorType | UrlRequiredErrorType | ErrorType;
/**
 * @description Creates an IPC transport that connects to a JSON-RPC API.
 */
export declare function ipc(path: string, config?: IpcTransportConfig): IpcTransport;
export {};
//# sourceMappingURL=ipc.d.ts.map