/**
 * The substrate-connect package makes it possible to connect to Substrate-compatible blockchains with a light client.
 *
 * Connecting to a chain is done in two steps:
 *
 * 1. Call {@link createScClient}, which gives you a so-called *client*.
 * 2. Call {@link ScClient.addChain addChain} or {@link ScClient.addWellKnownChain addWellKnownChain} on this client.
 *
 * Note that this library is a low-level library where you directly send JSON-RPC requests and
 * receive responses.
 *
 * # Adding parachains
 *
 * Connecting to a parachain is done by obtaining a relay chain instance and then calling {@link Chain.addChain addChain}.
 *
 * ```js
 * const client = createScClient();
 * const relayChain = await client.addChain(relayChainSpec);
 * const parachain = await relayChain.addChain(parachainSpec);
 * ```
 *
 * While this will **not** work, and an exception will be thrown when adding the parachain:
 *
 * ```js
 * await createScClient().addChain(relayChainSpec);
 * await createScClient().addChain(parachainSpec);
 * ```
 *
 * # Resources sharing
 *
 * While calling {@link createScClient} multiple times leads to a different observable behaviour
 * when it comes to parachains (see previous section), internally resources are shared
 * between all the clients.
 *
 * In order words, it is not a problem to do this:
 *
 * ```js
 * const relayChainSpec = ...;
 * const chain1 = await createScClient().addChain(relayChainSpec);
 * const chain2 = await createScClient().addChain(relayChainSpec);
 * ```
 *
 * From an API perspective, `chain1` and `chain2` should be treated as two completely separate
 * connections to the same chain. Internally, however, only one "actual" connection to that chain
 * will exist.
 *
 * This means that there is no problem in calling {@link createScClient} from within a library for
 * example.
 *
 * # Well-known chains
 *
 * This package contains a list of so-called {@link WellKnownChain}s. This is a list of popular chains
 * that users are likely to connect to. Instead of calling `addChain` with a chain specification,
 * one can call `addWellKnownChain`, passing only the name of a well-known chain as parameter.
 *
 * Using {@link WellKnownChain}s doesn't provide any benefit when the substrate-connect extension is not
 * installed.
 *
 * If, however, the substrate-connect extension is installed, using {@link ScClient.addWellKnownChain addWellKnownChain} has several
 * benefits:
 *
 * - The web page that uses substrate-connect doesn't need to download the chain specification of
 * a well-known chain from the web server, as this chain specification is already known by the
 * extension.
 * - The extension starts connect to well-known chains when the browser initializes, meaning that
 * when {@link ScClient.addWellKnownChain addWellKnownChain} is called, it is likely that the chain in question has already been
 * fully synchronized.
 * - Furthermore, the extension stores the state of all the well-known chains in the browser's
 * local storage. This leads to a very quick initialization time.
 *
 * # Usage with a worker
 * By default, when the substrate-connect extension is not installed, {@link createScClient} will run the smoldot light
 * client entirely in the current thread. This can cause performance issues if other CPU-heavy operations are done in
 * that thread.
 *
 * In order to help with this, it possible to run the smoldot light client in a worker.
 * To do so, you must provide a {@link EmbeddedNodeConfig.workerFactory workerFactory} to {@link createScClient}
 * and setup the worker to import `@substrate/connect/worker`.
 *
 * For example
 *
 * ```js
 * // worker.mjs
 * import "@substrate/connect/worker"
 *
 * // main.mjs
 * import { createScClient } from "@substrate/connect"
 * createScClient({
 *   embeddedNodeConfig: {
 *     workerFactory: () => new Worker("./worker.mjs"),
 *   },
 * })
 * ```
 *
 * @packageDocumentation
 */

export { WellKnownChain } from "./WellKnownChain.js"
export * from "./connector/index.js"
