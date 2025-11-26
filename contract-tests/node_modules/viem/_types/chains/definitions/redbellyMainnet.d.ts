export declare const redbellyMainnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Routescan";
            readonly url: "https://redbelly.routescan.io";
            readonly apiUrl: "https://api.routescan.io/v2/network/mainnet/evm/151/etherscan/api";
        };
    };
    contracts?: import("../index.js").Prettify<{
        [key: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
    } & {
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        universalSignatureVerifier?: import("../../index.js").ChainContract | undefined;
    }> | undefined;
    id: 151;
    name: "Redbelly Network Mainnet";
    nativeCurrency: {
        readonly name: "Redbelly Native Coin";
        readonly symbol: "RBNT";
        readonly decimals: 18;
    };
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://governors.mainnet.redbelly.network"];
        };
    };
    sourceId?: number | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=redbellyMainnet.d.ts.map