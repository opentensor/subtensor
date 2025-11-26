export declare const confluxESpaceTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "ConfluxScan";
            readonly url: "https://evmtestnet.confluxscan.org";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xEFf0078910f638cd81996cc117bccD3eDf2B072F";
            readonly blockCreated: 117499050;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 71;
    name: "Conflux eSpace Testnet";
    nativeCurrency: {
        readonly name: "Conflux";
        readonly symbol: "CFX";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://evmtestnet.confluxrpc.com"];
            readonly webSocket: readonly ["wss://evmtestnet.confluxrpc.com/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "cfx-espace-testnet";
};
//# sourceMappingURL=confluxESpaceTestnet.d.ts.map