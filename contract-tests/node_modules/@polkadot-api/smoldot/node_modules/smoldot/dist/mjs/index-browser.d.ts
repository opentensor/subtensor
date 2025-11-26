/// <reference lib="dom" />
import { Client, ClientOptions } from './public-types.js';
export { AddChainError, AddChainOptions, AlreadyDestroyedError, Chain, Client, ClientOptions, ClientOptionsWithBytecode, SmoldotBytecode, CrashError, JsonRpcDisabledError, QueueFullError, LogCallback } from './public-types.js';
/**
 * Initializes a new client. This is a pre-requisite to connecting to a blockchain.
 *
 * Can never fail.
 *
 * @param options Configuration of the client. Defaults to `{}`.
 */
export declare function start(options?: ClientOptions): Client;
