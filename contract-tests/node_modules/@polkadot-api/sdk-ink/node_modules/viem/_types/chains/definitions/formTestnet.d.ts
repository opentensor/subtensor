export declare const formTestnet: {
    blockExplorers: {
        readonly default: {
            readonly name: "Form Testnet Explorer";
            readonly url: "https://sepolia-explorer.form.network";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly addressManager: {
            readonly 11155111: {
                readonly address: "0xd5C38fa934f7fd7477D4800F4f38a1c5BFdF1373";
            };
        };
        readonly l1CrossDomainMessenger: {
            readonly 11155111: {
                readonly address: "0x37A68565c4BE9700b3E3Ec60cC4416cAC3052FAa";
            };
        };
        readonly l2OutputOracle: {
            readonly 11155111: {
                readonly address: "0x9eA2239E65a59EC9C7F1ED4C116dD58Da71Fc1e2";
            };
        };
        readonly portal: {
            readonly 11155111: {
                readonly address: "0x60377e3cE15dF4CCA24c4beF076b60314240b032";
            };
        };
        readonly l1StandardBridge: {
            readonly 11155111: {
                readonly address: "0xD4531f633942b2725896F47cD2aFd260b44Ab1F7";
            };
        };
        readonly multicall3: {
            readonly address: "0xcA11bde05977b3631167028862bE2a173976CA11";
        };
        readonly gasPriceOracle: {
            readonly address: "0x420000000000000000000000000000000000000F";
        };
        readonly l1Block: {
            readonly address: "0x4200000000000000000000000000000000000015";
        };
        readonly l2CrossDomainMessenger: {
            readonly address: "0x4200000000000000000000000000000000000007";
        };
        readonly l2Erc721Bridge: {
            readonly address: "0x4200000000000000000000000000000000000014";
        };
        readonly l2StandardBridge: {
            readonly address: "0x4200000000000000000000000000000000000010";
        };
        readonly l2ToL1MessagePasser: {
            readonly address: "0x4200000000000000000000000000000000000016";
        };
    };
    ensTlds?: readonly string[] | undefined;
    id: 132902;
    name: "Form Testnet";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ethereum";
        readonly symbol: "ETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://sepolia-rpc.form.network/http"];
            readonly webSocket: readonly ["wss://sepolia-rpc.form.network/ws"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=formTestnet.d.ts.map