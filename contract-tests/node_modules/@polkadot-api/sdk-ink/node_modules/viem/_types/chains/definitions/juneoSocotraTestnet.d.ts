export declare const juneoSocotraTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Juneo Scan";
            readonly url: "https://socotra.juneoscan.io/chain/2";
            readonly apiUrl: "https://socotra.juneoscan.io/chain/2/api";
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
    id: 101003;
    name: "Socotra JUNE-Chain";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Socotra JUNE-Chain";
        readonly symbol: "JUNE";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.socotra-testnet.network/ext/bc/JUNE/rpc"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=juneoSocotraTestnet.d.ts.map