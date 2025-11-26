"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createClient = createClient;
exports.rpcSchema = rpcSchema;
const parseAccount_js_1 = require("../accounts/utils/parseAccount.js");
const uid_js_1 = require("../utils/uid.js");
function createClient(parameters) {
    const { batch, chain, ccipRead, key = 'base', name = 'Base Client', type = 'base', } = parameters;
    const experimental_blockTag = parameters.experimental_blockTag ??
        (typeof chain?.experimental_preconfirmationTime === 'number'
            ? 'pending'
            : undefined);
    const blockTime = chain?.blockTime ?? 12_000;
    const defaultPollingInterval = Math.min(Math.max(Math.floor(blockTime / 2), 500), 4_000);
    const pollingInterval = parameters.pollingInterval ?? defaultPollingInterval;
    const cacheTime = parameters.cacheTime ?? pollingInterval;
    const account = parameters.account
        ? (0, parseAccount_js_1.parseAccount)(parameters.account)
        : undefined;
    const { config, request, value } = parameters.transport({
        chain,
        pollingInterval,
    });
    const transport = { ...config, ...value };
    const client = {
        account,
        batch,
        cacheTime,
        ccipRead,
        chain,
        key,
        name,
        pollingInterval,
        request,
        transport,
        type,
        uid: (0, uid_js_1.uid)(),
        ...(experimental_blockTag ? { experimental_blockTag } : {}),
    };
    function extend(base) {
        return (extendFn) => {
            const extended = extendFn(base);
            for (const key in client)
                delete extended[key];
            const combined = { ...base, ...extended };
            return Object.assign(combined, { extend: extend(combined) });
        };
    }
    return Object.assign(client, { extend: extend(client) });
}
function rpcSchema() {
    return null;
}
//# sourceMappingURL=createClient.js.map