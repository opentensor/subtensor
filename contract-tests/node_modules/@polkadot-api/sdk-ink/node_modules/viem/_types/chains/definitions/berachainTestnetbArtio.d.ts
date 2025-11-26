export declare const berachainTestnetbArtio: {
    blockExplorers: {
        readonly default: {
            readonly name: "Berachain bArtio Beratrail";
            readonly url: "https://bartio.beratrail.io";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
            readonly blockCreated: 109269;
        };
        readonly ensRegistry: {
            readonly address: "0xB0eef18971290b333450586D33dcA6cE122651D2";
            readonly blockCreated: 7736794;
        };
        readonly ensUniversalResolver: {
            readonly address: "0x41692Ef1EA0C79E6b73077E4A67572D2BDbD7057";
            readonly blockCreated: 7736795;
        };
    };
    ensTlds: readonly [".bera"];
    id: 80084;
    name: "Berachain bArtio";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "BERA Token";
        readonly symbol: "BERA";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://bartio.rpc.berachain.com"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=berachainTestnetbArtio.d.ts.map