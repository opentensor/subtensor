/**
 * EntryPoint version.
 *
 * @see https://github.com/eth-infinitism/account-abstraction/releases
 */
export type Version = '0.6' | '0.7' | '0.8';
/** EntryPoint 0.6 ABI. */
export declare const abiV06: readonly [{
    readonly inputs: readonly [{
        readonly name: "preOpGas";
        readonly type: "uint256";
    }, {
        readonly name: "paid";
        readonly type: "uint256";
    }, {
        readonly name: "validAfter";
        readonly type: "uint48";
    }, {
        readonly name: "validUntil";
        readonly type: "uint48";
    }, {
        readonly name: "targetSuccess";
        readonly type: "bool";
    }, {
        readonly name: "targetResult";
        readonly type: "bytes";
    }];
    readonly name: "ExecutionResult";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "opIndex";
        readonly type: "uint256";
    }, {
        readonly name: "reason";
        readonly type: "string";
    }];
    readonly name: "FailedOp";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "sender";
        readonly type: "address";
    }];
    readonly name: "SenderAddressResult";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureValidationFailed";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "preOpGas";
            readonly type: "uint256";
        }, {
            readonly name: "prefund";
            readonly type: "uint256";
        }, {
            readonly name: "sigFailed";
            readonly type: "bool";
        }, {
            readonly name: "validAfter";
            readonly type: "uint48";
        }, {
            readonly name: "validUntil";
            readonly type: "uint48";
        }, {
            readonly name: "paymasterContext";
            readonly type: "bytes";
        }];
        readonly name: "returnInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "senderInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "factoryInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "paymasterInfo";
        readonly type: "tuple";
    }];
    readonly name: "ValidationResult";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "preOpGas";
            readonly type: "uint256";
        }, {
            readonly name: "prefund";
            readonly type: "uint256";
        }, {
            readonly name: "sigFailed";
            readonly type: "bool";
        }, {
            readonly name: "validAfter";
            readonly type: "uint48";
        }, {
            readonly name: "validUntil";
            readonly type: "uint48";
        }, {
            readonly name: "paymasterContext";
            readonly type: "bytes";
        }];
        readonly name: "returnInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "senderInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "factoryInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "stake";
            readonly type: "uint256";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint256";
        }];
        readonly name: "paymasterInfo";
        readonly type: "tuple";
    }, {
        readonly components: readonly [{
            readonly name: "aggregator";
            readonly type: "address";
        }, {
            readonly components: readonly [{
                readonly name: "stake";
                readonly type: "uint256";
            }, {
                readonly name: "unstakeDelaySec";
                readonly type: "uint256";
            }];
            readonly name: "stakeInfo";
            readonly type: "tuple";
        }];
        readonly name: "aggregatorInfo";
        readonly type: "tuple";
    }];
    readonly name: "ValidationResultWithAggregation";
    readonly type: "error";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "factory";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "paymaster";
        readonly type: "address";
    }];
    readonly name: "AccountDeployed";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "BeforeExecution";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "totalDeposit";
        readonly type: "uint256";
    }];
    readonly name: "Deposited";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureAggregatorChanged";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "totalStaked";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "unstakeDelaySec";
        readonly type: "uint256";
    }];
    readonly name: "StakeLocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawTime";
        readonly type: "uint256";
    }];
    readonly name: "StakeUnlocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "StakeWithdrawn";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly name: "paymaster";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "success";
        readonly type: "bool";
    }, {
        readonly indexed: false;
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "actualGasUsed";
        readonly type: "uint256";
    }];
    readonly name: "UserOperationEvent";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "revertReason";
        readonly type: "bytes";
    }];
    readonly name: "UserOperationRevertReason";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "Withdrawn";
    readonly type: "event";
}, {
    readonly inputs: readonly [];
    readonly name: "SIG_VALIDATION_FAILED";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "initCode";
        readonly type: "bytes";
    }, {
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly name: "paymasterAndData";
        readonly type: "bytes";
    }];
    readonly name: "_validateSenderAndPaymaster";
    readonly outputs: readonly [];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "unstakeDelaySec";
        readonly type: "uint32";
    }];
    readonly name: "addStake";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "balanceOf";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "depositTo";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "";
        readonly type: "address";
    }];
    readonly name: "deposits";
    readonly outputs: readonly [{
        readonly name: "deposit";
        readonly type: "uint112";
    }, {
        readonly name: "staked";
        readonly type: "bool";
    }, {
        readonly name: "stake";
        readonly type: "uint112";
    }, {
        readonly name: "unstakeDelaySec";
        readonly type: "uint32";
    }, {
        readonly name: "withdrawTime";
        readonly type: "uint48";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "getDepositInfo";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly name: "deposit";
            readonly type: "uint112";
        }, {
            readonly name: "staked";
            readonly type: "bool";
        }, {
            readonly name: "stake";
            readonly type: "uint112";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint32";
        }, {
            readonly name: "withdrawTime";
            readonly type: "uint48";
        }];
        readonly name: "info";
        readonly type: "tuple";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "getNonce";
    readonly outputs: readonly [{
        readonly name: "nonce";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "initCode";
        readonly type: "bytes";
    }];
    readonly name: "getSenderAddress";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "callGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "verificationGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxPriorityFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "userOp";
        readonly type: "tuple";
    }];
    readonly name: "getUserOpHash";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly name: "initCode";
                readonly type: "bytes";
            }, {
                readonly name: "callData";
                readonly type: "bytes";
            }, {
                readonly name: "callGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "verificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly name: "maxFeePerGas";
                readonly type: "uint256";
            }, {
                readonly name: "maxPriorityFeePerGas";
                readonly type: "uint256";
            }, {
                readonly name: "paymasterAndData";
                readonly type: "bytes";
            }, {
                readonly name: "signature";
                readonly type: "bytes";
            }];
            readonly name: "userOps";
            readonly type: "tuple[]";
        }, {
            readonly name: "aggregator";
            readonly type: "address";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "opsPerAggregator";
        readonly type: "tuple[]";
    }, {
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleAggregatedOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "callGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "verificationGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxPriorityFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "ops";
        readonly type: "tuple[]";
    }, {
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "incrementNonce";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "callData";
        readonly type: "bytes";
    }, {
        readonly components: readonly [{
            readonly components: readonly [{
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly name: "callGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "verificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly name: "paymaster";
                readonly type: "address";
            }, {
                readonly name: "maxFeePerGas";
                readonly type: "uint256";
            }, {
                readonly name: "maxPriorityFeePerGas";
                readonly type: "uint256";
            }];
            readonly name: "mUserOp";
            readonly type: "tuple";
        }, {
            readonly name: "userOpHash";
            readonly type: "bytes32";
        }, {
            readonly name: "prefund";
            readonly type: "uint256";
        }, {
            readonly name: "contextOffset";
            readonly type: "uint256";
        }, {
            readonly name: "preOpGas";
            readonly type: "uint256";
        }];
        readonly name: "opInfo";
        readonly type: "tuple";
    }, {
        readonly name: "context";
        readonly type: "bytes";
    }];
    readonly name: "innerHandleOp";
    readonly outputs: readonly [{
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "";
        readonly type: "address";
    }, {
        readonly name: "";
        readonly type: "uint192";
    }];
    readonly name: "nonceSequenceNumber";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "callGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "verificationGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxPriorityFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "op";
        readonly type: "tuple";
    }, {
        readonly name: "target";
        readonly type: "address";
    }, {
        readonly name: "targetCallData";
        readonly type: "bytes";
    }];
    readonly name: "simulateHandleOp";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "callGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "verificationGasLimit";
            readonly type: "uint256";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "maxPriorityFeePerGas";
            readonly type: "uint256";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "userOp";
        readonly type: "tuple";
    }];
    readonly name: "simulateValidation";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "unlockStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "withdrawAddress";
        readonly type: "address";
    }];
    readonly name: "withdrawStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly name: "withdrawAmount";
        readonly type: "uint256";
    }];
    readonly name: "withdrawTo";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly stateMutability: "payable";
    readonly type: "receive";
}];
/** EntryPoint 0.7 ABI. */
export declare const abiV07: readonly [{
    readonly inputs: readonly [{
        readonly name: "success";
        readonly type: "bool";
    }, {
        readonly name: "ret";
        readonly type: "bytes";
    }];
    readonly name: "DelegateAndRevert";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "opIndex";
        readonly type: "uint256";
    }, {
        readonly name: "reason";
        readonly type: "string";
    }];
    readonly name: "FailedOp";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "opIndex";
        readonly type: "uint256";
    }, {
        readonly name: "reason";
        readonly type: "string";
    }, {
        readonly name: "inner";
        readonly type: "bytes";
    }];
    readonly name: "FailedOpWithRevert";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "returnData";
        readonly type: "bytes";
    }];
    readonly name: "PostOpReverted";
    readonly type: "error";
}, {
    readonly inputs: readonly [];
    readonly name: "ReentrancyGuardReentrantCall";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "sender";
        readonly type: "address";
    }];
    readonly name: "SenderAddressResult";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureValidationFailed";
    readonly type: "error";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "factory";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "paymaster";
        readonly type: "address";
    }];
    readonly name: "AccountDeployed";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "BeforeExecution";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "totalDeposit";
        readonly type: "uint256";
    }];
    readonly name: "Deposited";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "revertReason";
        readonly type: "bytes";
    }];
    readonly name: "PostOpRevertReason";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureAggregatorChanged";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "totalStaked";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "unstakeDelaySec";
        readonly type: "uint256";
    }];
    readonly name: "StakeLocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawTime";
        readonly type: "uint256";
    }];
    readonly name: "StakeUnlocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "StakeWithdrawn";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly name: "paymaster";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "success";
        readonly type: "bool";
    }, {
        readonly indexed: false;
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "actualGasUsed";
        readonly type: "uint256";
    }];
    readonly name: "UserOperationEvent";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }];
    readonly name: "UserOperationPrefundTooLow";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly name: "revertReason";
        readonly type: "bytes";
    }];
    readonly name: "UserOperationRevertReason";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "Withdrawn";
    readonly type: "event";
}, {
    readonly inputs: readonly [{
        readonly name: "unstakeDelaySec";
        readonly type: "uint32";
    }];
    readonly name: "addStake";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "balanceOf";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "target";
        readonly type: "address";
    }, {
        readonly name: "data";
        readonly type: "bytes";
    }];
    readonly name: "delegateAndRevert";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "depositTo";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "";
        readonly type: "address";
    }];
    readonly name: "deposits";
    readonly outputs: readonly [{
        readonly name: "deposit";
        readonly type: "uint256";
    }, {
        readonly name: "staked";
        readonly type: "bool";
    }, {
        readonly name: "stake";
        readonly type: "uint112";
    }, {
        readonly name: "unstakeDelaySec";
        readonly type: "uint32";
    }, {
        readonly name: "withdrawTime";
        readonly type: "uint48";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "getDepositInfo";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly name: "deposit";
            readonly type: "uint256";
        }, {
            readonly name: "staked";
            readonly type: "bool";
        }, {
            readonly name: "stake";
            readonly type: "uint112";
        }, {
            readonly name: "unstakeDelaySec";
            readonly type: "uint32";
        }, {
            readonly name: "withdrawTime";
            readonly type: "uint48";
        }];
        readonly name: "info";
        readonly type: "tuple";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "getNonce";
    readonly outputs: readonly [{
        readonly name: "nonce";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "initCode";
        readonly type: "bytes";
    }];
    readonly name: "getSenderAddress";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "accountGasLimits";
            readonly type: "bytes32";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "gasFees";
            readonly type: "bytes32";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "userOp";
        readonly type: "tuple";
    }];
    readonly name: "getUserOpHash";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly name: "initCode";
                readonly type: "bytes";
            }, {
                readonly name: "callData";
                readonly type: "bytes";
            }, {
                readonly name: "accountGasLimits";
                readonly type: "bytes32";
            }, {
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly name: "gasFees";
                readonly type: "bytes32";
            }, {
                readonly name: "paymasterAndData";
                readonly type: "bytes";
            }, {
                readonly name: "signature";
                readonly type: "bytes";
            }];
            readonly name: "userOps";
            readonly type: "tuple[]";
        }, {
            readonly name: "aggregator";
            readonly type: "address";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "opsPerAggregator";
        readonly type: "tuple[]";
    }, {
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleAggregatedOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly name: "accountGasLimits";
            readonly type: "bytes32";
        }, {
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly name: "gasFees";
            readonly type: "bytes32";
        }, {
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly name: "ops";
        readonly type: "tuple[]";
    }, {
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "incrementNonce";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "callData";
        readonly type: "bytes";
    }, {
        readonly components: readonly [{
            readonly components: readonly [{
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly name: "verificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "callGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "paymasterVerificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "paymasterPostOpGasLimit";
                readonly type: "uint256";
            }, {
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly name: "paymaster";
                readonly type: "address";
            }, {
                readonly name: "maxFeePerGas";
                readonly type: "uint256";
            }, {
                readonly name: "maxPriorityFeePerGas";
                readonly type: "uint256";
            }];
            readonly name: "mUserOp";
            readonly type: "tuple";
        }, {
            readonly name: "userOpHash";
            readonly type: "bytes32";
        }, {
            readonly name: "prefund";
            readonly type: "uint256";
        }, {
            readonly name: "contextOffset";
            readonly type: "uint256";
        }, {
            readonly name: "preOpGas";
            readonly type: "uint256";
        }];
        readonly name: "opInfo";
        readonly type: "tuple";
    }, {
        readonly name: "context";
        readonly type: "bytes";
    }];
    readonly name: "innerHandleOp";
    readonly outputs: readonly [{
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "";
        readonly type: "address";
    }, {
        readonly name: "";
        readonly type: "uint192";
    }];
    readonly name: "nonceSequenceNumber";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "interfaceId";
        readonly type: "bytes4";
    }];
    readonly name: "supportsInterface";
    readonly outputs: readonly [{
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "unlockStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "withdrawAddress";
        readonly type: "address";
    }];
    readonly name: "withdrawStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly name: "withdrawAmount";
        readonly type: "uint256";
    }];
    readonly name: "withdrawTo";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly stateMutability: "payable";
    readonly type: "receive";
}];
/** EntryPoint 0.8 ABI. */
export declare const abiV08: readonly [{
    readonly inputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "constructor";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bool";
        readonly name: "success";
        readonly type: "bool";
    }, {
        readonly internalType: "bytes";
        readonly name: "ret";
        readonly type: "bytes";
    }];
    readonly name: "DelegateAndRevert";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "opIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "string";
        readonly name: "reason";
        readonly type: "string";
    }];
    readonly name: "FailedOp";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "opIndex";
        readonly type: "uint256";
    }, {
        readonly internalType: "string";
        readonly name: "reason";
        readonly type: "string";
    }, {
        readonly internalType: "bytes";
        readonly name: "inner";
        readonly type: "bytes";
    }];
    readonly name: "FailedOpWithRevert";
    readonly type: "error";
}, {
    readonly inputs: readonly [];
    readonly name: "InvalidShortString";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes";
        readonly name: "returnData";
        readonly type: "bytes";
    }];
    readonly name: "PostOpReverted";
    readonly type: "error";
}, {
    readonly inputs: readonly [];
    readonly name: "ReentrancyGuardReentrantCall";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }];
    readonly name: "SenderAddressResult";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureValidationFailed";
    readonly type: "error";
}, {
    readonly inputs: readonly [{
        readonly internalType: "string";
        readonly name: "str";
        readonly type: "string";
    }];
    readonly name: "StringTooLong";
    readonly type: "error";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "factory";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "paymaster";
        readonly type: "address";
    }];
    readonly name: "AccountDeployed";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "BeforeExecution";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "totalDeposit";
        readonly type: "uint256";
    }];
    readonly name: "Deposited";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [];
    readonly name: "EIP712DomainChanged";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes";
        readonly name: "revertReason";
        readonly type: "bytes";
    }];
    readonly name: "PostOpRevertReason";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "aggregator";
        readonly type: "address";
    }];
    readonly name: "SignatureAggregatorChanged";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "totalStaked";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "unstakeDelaySec";
        readonly type: "uint256";
    }];
    readonly name: "StakeLocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "withdrawTime";
        readonly type: "uint256";
    }];
    readonly name: "StakeUnlocked";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "StakeWithdrawn";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "paymaster";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bool";
        readonly name: "success";
        readonly type: "bool";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "actualGasUsed";
        readonly type: "uint256";
    }];
    readonly name: "UserOperationEvent";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "nonce";
        readonly type: "uint256";
    }];
    readonly name: "UserOperationPrefundTooLow";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "bytes32";
        readonly name: "userOpHash";
        readonly type: "bytes32";
    }, {
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "nonce";
        readonly type: "uint256";
    }, {
        readonly indexed: false;
        readonly internalType: "bytes";
        readonly name: "revertReason";
        readonly type: "bytes";
    }];
    readonly name: "UserOperationRevertReason";
    readonly type: "event";
}, {
    readonly anonymous: false;
    readonly inputs: readonly [{
        readonly indexed: true;
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "address";
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly indexed: false;
        readonly internalType: "uint256";
        readonly name: "amount";
        readonly type: "uint256";
    }];
    readonly name: "Withdrawn";
    readonly type: "event";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint32";
        readonly name: "unstakeDelaySec";
        readonly type: "uint32";
    }];
    readonly name: "addStake";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
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
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "target";
        readonly type: "address";
    }, {
        readonly internalType: "bytes";
        readonly name: "data";
        readonly type: "bytes";
    }];
    readonly name: "delegateAndRevert";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "depositTo";
    readonly outputs: readonly [];
    readonly stateMutability: "payable";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "eip712Domain";
    readonly outputs: readonly [{
        readonly internalType: "bytes1";
        readonly name: "fields";
        readonly type: "bytes1";
    }, {
        readonly internalType: "string";
        readonly name: "name";
        readonly type: "string";
    }, {
        readonly internalType: "string";
        readonly name: "version";
        readonly type: "string";
    }, {
        readonly internalType: "uint256";
        readonly name: "chainId";
        readonly type: "uint256";
    }, {
        readonly internalType: "address";
        readonly name: "verifyingContract";
        readonly type: "address";
    }, {
        readonly internalType: "bytes32";
        readonly name: "salt";
        readonly type: "bytes32";
    }, {
        readonly internalType: "uint256[]";
        readonly name: "extensions";
        readonly type: "uint256[]";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "account";
        readonly type: "address";
    }];
    readonly name: "getDepositInfo";
    readonly outputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "uint256";
            readonly name: "deposit";
            readonly type: "uint256";
        }, {
            readonly internalType: "bool";
            readonly name: "staked";
            readonly type: "bool";
        }, {
            readonly internalType: "uint112";
            readonly name: "stake";
            readonly type: "uint112";
        }, {
            readonly internalType: "uint32";
            readonly name: "unstakeDelaySec";
            readonly type: "uint32";
        }, {
            readonly internalType: "uint48";
            readonly name: "withdrawTime";
            readonly type: "uint48";
        }];
        readonly internalType: "struct IStakeManager.DepositInfo";
        readonly name: "info";
        readonly type: "tuple";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getDomainSeparatorV4";
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
        readonly name: "sender";
        readonly type: "address";
    }, {
        readonly internalType: "uint192";
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "getNonce";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "nonce";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "getPackedUserOpTypeHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "pure";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes";
        readonly name: "initCode";
        readonly type: "bytes";
    }];
    readonly name: "getSenderAddress";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "uint256";
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes32";
            readonly name: "accountGasLimits";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "gasFees";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes";
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly internalType: "struct PackedUserOperation";
        readonly name: "userOp";
        readonly type: "tuple";
    }];
    readonly name: "getUserOpHash";
    readonly outputs: readonly [{
        readonly internalType: "bytes32";
        readonly name: "";
        readonly type: "bytes32";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly internalType: "uint256";
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly internalType: "bytes";
                readonly name: "initCode";
                readonly type: "bytes";
            }, {
                readonly internalType: "bytes";
                readonly name: "callData";
                readonly type: "bytes";
            }, {
                readonly internalType: "bytes32";
                readonly name: "accountGasLimits";
                readonly type: "bytes32";
            }, {
                readonly internalType: "uint256";
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly internalType: "bytes32";
                readonly name: "gasFees";
                readonly type: "bytes32";
            }, {
                readonly internalType: "bytes";
                readonly name: "paymasterAndData";
                readonly type: "bytes";
            }, {
                readonly internalType: "bytes";
                readonly name: "signature";
                readonly type: "bytes";
            }];
            readonly internalType: "struct PackedUserOperation[]";
            readonly name: "userOps";
            readonly type: "tuple[]";
        }, {
            readonly internalType: "contract IAggregator";
            readonly name: "aggregator";
            readonly type: "address";
        }, {
            readonly internalType: "bytes";
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly internalType: "struct IEntryPoint.UserOpsPerAggregator[]";
        readonly name: "opsPerAggregator";
        readonly type: "tuple[]";
    }, {
        readonly internalType: "address payable";
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleAggregatedOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly components: readonly [{
            readonly internalType: "address";
            readonly name: "sender";
            readonly type: "address";
        }, {
            readonly internalType: "uint256";
            readonly name: "nonce";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes";
            readonly name: "initCode";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "callData";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes32";
            readonly name: "accountGasLimits";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "preVerificationGas";
            readonly type: "uint256";
        }, {
            readonly internalType: "bytes32";
            readonly name: "gasFees";
            readonly type: "bytes32";
        }, {
            readonly internalType: "bytes";
            readonly name: "paymasterAndData";
            readonly type: "bytes";
        }, {
            readonly internalType: "bytes";
            readonly name: "signature";
            readonly type: "bytes";
        }];
        readonly internalType: "struct PackedUserOperation[]";
        readonly name: "ops";
        readonly type: "tuple[]";
    }, {
        readonly internalType: "address payable";
        readonly name: "beneficiary";
        readonly type: "address";
    }];
    readonly name: "handleOps";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "uint192";
        readonly name: "key";
        readonly type: "uint192";
    }];
    readonly name: "incrementNonce";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes";
        readonly name: "callData";
        readonly type: "bytes";
    }, {
        readonly components: readonly [{
            readonly components: readonly [{
                readonly internalType: "address";
                readonly name: "sender";
                readonly type: "address";
            }, {
                readonly internalType: "uint256";
                readonly name: "nonce";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "verificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "callGasLimit";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "paymasterVerificationGasLimit";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "paymasterPostOpGasLimit";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "preVerificationGas";
                readonly type: "uint256";
            }, {
                readonly internalType: "address";
                readonly name: "paymaster";
                readonly type: "address";
            }, {
                readonly internalType: "uint256";
                readonly name: "maxFeePerGas";
                readonly type: "uint256";
            }, {
                readonly internalType: "uint256";
                readonly name: "maxPriorityFeePerGas";
                readonly type: "uint256";
            }];
            readonly internalType: "struct EntryPoint.MemoryUserOp";
            readonly name: "mUserOp";
            readonly type: "tuple";
        }, {
            readonly internalType: "bytes32";
            readonly name: "userOpHash";
            readonly type: "bytes32";
        }, {
            readonly internalType: "uint256";
            readonly name: "prefund";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "contextOffset";
            readonly type: "uint256";
        }, {
            readonly internalType: "uint256";
            readonly name: "preOpGas";
            readonly type: "uint256";
        }];
        readonly internalType: "struct EntryPoint.UserOpInfo";
        readonly name: "opInfo";
        readonly type: "tuple";
    }, {
        readonly internalType: "bytes";
        readonly name: "context";
        readonly type: "bytes";
    }];
    readonly name: "innerHandleOp";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "actualGasCost";
        readonly type: "uint256";
    }];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address";
        readonly name: "";
        readonly type: "address";
    }, {
        readonly internalType: "uint192";
        readonly name: "";
        readonly type: "uint192";
    }];
    readonly name: "nonceSequenceNumber";
    readonly outputs: readonly [{
        readonly internalType: "uint256";
        readonly name: "";
        readonly type: "uint256";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "senderCreator";
    readonly outputs: readonly [{
        readonly internalType: "contract ISenderCreator";
        readonly name: "";
        readonly type: "address";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "bytes4";
        readonly name: "interfaceId";
        readonly type: "bytes4";
    }];
    readonly name: "supportsInterface";
    readonly outputs: readonly [{
        readonly internalType: "bool";
        readonly name: "";
        readonly type: "bool";
    }];
    readonly stateMutability: "view";
    readonly type: "function";
}, {
    readonly inputs: readonly [];
    readonly name: "unlockStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address payable";
        readonly name: "withdrawAddress";
        readonly type: "address";
    }];
    readonly name: "withdrawStake";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly inputs: readonly [{
        readonly internalType: "address payable";
        readonly name: "withdrawAddress";
        readonly type: "address";
    }, {
        readonly internalType: "uint256";
        readonly name: "withdrawAmount";
        readonly type: "uint256";
    }];
    readonly name: "withdrawTo";
    readonly outputs: readonly [];
    readonly stateMutability: "nonpayable";
    readonly type: "function";
}, {
    readonly stateMutability: "payable";
    readonly type: "receive";
}];
/** EntryPoint 0.6 address. */
export declare const addressV06: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
/** EntryPoint 0.7 address. */
export declare const addressV07: "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
/** EntryPoint 0.8 address. */
export declare const addressV08: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
//# sourceMappingURL=EntryPoint.d.ts.map