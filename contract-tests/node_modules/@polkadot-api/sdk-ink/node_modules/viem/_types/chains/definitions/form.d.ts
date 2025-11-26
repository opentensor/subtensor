export declare const form: {
    blockExplorers: {
        readonly default: {
            readonly name: "Form Explorer";
            readonly url: "https://explorer.form.network";
        };
    };
    blockTime?: number | undefined | undefined;
    contracts: {
        readonly addressManager: {
            readonly 1: {
                readonly address: "0x15c249E46A2F924C2dB3A1560CF86729bAD1f07B";
            };
        };
        readonly l1CrossDomainMessenger: {
            readonly 1: {
                readonly address: "0xF333158DCCad1dF6C3F0a3aEe8BC31fA94d9eD5c";
            };
        };
        readonly l2OutputOracle: {
            readonly 1: {
                readonly address: "0x4ccAAF69F41c5810cA875183648B577CaCf1F67E";
            };
        };
        readonly portal: {
            readonly 1: {
                readonly address: "0x4E259Ee5F4136408908160dD32295A5031Fa426F";
            };
        };
        readonly l1StandardBridge: {
            readonly 1: {
                readonly address: "0xdc20aA63D3DE59574E065957190D8f24e0F7B8Ba";
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
    id: 478;
    name: "Form Network";
    nativeCurrency: {
        readonly decimals: 18;
        readonly name: "Ethereum";
        readonly symbol: "ETH";
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://rpc.form.network/http"];
            readonly webSocket: readonly ["wss://rpc.form.network/ws"];
        };
    };
    sourceId: 1;
    testnet?: boolean | undefined | undefined;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters?: undefined;
    serializers?: import("../../index.js").ChainSerializers<undefined, import("../../index.js").TransactionSerializable> | undefined;
};
//# sourceMappingURL=form.d.ts.map