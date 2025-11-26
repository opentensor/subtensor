export declare const contractDeployerAbi: ({
    anonymous: boolean;
    inputs: {
        indexed: boolean;
        internalType: string;
        name: string;
        type: string;
    }[];
    name: string;
    type: string;
    outputs?: never;
    stateMutability?: never;
} | {
    inputs: {
        internalType: string;
        name: string;
        type: string;
    }[];
    name: string;
    outputs: {
        internalType: string;
        name: string;
        type: string;
    }[];
    stateMutability: string;
    type: string;
    anonymous?: never;
} | {
    inputs: ({
        components: {
            internalType: string;
            name: string;
            type: string;
        }[];
        internalType: string;
        name: string;
        type: string;
    } | {
        internalType: string;
        name: string;
        type: string;
        components?: never;
    })[];
    name: string;
    outputs: never[];
    stateMutability: string;
    type: string;
    anonymous?: never;
} | {
    inputs: {
        internalType: string;
        name: string;
        type: string;
    }[];
    name: string;
    outputs: {
        components: {
            internalType: string;
            name: string;
            type: string;
        }[];
        internalType: string;
        name: string;
        type: string;
    }[];
    stateMutability: string;
    type: string;
    anonymous?: never;
})[];
export declare const paymasterAbi: {
    inputs: {
        internalType: string;
        name: string;
        type: string;
    }[];
    name: string;
    outputs: never[];
    stateMutability: string;
    type: string;
}[];
export declare const l2SharedBridgeAbi: readonly [{
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l1Sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l2Receiver";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l2Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "FinalizeDeposit";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l2Sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l1Receiver";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l2Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "WithdrawalInitiated";
    readonly type: "event";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l1Sender";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l2Receiver";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes";
        readonly name: "_data";
        readonly type: "bytes";
    }];
    readonly name: "finalizeDeposit";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "l1Bridge";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "l1SharedBridge";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l2Token";
        readonly type: "address";
    }];
    readonly name: "l1TokenAddress";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }];
    readonly name: "l2TokenAddress";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l1Receiver";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l2Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }];
    readonly name: "withdraw";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}];
export declare const l1SharedBridgeAbi: readonly [{
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "from";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "BridgehubDepositBaseTokenInitiated";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "txDataHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "l2DepositTxHash";
        readonly type: "bytes32";
    }];
    readonly name: "BridgehubDepositFinalized";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "txDataHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "from";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "to";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "BridgehubDepositInitiated";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "to";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "ClaimedFailedDepositSharedBridge";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "l2DepositTxHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "from";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "to";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "LegacyDepositInitiated";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "to";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "WithdrawalFinalizedSharedBridge";
    readonly type: "event";
}, {
    readonly inputs: readonly [];
    readonly name: "BRIDGE_HUB";
    readonly outputs: readonly [{
        readonly internalType: "contract IBridgehub";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "L1_WETH_TOKEN";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_txDataHash";
        readonly type: "bytes32";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_txHash";
        readonly type: "bytes32";
    }];
    readonly name: "bridgehubConfirmL2Transaction";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_prevMsgSender";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2Value";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes";
        readonly name: "_data";
        readonly type: "bytes";
    }];
    readonly name: "bridgehubDeposit";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "bytes32";
            readonly name: "magicValue";
            readonly type: "bytes32";
        }, {
            readonly internalType: "address";
            readonly name: "l2Contract";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "l2Calldata";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes[]";
            readonly name: "factoryDeps";
            readonly type: "bytes[]";
        }, {
            readonly internalType: "bytes32";
            readonly name: "txDataHash";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct L2TransactionRequestTwoBridgesInner";
        readonly name: "request";
        readonly type: "tuple";
    }];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_prevMsgSender";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }];
    readonly name: "bridgehubDepositBaseToken";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_depositSender";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_l2TxHash";
        readonly type: "bytes32";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }];
    readonly name: "claimFailedDeposit";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_depositSender";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_l2TxHash";
        readonly type: "bytes32";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }];
    readonly name: "claimFailedDepositLegacyErc20Bridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_l2TxHash";
        readonly type: "bytes32";
    }];
    readonly name: "depositHappened";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_msgSender";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l2Receiver";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2TxGasLimit";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2TxGasPerPubdataByte";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_refundRecipient";
        readonly type: "address";
    }];
    readonly name: "depositLegacyErc20Bridge";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "txHash";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes";
        readonly name: "_message";
        readonly type: "bytes";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }];
    readonly name: "finalizeWithdrawal";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes";
        readonly name: "_message";
        readonly type: "bytes";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }];
    readonly name: "finalizeWithdrawalLegacyErc20Bridge";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "l1Receiver";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "l1Token";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }];
    readonly name: "isWithdrawalFinalized";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }];
    readonly name: "l2BridgeAddress";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "legacyBridge";
    readonly outputs: readonly [{
        readonly internalType: "contract IL1ERC20Bridge";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }];
    readonly name: "receiveEth";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_eraLegacyBridgeLastDepositBatch";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_eraLegacyBridgeLastDepositTxNumber";
        readonly type: "uint256";
    }];
    readonly name: "setEraLegacyBridgeLastDepositTime";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_eraPostDiamondUpgradeFirstBatch";
        readonly type: "uint256";
    }];
    readonly name: "setEraPostDiamondUpgradeFirstBatch";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_eraPostLegacyBridgeUpgradeFirstBatch";
        readonly type: "uint256";
    }];
    readonly name: "setEraPostLegacyBridgeUpgradeFirstBatch";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}];
