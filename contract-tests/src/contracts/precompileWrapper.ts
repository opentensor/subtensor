export const PRECOMPILE_WRAPPER_ABI = [
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "delegate",
                "type": "bytes32"
            },
            {
                "internalType": "uint8",
                "name": "proxy_type",
                "type": "uint8"
            },
            {
                "internalType": "uint32",
                "name": "delay",
                "type": "uint32"
            }
        ],
        "name": "addProxy",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "addStake",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "address",
                "name": "target_address",
                "type": "address"
            }
        ],
        "name": "addressMapping",
        "outputs": [
            {
                "internalType": "bytes32",
                "name": "",
                "type": "bytes32"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "addressMappingPrecompile",
        "outputs": [
            {
                "internalType": "contract IAddressMapping",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "alpha",
        "outputs": [
            {
                "internalType": "contract IAlpha",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "balanceTransfer",
        "outputs": [
            {
                "internalType": "contract ISubtensorBalanceTransfer",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            },
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            }
        ],
        "name": "burnedRegister",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint64",
                "name": "deposit",
                "type": "uint64"
            },
            {
                "internalType": "uint64",
                "name": "minContribution",
                "type": "uint64"
            },
            {
                "internalType": "uint64",
                "name": "cap",
                "type": "uint64"
            },
            {
                "internalType": "uint32",
                "name": "end",
                "type": "uint32"
            },
            {
                "internalType": "address",
                "name": "targetAddress",
                "type": "address"
            }
        ],
        "name": "createCrowdloan",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint64",
                "name": "crowdloanDeposit",
                "type": "uint64"
            },
            {
                "internalType": "uint64",
                "name": "crowdloanMinContribution",
                "type": "uint64"
            },
            {
                "internalType": "uint64",
                "name": "crowdloanCap",
                "type": "uint64"
            },
            {
                "internalType": "uint32",
                "name": "crowdloanEnd",
                "type": "uint32"
            },
            {
                "internalType": "uint8",
                "name": "leasingEmissionsShare",
                "type": "uint8"
            },
            {
                "internalType": "bool",
                "name": "hasLeasingEndBlock",
                "type": "bool"
            },
            {
                "internalType": "uint32",
                "name": "leasingEndBlock",
                "type": "uint32"
            }
        ],
        "name": "createLeaseCrowdloan",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "crowdloan",
        "outputs": [
            {
                "internalType": "contract ICrowdloan",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            }
        ],
        "name": "getAlphaPrice",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint32",
                "name": "crowdloanId",
                "type": "uint32"
            },
            {
                "internalType": "bytes32",
                "name": "coldkey",
                "type": "bytes32"
            }
        ],
        "name": "getContribution",
        "outputs": [
            {
                "internalType": "uint64",
                "name": "",
                "type": "uint64"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint32",
                "name": "leaseId",
                "type": "uint32"
            },
            {
                "internalType": "bytes32",
                "name": "contributor",
                "type": "bytes32"
            }
        ],
        "name": "getContributorShare",
        "outputs": [
            {
                "internalType": "uint128",
                "name": "",
                "type": "uint128"
            },
            {
                "internalType": "uint128",
                "name": "",
                "type": "uint128"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint32",
                "name": "crowdloanId",
                "type": "uint32"
            }
        ],
        "name": "getCrowdloan",
        "outputs": [
            {
                "components": [
                    {
                        "internalType": "bytes32",
                        "name": "creator",
                        "type": "bytes32"
                    },
                    {
                        "internalType": "uint64",
                        "name": "deposit",
                        "type": "uint64"
                    },
                    {
                        "internalType": "uint64",
                        "name": "min_contribution",
                        "type": "uint64"
                    },
                    {
                        "internalType": "uint32",
                        "name": "end",
                        "type": "uint32"
                    },
                    {
                        "internalType": "uint64",
                        "name": "cap",
                        "type": "uint64"
                    },
                    {
                        "internalType": "bytes32",
                        "name": "funds_account",
                        "type": "bytes32"
                    },
                    {
                        "internalType": "uint64",
                        "name": "raised",
                        "type": "uint64"
                    },
                    {
                        "internalType": "bool",
                        "name": "has_target_address",
                        "type": "bool"
                    },
                    {
                        "internalType": "bytes32",
                        "name": "target_address",
                        "type": "bytes32"
                    },
                    {
                        "internalType": "bool",
                        "name": "finalized",
                        "type": "bool"
                    },
                    {
                        "internalType": "uint32",
                        "name": "contributors_count",
                        "type": "uint32"
                    }
                ],
                "internalType": "struct CrowdloanInfo",
                "name": "",
                "type": "tuple"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "account",
                "type": "bytes32"
            }
        ],
        "name": "getProxies",
        "outputs": [
            {
                "components": [
                    {
                        "internalType": "bytes32",
                        "name": "delegate",
                        "type": "bytes32"
                    },
                    {
                        "internalType": "uint256",
                        "name": "proxy_type",
                        "type": "uint256"
                    },
                    {
                        "internalType": "uint256",
                        "name": "delay",
                        "type": "uint256"
                    }
                ],
                "internalType": "struct IProxy.ProxyInfo[]",
                "name": "",
                "type": "tuple[]"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            }
        ],
        "name": "getServingRateLimit",
        "outputs": [
            {
                "internalType": "uint64",
                "name": "",
                "type": "uint64"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            }
        ],
        "name": "getNetworkRegistrationBlock",
        "outputs": [
            {
                "internalType": "uint64",
                "name": "",
                "type": "uint64"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "coldkey",
                "type": "bytes32"
            }
        ],
        "name": "getTotalColdkeyStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            }
        ],
        "name": "getTotalHotkeyStake",
        "outputs": [
            {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            }
        ],
        "name": "getUidCount",
        "outputs": [
            {
                "internalType": "uint16",
                "name": "",
                "type": "uint16"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "leasing",
        "outputs": [
            {
                "internalType": "contract ILeasing",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "metagraph",
        "outputs": [
            {
                "internalType": "contract IMetagraph",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "neuron",
        "outputs": [
            {
                "internalType": "contract INeuron",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "proxy",
        "outputs": [
            {
                "internalType": "contract IProxy",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "real",
                "type": "bytes32"
            },
            {
                "internalType": "uint8[]",
                "name": "force_proxy_type",
                "type": "uint8[]"
            },
            {
                "internalType": "uint8[]",
                "name": "call",
                "type": "uint8[]"
            }
        ],
        "name": "proxyCall",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "string",
                "name": "subnetName",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "githubRepo",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "subnetContact",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "subnetUrl",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "discord",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "description",
                "type": "string"
            },
            {
                "internalType": "string",
                "name": "additional",
                "type": "string"
            }
        ],
        "name": "registerNetworkWithDetails",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "hotkey",
                "type": "bytes32"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "netuid",
                "type": "uint256"
            }
        ],
        "name": "removeStake",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "staking",
        "outputs": [
            {
                "internalType": "contract IStaking",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "subnet",
        "outputs": [
            {
                "internalType": "contract ISubnet",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "data",
                "type": "bytes32"
            }
        ],
        "name": "transfer",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "uint16",
                "name": "netuid",
                "type": "uint16"
            },
            {
                "internalType": "address",
                "name": "evm_address",
                "type": "address"
            },
            {
                "internalType": "uint16",
                "name": "limit",
                "type": "uint16"
            }
        ],
        "name": "uidLookup",
        "outputs": [
            {
                "components": [
                    {
                        "internalType": "uint16",
                        "name": "uid",
                        "type": "uint16"
                    },
                    {
                        "internalType": "uint64",
                        "name": "block_associated",
                        "type": "uint64"
                    }
                ],
                "internalType": "struct LookupItem[]",
                "name": "",
                "type": "tuple[]"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "uidLookupPrecompile",
        "outputs": [
            {
                "internalType": "contract IUidLookup",
                "name": "",
                "type": "address"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    }
];

export const PRECOMPILE_WRAPPER_BYTECODE = "6080604052348015600e575f5ffd5b506119368061001c5f395ff3fe6080604052600436106101da575f3560e01c80638bba466c116100fd578063b1f789ef11610092578063d75e3e0d11610062578063d75e3e0d14610547578063db1d0fd51461055c578063ec55688914610571578063fc6679fb14610586575f5ffd5b8063b1f789ef146104de578063bfe252a21461050a578063caf2ebf21461051f578063cd6f4eb114610534575f5ffd5b8063a2176276116100cd578063a217627614610482578063ac3166bf14610497578063afed65f9146104ac578063b0c751b0146104bf575f5ffd5b80638bba466c146103ec57806394e3ac6f14610418578063998538c4146104445780639f246f6f14610463575f5ffd5b80634cf088d91161017357806369e38bc31161014357806369e38bc31461038857806371214e27146103a75780637444dadc146103ba5780637d691e30146103d9575f5ffd5b80634cf088d9146103145780635b53ddde146103295780635b7210c51461033e5780635e25f3f814610375575f5ffd5b80631fc9b141116101ae5780631fc9b141146102825780633175bd98146102955780634054ecca146102d45780634c378a96146102e7575f5ffd5b80620ae759146101de5780630494cd9a146101ff5780630cadeda5146102315780631f19357214610250575b5f5ffd5b3480156101e9575f5ffd5b506101fd6101f8366004610e85565b61059b565b005b34801561020a575f5ffd5b5061021e610219366004610f06565b6105f4565b6040519081526020015b60405180910390f35b34801561023c575f5ffd5b506101fd61024b366004610f33565b610665565b34801561025b575f5ffd5b5061026f61026a366004610f7f565b6106a0565b60405161ffff9091168152602001610228565b6101fd610290366004610f9a565b610705565b3480156102a0575f5ffd5b506102b46102af366004610fc3565b610739565b604080516001600160801b03938416815292909116602083015201610228565b6101fd6102e2366004610fed565b6107b3565b3480156102f2575f5ffd5b506102fc61080481565b6040516001600160a01b039091168152602001610228565b34801561031f575f5ffd5b506102fc61080581565b348015610334575f5ffd5b506102fc61080a81565b348015610349575f5ffd5b5061035d610358366004610fc3565b6107f7565b6040516001600160401b039091168152602001610228565b6101fd610383366004611074565b61086c565b348015610393575f5ffd5b5061021e6103a2366004610f7f565b6108d6565b6101fd6103b53660046111c2565b610901565b3480156103c5575f5ffd5b5061035d6103d4366004610f7f565b610989565b6101fd6103e7366004610f9a565b6109ef565b3480156103f7575f5ffd5b5061040b61040636600461122b565b610a23565b6040516102289190611246565b348015610423575f5ffd5b50610437610432366004611334565b610add565b604051610228919061134b565b34801561044f575f5ffd5b5061021e61045e366004611334565b610b42565b34801561046e575f5ffd5b5061021e61047d366004611334565b610b6a565b34801561048d575f5ffd5b506102fc61080681565b3480156104a2575f5ffd5b506102fc61080c81565b6101fd6104ba3660046113b6565b610b92565b3480156104ca575f5ffd5b5061035d6104d9366004610f7f565b610c26565b3480156104e9575f5ffd5b506104fd6104f8366004611445565b610c51565b6040516102289190611480565b348015610515575f5ffd5b506102fc61080981565b34801561052a575f5ffd5b506102fc61080381565b6101fd610542366004611334565b610cd8565b348015610552575f5ffd5b506102fc61080081565b348015610567575f5ffd5b506102fc61080881565b34801561057c575f5ffd5b506102fc61080b81565b348015610591575f5ffd5b506102fc61080281565b604051620ae75960e01b815261080b90620ae759906105c29086908690869060040161150d565b5f604051808303815f87803b1580156105d9575f5ffd5b505af11580156105eb573d5f5f3e3d5ffd5b50505050505050565b60405163024a66cd60e11b81526001600160a01b03821660048201525f9061080c90630494cd9a906024015b602060405180830381865afa15801561063b573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061065f9190611541565b92915050565b604051630cadeda560e01b81526004810184905260ff8316602482015263ffffffff8216604482015261080b90630cadeda5906064016105c2565b604051630f8c9ab960e11b815261ffff821660048201525f9061080290631f19357290602401602060405180830381865afa1580156106e1573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061065f9190611558565b604051631fc9b14160e01b815260048101849052602481018390526044810182905261080590631fc9b141906064016105c2565b60405163062eb7b360e31b815263ffffffff83166004820152602481018290525f90819061080a90633175bd98906044016040805180830381865afa158015610784573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906107a89190611589565b915091509250929050565b60405163202a766560e11b815261ffff831660048201526024810182905261080490634054ecca9034906044015f604051808303818588803b1580156105d9575f5ffd5b604051635b7210c560e01b815263ffffffff83166004820152602481018290525f9061080990635b7210c590604401602060405180830381865afa158015610841573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061086591906115c5565b9392505050565b604051631cf98c6b60e01b815261080390631cf98c6b9061089f908b908b908b908b908b908b908b908b9060040161160e565b5f604051808303815f87803b1580156108b6575f5ffd5b505af11580156108c8573d5f5f3e3d5ffd5b505050505050505050505050565b6040516369e38bc360e01b815261ffff821660048201525f90610808906369e38bc390602401610620565b60405163127e1adb60e01b81526001600160401b03808716600483015280861660248301528416604482015263ffffffff831660648201526001600160a01b03821660848201526108099063127e1adb9060a4015f604051808303815f87803b15801561096c575f5ffd5b505af115801561097e573d5f5f3e3d5ffd5b505050505050505050565b604051631d1136b760e21b815261ffff821660048201525f9061080390637444dadc906024015b602060405180830381865afa1580156109cb573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061065f91906115c5565b6040516307d691e360e41b815260048101849052602481018390526044810182905261080590637d691e30906064016105c2565b60408051610160810182525f80825260208201819052818301819052606082018190526080820181905260a0820181905260c0820181905260e082018190526101008201819052610120820181905261014082015290516322ee919b60e21b815263ffffffff8316600482015261080990638bba466c9060240161016060405180830381865afa158015610ab9573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061065f91906116c3565b6040516394e3ac6f60e01b81526004810182905260609061080b906394e3ac6f906024015f60405180830381865afa158015610b1b573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f1916820160405261065f919081019061178a565b6040516326614e3160e21b8152600481018290525f906108059063998538c490602401610620565b604051639f246f6f60e01b8152600481018290525f9061080590639f246f6f90602401610620565b60405163afed65f960e01b81526001600160401b03808916600483015280881660248301528616604482015263ffffffff808616606483015260ff8516608483015283151560a4830152821660c482015261080a9063afed65f99060e4015f604051808303815f87803b158015610c07575f5ffd5b505af1158015610c19573d5f5f3e3d5ffd5b5050505050505050505050565b604051630b0c751b60e41b815261ffff821660048201525f906108039063b0c751b0906024016109b0565b60405163b1f789ef60e01b815261ffff80851660048301526001600160a01b0384166024830152821660448201526060906108069063b1f789ef906064015f60405180830381865afa158015610ca9573d5f5f3e3d5ffd5b505050506040513d5f823e601f3d908101601f19168201604052610cd0919081019061183f565b949350505050565b60405163cd6f4eb160e01b8152600481018290526108009063cd6f4eb19034906024015f604051808303818588803b158015610d12575f5ffd5b505af1158015610d24573d5f5f3e3d5ffd5b505050505050565b634e487b7160e01b5f52604160045260245ffd5b60405161016081016001600160401b0381118282101715610d6357610d63610d2c565b60405290565b604051606081016001600160401b0381118282101715610d6357610d63610d2c565b604080519081016001600160401b0381118282101715610d6357610d63610d2c565b604051601f8201601f191681016001600160401b0381118282101715610dd557610dd5610d2c565b604052919050565b5f6001600160401b03821115610df557610df5610d2c565b5060051b60200190565b803560ff81168114610e0f575f5ffd5b919050565b5f82601f830112610e23575f5ffd5b8135610e36610e3182610ddd565b610dad565b8082825260208201915060208360051b860101925085831115610e57575f5ffd5b602085015b83811015610e7b57610e6d81610dff565b835260209283019201610e5c565b5095945050505050565b5f5f5f60608486031215610e97575f5ffd5b8335925060208401356001600160401b03811115610eb3575f5ffd5b610ebf86828701610e14565b92505060408401356001600160401b03811115610eda575f5ffd5b610ee686828701610e14565b9150509250925092565b80356001600160a01b0381168114610e0f575f5ffd5b5f60208284031215610f16575f5ffd5b61086582610ef0565b63ffffffff81168114610f30575f5ffd5b50565b5f5f5f60608486031215610f45575f5ffd5b83359250610f5560208501610dff565b91506040840135610f6581610f1f565b809150509250925092565b61ffff81168114610f30575f5ffd5b5f60208284031215610f8f575f5ffd5b813561086581610f70565b5f5f5f60608486031215610fac575f5ffd5b505081359360208301359350604090920135919050565b5f5f60408385031215610fd4575f5ffd5b8235610fdf81610f1f565b946020939093013593505050565b5f5f60408385031215610ffe575f5ffd5b8235610fdf81610f70565b5f82601f830112611018575f5ffd5b81356001600160401b0381111561103157611031610d2c565b611044601f8201601f1916602001610dad565b818152846020838601011115611058575f5ffd5b816020850160208301375f918101602001919091529392505050565b5f5f5f5f5f5f5f5f610100898b03121561108c575f5ffd5b8835975060208901356001600160401b038111156110a8575f5ffd5b6110b48b828c01611009565b97505060408901356001600160401b038111156110cf575f5ffd5b6110db8b828c01611009565b96505060608901356001600160401b038111156110f6575f5ffd5b6111028b828c01611009565b95505060808901356001600160401b0381111561111d575f5ffd5b6111298b828c01611009565b94505060a08901356001600160401b03811115611144575f5ffd5b6111508b828c01611009565b93505060c08901356001600160401b0381111561116b575f5ffd5b6111778b828c01611009565b92505060e08901356001600160401b03811115611192575f5ffd5b61119e8b828c01611009565b9150509295985092959890939650565b6001600160401b0381168114610f30575f5ffd5b5f5f5f5f5f60a086880312156111d6575f5ffd5b85356111e1816111ae565b945060208601356111f1816111ae565b93506040860135611201816111ae565b9250606086013561121181610f1f565b915061121f60808701610ef0565b90509295509295909350565b5f6020828403121561123b575f5ffd5b813561086581610f1f565b8151815260208083015161016083019161126a908401826001600160401b03169052565b50604083015161128560408401826001600160401b03169052565b50606083015161129d606084018263ffffffff169052565b5060808301516112b860808401826001600160401b03169052565b5060a083015160a083015260c08301516112dd60c08401826001600160401b03169052565b5060e08301516112f160e084018215159052565b5061010083015161010083015261012083015161131361012084018215159052565b5061014083015161132d61014084018263ffffffff169052565b5092915050565b5f60208284031215611344575f5ffd5b5035919050565b602080825282518282018190525f918401906040840190835b8181101561139e57835180518452602081015160208501526040810151604085015250606083019250602084019350600181019050611364565b509095945050505050565b8015158114610f30575f5ffd5b5f5f5f5f5f5f5f60e0888a0312156113cc575f5ffd5b87356113d7816111ae565b965060208801356113e7816111ae565b955060408801356113f7816111ae565b9450606088013561140781610f1f565b935061141560808901610dff565b925060a0880135611425816113a9565b915060c088013561143581610f1f565b8091505092959891949750929550565b5f5f5f60608486031215611457575f5ffd5b833561146281610f70565b925061147060208501610ef0565b91506040840135610f6581610f70565b602080825282518282018190525f918401906040840190835b8181101561139e578351805161ffff1684526020908101516001600160401b03168185015290930192604090920191600101611499565b5f8151808452602084019350602083015f5b8281101561150357815160ff168652602095860195909101906001016114e2565b5093949350505050565b838152606060208201525f61152560608301856114d0565b828103604084015261153781856114d0565b9695505050505050565b5f60208284031215611551575f5ffd5b5051919050565b5f60208284031215611568575f5ffd5b815161086581610f70565b80516001600160801b0381168114610e0f575f5ffd5b5f5f6040838503121561159a575f5ffd5b6115a383611573565b91506115b160208401611573565b90509250929050565b8051610e0f816111ae565b5f602082840312156115d5575f5ffd5b8151610865816111ae565b5f81518084528060208401602086015e5f602082860101526020601f19601f83011685010191505092915050565b88815261010060208201525f61162861010083018a6115e0565b828103604084015261163a818a6115e0565b9050828103606084015261164e81896115e0565b9050828103608084015261166281886115e0565b905082810360a084015261167681876115e0565b905082810360c084015261168a81866115e0565b905082810360e084015261169e81856115e0565b9b9a5050505050505050505050565b8051610e0f81610f1f565b8051610e0f816113a9565b5f6101608284031280156116d5575f5ffd5b506116de610d40565b825181526116ee602084016115ba565b60208201526116ff604084016115ba565b6040820152611710606084016116ad565b6060820152611721608084016115ba565b608082015260a0838101519082015261173c60c084016115ba565b60c082015261174d60e084016116b8565b60e0820152610100838101519082015261176a61012084016116b8565b61012082015261177d61014084016116ad565b6101408201529392505050565b5f6020828403121561179a575f5ffd5b81516001600160401b038111156117af575f5ffd5b8201601f810184136117bf575f5ffd5b80516117cd610e3182610ddd565b808282526020820191506020606084028501019250868311156117ee575f5ffd5b6020840193505b82841015611537576060848803121561180c575f5ffd5b611814610d69565b84518152602080860151818301526040808701519083015290835260609094019391909101906117f5565b5f6020828403121561184f575f5ffd5b81516001600160401b03811115611864575f5ffd5b8201601f81018413611874575f5ffd5b8051611882610e3182610ddd565b8082825260208201915060208360061b8501019250868311156118a3575f5ffd5b6020840193505b8284101561153757604084880312156118c1575f5ffd5b6118c9610d8b565b84516118d481610f70565b815260208501516118e4816111ae565b80602083015250808352506020820191506040840193506118aa56fea264697066735822122026460b0cf8f5e17c58e4083c1b1155431c8d2cb9962cd9d5f6105ce473df73ee64736f6c63430008230033";
