/// <reference lib="dom" />
/**
 * Runs the CPU-heavy parts of smoldot. Must be passed a port whose other end is passed to
 * `ClientOptions.portToWorker`.
 *
 * Returns a `Promise` that is ready when the smoldot client is shut down (either because it
 * crashes or intentionally with a call to `Client.terminate`).
 * Since this function is asynchronous, this `Promise` is wrapped around another `Promise`. In
 * other words, the outer `Promise` is ready when execution starts, and the inner `Promise` is
 * ready when execution ends.
 */
export declare function run(messagePort: MessagePort): Promise<Promise<void>>;
