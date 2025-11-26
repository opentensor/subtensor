import type { JsonRpcRequest, JsonRpcResponse } from '../types.js';
/** @internal */
export declare class RpcCoder {
    #private;
    decodeResponse<T>(response?: JsonRpcResponse<T>): T;
    encodeJson(method: string, params: unknown[]): [number, string];
    encodeObject(method: string, params: unknown[]): [number, JsonRpcRequest];
}
