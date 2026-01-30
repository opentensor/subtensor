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
                "internalType": "bytes",
                "name": "call",
                "type": "bytes"
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

export const PRECOMPILE_WRAPPER_BYTECODE = "6080604052348015600e575f5ffd5b50612cab8061001c5f395ff3fe6080604052600436106101d7575f3560e01c80638bba466c11610101578063bfe252a211610094578063db1d0fd511610063578063db1d0fd5146106ac578063ec556889146106d6578063fa83f46714610700578063fc6679fb14610728576101d7565b8063bfe252a214610612578063caf2ebf21461063c578063cd6f4eb114610666578063d75e3e0d14610682576101d7565b8063a2176276116100d0578063a217627614610566578063ac3166bf14610590578063afed65f9146105ba578063b1f789ef146105d6576101d7565b80638bba466c1461047657806394e3ac6f146104b2578063998538c4146104ee5780639f246f6f1461052a576101d7565b80634cf088d91161017957806369e38bc31161014857806369e38bc3146103c657806371214e27146104025780637444dadc1461041e5780637d691e301461045a576101d7565b80634cf088d91461031a5780635b53ddde146103445780635b7210c51461036e5780635e25f3f8146103aa576101d7565b80631fc9b141116101b55780631fc9b1411461027b5780633175bd98146102975780634054ecca146102d45780634c378a96146102f0576101d7565b80630494cd9a146101db5780630cadeda5146102175780631f1935721461023f575b5f5ffd5b3480156101e6575f5ffd5b5061020160048036038101906101fc919061124e565b610752565b60405161020e9190611291565b60405180910390f35b348015610222575f5ffd5b5061023d60048036038101906102389190611343565b6107d4565b005b34801561024a575f5ffd5b50610265600480360381019061026091906113ca565b610845565b6040516102729190611404565b60405180910390f35b61029560048036038101906102909190611450565b6108c7565b005b3480156102a2575f5ffd5b506102bd60048036038101906102b891906114a0565b610938565b6040516102cb929190611508565b60405180910390f35b6102ee60048036038101906102e9919061152f565b6109c0565b005b3480156102fb575f5ffd5b50610304610a2e565b60405161031191906115c8565b60405180910390f35b348015610325575f5ffd5b5061032e610a34565b60405161033b9190611601565b60405180910390f35b34801561034f575f5ffd5b50610358610a3a565b604051610365919061163a565b60405180910390f35b348015610379575f5ffd5b50610394600480360381019061038f91906114a0565b610a40565b6040516103a19190611675565b60405180910390f35b6103c460048036038101906103bf91906117ca565b610ac5565b005b3480156103d1575f5ffd5b506103ec60048036038101906103e791906113ca565b610b45565b6040516103f9919061194e565b60405180910390f35b61041c60048036038101906104179190611991565b610bc7565b005b348015610429575f5ffd5b50610444600480360381019061043f91906113ca565b610c3e565b6040516104519190611675565b60405180910390f35b610474600480360381019061046f9190611450565b610cc0565b005b348015610481575f5ffd5b5061049c60048036038101906104979190611a08565b610d31565b6040516104a99190611b59565b60405180910390f35b3480156104bd575f5ffd5b506104d860048036038101906104d39190611b73565b610dbb565b6040516104e59190611c95565b60405180910390f35b3480156104f9575f5ffd5b50610514600480360381019061050f9190611b73565b610e41565b604051610521919061194e565b60405180910390f35b348015610535575f5ffd5b50610550600480360381019061054b9190611b73565b610ec3565b60405161055d919061194e565b60405180910390f35b348015610571575f5ffd5b5061057a610f45565b6040516105879190611cd5565b60405180910390f35b34801561059b575f5ffd5b506105a4610f4b565b6040516105b19190611d0e565b60405180910390f35b6105d460048036038101906105cf9190611d51565b610f51565b005b3480156105e1575f5ffd5b506105fc60048036038101906105f79190611dee565b610fce565b6040516106099190611f22565b60405180910390f35b34801561061d575f5ffd5b5061062661105a565b6040516106339190611f62565b60405180910390f35b348015610647575f5ffd5b50610650611060565b60405161065d9190611f9b565b60405180910390f35b610680600480360381019061067b9190611b73565b611066565b005b34801561068d575f5ffd5b506106966110d3565b6040516106a39190611fd4565b60405180910390f35b3480156106b7575f5ffd5b506106c06110d9565b6040516106cd919061200d565b60405180910390f35b3480156106e1575f5ffd5b506106ea6110df565b6040516106f79190612046565b60405180910390f35b34801561070b575f5ffd5b50610726600480360381019061072191906121c1565b6110e5565b005b348015610733575f5ffd5b5061073c611156565b6040516107499190612269565b60405180910390f35b5f61080c73ffffffffffffffffffffffffffffffffffffffff16630494cd9a836040518263ffffffff1660e01b815260040161078e9190612291565b602060405180830381865afa1580156107a9573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906107cd91906122be565b9050919050565b61080b73ffffffffffffffffffffffffffffffffffffffff16630cadeda58484846040518463ffffffff1660e01b815260040161081393929190612307565b5f604051808303815f87803b15801561082a575f5ffd5b505af115801561083c573d5f5f3e3d5ffd5b50505050505050565b5f61080273ffffffffffffffffffffffffffffffffffffffff16631f193572836040518263ffffffff1660e01b81526004016108819190611404565b602060405180830381865afa15801561089c573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906108c09190612350565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16631fc9b1418484846040518463ffffffff1660e01b81526004016109069392919061237b565b5f604051808303815f87803b15801561091d575f5ffd5b505af115801561092f573d5f5f3e3d5ffd5b50505050505050565b5f5f61080a73ffffffffffffffffffffffffffffffffffffffff16633175bd9885856040518363ffffffff1660e01b81526004016109779291906123b0565b6040805180830381865afa158015610991573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109b59190612401565b915091509250929050565b61080473ffffffffffffffffffffffffffffffffffffffff16634054ecca83836040518363ffffffff1660e01b81526004016109fd92919061243f565b5f604051808303815f87803b158015610a14575f5ffd5b505af1158015610a26573d5f5f3e3d5ffd5b505050505050565b61080481565b61080581565b61080a81565b5f61080973ffffffffffffffffffffffffffffffffffffffff16635b7210c584846040518363ffffffff1660e01b8152600401610a7e9291906123b0565b602060405180830381865afa158015610a99573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610abd919061247a565b905092915050565b61080373ffffffffffffffffffffffffffffffffffffffff16631cf98c6b89898989898989896040518963ffffffff1660e01b8152600401610b0e989796959493929190612505565b5f604051808303815f87803b158015610b25575f5ffd5b505af1158015610b37573d5f5f3e3d5ffd5b505050505050505050505050565b5f61080873ffffffffffffffffffffffffffffffffffffffff166369e38bc3836040518263ffffffff1660e01b8152600401610b819190611404565b602060405180830381865afa158015610b9c573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610bc091906125c6565b9050919050565b61080973ffffffffffffffffffffffffffffffffffffffff1663127e1adb86868686866040518663ffffffff1660e01b8152600401610c0a9594939291906125f1565b5f604051808303815f87803b158015610c21575f5ffd5b505af1158015610c33573d5f5f3e3d5ffd5b505050505050505050565b5f61080373ffffffffffffffffffffffffffffffffffffffff16637444dadc836040518263ffffffff1660e01b8152600401610c7a9190611404565b602060405180830381865afa158015610c95573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610cb9919061247a565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16637d691e308484846040518463ffffffff1660e01b8152600401610cff9392919061237b565b5f604051808303815f87803b158015610d16575f5ffd5b505af1158015610d28573d5f5f3e3d5ffd5b50505050505050565b610d3961115c565b61080973ffffffffffffffffffffffffffffffffffffffff16638bba466c836040518263ffffffff1660e01b8152600401610d749190612642565b61016060405180830381865afa158015610d90573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610db49190612790565b9050919050565b606061080b73ffffffffffffffffffffffffffffffffffffffff166394e3ac6f836040518263ffffffff1660e01b8152600401610df89190611291565b5f60405180830381865afa158015610e12573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f82011682018060405250810190610e3a91906128dd565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff1663998538c4836040518263ffffffff1660e01b8152600401610e7d9190611291565b602060405180830381865afa158015610e98573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610ebc91906125c6565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff16639f246f6f836040518263ffffffff1660e01b8152600401610eff9190611291565b602060405180830381865afa158015610f1a573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610f3e91906125c6565b9050919050565b61080681565b61080c81565b61080a73ffffffffffffffffffffffffffffffffffffffff1663afed65f9888888888888886040518863ffffffff1660e01b8152600401610f989796959493929190612933565b5f604051808303815f87803b158015610faf575f5ffd5b505af1158015610fc1573d5f5f3e3d5ffd5b5050505050505050505050565b606061080673ffffffffffffffffffffffffffffffffffffffff1663b1f789ef8585856040518463ffffffff1660e01b815260040161100f939291906129a0565b5f60405180830381865afa158015611029573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f820116820180604052508101906110519190612ae2565b90509392505050565b61080981565b61080381565b61080073ffffffffffffffffffffffffffffffffffffffff1663cd6f4eb134836040518363ffffffff1660e01b81526004016110a29190611291565b5f604051808303818588803b1580156110b9575f5ffd5b505af11580156110cb573d5f5f3e3d5ffd5b505050505050565b61080081565b61080881565b61080b81565b61080b73ffffffffffffffffffffffffffffffffffffffff1663fa83f4678484846040518463ffffffff1660e01b815260040161112493929190612c32565b5f604051808303815f87803b15801561113b575f5ffd5b505af115801561114d573d5f5f3e3d5ffd5b50505050505050565b61080281565b6040518061016001604052805f81526020015f67ffffffffffffffff1681526020015f67ffffffffffffffff1681526020015f63ffffffff1681526020015f67ffffffffffffffff1681526020015f81526020015f67ffffffffffffffff1681526020015f151581526020015f81526020015f151581526020015f63ffffffff1681525090565b5f604051905090565b5f5ffd5b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61121d826111f4565b9050919050565b61122d81611213565b8114611237575f5ffd5b50565b5f8135905061124881611224565b92915050565b5f60208284031215611263576112626111ec565b5b5f6112708482850161123a565b91505092915050565b5f819050919050565b61128b81611279565b82525050565b5f6020820190506112a45f830184611282565b92915050565b6112b381611279565b81146112bd575f5ffd5b50565b5f813590506112ce816112aa565b92915050565b5f60ff82169050919050565b6112e9816112d4565b81146112f3575f5ffd5b50565b5f81359050611304816112e0565b92915050565b5f63ffffffff82169050919050565b6113228161130a565b811461132c575f5ffd5b50565b5f8135905061133d81611319565b92915050565b5f5f5f6060848603121561135a576113596111ec565b5b5f611367868287016112c0565b9350506020611378868287016112f6565b92505060406113898682870161132f565b9150509250925092565b5f61ffff82169050919050565b6113a981611393565b81146113b3575f5ffd5b50565b5f813590506113c4816113a0565b92915050565b5f602082840312156113df576113de6111ec565b5b5f6113ec848285016113b6565b91505092915050565b6113fe81611393565b82525050565b5f6020820190506114175f8301846113f5565b92915050565b5f819050919050565b61142f8161141d565b8114611439575f5ffd5b50565b5f8135905061144a81611426565b92915050565b5f5f5f60608486031215611467576114666111ec565b5b5f611474868287016112c0565b93505060206114858682870161143c565b92505060406114968682870161143c565b9150509250925092565b5f5f604083850312156114b6576114b56111ec565b5b5f6114c38582860161132f565b92505060206114d4858286016112c0565b9150509250929050565b5f6fffffffffffffffffffffffffffffffff82169050919050565b611502816114de565b82525050565b5f60408201905061151b5f8301856114f9565b61152860208301846114f9565b9392505050565b5f5f60408385031215611545576115446111ec565b5b5f611552858286016113b6565b9250506020611563858286016112c0565b9150509250929050565b5f819050919050565b5f61159061158b611586846111f4565b61156d565b6111f4565b9050919050565b5f6115a182611576565b9050919050565b5f6115b282611597565b9050919050565b6115c2816115a8565b82525050565b5f6020820190506115db5f8301846115b9565b92915050565b5f6115eb82611597565b9050919050565b6115fb816115e1565b82525050565b5f6020820190506116145f8301846115f2565b92915050565b5f61162482611597565b9050919050565b6116348161161a565b82525050565b5f60208201905061164d5f83018461162b565b92915050565b5f67ffffffffffffffff82169050919050565b61166f81611653565b82525050565b5f6020820190506116885f830184611666565b92915050565b5f5ffd5b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b6116dc82611696565b810181811067ffffffffffffffff821117156116fb576116fa6116a6565b5b80604052505050565b5f61170d6111e3565b905061171982826116d3565b919050565b5f67ffffffffffffffff821115611738576117376116a6565b5b61174182611696565b9050602081019050919050565b828183375f83830152505050565b5f61176e6117698461171e565b611704565b90508281526020810184848401111561178a57611789611692565b5b61179584828561174e565b509392505050565b5f82601f8301126117b1576117b061168e565b5b81356117c184826020860161175c565b91505092915050565b5f5f5f5f5f5f5f5f610100898b0312156117e7576117e66111ec565b5b5f6117f48b828c016112c0565b985050602089013567ffffffffffffffff811115611815576118146111f0565b5b6118218b828c0161179d565b975050604089013567ffffffffffffffff811115611842576118416111f0565b5b61184e8b828c0161179d565b965050606089013567ffffffffffffffff81111561186f5761186e6111f0565b5b61187b8b828c0161179d565b955050608089013567ffffffffffffffff81111561189c5761189b6111f0565b5b6118a88b828c0161179d565b94505060a089013567ffffffffffffffff8111156118c9576118c86111f0565b5b6118d58b828c0161179d565b93505060c089013567ffffffffffffffff8111156118f6576118f56111f0565b5b6119028b828c0161179d565b92505060e089013567ffffffffffffffff811115611923576119226111f0565b5b61192f8b828c0161179d565b9150509295985092959890939650565b6119488161141d565b82525050565b5f6020820190506119615f83018461193f565b92915050565b61197081611653565b811461197a575f5ffd5b50565b5f8135905061198b81611967565b92915050565b5f5f5f5f5f60a086880312156119aa576119a96111ec565b5b5f6119b78882890161197d565b95505060206119c88882890161197d565b94505060406119d98882890161197d565b93505060606119ea8882890161132f565b92505060806119fb8882890161123a565b9150509295509295909350565b5f60208284031215611a1d57611a1c6111ec565b5b5f611a2a8482850161132f565b91505092915050565b611a3c81611279565b82525050565b611a4b81611653565b82525050565b611a5a8161130a565b82525050565b5f8115159050919050565b611a7481611a60565b82525050565b61016082015f820151611a8f5f850182611a33565b506020820151611aa26020850182611a42565b506040820151611ab56040850182611a42565b506060820151611ac86060850182611a51565b506080820151611adb6080850182611a42565b5060a0820151611aee60a0850182611a33565b5060c0820151611b0160c0850182611a42565b5060e0820151611b1460e0850182611a6b565b50610100820151611b29610100850182611a33565b50610120820151611b3e610120850182611a6b565b50610140820151611b53610140850182611a51565b50505050565b5f61016082019050611b6d5f830184611a7a565b92915050565b5f60208284031215611b8857611b876111ec565b5b5f611b95848285016112c0565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b611bd08161141d565b82525050565b606082015f820151611bea5f850182611a33565b506020820151611bfd6020850182611bc7565b506040820151611c106040850182611bc7565b50505050565b5f611c218383611bd6565b60608301905092915050565b5f602082019050919050565b5f611c4382611b9e565b611c4d8185611ba8565b9350611c5883611bb8565b805f5b83811015611c88578151611c6f8882611c16565b9750611c7a83611c2d565b925050600181019050611c5b565b5085935050505092915050565b5f6020820190508181035f830152611cad8184611c39565b905092915050565b5f611cbf82611597565b9050919050565b611ccf81611cb5565b82525050565b5f602082019050611ce85f830184611cc6565b92915050565b5f611cf882611597565b9050919050565b611d0881611cee565b82525050565b5f602082019050611d215f830184611cff565b92915050565b611d3081611a60565b8114611d3a575f5ffd5b50565b5f81359050611d4b81611d27565b92915050565b5f5f5f5f5f5f5f60e0888a031215611d6c57611d6b6111ec565b5b5f611d798a828b0161197d565b9750506020611d8a8a828b0161197d565b9650506040611d9b8a828b0161197d565b9550506060611dac8a828b0161132f565b9450506080611dbd8a828b016112f6565b93505060a0611dce8a828b01611d3d565b92505060c0611ddf8a828b0161132f565b91505092959891949750929550565b5f5f5f60608486031215611e0557611e046111ec565b5b5f611e12868287016113b6565b9350506020611e238682870161123a565b9250506040611e34868287016113b6565b9150509250925092565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b611e7081611393565b82525050565b604082015f820151611e8a5f850182611e67565b506020820151611e9d6020850182611a42565b50505050565b5f611eae8383611e76565b60408301905092915050565b5f602082019050919050565b5f611ed082611e3e565b611eda8185611e48565b9350611ee583611e58565b805f5b83811015611f15578151611efc8882611ea3565b9750611f0783611eba565b925050600181019050611ee8565b5085935050505092915050565b5f6020820190508181035f830152611f3a8184611ec6565b905092915050565b5f611f4c82611597565b9050919050565b611f5c81611f42565b82525050565b5f602082019050611f755f830184611f53565b92915050565b5f611f8582611597565b9050919050565b611f9581611f7b565b82525050565b5f602082019050611fae5f830184611f8c565b92915050565b5f611fbe82611597565b9050919050565b611fce81611fb4565b82525050565b5f602082019050611fe75f830184611fc5565b92915050565b5f611ff782611597565b9050919050565b61200781611fed565b82525050565b5f6020820190506120205f830184611ffe565b92915050565b5f61203082611597565b9050919050565b61204081612026565b82525050565b5f6020820190506120595f830184612037565b92915050565b5f67ffffffffffffffff821115612079576120786116a6565b5b602082029050602081019050919050565b5f5ffd5b5f6120a061209b8461205f565b611704565b905080838252602082019050602084028301858111156120c3576120c261208a565b5b835b818110156120ec57806120d888826112f6565b8452602084019350506020810190506120c5565b5050509392505050565b5f82601f83011261210a5761210961168e565b5b813561211a84826020860161208e565b91505092915050565b5f67ffffffffffffffff82111561213d5761213c6116a6565b5b61214682611696565b9050602081019050919050565b5f61216561216084612123565b611704565b90508281526020810184848401111561218157612180611692565b5b61218c84828561174e565b509392505050565b5f82601f8301126121a8576121a761168e565b5b81356121b8848260208601612153565b91505092915050565b5f5f5f606084860312156121d8576121d76111ec565b5b5f6121e5868287016112c0565b935050602084013567ffffffffffffffff811115612206576122056111f0565b5b612212868287016120f6565b925050604084013567ffffffffffffffff811115612233576122326111f0565b5b61223f86828701612194565b9150509250925092565b5f61225382611597565b9050919050565b61226381612249565b82525050565b5f60208201905061227c5f83018461225a565b92915050565b61228b81611213565b82525050565b5f6020820190506122a45f830184612282565b92915050565b5f815190506122b8816112aa565b92915050565b5f602082840312156122d3576122d26111ec565b5b5f6122e0848285016122aa565b91505092915050565b6122f2816112d4565b82525050565b6123018161130a565b82525050565b5f60608201905061231a5f830186611282565b61232760208301856122e9565b61233460408301846122f8565b949350505050565b5f8151905061234a816113a0565b92915050565b5f60208284031215612365576123646111ec565b5b5f6123728482850161233c565b91505092915050565b5f60608201905061238e5f830186611282565b61239b602083018561193f565b6123a8604083018461193f565b949350505050565b5f6040820190506123c35f8301856122f8565b6123d06020830184611282565b9392505050565b6123e0816114de565b81146123ea575f5ffd5b50565b5f815190506123fb816123d7565b92915050565b5f5f60408385031215612417576124166111ec565b5b5f612424858286016123ed565b9250506020612435858286016123ed565b9150509250929050565b5f6040820190506124525f8301856113f5565b61245f6020830184611282565b9392505050565b5f8151905061247481611967565b92915050565b5f6020828403121561248f5761248e6111ec565b5b5f61249c84828501612466565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f6124d7826124a5565b6124e181856124af565b93506124f18185602086016124bf565b6124fa81611696565b840191505092915050565b5f610100820190506125195f83018b611282565b818103602083015261252b818a6124cd565b9050818103604083015261253f81896124cd565b9050818103606083015261255381886124cd565b9050818103608083015261256781876124cd565b905081810360a083015261257b81866124cd565b905081810360c083015261258f81856124cd565b905081810360e08301526125a381846124cd565b90509998505050505050505050565b5f815190506125c081611426565b92915050565b5f602082840312156125db576125da6111ec565b5b5f6125e8848285016125b2565b91505092915050565b5f60a0820190506126045f830188611666565b6126116020830187611666565b61261e6040830186611666565b61262b60608301856122f8565b6126386080830184612282565b9695505050505050565b5f6020820190506126555f8301846122f8565b92915050565b5f5ffd5b5f8151905061266d81611319565b92915050565b5f8151905061268181611d27565b92915050565b5f610160828403121561269d5761269c61265b565b5b6126a8610160611704565b90505f6126b7848285016122aa565b5f8301525060206126ca84828501612466565b60208301525060406126de84828501612466565b60408301525060606126f28482850161265f565b606083015250608061270684828501612466565b60808301525060a061271a848285016122aa565b60a08301525060c061272e84828501612466565b60c08301525060e061274284828501612673565b60e083015250610100612757848285016122aa565b6101008301525061012061276d84828501612673565b610120830152506101406127838482850161265f565b6101408301525092915050565b5f61016082840312156127a6576127a56111ec565b5b5f6127b384828501612687565b91505092915050565b5f67ffffffffffffffff8211156127d6576127d56116a6565b5b602082029050602081019050919050565b5f606082840312156127fc576127fb61265b565b5b6128066060611704565b90505f612815848285016122aa565b5f830152506020612828848285016125b2565b602083015250604061283c848285016125b2565b60408301525092915050565b5f61285a612855846127bc565b611704565b9050808382526020820190506060840283018581111561287d5761287c61208a565b5b835b818110156128a6578061289288826127e7565b84526020840193505060608101905061287f565b5050509392505050565b5f82601f8301126128c4576128c361168e565b5b81516128d4848260208601612848565b91505092915050565b5f602082840312156128f2576128f16111ec565b5b5f82015167ffffffffffffffff81111561290f5761290e6111f0565b5b61291b848285016128b0565b91505092915050565b61292d81611a60565b82525050565b5f60e0820190506129465f83018a611666565b6129536020830189611666565b6129606040830188611666565b61296d60608301876122f8565b61297a60808301866122e9565b61298760a0830185612924565b61299460c08301846122f8565b98975050505050505050565b5f6060820190506129b35f8301866113f5565b6129c06020830185612282565b6129cd60408301846113f5565b949350505050565b5f67ffffffffffffffff8211156129ef576129ee6116a6565b5b602082029050602081019050919050565b5f60408284031215612a1557612a1461265b565b5b612a1f6040611704565b90505f612a2e8482850161233c565b5f830152506020612a4184828501612466565b60208301525092915050565b5f612a5f612a5a846129d5565b611704565b90508083825260208201905060408402830185811115612a8257612a8161208a565b5b835b81811015612aab5780612a978882612a00565b845260208401935050604081019050612a84565b5050509392505050565b5f82601f830112612ac957612ac861168e565b5b8151612ad9848260208601612a4d565b91505092915050565b5f60208284031215612af757612af66111ec565b5b5f82015167ffffffffffffffff811115612b1457612b136111f0565b5b612b2084828501612ab5565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b612b5b816112d4565b82525050565b5f612b6c8383612b52565b60208301905092915050565b5f602082019050919050565b5f612b8e82612b29565b612b988185612b33565b9350612ba383612b43565b805f5b83811015612bd3578151612bba8882612b61565b9750612bc583612b78565b925050600181019050612ba6565b5085935050505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f612c0482612be0565b612c0e8185612bea565b9350612c1e8185602086016124bf565b612c2781611696565b840191505092915050565b5f606082019050612c455f830186611282565b8181036020830152612c578185612b84565b90508181036040830152612c6b8184612bfa565b905094935050505056fea2646970667358221220173b5d48037ae11a1b4ddc1d3fa630e53a41fbdde44b962f2ec9d9038e92aaf264736f6c634300081e0033";
