export declare const creditCoin3Testnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://creditcoin-testnet.blockscout.com";
            readonly apiUrl: "https://creditcoin-testnet.blockscout.com/api";
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
    id: 102031;
    name: "Creditcoin Testnet";
    nativeCurrency: {
        readonly name: "Creditcoin Testnet";
        readonly symbol: "tCTC";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.cc3-testnet.creditcoin.network"];
            readonly webSocket: readonly ["wss://rpc.cc3-testnet.creditcoin.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=creditCoin3Testnet.d.ts.map