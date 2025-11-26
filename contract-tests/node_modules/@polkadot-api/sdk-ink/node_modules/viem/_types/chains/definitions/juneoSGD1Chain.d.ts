export declare const juneoSGD1Chain: {
    blockExplorers: {
        readonly default: {
            readonly name: "Juneo Scan";
            readonly url: "https://juneoscan.io/chain/7";
            readonly apiUrl: "https://juneoscan.io/chain/7/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts?: {
        [x: string]: import("../../index.js").ChainContract | {
            [sourceId: number]: import("../../index.js").ChainContract | undefined;
        } | undefined;
        ensRegistry?: import("../../index.js").ChainContract | undefined;
        ensUniversalResolver?: import("../../index.js").ChainContract | undefined;
        multicall3?: import("../../index.js").ChainContract | undefined;
        erc6492Verifier?: import("../../index.js").ChainContract | undefined;
    } | undefined;
    ensTlds?: readonly string[] | undefined;
    id: 45012;
    name: "Juneo SGD1-Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Juneo SGD1-Chain";
        readonly symbol: "SGD1";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.juneo-mainnet.network/ext/bc/SGD1/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=juneoSGD1Chain.d.ts.map