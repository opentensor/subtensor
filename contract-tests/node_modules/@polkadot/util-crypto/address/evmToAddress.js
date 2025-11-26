import { u8aConcat } from '@polkadot/util';
import { hasher } from '../secp256k1/hasher.js';
import { encodeAddress } from './encode.js';
/**
 * @name evmToAddress
 * @summary Converts an EVM address to its corresponding SS58 address.
 */
export function evmToAddress(evmAddress, ss58Format, hashType = 'blake2') {
    const message = u8aConcat('evm:', evmAddress);
    if (message.length !== 24) {
        throw new Error(`Converting ${evmAddress}: Invalid evm address length`);
    }
    return encodeAddress(hasher(hashType, message), ss58Format);
}
