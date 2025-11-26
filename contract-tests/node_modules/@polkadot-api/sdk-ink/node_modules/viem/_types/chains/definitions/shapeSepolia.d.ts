export declare const shapeSepolia: {
    blockExplorers: {
        readonly default: {
            readonly name: "blockscout";
            readonly url: "https://explorer-sepolia.shape.network/";
            readonly apiUrl: "https://explorer-sepolia.shape.network/api/v2";
        };
    };
    blockTime: 2000;
    contracts: {
        readonly multicall3: {
            readonly address: "0xca11bde05977b3631167028862be2a173976ca11";
            readonly blockCreated: 1;
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
    id: 11011;
    name: "Shape Sepolia Testnet";
    nativeCurrency: {
        readonly name: "Sepolia Ether";
        readonly symbol: "ETH";
        readonly decimals: 18;
    };
    experimental_preconfirmationTime?: number | undefined | undefined;
    rpcUrls: {
        readonly default: {
            readonly http: readonly ["https://sepolia.shape.network"];
        };
    };
    sourceId: 11155111;
    testnet: true;
    custom?: Record<string, unknown> | undefined;
    fees?: import("../../index.js").ChainFees<undefined> | undefined;
    formatters: {
        readonly block: {
            exclude: [] | undefined;
            format: (args: import("../index.js").OpStackRpcBlock, action?: string | undefined) => {
                baseFeePerGas: bigint | null;
                blobGasUsed: bigint;
                difficulty: bigint;
                excessBlobGas: bigint;
                extraData: import("../../index.js").Hex;
                gasLimit: bigint;
                gasUsed: bigint;
                hash: `0x${string}` | null;
                logsBloom: `0x${string}` | null;
                miner: import("abitype").Address;
                mixHash: import("../../index.js").Hash;
                nonce: `0x${string}` | null;
                number: bigint | null;
                parentBeaconBlockRoot?: `0x${string}` | undefined;
                parentHash: import("../../index.js").Hash;
                receiptsRoot: import("../../index.js").Hex;
                sealFields: import("../../index.js").Hex[];
                sha3Uncles: import("../../index.js").Hash;
                size: bigint;
                stateRoot: import("../../index.js").Hash;
                timestamp: bigint;
                totalDifficulty: bigint | null;
                transactions: `0x${string}`[] | import("../index.js").OpStackTransaction<boolean>[];
                transactionsRoot: import("../../index.js").Hash;
                uncles: import("../../index.js").Hash[];
                withdrawals?: import("../../index.js").Withdrawal[] | undefined | undefined;
                withdrawalsRoot?: `0x${string}` | undefined;
            } & {};
            type: "block";
        };
        readonly transaction: {
            exclude: [] | undefined;
            format: (args: import("../index.js").OpStackRpcTransaction, action?: string | undefined) => ({
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                from: import("abitype").Address;
                gas: bigint;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                nonce: number;
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                to: import("abitype").Address | null;
                transactionIndex: number | null;
                typeHex: import("../../index.js").Hex | null;
                v: bigint;
                value: bigint;
                yParity: number;
                gasPrice?: undefined | undefined;
                maxFeePerBlobGas?: undefined | undefined;
                maxFeePerGas: bigint;
                maxPriorityFeePerGas: bigint;
                isSystemTx?: boolean;
                mint?: bigint | undefined | undefined;
                sourceHash: import("../../index.js").Hex;
                type: "deposit";
            } | {
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                v: bigint;
                value: bigint;
                gas: bigint;
                to: import("abitype").Address | null;
                from: import("abitype").Address;
                nonce: number;
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                transactionIndex: number | null;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                typeHex: import("../../index.js").Hex | null;
                accessList?: undefined | undefined;
                authorizationList?: undefined | undefined;
                blobVersionedHashes?: undefined | undefined;
                chainId?: number | undefined;
                yParity?: undefined | undefined;
                type: "legacy";
                gasPrice: bigint;
                maxFeePerBlobGas?: undefined | undefined;
                maxFeePerGas?: undefined | undefined;
                maxPriorityFeePerGas?: undefined | undefined;
                isSystemTx?: undefined | undefined;
                mint?: undefined | undefined;
                sourceHash?: undefined | undefined;
            } | {
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                from: import("abitype").Address;
                gas: bigint;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                nonce: number;
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                to: import("abitype").Address | null;
                transactionIndex: number | null;
                typeHex: import("../../index.js").Hex | null;
                v: bigint;
                value: bigint;
                yParity: number;
                accessList: import("../../index.js").AccessList;
                authorizationList?: undefined | undefined;
                blobVersionedHashes?: undefined | undefined;
                chainId: number;
                type: "eip2930";
                gasPrice: bigint;
                maxFeePerBlobGas?: undefined | undefined;
                maxFeePerGas?: undefined | undefined;
                maxPriorityFeePerGas?: undefined | undefined;
                isSystemTx?: undefined | undefined;
                mint?: undefined | undefined;
                sourceHash?: undefined | undefined;
            } | {
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                from: import("abitype").Address;
                gas: bigint;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                nonce: number;
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                to: import("abitype").Address | null;
                transactionIndex: number | null;
                typeHex: import("../../index.js").Hex | null;
                v: bigint;
                value: bigint;
                yParity: number;
                accessList: import("../../index.js").AccessList;
                authorizationList?: undefined | undefined;
                blobVersionedHashes?: undefined | undefined;
                chainId: number;
                type: "eip1559";
                gasPrice?: undefined | undefined;
                maxFeePerBlobGas?: undefined | undefined;
                maxFeePerGas: bigint;
                maxPriorityFeePerGas: bigint;
                isSystemTx?: undefined | undefined;
                mint?: undefined | undefined;
                sourceHash?: undefined | undefined;
            } | {
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                from: import("abitype").Address;
                gas: bigint;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                nonce: number;
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                to: import("abitype").Address | null;
                transactionIndex: number | null;
                typeHex: import("../../index.js").Hex | null;
                v: bigint;
                value: bigint;
                yParity: number;
                accessList: import("../../index.js").AccessList;
                authorizationList?: undefined | undefined;
                blobVersionedHashes: readonly import("../../index.js").Hex[];
                chainId: number;
                type: "eip4844";
                gasPrice?: undefined | undefined;
                maxFeePerBlobGas: bigint;
                maxFeePerGas: bigint;
                maxPriorityFeePerGas: bigint;
                isSystemTx?: undefined | undefined;
                mint?: undefined | undefined;
                sourceHash?: undefined | undefined;
            } | {
                blockHash: `0x${string}` | null;
                blockNumber: bigint | null;
                from: import("abitype").Address;
                gas: bigint;
                hash: import("../../index.js").Hash;
                input: import("../../index.js").Hex;
                nonce: number;
                r: import("../../index.js").Hex;
                s: import("../../index.js").Hex;
                to: import("abitype").Address | null;
                transactionIndex: number | null;
                typeHex: import("../../index.js").Hex | null;
                v: bigint;
                value: bigint;
                yParity: number;
                accessList: import("../../index.js").AccessList;
                authorizationList: import("../../index.js").SignedAuthorizationList;
                blobVersionedHashes?: undefined | undefined;
                chainId: number;
                type: "eip7702";
                gasPrice?: undefined | undefined;
                maxFeePerBlobGas?: undefined | undefined;
                maxFeePerGas: bigint;
                maxPriorityFeePerGas: bigint;
                isSystemTx?: undefined | undefined;
                mint?: undefined | undefined;
                sourceHash?: undefined | undefined;
            }) & {};
            type: "transaction";
        };
        readonly transactionReceipt: {
            exclude: [] | undefined;
            format: (args: import("../index.js").OpStackRpcTransactionReceipt, action?: string | undefined) => {
                blobGasPrice?: bigint | undefined;
                blobGasUsed?: bigint | undefined;
                blockHash: import("../../index.js").Hash;
                blockNumber: bigint;
                contractAddress: import("abitype").Address | null | undefined;
                cumulativeGasUsed: bigint;
                effectiveGasPrice: bigint;
                from: import("abitype").Address;
                gasUsed: bigint;
                logs: import("../../index.js").Log<bigint, number, false>[];
                logsBloom: import("../../index.js").Hex;
                root?: `0x${string}` | undefined;
                status: "success" | "reverted";
                to: import("abitype").Address | null;
                transactionHash: import("../../index.js").Hash;
                transactionIndex: number;
                type: import("../../index.js").TransactionType;
                l1GasPrice: bigint | null;
                l1GasUsed: bigint | null;
                l1Fee: bigint | null;
                l1FeeScalar: number | null;
            } & {};
            type: "transactionReceipt";
        };
    };
    serializers: {
        readonly transaction: typeof import("../index.js").serializeTransactionOpStack;
    };
};
//# sourceMappingURL=shapeSepolia.d.ts.map