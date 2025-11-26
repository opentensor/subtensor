"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.connect = connect;
const requestAddresses_js_1 = require("../../../actions/wallet/requestAddresses.js");
const toHex_js_1 = require("../../../utils/encoding/toHex.js");
async function connect(client, parameters = {}) {
    const capabilities = formatRequestCapabilities(parameters.capabilities);
    const response = await (async () => {
        try {
            return await client.request({ method: 'wallet_connect', params: [{ capabilities, version: '1' }] }, { dedupe: true, retryCount: 0 });
        }
        catch (e) {
            const error = e;
            if (!parameters.capabilities &&
                (error.name === 'InvalidInputRpcError' ||
                    error.name === 'InvalidParamsRpcError' ||
                    error.name === 'MethodNotFoundRpcError' ||
                    error.name === 'MethodNotSupportedRpcError' ||
                    error.name === 'UnsupportedProviderMethodError')) {
                const addresses = await (0, requestAddresses_js_1.requestAddresses)(client);
                return {
                    accounts: addresses.map((address) => ({
                        address,
                        capabilities: {},
                    })),
                };
            }
            throw error;
        }
    })();
    return {
        ...response,
        accounts: (response.accounts ?? []).map((account) => ({
            ...account,
            capabilities: formatResponseCapabilities(account.capabilities),
        })),
    };
}
function formatRequestCapabilities(capabilities) {
    const { unstable_addSubAccount, unstable_getSubAccounts: getSubAccounts, unstable_signInWithEthereum, ...rest } = capabilities ?? {};
    const addSubAccount = unstable_addSubAccount
        ? {
            ...unstable_addSubAccount,
            account: {
                ...unstable_addSubAccount.account,
                ...(unstable_addSubAccount.account.chainId
                    ? {
                        chainId: (0, toHex_js_1.numberToHex)(unstable_addSubAccount.account.chainId),
                    }
                    : {}),
            },
        }
        : undefined;
    const { chainId, expirationTime, issuedAt, notBefore } = unstable_signInWithEthereum ?? {};
    const signInWithEthereum = unstable_signInWithEthereum
        ? {
            ...unstable_signInWithEthereum,
            chainId: (0, toHex_js_1.numberToHex)(chainId),
            ...(expirationTime
                ? {
                    expirationTime: expirationTime.toISOString(),
                }
                : {}),
            ...(issuedAt
                ? {
                    issuedAt: issuedAt.toISOString(),
                }
                : {}),
            ...(notBefore
                ? {
                    notBefore: notBefore.toISOString(),
                }
                : {}),
        }
        : undefined;
    return {
        ...rest,
        ...(addSubAccount
            ? {
                addSubAccount,
            }
            : {}),
        ...(getSubAccounts
            ? {
                getSubAccounts,
            }
            : {}),
        ...(signInWithEthereum
            ? {
                signInWithEthereum,
            }
            : {}),
    };
}
function formatResponseCapabilities(capabilities) {
    return Object.entries(capabilities ?? {}).reduce((capabilities, [key, value]) => {
        const k = (() => {
            if (key === 'signInWithEthereum')
                return 'unstable_signInWithEthereum';
            if (key === 'subAccounts')
                return 'unstable_subAccounts';
            return key;
        })();
        capabilities[k] = value;
        return capabilities;
    }, {});
}
//# sourceMappingURL=connect.js.map