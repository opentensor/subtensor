import { InvalidAbiParameterError } from './errors/abiParameter.js';
import { isStructSignature, modifiers } from './runtime/signatures.js';
import { parseStructs } from './runtime/structs.js';
import { parseAbiParameter as parseAbiParameter_ } from './runtime/utils.js';
/**
 * Parses human-readable ABI parameter into {@link AbiParameter}
 *
 * @param param - Human-readable ABI parameter
 * @returns Parsed {@link AbiParameter}
 *
 * @example
 * const abiParameter = parseAbiParameter('address from')
 * //    ^? const abiParameter: { type: "address"; name: "from"; }
 *
 * @example
 * const abiParameter = parseAbiParameter([
 *   //  ^? const abiParameter: { type: "tuple"; components: [{ type: "string"; name:...
 *   'Baz bar',
 *   'struct Baz { string name; }',
 * ])
 */
export function parseAbiParameter(param) {
    let abiParameter;
    if (typeof param === 'string')
        abiParameter = parseAbiParameter_(param, {
            modifiers,
        });
    else {
        const structs = parseStructs(param);
        const length = param.length;
        for (let i = 0; i < length; i++) {
            const signature = param[i];
            if (isStructSignature(signature))
                continue;
            abiParameter = parseAbiParameter_(signature, { modifiers, structs });
            break;
        }
    }
    if (!abiParameter)
        throw new InvalidAbiParameterError({ param });
    return abiParameter;
}
//# sourceMappingURL=parseAbiParameter.js.map