export declare const ethTokenAbi: readonly [{
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "Mint";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "from";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "to";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "value";
        readonly type: "uint256";
    }];
    readonly name: "Transfer";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "_l2Sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "_l1Receiver";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }];
    readonly name: "Withdrawal";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "_l2Sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "_l1Receiver";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes";
        readonly name: "_additionalData";
        readonly type: "bytes";
    }];
    readonly name: "WithdrawalWithMessage";
    readonly type: "event";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly name: "balanceOf";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "decimals";
    readonly outputs: readonly [{
        readonly internalType: "uint8";
        readonly name: "";
        readonly type: "uint8";
    }];
    readonly stateMutability: "pure";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_account";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }];
    readonly name: "mint";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "name";
    readonly outputs: readonly [{
        readonly internalType: "string";
        readonly name: "";
        readonly type: "string";
    }];
    readonly stateMutability: "pure";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "symbol";
    readonly outputs: readonly [{
        readonly internalType: "string";
        readonly name: "";
        readonly type: "string";
    }];
    readonly stateMutability: "pure";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "totalSupply";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_from";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_to";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_amount";
        readonly type: "uint256";
    }];
    readonly name: "transferFromTo";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l1Receiver";
        readonly type: "address";
    }];
    readonly name: "withdraw";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_l1Receiver";
        readonly type: "address";
    }, {
        readonly internalType: "bytes";
        readonly name: "_additionalData";
        readonly type: "bytes";
    }];
    readonly name: "withdrawWithMessage";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}];
