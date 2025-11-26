import { camelCaseKeys } from '../utils/camelCaseKeys.js';
/* @deprecated Use `debug_getRawTransaction` and `debug_getRawTransactions` instead. */
export async function getRawBlockTransactions(client, parameters) {
    const result = await client.request({
        method: 'zks_getRawBlockTransactions',
        params: [parameters.number],
    });
    return camelCaseKeys(result);
}
//# sourceMappingURL=getRawBlockTransactions.js.map