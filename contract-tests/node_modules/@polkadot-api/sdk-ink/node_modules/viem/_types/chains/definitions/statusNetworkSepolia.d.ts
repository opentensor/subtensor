export declare const statusSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Blockscout";
            readonly url: "https://sepoliascan.status.network";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1578364;
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 1660990954;
    name: "Status Network Sepolia";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ether";
        readonly symbol: "ETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://public.sepolia.rpc.status.network"];
            readonly webSocket: readonly ["wss://public.sepolia.rpc.status.network/ws"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees: {
        readonly estimateFeesPerGas: ({ client, multiply, request, type, }: Parameters<import("../../index.js").ChainEstimateFeesPerGasFn>[0]) => ReturnType<import("../../index.js").ChainEstimateFeesPerGasFn>;
        readonly maxPriorityFeePerGas: ({ block, client, request }: import("../../index.js").ChainFeesFnParameters<import("../../index.js").ChainFormatters | undefined>) => Promise<bigint | null>;
    };
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=statusNetworkSepolia.d.ts.map