export declare const bridgehubAbi: readonly [{
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "oldAdmin";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "newAdmin";
        readonly type: "address";
    }];
    readonly name: "NewAdmin";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "stateTransitionManager";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "chainGovernance";
        readonly type: "address";
    }];
    readonly name: "NewChain";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "oldPendingAdmin";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "newPendingAdmin";
        readonly type: "address";
    }];
    readonly name: "NewPendingAdmin";
    readonly type: "event";
}, {
    readonly inputs: readonly [];
    readonly name: "acceptAdmin";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_stateTransitionManager";
        readonly type: "address";
    }];
    readonly name: "addStateTransitionManager";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_token";
        readonly type: "address";
    }];
    readonly name: "addToken";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }];
    readonly name: "baseToken";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_stateTransitionManager";
        readonly type: "address";
    }, {
        readonly internalType: "address";
        readonly name: "_baseToken";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_salt";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "_admin";
        readonly type: "address";
    }, {
        readonly internalType: "bytes";
        readonly name: "_initData";
        readonly type: "bytes";
    }];
    readonly name: "createNewChain";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }];
    readonly name: "getHyperchain";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_gasPrice";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasLimit";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasPerPubdataByteLimit";
        readonly type: "uint256";
    }];
    readonly name: "l2TransactionBaseCost";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes32";
        readonly name: "_l2TxHash";
        readonly type: "bytes32";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }, {
        readonly internalType: "enum TxStatus";
        readonly name: "_status";
        readonly type: "uint8";
    }];
    readonly name: "proveL1ToL2TransactionStatus";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_index";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint8";
            readonly name: "l2ShardId";
            readonly type: "uint8";
        }, {
            readonly internalType: "bool";
            readonly name: "isService";
            readonly type: "bool";
        }, {
            readonly internalType: "uint16";
            readonly name: "txNumberInBatch";
            readonly type: "uint16";
        }, {
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "bytes32";
            readonly name: "key";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "value";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct L2Log";
        readonly name: "_log";
        readonly type: "tuple";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_proof";
        readonly type: "bytes32[]";
    }];
    readonly name: "proveL2LogInclusion";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_index";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint16";
            readonly name: "txNumberInBatch";
            readonly type: "uint16";
        }, {
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "data";
            readonly type: "bytes";
        }];
        readonly internalType: "struct L2Message";
        readonly name: "_message";
        readonly type: "tuple";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_proof";
        readonly type: "bytes32[]";
    }];
    readonly name: "proveL2MessageInclusion";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_stateTransitionManager";
        readonly type: "address";
    }];
    readonly name: "removeStateTransitionManager";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint256";
            readonly name: "chainId";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "mintValue";
            readonly type: "uint256";
        }, {
            readonly internalType: "address";
            readonly name: "l2Contract";
            readonly type: "address";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2Value";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes";
            readonly name: "l2Calldata";
            readonly type: "bytes";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasPerPubdataByteLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes[]";
            readonly name: "factoryDeps";
            readonly type: "bytes[]";
        }, {
            readonly internalType: "address";
            readonly name: "refundRecipient";
            readonly type: "address";
        }];
        readonly internalType: "struct L2TransactionRequestDirect";
        readonly name: "_request";
        readonly type: "tuple";
    }];
    readonly name: "requestL2TransactionDirect";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "canonicalTxHash";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint256";
            readonly name: "chainId";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "mintValue";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2Value";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasPerPubdataByteLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "address";
            readonly name: "refundRecipient";
            readonly type: "address";
        }, {
            readonly internalType: "address";
            readonly name: "secondBridgeAddress";
            readonly type: "address";
        }, {
            readonly internalType: "uint256";
            readonly name: "secondBridgeValue";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes";
            readonly name: "secondBridgeCalldata";
            readonly type: "bytes";
        }];
        readonly internalType: "struct L2TransactionRequestTwoBridgesOuter";
        readonly name: "_request";
        readonly type: "tuple";
    }];
    readonly name: "requestL2TransactionTwoBridges";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "canonicalTxHash";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_newPendingAdmin";
        readonly type: "address";
    }];
    readonly name: "setPendingAdmin";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_sharedBridge";
        readonly type: "address";
    }];
    readonly name: "setSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "sharedBridge";
    readonly outputs: readonly [{
        readonly internalType: "contract IL1SharedBridge";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }];
    readonly name: "stateTransitionManager";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_stateTransitionManager";
        readonly type: "address";
    }];
    readonly name: "stateTransitionManagerIsRegistered";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_baseToken";
        readonly type: "address";
    }];
    readonly name: "tokenIsRegistered";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}];
