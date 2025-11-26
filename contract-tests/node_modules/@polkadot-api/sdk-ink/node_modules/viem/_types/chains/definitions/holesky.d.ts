export declare const holesky: {
    blockExplorers: {
        readonly default: {
            readonly name: "Etherscan";
            readonly url: "https://holesky.etherscan.io";
            readonly apiUrl: "https://api-holesky.etherscan.io/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 77;
        };
        readonly ensUniversalResolver: {
            readonly address: "0xeeeeeeee14d718c2b47d9923deab1335e144eeee";
            readonly blockCreated: 4295055;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 17000;
    name: "Holesky";
    nativeCurrency: {
        readonly name: "Holesky Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://ethereum-holesky-rpc.publicnode.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=holesky.d.ts.map