export declare const karura: {
    blockExplorers: {
        readonly default: {
            readonly name: "Karura Blockscout";
            readonly url: "https://blockscout.karura.network";
            readonly apiUrl: "https://blockscout.karura.network/api";
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
    id: 686;
    name: "Karura";
    nativeCurrency: {
        readonly name: "Karura";
        readonly symbol: "KAR";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://eth-rpc-karura.aca-api.network"];
            readonly webSocket: readonly ["wss://eth-rpc-karura.aca-api.network"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: false;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
    readonly network: "karura";
};
//# sourceMappingURL=karura.d.ts.map