export declare const zksyncAbi: readonly [{
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "batchNumber";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "batchHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "commitment";
        readonly type: "bytes32";
    }];
    readonly name: "BlockCommit";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "batchNumber";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "batchHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "commitment";
        readonly type: "bytes32";
    }];
    readonly name: "BlockExecution";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "totalBatchesCommitted";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "totalBatchesVerified";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "totalBatchesExecuted";
        readonly type: "uint256";
    }];
    readonly name: "BlocksRevert";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "previousLastVerifiedBatch";
        readonly type: "uint256";
    }, {
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "currentLastVerifiedBatch";
        readonly type: "uint256";
    }];
    readonly name: "BlocksVerification";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "facet";
                readonly type: "address";
            }, {
                readonly internalType: "enum Diamond.Action";
                readonly name: "action";
                readonly type: "uint8";
            }, {
                readonly internalType: "bool";
                readonly name: "isFreezable";
                readonly type: "bool";
            }, {
                readonly internalType: "bytes4[]";
                readonly name: "selectors";
                readonly type: "bytes4[]";
            }];
            readonly internalType: "struct Diamond.FacetCut[]";
            readonly name: "facetCuts";
            readonly type: "tuple[]";
        }, {
            readonly internalType: "address";
            readonly name: "initAddress";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCalldata";
            readonly type: "bytes";
        }];
        readonly indexed: false;
        readonly internalType: "struct Diamond.DiamondCutData";
        readonly name: "diamondCut";
        readonly type: "tuple";
    }];
    readonly name: "ExecuteUpgrade";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "Freeze";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "bool";
        readonly name: "isPorterAvailable";
        readonly type: "bool";
    }];
    readonly name: "IsPorterAvailableStatusUpdate";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "oldAdmin";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "newAdmin";
        readonly type: "address";
    }];
    readonly name: "NewAdmin";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "uint128";
        readonly name: "oldNominator";
        readonly type: "uint128";
    }, {
        readonly indexed: false;
        readonly internalType: "uint128";
        readonly name: "oldDenominator";
        readonly type: "uint128";
    }, {
        readonly indexed: false;
        readonly internalType: "uint128";
        readonly name: "newNominator";
        readonly type: "uint128";
    }, {
        readonly indexed: false;
        readonly internalType: "uint128";
        readonly name: "newDenominator";
        readonly type: "uint128";
    }];
    readonly name: "NewBaseTokenMultiplier";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "enum PubdataPricingMode";
            readonly name: "pubdataPricingMode";
            readonly type: "uint8";
        }, {
            readonly internalType: "uint32";
            readonly name: "batchOverheadL1Gas";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxPubdataPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxL2GasPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "priorityTxMaxPubdata";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint64";
            readonly name: "minimalL2GasPrice";
            readonly type: "uint64";
        }];
        readonly indexed: false;
        readonly internalType: "struct FeeParams";
        readonly name: "oldFeeParams";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly internalType: "enum PubdataPricingMode";
            readonly name: "pubdataPricingMode";
            readonly type: "uint8";
        }, {
            readonly internalType: "uint32";
            readonly name: "batchOverheadL1Gas";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxPubdataPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxL2GasPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "priorityTxMaxPubdata";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint64";
            readonly name: "minimalL2GasPrice";
            readonly type: "uint64";
        }];
        readonly indexed: false;
        readonly internalType: "struct FeeParams";
        readonly name: "newFeeParams";
        readonly type: "tuple";
    }];
    readonly name: "NewFeeParams";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "oldPendingAdmin";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "newPendingAdmin";
        readonly type: "address";
    }];
    readonly name: "NewPendingAdmin";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "txId";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes32";
        readonly name: "txHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: false;
        readonly internalType: "uint64";
        readonly name: "expirationTimestamp";
        readonly type: "uint64";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint256";
            readonly name: "txType";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "from";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "to";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "gasLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "gasPerPubdataByteLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "maxFeePerGas";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "maxPriorityFeePerGas";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "paymaster";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "value";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256[4]";
            readonly name: "reserved";
            readonly type: "uint256[4]";
        }, {
            readonly internalType: "bytes";
            readonly name: "data";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "signature";
            readonly type: "bytes";
        }, {
            readonly internalType: "uint256[]";
            readonly name: "factoryDeps";
            readonly type: "uint256[]";
        }, {
            readonly internalType: "bytes";
            readonly name: "paymasterInput";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "reservedDynamic";
            readonly type: "bytes";
        }];
        readonly indexed: false;
        readonly internalType: "struct L2CanonicalTransaction";
        readonly name: "transaction";
        readonly type: "tuple";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes[]";
        readonly name: "factoryDeps";
        readonly type: "bytes[]";
    }];
    readonly name: "NewPriorityRequest";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "oldPriorityTxMaxGasLimit";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "newPriorityTxMaxGasLimit";
        readonly type: "uint256";
    }];
    readonly name: "NewPriorityTxMaxGasLimit";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "oldTransactionFilterer";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "newTransactionFilterer";
        readonly type: "address";
    }];
    readonly name: "NewTransactionFilterer";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "facet";
                readonly type: "address";
            }, {
                readonly internalType: "enum Diamond.Action";
                readonly name: "action";
                readonly type: "uint8";
            }, {
                readonly internalType: "bool";
                readonly name: "isFreezable";
                readonly type: "bool";
            }, {
                readonly internalType: "bytes4[]";
                readonly name: "selectors";
                readonly type: "bytes4[]";
            }];
            readonly internalType: "struct Diamond.FacetCut[]";
            readonly name: "facetCuts";
            readonly type: "tuple[]";
        }, {
            readonly internalType: "address";
            readonly name: "initAddress";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCalldata";
            readonly type: "bytes";
        }];
        readonly indexed: false;
        readonly internalType: "struct Diamond.DiamondCutData";
        readonly name: "diamondCut";
        readonly type: "tuple";
    }, {
        readonly indexed: true;
        readonly internalType: "uint256";
        readonly name: "proposalId";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes32";
        readonly name: "proposalSalt";
        readonly type: "bytes32";
    }];
    readonly name: "ProposeTransparentUpgrade";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "Unfreeze";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "validatorAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "bool";
        readonly name: "isActive";
        readonly type: "bool";
    }];
    readonly name: "ValidatorStatusUpdate";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: false;
        readonly internalType: "enum PubdataPricingMode";
        readonly name: "validiumMode";
        readonly type: "uint8";
    }];
    readonly name: "ValidiumModeStatusUpdate";
    readonly type: "event";
}, {
    readonly inputs: readonly [];
    readonly name: "acceptAdmin";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "baseTokenGasPriceMultiplierDenominator";
    readonly outputs: readonly [{
        readonly internalType: "uint128";
        readonly name: "";
        readonly type: "uint128";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "baseTokenGasPriceMultiplierNominator";
    readonly outputs: readonly [{
        readonly internalType: "uint128";
        readonly name: "";
        readonly type: "uint128";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "address";
            readonly name: "contractL2";
            readonly type: "address";
        }, {
            readonly internalType: "uint256";
            readonly name: "mintValue";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2Value";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes";
            readonly name: "l2Calldata";
            readonly type: "bytes";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "l2GasPerPubdataByteLimit";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes[]";
            readonly name: "factoryDeps";
            readonly type: "bytes[]";
        }, {
            readonly internalType: "address";
            readonly name: "refundRecipient";
            readonly type: "address";
        }];
        readonly internalType: "struct BridgehubL2TransactionRequest";
        readonly name: "_request";
        readonly type: "tuple";
    }];
    readonly name: "bridgehubRequestL2Transaction";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "canonicalTxHash";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "enum PubdataPricingMode";
            readonly name: "pubdataPricingMode";
            readonly type: "uint8";
        }, {
            readonly internalType: "uint32";
            readonly name: "batchOverheadL1Gas";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxPubdataPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "maxL2GasPerBatch";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint32";
            readonly name: "priorityTxMaxPubdata";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint64";
            readonly name: "minimalL2GasPrice";
            readonly type: "uint64";
        }];
        readonly internalType: "struct FeeParams";
        readonly name: "_newFeeParams";
        readonly type: "tuple";
    }];
    readonly name: "changeFeeParams";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo";
        readonly name: "_lastCommittedBatchData";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint64";
            readonly name: "timestamp";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "newStateRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "bootloaderHeapInitialContentsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "eventsQueueStateHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes";
            readonly name: "systemLogs";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "pubdataCommitments";
            readonly type: "bytes";
        }];
        readonly internalType: "struct IExecutor.CommitBatchInfo[]";
        readonly name: "_newBatchesData";
        readonly type: "tuple[]";
    }];
    readonly name: "commitBatches";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo";
        readonly name: "_lastCommittedBatchData";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint64";
            readonly name: "timestamp";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "newStateRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "bootloaderHeapInitialContentsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "eventsQueueStateHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes";
            readonly name: "systemLogs";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "pubdataCommitments";
            readonly type: "bytes";
        }];
        readonly internalType: "struct IExecutor.CommitBatchInfo[]";
        readonly name: "_newBatchesData";
        readonly type: "tuple[]";
    }];
    readonly name: "commitBatchesSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo[]";
        readonly name: "_batchesData";
        readonly type: "tuple[]";
    }];
    readonly name: "executeBatches";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo[]";
        readonly name: "_batchesData";
        readonly type: "tuple[]";
    }];
    readonly name: "executeBatchesSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "facet";
                readonly type: "address";
            }, {
                readonly internalType: "enum Diamond.Action";
                readonly name: "action";
                readonly type: "uint8";
            }, {
                readonly internalType: "bool";
                readonly name: "isFreezable";
                readonly type: "bool";
            }, {
                readonly internalType: "bytes4[]";
                readonly name: "selectors";
                readonly type: "bytes4[]";
            }];
            readonly internalType: "struct Diamond.FacetCut[]";
            readonly name: "facetCuts";
            readonly type: "tuple[]";
        }, {
            readonly internalType: "address";
            readonly name: "initAddress";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCalldata";
            readonly type: "bytes";
        }];
        readonly internalType: "struct Diamond.DiamondCutData";
        readonly name: "_diamondCut";
        readonly type: "tuple";
    }];
    readonly name: "executeUpgrade";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes4";
        readonly name: "_selector";
        readonly type: "bytes4";
    }];
    readonly name: "facetAddress";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "facet";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "facetAddresses";
    readonly outputs: readonly [{
        readonly internalType: "address[]";
        readonly name: "facets";
        readonly type: "address[]";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_facet";
        readonly type: "address";
    }];
    readonly name: "facetFunctionSelectors";
    readonly outputs: readonly [{
        readonly internalType: "bytes4[]";
        readonly name: "";
        readonly type: "bytes4[]";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "facets";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "address";
            readonly name: "addr";
            readonly type: "address";
        }, {
            readonly internalType: "bytes4[]";
            readonly name: "selectors";
            readonly type: "bytes4[]";
        }];
        readonly internalType: "struct IGetters.Facet[]";
        readonly name: "";
        readonly type: "tuple[]";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes";
        readonly name: "_message";
        readonly type: "bytes";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }];
    readonly name: "finalizeEthWithdrawal";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "freezeDiamond";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getAdmin";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getBaseToken";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getBaseTokenBridge";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getBridgehub";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getFirstUnprocessedPriorityTx";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getL2BootloaderBytecodeHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getL2DefaultAccountBytecodeHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getL2SystemContractsUpgradeBatchNumber";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getL2SystemContractsUpgradeTxHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getName";
    readonly outputs: readonly [{
        readonly internalType: "string";
        readonly name: "";
        readonly type: "string";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getPendingAdmin";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getPriorityQueueSize";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getPriorityTxMaxGasLimit";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getProtocolVersion";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getPubdataPricingMode";
    readonly outputs: readonly [{
        readonly internalType: "enum PubdataPricingMode";
        readonly name: "";
        readonly type: "uint8";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getSemverProtocolVersion";
    readonly outputs: readonly [{
        readonly internalType: "uint32";
        readonly name: "";
        readonly type: "uint32";
    }, {
        readonly internalType: "uint32";
        readonly name: "";
        readonly type: "uint32";
    }, {
        readonly internalType: "uint32";
        readonly name: "";
        readonly type: "uint32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getStateTransitionManager";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getTotalBatchesCommitted";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getTotalBatchesExecuted";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getTotalBatchesVerified";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getTotalPriorityTxs";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getVerifier";
    readonly outputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getVerifierParams";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "bytes32";
            readonly name: "recursionNodeLevelVkHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "recursionLeafLevelVkHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "recursionCircuitsSetVksHash";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct VerifierParams";
        readonly name: "";
        readonly type: "tuple";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "isDiamondStorageFrozen";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }];
    readonly name: "isEthWithdrawalFinalized";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_facet";
        readonly type: "address";
    }];
    readonly name: "isFacetFreezable";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "isFreezable";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes4";
        readonly name: "_selector";
        readonly type: "bytes4";
    }];
    readonly name: "isFunctionFreezable";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_address";
        readonly type: "address";
    }];
    readonly name: "isValidator";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }];
    readonly name: "l2LogsRootHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "merkleRoot";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_gasPrice";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasLimit";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasPerPubdataByteLimit";
        readonly type: "uint256";
    }];
    readonly name: "l2TransactionBaseCost";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "priorityQueueFrontOperation";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "bytes32";
            readonly name: "canonicalTxHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "expirationTimestamp";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint192";
            readonly name: "layer2Tip";
            readonly type: "uint192";
        }];
        readonly internalType: "struct PriorityOperation";
        readonly name: "";
        readonly type: "tuple";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo";
        readonly name: "_prevBatch";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo[]";
        readonly name: "_committedBatches";
        readonly type: "tuple[]";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint256[]";
            readonly name: "recursiveAggregationInput";
            readonly type: "uint256[]";
        }, {
            readonly internalType: "uint256[]";
            readonly name: "serializedProof";
            readonly type: "uint256[]";
        }];
        readonly internalType: "struct IExecutor.ProofInput";
        readonly name: "_proof";
        readonly type: "tuple";
    }];
    readonly name: "proveBatches";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo";
        readonly name: "_prevBatch";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint64";
            readonly name: "batchNumber";
            readonly type: "uint64";
        }, {
            readonly internalType: "bytes32";
            readonly name: "batchHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint64";
            readonly name: "indexRepeatedStorageChanges";
            readonly type: "uint64";
        }, {
            readonly internalType: "uint256";
            readonly name: "numberOfLayer1Txs";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "priorityOperationsHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "l2LogsTreeRoot";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "timestamp";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "commitment";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct IExecutor.StoredBatchInfo[]";
        readonly name: "_committedBatches";
        readonly type: "tuple[]";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint256[]";
            readonly name: "recursiveAggregationInput";
            readonly type: "uint256[]";
        }, {
            readonly internalType: "uint256[]";
            readonly name: "serializedProof";
            readonly type: "uint256[]";
        }];
        readonly internalType: "struct IExecutor.ProofInput";
        readonly name: "_proof";
        readonly type: "tuple";
    }];
    readonly name: "proveBatchesSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "_l2TxHash";
        readonly type: "bytes32";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2BatchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2MessageIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint16";
        readonly name: "_l2TxNumberInBatch";
        readonly type: "uint16";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_merkleProof";
        readonly type: "bytes32[]";
    }, {
        readonly internalType: "enum TxStatus";
        readonly name: "_status";
        readonly type: "uint8";
    }];
    readonly name: "proveL1ToL2TransactionStatus";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_index";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint8";
            readonly name: "l2ShardId";
            readonly type: "uint8";
        }, {
            readonly internalType: "bool";
            readonly name: "isService";
            readonly type: "bool";
        }, {
            readonly internalType: "uint16";
            readonly name: "txNumberInBatch";
            readonly type: "uint16";
        }, {
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "bytes32";
            readonly name: "key";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes32";
            readonly name: "value";
            readonly type: "bytes32";
        }];
        readonly internalType: "struct L2Log";
        readonly name: "_log";
        readonly type: "tuple";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_proof";
        readonly type: "bytes32[]";
    }];
    readonly name: "proveL2LogInclusion";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_index";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly internalType: "uint16";
            readonly name: "txNumberInBatch";
            readonly type: "uint16";
        }, {
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "data";
            readonly type: "bytes";
        }];
        readonly internalType: "struct L2Message";
        readonly name: "_message";
        readonly type: "tuple";
    }, {
        readonly internalType: "bytes32[]";
        readonly name: "_proof";
        readonly type: "bytes32[]";
    }];
    readonly name: "proveL2MessageInclusion";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_contractL2";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2Value";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes";
        readonly name: "_calldata";
        readonly type: "bytes";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasLimit";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_l2GasPerPubdataByteLimit";
        readonly type: "uint256";
    }, {
        readonly internalType: "bytes[]";
        readonly name: "_factoryDeps";
        readonly type: "bytes[]";
    }, {
        readonly internalType: "address";
        readonly name: "_refundRecipient";
        readonly type: "address";
    }];
    readonly name: "requestL2Transaction";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "canonicalTxHash";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_newLastBatch";
        readonly type: "uint256";
    }];
    readonly name: "revertBatches";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "uint256";
        readonly name: "_newLastBatch";
        readonly type: "uint256";
    }];
    readonly name: "revertBatchesSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_newPendingAdmin";
        readonly type: "address";
    }];
    readonly name: "setPendingAdmin";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bool";
        readonly name: "_zkPorterIsAvailable";
        readonly type: "bool";
    }];
    readonly name: "setPorterAvailability";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_newPriorityTxMaxGasLimit";
        readonly type: "uint256";
    }];
    readonly name: "setPriorityTxMaxGasLimit";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "enum PubdataPricingMode";
        readonly name: "_pricingMode";
        readonly type: "uint8";
    }];
    readonly name: "setPubdataPricingMode";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint128";
        readonly name: "_nominator";
        readonly type: "uint128";
    }, {
        readonly internalType: "uint128";
        readonly name: "_denominator";
        readonly type: "uint128";
    }];
    readonly name: "setTokenMultiplier";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_transactionFilterer";
        readonly type: "address";
    }];
    readonly name: "setTransactionFilterer";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "_validator";
        readonly type: "address";
    }, {
        readonly internalType: "bool";
        readonly name: "_active";
        readonly type: "bool";
    }];
    readonly name: "setValidator";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_batchNumber";
        readonly type: "uint256";
    }];
    readonly name: "storedBatchHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "transferEthToSharedBridge";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "unfreezeDiamond";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "_protocolVersion";
        readonly type: "uint256";
    }, {
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "facet";
                readonly type: "address";
            }, {
                readonly internalType: "enum Diamond.Action";
                readonly name: "action";
                readonly type: "uint8";
            }, {
                readonly internalType: "bool";
                readonly name: "isFreezable";
                readonly type: "bool";
            }, {
                readonly internalType: "bytes4[]";
                readonly name: "selectors";
                readonly type: "bytes4[]";
            }];
            readonly internalType: "struct Diamond.FacetCut[]";
            readonly name: "facetCuts";
            readonly type: "tuple[]";
        }, {
            readonly internalType: "address";
            readonly name: "initAddress";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCalldata";
            readonly type: "bytes";
        }];
        readonly internalType: "struct Diamond.DiamondCutData";
        readonly name: "_cutData";
        readonly type: "tuple";
    }];
    readonly name: "upgradeChainFromVersion";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}];
//# sourceMappingURL=abis.d.ts.map