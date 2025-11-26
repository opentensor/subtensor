export declare const abi: readonly [{
    readonly type: "fallback";
    readonly stateMutability: "payable";
}, {
    readonly type: "receive";
    readonly stateMutability: "payable";
}, {
    readonly type: "function";
    readonly name: "execute";
    readonly inputs: readonly [{
        readonly name: "mode";
        readonly type: "bytes32";
        readonly internalType: "bytes32";
    }, {
        readonly name: "executionData";
        readonly type: "bytes";
        readonly internalType: "bytes";
    }];
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
}, {
    readonly type: "function";
    readonly name: "supportsExecutionMode";
    readonly inputs: readonly [{
        readonly name: "mode";
        readonly type: "bytes32";
        readonly internalType: "bytes32";
    }];
    readonly outputs: readonly [{
        readonly name: "result";
        readonly type: "bool";
        readonly internalType: "bool";
    }];
    readonly stateMutability: "view";
}, {
    readonly type: "error";
    readonly name: "FnSelectorNotRecognized";
    readonly inputs: readonly [];
}, {
    readonly type: "error";
    readonly name: "UnsupportedExecutionMode";
    readonly inputs: readonly [];
}];
export declare const executionMode: {
    readonly default: "0x0100000000000000000000000000000000000000000000000000000000000000";
    readonly opData: "0x0100000000007821000100000000000000000000000000000000000000000000";
    readonly batchOfBatches: "0x0100000000007821000200000000000000000000000000000000000000000000";
};
//# sourceMappingURL=constants.d.ts.map