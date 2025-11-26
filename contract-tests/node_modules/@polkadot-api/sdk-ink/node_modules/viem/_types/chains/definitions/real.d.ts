export declare const real: {
    blockExplorers: {
        readonly default: {
            readonly name: "re.al Explorer";
            readonly url: "https://explorer.re.al";
            readonly apiUrl: "https://explorer.re.al/api/v2";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 695;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 111188;
    name: "re.al";
    nativeCurrency: {
        readonly name: "reETH";
        readonly decimals: 18;
        readonly symbol: "reETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.realforreal.gelato.digital"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=real.d.ts.map