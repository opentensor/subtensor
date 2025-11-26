/** @entrypointCategory ERCs */
// biome-ignore lint/complexity/noUselessEmptyExport: tsdoc
export type {}

/**
 * Utility functions for working with [ERC-6492 wrapped signatures](https://eips.ethereum.org/EIPS/eip-6492#specification).
 *
 * @example
 * ```ts twoslash
 * import { PersonalMessage, Secp256k1, Signature } from 'ox'
 * import { SignatureErc6492 } from 'ox/erc6492' // [!code focus]
 *
 * const signature = Secp256k1.sign({
 *   payload: PersonalMessage.getSignPayload('0xdeadbeef'),
 *   privateKey: '0x...',
 * })
 *
 * const wrapped = SignatureErc6492.wrap({ // [!code focus]
 *   data: '0xcafebabe', // [!code focus]
 *   signature: Signature.toHex(signature), // [!code focus]
 *   to: '0xcafebabecafebabecafebabecafebabecafebabe', // [!code focus]
 * }) // [!code focus]
 * // @log: '0x000000000000000000000000cafebabecafebabecafebabecafebabecafebabe000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000004deadbeef000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000041fa78c5905fb0b9d6066ef531f962a62bc6ef0d5eb59ecb134056d206f75aaed7780926ff2601a935c2c79707d9e1799948c9f19dcdde1e090e903b19a07923d01c000000000000000000000000000000000000000000000000000000000000006492649264926492649264926492649264926492649264926492649264926492'
 * ```
 *
 * @category ERC-6492
 */
export * as SignatureErc6492 from './SignatureErc6492.js'
