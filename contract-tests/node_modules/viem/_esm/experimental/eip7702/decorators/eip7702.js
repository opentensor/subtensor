import { prepareAuthorization, } from '../actions/prepareAuthorization.js';
import { signAuthorization, } from '../actions/signAuthorization.js';
/**
 * A suite of EIP-7702 Actions.
 *
 * @example
 * import { createClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { eip7702Actions } from 'viem/experimental'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(eip7702Actions())
 *
 * const hash = await client.signAuthorization({ ... })
 */
export function eip7702Actions() {
    return (client) => {
        return {
            prepareAuthorization: (parameters) => prepareAuthorization(client, parameters),
            signAuthorization: (parameters) => signAuthorization(client, parameters),
        };
    };
}
//# sourceMappingURL=eip7702.js.map