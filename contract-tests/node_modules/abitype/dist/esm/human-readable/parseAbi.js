import { isStructSignature } from './runtime/signatures.js';
import { parseStructs } from './runtime/structs.js';
import { parseSignature } from './runtime/utils.js';
/**
 * Parses human-readable ABI into JSON {@link Abi}
 *
 * @param signatures - Human-Readable ABI
 * @returns Parsed {@link Abi}
 *
 * @example
 * const abi = parseAbi([
 *   //  ^? const abi: readonly [{ name: "balanceOf"; type: "function"; stateMutability:...
 *   'function balanceOf(address owner) view returns (uint256)',
 *   'event Transfer(address indexed from, address indexed to, uint256 amount)',
 * ])
 */
export function parseAbi(signatures) {
    const structs = parseStructs(signatures);
    const abi = [];
    const length = signatures.length;
    for (let i = 0; i < length; i++) {
        const signature = signatures[i];
        if (isStructSignature(signature))
            continue;
        abi.push(parseSignature(signature, structs));
    }
    return abi;
}
//# sourceMappingURL=parseAbi.js.map