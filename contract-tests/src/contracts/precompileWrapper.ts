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

export const PRECOMPILE_WRAPPER_BYTECODE = "6080604052348015600e575f5ffd5b50612bb98061001c5f395ff3fe6080604052600436106101d6575f3560e01c80637d691e3011610101578063b1f789ef11610094578063d75e3e0d11610063578063d75e3e0d146106a9578063db1d0fd5146106d3578063ec556889146106fd578063fc6679fb14610727576101d6565b8063b1f789ef146105fd578063bfe252a214610639578063caf2ebf214610663578063cd6f4eb11461068d576101d6565b80639f246f6f116100d05780639f246f6f14610551578063a21762761461058d578063ac3166bf146105b7578063afed65f9146105e1576101d6565b80637d691e30146104815780638bba466c1461049d57806394e3ac6f146104d9578063998538c414610515576101d6565b80634c378a96116101795780635e25f3f8116101485780635e25f3f8146103d157806369e38bc3146103ed57806371214e27146104295780637444dadc14610445576101d6565b80634c378a96146103175780634cf088d9146103415780635b53ddde1461036b5780635b7210c514610395576101d6565b80631f193572116101b55780631f193572146102665780631fc9b141146102a25780633175bd98146102be5780634054ecca146102fb576101d6565b80620ae759146101da5780630494cd9a146102025780630cadeda51461023e575b5f5ffd5b3480156101e5575f5ffd5b5061020060048036038101906101fb91906113ab565b610751565b005b34801561020d575f5ffd5b506102286004803603810190610223919061148d565b6107c1565b60405161023591906114c7565b60405180910390f35b348015610249575f5ffd5b50610264600480360381019061025f9190611519565b610843565b005b348015610271575f5ffd5b5061028c600480360381019061028791906115a0565b6108b4565b60405161029991906115da565b60405180910390f35b6102bc60048036038101906102b79190611626565b610936565b005b3480156102c9575f5ffd5b506102e460048036038101906102df9190611676565b6109a7565b6040516102f29291906116de565b60405180910390f35b61031560048036038101906103109190611705565b610a2f565b005b348015610322575f5ffd5b5061032b610a9d565b604051610338919061179e565b60405180910390f35b34801561034c575f5ffd5b50610355610aa3565b60405161036291906117d7565b60405180910390f35b348015610376575f5ffd5b5061037f610aa9565b60405161038c9190611810565b60405180910390f35b3480156103a0575f5ffd5b506103bb60048036038101906103b69190611676565b610aaf565b6040516103c8919061184b565b60405180910390f35b6103eb60048036038101906103e69190611914565b610b34565b005b3480156103f8575f5ffd5b50610413600480360381019061040e91906115a0565b610bb4565b6040516104209190611a98565b60405180910390f35b610443600480360381019061043e9190611adb565b610c36565b005b348015610450575f5ffd5b5061046b600480360381019061046691906115a0565b610cad565b604051610478919061184b565b60405180910390f35b61049b60048036038101906104969190611626565b610d2f565b005b3480156104a8575f5ffd5b506104c360048036038101906104be9190611b52565b610da0565b6040516104d09190611ca3565b60405180910390f35b3480156104e4575f5ffd5b506104ff60048036038101906104fa9190611cbd565b610e2a565b60405161050c9190611ddf565b60405180910390f35b348015610520575f5ffd5b5061053b60048036038101906105369190611cbd565b610eb0565b6040516105489190611a98565b60405180910390f35b34801561055c575f5ffd5b5061057760048036038101906105729190611cbd565b610f32565b6040516105849190611a98565b60405180910390f35b348015610598575f5ffd5b506105a1610fb4565b6040516105ae9190611e1f565b60405180910390f35b3480156105c2575f5ffd5b506105cb610fba565b6040516105d89190611e58565b60405180910390f35b6105fb60048036038101906105f69190611e9b565b610fc0565b005b348015610608575f5ffd5b50610623600480360381019061061e9190611f38565b61103d565b604051610630919061206c565b60405180910390f35b348015610644575f5ffd5b5061064d6110c9565b60405161065a91906120ac565b60405180910390f35b34801561066e575f5ffd5b506106776110cf565b60405161068491906120e5565b60405180910390f35b6106a760048036038101906106a29190611cbd565b6110d5565b005b3480156106b4575f5ffd5b506106bd611142565b6040516106ca919061211e565b60405180910390f35b3480156106de575f5ffd5b506106e7611148565b6040516106f49190612157565b60405180910390f35b348015610708575f5ffd5b5061071161114e565b60405161071e9190612190565b60405180910390f35b348015610732575f5ffd5b5061073b611154565b60405161074891906121c9565b60405180910390f35b61080b73ffffffffffffffffffffffffffffffffffffffff16620ae7598484846040518463ffffffff1660e01b815260040161078f93929190612299565b5f604051808303815f87803b1580156107a6575f5ffd5b505af11580156107b8573d5f5f3e3d5ffd5b50505050505050565b5f61080c73ffffffffffffffffffffffffffffffffffffffff16630494cd9a836040518263ffffffff1660e01b81526004016107fd91906122eb565b602060405180830381865afa158015610818573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061083c9190612318565b9050919050565b61080b73ffffffffffffffffffffffffffffffffffffffff16630cadeda58484846040518463ffffffff1660e01b815260040161088293929190612361565b5f604051808303815f87803b158015610899575f5ffd5b505af11580156108ab573d5f5f3e3d5ffd5b50505050505050565b5f61080273ffffffffffffffffffffffffffffffffffffffff16631f193572836040518263ffffffff1660e01b81526004016108f091906115da565b602060405180830381865afa15801561090b573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061092f91906123aa565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16631fc9b1418484846040518463ffffffff1660e01b8152600401610975939291906123d5565b5f604051808303815f87803b15801561098c575f5ffd5b505af115801561099e573d5f5f3e3d5ffd5b50505050505050565b5f5f61080a73ffffffffffffffffffffffffffffffffffffffff16633175bd9885856040518363ffffffff1660e01b81526004016109e692919061240a565b6040805180830381865afa158015610a00573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610a24919061245b565b915091509250929050565b61080473ffffffffffffffffffffffffffffffffffffffff16634054ecca83836040518363ffffffff1660e01b8152600401610a6c929190612499565b5f604051808303815f87803b158015610a83575f5ffd5b505af1158015610a95573d5f5f3e3d5ffd5b505050505050565b61080481565b61080581565b61080a81565b5f61080973ffffffffffffffffffffffffffffffffffffffff16635b7210c584846040518363ffffffff1660e01b8152600401610aed92919061240a565b602060405180830381865afa158015610b08573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610b2c91906124d4565b905092915050565b61080373ffffffffffffffffffffffffffffffffffffffff16631cf98c6b89898989898989896040518963ffffffff1660e01b8152600401610b7d98979695949392919061255f565b5f604051808303815f87803b158015610b94575f5ffd5b505af1158015610ba6573d5f5f3e3d5ffd5b505050505050505050505050565b5f61080873ffffffffffffffffffffffffffffffffffffffff166369e38bc3836040518263ffffffff1660e01b8152600401610bf091906115da565b602060405180830381865afa158015610c0b573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610c2f9190612620565b9050919050565b61080973ffffffffffffffffffffffffffffffffffffffff1663127e1adb86868686866040518663ffffffff1660e01b8152600401610c7995949392919061264b565b5f604051808303815f87803b158015610c90575f5ffd5b505af1158015610ca2573d5f5f3e3d5ffd5b505050505050505050565b5f61080373ffffffffffffffffffffffffffffffffffffffff16637444dadc836040518263ffffffff1660e01b8152600401610ce991906115da565b602060405180830381865afa158015610d04573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610d2891906124d4565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16637d691e308484846040518463ffffffff1660e01b8152600401610d6e939291906123d5565b5f604051808303815f87803b158015610d85575f5ffd5b505af1158015610d97573d5f5f3e3d5ffd5b50505050505050565b610da861115a565b61080973ffffffffffffffffffffffffffffffffffffffff16638bba466c836040518263ffffffff1660e01b8152600401610de3919061269c565b61016060405180830381865afa158015610dff573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610e2391906127ea565b9050919050565b606061080b73ffffffffffffffffffffffffffffffffffffffff166394e3ac6f836040518263ffffffff1660e01b8152600401610e6791906114c7565b5f60405180830381865afa158015610e81573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f82011682018060405250810190610ea99190612937565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff1663998538c4836040518263ffffffff1660e01b8152600401610eec91906114c7565b602060405180830381865afa158015610f07573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610f2b9190612620565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff16639f246f6f836040518263ffffffff1660e01b8152600401610f6e91906114c7565b602060405180830381865afa158015610f89573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610fad9190612620565b9050919050565b61080681565b61080c81565b61080a73ffffffffffffffffffffffffffffffffffffffff1663afed65f9888888888888886040518863ffffffff1660e01b8152600401611007979695949392919061298d565b5f604051808303815f87803b15801561101e575f5ffd5b505af1158015611030573d5f5f3e3d5ffd5b5050505050505050505050565b606061080673ffffffffffffffffffffffffffffffffffffffff1663b1f789ef8585856040518463ffffffff1660e01b815260040161107e939291906129fa565b5f60405180830381865afa158015611098573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f820116820180604052508101906110c09190612b3c565b90509392505050565b61080981565b61080381565b61080073ffffffffffffffffffffffffffffffffffffffff1663cd6f4eb134836040518363ffffffff1660e01b815260040161111191906114c7565b5f604051808303818588803b158015611128575f5ffd5b505af115801561113a573d5f5f3e3d5ffd5b505050505050565b61080081565b61080881565b61080b81565b61080281565b6040518061016001604052805f81526020015f67ffffffffffffffff1681526020015f67ffffffffffffffff1681526020015f63ffffffff1681526020015f67ffffffffffffffff1681526020015f81526020015f67ffffffffffffffff1681526020015f151581526020015f81526020015f151581526020015f63ffffffff1681525090565b5f604051905090565b5f5ffd5b5f5ffd5b5f819050919050565b611204816111f2565b811461120e575f5ffd5b50565b5f8135905061121f816111fb565b92915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b61126f82611229565b810181811067ffffffffffffffff8211171561128e5761128d611239565b5b80604052505050565b5f6112a06111e1565b90506112ac8282611266565b919050565b5f67ffffffffffffffff8211156112cb576112ca611239565b5b602082029050602081019050919050565b5f5ffd5b5f60ff82169050919050565b6112f5816112e0565b81146112ff575f5ffd5b50565b5f81359050611310816112ec565b92915050565b5f611328611323846112b1565b611297565b9050808382526020820190506020840283018581111561134b5761134a6112dc565b5b835b8181101561137457806113608882611302565b84526020840193505060208101905061134d565b5050509392505050565b5f82601f83011261139257611391611225565b5b81356113a2848260208601611316565b91505092915050565b5f5f5f606084860312156113c2576113c16111ea565b5b5f6113cf86828701611211565b935050602084013567ffffffffffffffff8111156113f0576113ef6111ee565b5b6113fc8682870161137e565b925050604084013567ffffffffffffffff81111561141d5761141c6111ee565b5b6114298682870161137e565b9150509250925092565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61145c82611433565b9050919050565b61146c81611452565b8114611476575f5ffd5b50565b5f8135905061148781611463565b92915050565b5f602082840312156114a2576114a16111ea565b5b5f6114af84828501611479565b91505092915050565b6114c1816111f2565b82525050565b5f6020820190506114da5f8301846114b8565b92915050565b5f63ffffffff82169050919050565b6114f8816114e0565b8114611502575f5ffd5b50565b5f81359050611513816114ef565b92915050565b5f5f5f606084860312156115305761152f6111ea565b5b5f61153d86828701611211565b935050602061154e86828701611302565b925050604061155f86828701611505565b9150509250925092565b5f61ffff82169050919050565b61157f81611569565b8114611589575f5ffd5b50565b5f8135905061159a81611576565b92915050565b5f602082840312156115b5576115b46111ea565b5b5f6115c28482850161158c565b91505092915050565b6115d481611569565b82525050565b5f6020820190506115ed5f8301846115cb565b92915050565b5f819050919050565b611605816115f3565b811461160f575f5ffd5b50565b5f81359050611620816115fc565b92915050565b5f5f5f6060848603121561163d5761163c6111ea565b5b5f61164a86828701611211565b935050602061165b86828701611612565b925050604061166c86828701611612565b9150509250925092565b5f5f6040838503121561168c5761168b6111ea565b5b5f61169985828601611505565b92505060206116aa85828601611211565b9150509250929050565b5f6fffffffffffffffffffffffffffffffff82169050919050565b6116d8816116b4565b82525050565b5f6040820190506116f15f8301856116cf565b6116fe60208301846116cf565b9392505050565b5f5f6040838503121561171b5761171a6111ea565b5b5f6117288582860161158c565b925050602061173985828601611211565b9150509250929050565b5f819050919050565b5f61176661176161175c84611433565b611743565b611433565b9050919050565b5f6117778261174c565b9050919050565b5f6117888261176d565b9050919050565b6117988161177e565b82525050565b5f6020820190506117b15f83018461178f565b92915050565b5f6117c18261176d565b9050919050565b6117d1816117b7565b82525050565b5f6020820190506117ea5f8301846117c8565b92915050565b5f6117fa8261176d565b9050919050565b61180a816117f0565b82525050565b5f6020820190506118235f830184611801565b92915050565b5f67ffffffffffffffff82169050919050565b61184581611829565b82525050565b5f60208201905061185e5f83018461183c565b92915050565b5f5ffd5b5f67ffffffffffffffff82111561188257611881611239565b5b61188b82611229565b9050602081019050919050565b828183375f83830152505050565b5f6118b86118b384611868565b611297565b9050828152602081018484840111156118d4576118d3611864565b5b6118df848285611898565b509392505050565b5f82601f8301126118fb576118fa611225565b5b813561190b8482602086016118a6565b91505092915050565b5f5f5f5f5f5f5f5f610100898b031215611931576119306111ea565b5b5f61193e8b828c01611211565b985050602089013567ffffffffffffffff81111561195f5761195e6111ee565b5b61196b8b828c016118e7565b975050604089013567ffffffffffffffff81111561198c5761198b6111ee565b5b6119988b828c016118e7565b965050606089013567ffffffffffffffff8111156119b9576119b86111ee565b5b6119c58b828c016118e7565b955050608089013567ffffffffffffffff8111156119e6576119e56111ee565b5b6119f28b828c016118e7565b94505060a089013567ffffffffffffffff811115611a1357611a126111ee565b5b611a1f8b828c016118e7565b93505060c089013567ffffffffffffffff811115611a4057611a3f6111ee565b5b611a4c8b828c016118e7565b92505060e089013567ffffffffffffffff811115611a6d57611a6c6111ee565b5b611a798b828c016118e7565b9150509295985092959890939650565b611a92816115f3565b82525050565b5f602082019050611aab5f830184611a89565b92915050565b611aba81611829565b8114611ac4575f5ffd5b50565b5f81359050611ad581611ab1565b92915050565b5f5f5f5f5f60a08688031215611af457611af36111ea565b5b5f611b0188828901611ac7565b9550506020611b1288828901611ac7565b9450506040611b2388828901611ac7565b9350506060611b3488828901611505565b9250506080611b4588828901611479565b9150509295509295909350565b5f60208284031215611b6757611b666111ea565b5b5f611b7484828501611505565b91505092915050565b611b86816111f2565b82525050565b611b9581611829565b82525050565b611ba4816114e0565b82525050565b5f8115159050919050565b611bbe81611baa565b82525050565b61016082015f820151611bd95f850182611b7d565b506020820151611bec6020850182611b8c565b506040820151611bff6040850182611b8c565b506060820151611c126060850182611b9b565b506080820151611c256080850182611b8c565b5060a0820151611c3860a0850182611b7d565b5060c0820151611c4b60c0850182611b8c565b5060e0820151611c5e60e0850182611bb5565b50610100820151611c73610100850182611b7d565b50610120820151611c88610120850182611bb5565b50610140820151611c9d610140850182611b9b565b50505050565b5f61016082019050611cb75f830184611bc4565b92915050565b5f60208284031215611cd257611cd16111ea565b5b5f611cdf84828501611211565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b611d1a816115f3565b82525050565b606082015f820151611d345f850182611b7d565b506020820151611d476020850182611d11565b506040820151611d5a6040850182611d11565b50505050565b5f611d6b8383611d20565b60608301905092915050565b5f602082019050919050565b5f611d8d82611ce8565b611d978185611cf2565b9350611da283611d02565b805f5b83811015611dd2578151611db98882611d60565b9750611dc483611d77565b925050600181019050611da5565b5085935050505092915050565b5f6020820190508181035f830152611df78184611d83565b905092915050565b5f611e098261176d565b9050919050565b611e1981611dff565b82525050565b5f602082019050611e325f830184611e10565b92915050565b5f611e428261176d565b9050919050565b611e5281611e38565b82525050565b5f602082019050611e6b5f830184611e49565b92915050565b611e7a81611baa565b8114611e84575f5ffd5b50565b5f81359050611e9581611e71565b92915050565b5f5f5f5f5f5f5f60e0888a031215611eb657611eb56111ea565b5b5f611ec38a828b01611ac7565b9750506020611ed48a828b01611ac7565b9650506040611ee58a828b01611ac7565b9550506060611ef68a828b01611505565b9450506080611f078a828b01611302565b93505060a0611f188a828b01611e87565b92505060c0611f298a828b01611505565b91505092959891949750929550565b5f5f5f60608486031215611f4f57611f4e6111ea565b5b5f611f5c8682870161158c565b9350506020611f6d86828701611479565b9250506040611f7e8682870161158c565b9150509250925092565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b611fba81611569565b82525050565b604082015f820151611fd45f850182611fb1565b506020820151611fe76020850182611b8c565b50505050565b5f611ff88383611fc0565b60408301905092915050565b5f602082019050919050565b5f61201a82611f88565b6120248185611f92565b935061202f83611fa2565b805f5b8381101561205f5781516120468882611fed565b975061205183612004565b925050600181019050612032565b5085935050505092915050565b5f6020820190508181035f8301526120848184612010565b905092915050565b5f6120968261176d565b9050919050565b6120a68161208c565b82525050565b5f6020820190506120bf5f83018461209d565b92915050565b5f6120cf8261176d565b9050919050565b6120df816120c5565b82525050565b5f6020820190506120f85f8301846120d6565b92915050565b5f6121088261176d565b9050919050565b612118816120fe565b82525050565b5f6020820190506121315f83018461210f565b92915050565b5f6121418261176d565b9050919050565b61215181612137565b82525050565b5f60208201905061216a5f830184612148565b92915050565b5f61217a8261176d565b9050919050565b61218a81612170565b82525050565b5f6020820190506121a35f830184612181565b92915050565b5f6121b38261176d565b9050919050565b6121c3816121a9565b82525050565b5f6020820190506121dc5f8301846121ba565b92915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b612214816112e0565b82525050565b5f612225838361220b565b60208301905092915050565b5f602082019050919050565b5f612247826121e2565b61225181856121ec565b935061225c836121fc565b805f5b8381101561228c578151612273888261221a565b975061227e83612231565b92505060018101905061225f565b5085935050505092915050565b5f6060820190506122ac5f8301866114b8565b81810360208301526122be818561223d565b905081810360408301526122d2818461223d565b9050949350505050565b6122e581611452565b82525050565b5f6020820190506122fe5f8301846122dc565b92915050565b5f81519050612312816111fb565b92915050565b5f6020828403121561232d5761232c6111ea565b5b5f61233a84828501612304565b91505092915050565b61234c816112e0565b82525050565b61235b816114e0565b82525050565b5f6060820190506123745f8301866114b8565b6123816020830185612343565b61238e6040830184612352565b949350505050565b5f815190506123a481611576565b92915050565b5f602082840312156123bf576123be6111ea565b5b5f6123cc84828501612396565b91505092915050565b5f6060820190506123e85f8301866114b8565b6123f56020830185611a89565b6124026040830184611a89565b949350505050565b5f60408201905061241d5f830185612352565b61242a60208301846114b8565b9392505050565b61243a816116b4565b8114612444575f5ffd5b50565b5f8151905061245581612431565b92915050565b5f5f60408385031215612471576124706111ea565b5b5f61247e85828601612447565b925050602061248f85828601612447565b9150509250929050565b5f6040820190506124ac5f8301856115cb565b6124b960208301846114b8565b9392505050565b5f815190506124ce81611ab1565b92915050565b5f602082840312156124e9576124e86111ea565b5b5f6124f6848285016124c0565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f612531826124ff565b61253b8185612509565b935061254b818560208601612519565b61255481611229565b840191505092915050565b5f610100820190506125735f83018b6114b8565b8181036020830152612585818a612527565b905081810360408301526125998189612527565b905081810360608301526125ad8188612527565b905081810360808301526125c18187612527565b905081810360a08301526125d58186612527565b905081810360c08301526125e98185612527565b905081810360e08301526125fd8184612527565b90509998505050505050505050565b5f8151905061261a816115fc565b92915050565b5f60208284031215612635576126346111ea565b5b5f6126428482850161260c565b91505092915050565b5f60a08201905061265e5f83018861183c565b61266b602083018761183c565b612678604083018661183c565b6126856060830185612352565b61269260808301846122dc565b9695505050505050565b5f6020820190506126af5f830184612352565b92915050565b5f5ffd5b5f815190506126c7816114ef565b92915050565b5f815190506126db81611e71565b92915050565b5f61016082840312156126f7576126f66126b5565b5b612702610160611297565b90505f61271184828501612304565b5f830152506020612724848285016124c0565b6020830152506040612738848285016124c0565b604083015250606061274c848285016126b9565b6060830152506080612760848285016124c0565b60808301525060a061277484828501612304565b60a08301525060c0612788848285016124c0565b60c08301525060e061279c848285016126cd565b60e0830152506101006127b184828501612304565b610100830152506101206127c7848285016126cd565b610120830152506101406127dd848285016126b9565b6101408301525092915050565b5f6101608284031215612800576127ff6111ea565b5b5f61280d848285016126e1565b91505092915050565b5f67ffffffffffffffff8211156128305761282f611239565b5b602082029050602081019050919050565b5f60608284031215612856576128556126b5565b5b6128606060611297565b90505f61286f84828501612304565b5f8301525060206128828482850161260c565b60208301525060406128968482850161260c565b60408301525092915050565b5f6128b46128af84612816565b611297565b905080838252602082019050606084028301858111156128d7576128d66112dc565b5b835b8181101561290057806128ec8882612841565b8452602084019350506060810190506128d9565b5050509392505050565b5f82601f83011261291e5761291d611225565b5b815161292e8482602086016128a2565b91505092915050565b5f6020828403121561294c5761294b6111ea565b5b5f82015167ffffffffffffffff811115612969576129686111ee565b5b6129758482850161290a565b91505092915050565b61298781611baa565b82525050565b5f60e0820190506129a05f83018a61183c565b6129ad602083018961183c565b6129ba604083018861183c565b6129c76060830187612352565b6129d46080830186612343565b6129e160a083018561297e565b6129ee60c0830184612352565b98975050505050505050565b5f606082019050612a0d5f8301866115cb565b612a1a60208301856122dc565b612a2760408301846115cb565b949350505050565b5f67ffffffffffffffff821115612a4957612a48611239565b5b602082029050602081019050919050565b5f60408284031215612a6f57612a6e6126b5565b5b612a796040611297565b90505f612a8884828501612396565b5f830152506020612a9b848285016124c0565b60208301525092915050565b5f612ab9612ab484612a2f565b611297565b90508083825260208201905060408402830185811115612adc57612adb6112dc565b5b835b81811015612b055780612af18882612a5a565b845260208401935050604081019050612ade565b5050509392505050565b5f82601f830112612b2357612b22611225565b5b8151612b33848260208601612aa7565b91505092915050565b5f60208284031215612b5157612b506111ea565b5b5f82015167ffffffffffffffff811115612b6e57612b6d6111ee565b5b612b7a84828501612b0f565b9150509291505056fea2646970667358221220768c64014d2253c661e44d07f480f7a203eb9e422f680d00272498325a4f6ad964736f6c634300081e0033";
