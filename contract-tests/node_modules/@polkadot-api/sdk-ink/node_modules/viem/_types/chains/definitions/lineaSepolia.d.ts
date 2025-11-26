export declare const lineaSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "Etherscan";
            readonly url: "https://sepolia.lineascan.build";
            readonly apiUrl: "https://api-sepolia.lineascan.build/api";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 227427;
        };
        readonly ensRegistry: {
            readonly address: "0x5B2636F0f2137B4aE722C01dd5122D7d3e9541f7";
            readonly blockCreated: 2395094;
        };
        readonly ensUniversalResolver: {
            readonly address: "0x4D41762915F83c76EcaF6776d9b08076aA32b492";
            readonly blockCreated: 17168484;
        };
    };
    ensTlds: readonly [".linea.eth"];
    id: 59141;
    name: "Linea Sepolia Testnet";
    nativeCurrency: {
        readonly name: "Linea Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.sepolia.linea.build"];
            readonly webSocket: readonly ["wss://rpc.sepolia.linea.build"];
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
//# sourceMappingURL=lineaSepolia.d.ts.map