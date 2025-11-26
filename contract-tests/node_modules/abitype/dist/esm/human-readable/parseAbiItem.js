import { InvalidAbiItemError } from './errors/abiItem.js';
import { isStructSignature } from './runtime/signatures.js';
import { parseStructs } from './runtime/structs.js';
import { parseSignature } from './runtime/utils.js';
/**
 * Parses human-readable ABI item (e.g. error, event, function) into {@link Abi} item
 *
 * @param signature - Human-readable ABI item
 * @returns Parsed {@link Abi} item
 *
 * @example
 * const abiItem = parseAbiItem('function balanceOf(address owner) view returns (uint256)')
 * //    ^? const abiItem: { name: "balanceOf"; type: "function"; stateMutability: "view";...
 *
 * @example
 * const abiItem = parseAbiItem([
 *   //  ^? const abiItem: { name: "foo"; type: "function"; stateMutability: "view"; inputs:...
 *   'function foo(Baz bar) view returns (string)',
 *   'struct Baz { string name; }',
 * ])
 */
export function parseAbiItem(signature) {
    let abiItem;
    if (typeof signature === 'string')
        abiItem = parseSignature(signature);
    else {
        const structs = parseStructs(signature);
        const length = signature.length;
        for (let i = 0; i < length; i++) {
            const signature_ = signature[i];
            if (isStructSignature(signature_))
                continue;
            abiItem = parseSignature(signature_, structs);
            break;
        }
    }
    if (!abiItem)
        throw new InvalidAbiItemError({ signature });
    return abiItem;
}
//# sourceMappingURL=parseAbiItem.js.map