export declare const megaethTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "MegaETH Testnet Explorer";
            readonly url: "https://www.megaexplorer.xyz/";
        };
    };
    blockTime: 1000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 6342;
    name: "MegaETH Testnet";
    nativeCurrency: {
        readonly name: "MegaETH Testnet Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://carrot.megaeth.com/rpc"];
            readonly webSocket: readonly ["wss://carrot.megaeth.com/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=megaethTestnet.d.ts.map