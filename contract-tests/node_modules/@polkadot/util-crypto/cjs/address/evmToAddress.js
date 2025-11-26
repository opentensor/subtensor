"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.evmToAddress = evmToAddress;
const util_1 = require("@polkadot/util");
const hasher_js_1 = require("../secp256k1/hasher.js");
const encode_js_1 = require("./encode.js");
/**
 * @name evmToAddress
 * @summary Converts an EVM address to its corresponding SS58 address.
 */
function evmToAddress(evmAddress, ss58Format, hashType = 'blake2') {
    const message = (0, util_1.u8aConcat)('evm:', evmAddress);
    if (message.length !== 24) {
        throw new Error(`Converting ${evmAddress}: Invalid evm address length`);
    }
    return (0, encode_js_1.encodeAddress)((0, hasher_js_1.hasher)(hashType, message), ss58Format);
}
