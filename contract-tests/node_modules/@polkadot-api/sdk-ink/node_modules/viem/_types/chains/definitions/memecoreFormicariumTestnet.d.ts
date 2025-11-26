export declare const formicarium: {
    blockExplorers: {
        readonly default: {
            readonly name: "MemeCore Testnet Explorer";
            readonly url: "https://formicarium.memecorescan.io";
        };
        readonly okx: {
            readonly name: "MemeCore Testnet Explorer";
            readonly url: "https://web3.okx.com/explorer/formicarium-testnet";
        };
        readonly memecore: {
            readonly name: "MemeCore Testnet Explorer";
            readonly url: "https://formicarium.blockscout.memecore.com";
            readonly apiUrl: "https://formicarium.blockscout.memecore.com/api";
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
    id: 43521;
    name: "Formicarium";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "M";
        readonly symbol: "M";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.formicarium.memecore.net"];
            readonly webSocket: readonly ["wss://ws.formicarium.memecore.net"];
        };
    };
    sourceId?: number | undefined | undefined;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=memecoreFormicariumTestnet.d.ts.map