import { Client, ClientOptionsWithBytecode } from './public-types.js';
export { AddChainError, AddChainOptions, AlreadyDestroyedError, Chain, Client, ClientOptions, ClientOptionsWithBytecode, SmoldotBytecode, CrashError, QueueFullError, JsonRpcDisabledError, LogCallback } from './public-types.js';
/**
 * Initializes a new client. This is a pre-requisite to connecting to a blockchain.
 *
 * Can never fail.
 *
 * @param options Configuration of the client.
 */
export declare function startWithBytecode(options: ClientOptionsWithBytecode): Client;
