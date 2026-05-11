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
        "name": "getNetworkRegisteredBlock",
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

export const PRECOMPILE_WRAPPER_BYTECODE = "6080604052348015600e575f5ffd5b50612c848061001c5f395ff3fe6080604052600436106101e1575f3560e01c80637d691e3011610101578063b1f789ef11610094578063d75e3e0d11610063578063d75e3e0d146106f0578063db1d0fd51461071a578063ec55688914610744578063fc6679fb1461076e576101e1565b8063b1f789ef14610644578063bfe252a214610680578063caf2ebf2146106aa578063cd6f4eb1146106d4576101e1565b80639f246f6f116100d05780639f246f6f14610598578063a2176276146105d4578063ac3166bf146105fe578063afed65f914610628576101e1565b80637d691e30146104c85780638bba466c146104e457806394e3ac6f14610520578063998538c41461055c576101e1565b80634c378a96116101795780635e25f3f8116101485780635e25f3f81461041857806369e38bc31461043457806371214e27146104705780637444dadc1461048c576101e1565b80634c378a961461035e5780634cf088d9146103885780635b53ddde146103b25780635b7210c5146103dc576101e1565b80631f193572116101b55780631f193572146102ad5780631fc9b141146102e95780633175bd98146103055780634054ecca14610342576101e1565b80620ae759146101e55780630494cd9a1461020d57806304eaf18c146102495780630cadeda514610285575b5f5ffd5b3480156101f0575f5ffd5b5061020b60048036038101906102069190611476565b610798565b005b348015610218575f5ffd5b50610233600480360381019061022e9190611558565b610808565b6040516102409190611592565b60405180910390f35b348015610254575f5ffd5b5061026f600480360381019061026a91906115e2565b61088a565b60405161027c919061162f565b60405180910390f35b348015610290575f5ffd5b506102ab60048036038101906102a69190611681565b61090c565b005b3480156102b8575f5ffd5b506102d360048036038101906102ce91906115e2565b61097d565b6040516102e091906116e0565b60405180910390f35b61030360048036038101906102fe919061172c565b6109ff565b005b348015610310575f5ffd5b5061032b6004803603810190610326919061177c565b610a70565b6040516103399291906117e4565b60405180910390f35b61035c6004803603810190610357919061180b565b610af8565b005b348015610369575f5ffd5b50610372610b68565b60405161037f91906118a4565b60405180910390f35b348015610393575f5ffd5b5061039c610b6e565b6040516103a991906118dd565b60405180910390f35b3480156103bd575f5ffd5b506103c6610b74565b6040516103d39190611916565b60405180910390f35b3480156103e7575f5ffd5b5061040260048036038101906103fd919061177c565b610b7a565b60405161040f919061162f565b60405180910390f35b610432600480360381019061042d91906119df565b610bff565b005b34801561043f575f5ffd5b5061045a600480360381019061045591906115e2565b610c7f565b6040516104679190611b63565b60405180910390f35b61048a60048036038101906104859190611ba6565b610d01565b005b348015610497575f5ffd5b506104b260048036038101906104ad91906115e2565b610d78565b6040516104bf919061162f565b60405180910390f35b6104e260048036038101906104dd919061172c565b610dfa565b005b3480156104ef575f5ffd5b5061050a60048036038101906105059190611c1d565b610e6b565b6040516105179190611d6e565b60405180910390f35b34801561052b575f5ffd5b5061054660048036038101906105419190611d88565b610ef5565b6040516105539190611eaa565b60405180910390f35b348015610567575f5ffd5b50610582600480360381019061057d9190611d88565b610f7b565b60405161058f9190611b63565b60405180910390f35b3480156105a3575f5ffd5b506105be60048036038101906105b99190611d88565b610ffd565b6040516105cb9190611b63565b60405180910390f35b3480156105df575f5ffd5b506105e861107f565b6040516105f59190611eea565b60405180910390f35b348015610609575f5ffd5b50610612611085565b60405161061f9190611f23565b60405180910390f35b610642600480360381019061063d9190611f66565b61108b565b005b34801561064f575f5ffd5b5061066a60048036038101906106659190612003565b611108565b6040516106779190612137565b60405180910390f35b34801561068b575f5ffd5b50610694611194565b6040516106a19190612177565b60405180910390f35b3480156106b5575f5ffd5b506106be61119a565b6040516106cb91906121b0565b60405180910390f35b6106ee60048036038101906106e99190611d88565b6111a0565b005b3480156106fb575f5ffd5b5061070461120d565b60405161071191906121e9565b60405180910390f35b348015610725575f5ffd5b5061072e611213565b60405161073b9190612222565b60405180910390f35b34801561074f575f5ffd5b50610758611219565b604051610765919061225b565b60405180910390f35b348015610779575f5ffd5b5061078261121f565b60405161078f9190612294565b60405180910390f35b61080b73ffffffffffffffffffffffffffffffffffffffff16620ae7598484846040518463ffffffff1660e01b81526004016107d693929190612364565b5f604051808303815f87803b1580156107ed575f5ffd5b505af11580156107ff573d5f5f3e3d5ffd5b50505050505050565b5f61080c73ffffffffffffffffffffffffffffffffffffffff16630494cd9a836040518263ffffffff1660e01b815260040161084491906123b6565b602060405180830381865afa15801561085f573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061088391906123e3565b9050919050565b5f61080373ffffffffffffffffffffffffffffffffffffffff166304eaf18c836040518263ffffffff1660e01b81526004016108c691906116e0565b602060405180830381865afa1580156108e1573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109059190612422565b9050919050565b61080b73ffffffffffffffffffffffffffffffffffffffff16630cadeda58484846040518463ffffffff1660e01b815260040161094b9392919061246b565b5f604051808303815f87803b158015610962575f5ffd5b505af1158015610974573d5f5f3e3d5ffd5b50505050505050565b5f61080273ffffffffffffffffffffffffffffffffffffffff16631f193572836040518263ffffffff1660e01b81526004016109b991906116e0565b602060405180830381865afa1580156109d4573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906109f891906124b4565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16631fc9b1418484846040518463ffffffff1660e01b8152600401610a3e939291906124df565b5f604051808303815f87803b158015610a55575f5ffd5b505af1158015610a67573d5f5f3e3d5ffd5b50505050505050565b5f5f61080a73ffffffffffffffffffffffffffffffffffffffff16633175bd9885856040518363ffffffff1660e01b8152600401610aaf929190612514565b6040805180830381865afa158015610ac9573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610aed9190612565565b915091509250929050565b61080473ffffffffffffffffffffffffffffffffffffffff16634054ecca3484846040518463ffffffff1660e01b8152600401610b369291906125a3565b5f604051808303818588803b158015610b4d575f5ffd5b505af1158015610b5f573d5f5f3e3d5ffd5b50505050505050565b61080481565b61080581565b61080a81565b5f61080973ffffffffffffffffffffffffffffffffffffffff16635b7210c584846040518363ffffffff1660e01b8152600401610bb8929190612514565b602060405180830381865afa158015610bd3573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610bf79190612422565b905092915050565b61080373ffffffffffffffffffffffffffffffffffffffff16631cf98c6b89898989898989896040518963ffffffff1660e01b8152600401610c4898979695949392919061262a565b5f604051808303815f87803b158015610c5f575f5ffd5b505af1158015610c71573d5f5f3e3d5ffd5b505050505050505050505050565b5f61080873ffffffffffffffffffffffffffffffffffffffff166369e38bc3836040518263ffffffff1660e01b8152600401610cbb91906116e0565b602060405180830381865afa158015610cd6573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610cfa91906126eb565b9050919050565b61080973ffffffffffffffffffffffffffffffffffffffff1663127e1adb86868686866040518663ffffffff1660e01b8152600401610d44959493929190612716565b5f604051808303815f87803b158015610d5b575f5ffd5b505af1158015610d6d573d5f5f3e3d5ffd5b505050505050505050565b5f61080373ffffffffffffffffffffffffffffffffffffffff16637444dadc836040518263ffffffff1660e01b8152600401610db491906116e0565b602060405180830381865afa158015610dcf573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610df39190612422565b9050919050565b61080573ffffffffffffffffffffffffffffffffffffffff16637d691e308484846040518463ffffffff1660e01b8152600401610e39939291906124df565b5f604051808303815f87803b158015610e50575f5ffd5b505af1158015610e62573d5f5f3e3d5ffd5b50505050505050565b610e73611225565b61080973ffffffffffffffffffffffffffffffffffffffff16638bba466c836040518263ffffffff1660e01b8152600401610eae9190612767565b61016060405180830381865afa158015610eca573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610eee91906128b5565b9050919050565b606061080b73ffffffffffffffffffffffffffffffffffffffff166394e3ac6f836040518263ffffffff1660e01b8152600401610f329190611592565b5f60405180830381865afa158015610f4c573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f82011682018060405250810190610f749190612a02565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff1663998538c4836040518263ffffffff1660e01b8152600401610fb79190611592565b602060405180830381865afa158015610fd2573d5f5f3e3d5ffd5b505050506040513d601f19601f82011682018060405250810190610ff691906126eb565b9050919050565b5f61080573ffffffffffffffffffffffffffffffffffffffff16639f246f6f836040518263ffffffff1660e01b81526004016110399190611592565b602060405180830381865afa158015611054573d5f5f3e3d5ffd5b505050506040513d601f19601f8201168201806040525081019061107891906126eb565b9050919050565b61080681565b61080c81565b61080a73ffffffffffffffffffffffffffffffffffffffff1663afed65f9888888888888886040518863ffffffff1660e01b81526004016110d29796959493929190612a58565b5f604051808303815f87803b1580156110e9575f5ffd5b505af11580156110fb573d5f5f3e3d5ffd5b5050505050505050505050565b606061080673ffffffffffffffffffffffffffffffffffffffff1663b1f789ef8585856040518463ffffffff1660e01b815260040161114993929190612ac5565b5f60405180830381865afa158015611163573d5f5f3e3d5ffd5b505050506040513d5f823e3d601f19601f8201168201806040525081019061118b9190612c07565b90509392505050565b61080981565b61080381565b61080073ffffffffffffffffffffffffffffffffffffffff1663cd6f4eb134836040518363ffffffff1660e01b81526004016111dc9190611592565b5f604051808303818588803b1580156111f3575f5ffd5b505af1158015611205573d5f5f3e3d5ffd5b505050505050565b61080081565b61080881565b61080b81565b61080281565b6040518061016001604052805f81526020015f67ffffffffffffffff1681526020015f67ffffffffffffffff1681526020015f63ffffffff1681526020015f67ffffffffffffffff1681526020015f81526020015f67ffffffffffffffff1681526020015f151581526020015f81526020015f151581526020015f63ffffffff1681525090565b5f604051905090565b5f5ffd5b5f5ffd5b5f819050919050565b6112cf816112bd565b81146112d9575f5ffd5b50565b5f813590506112ea816112c6565b92915050565b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b61133a826112f4565b810181811067ffffffffffffffff8211171561135957611358611304565b5b80604052505050565b5f61136b6112ac565b90506113778282611331565b919050565b5f67ffffffffffffffff82111561139657611395611304565b5b602082029050602081019050919050565b5f5ffd5b5f60ff82169050919050565b6113c0816113ab565b81146113ca575f5ffd5b50565b5f813590506113db816113b7565b92915050565b5f6113f36113ee8461137c565b611362565b90508083825260208201905060208402830185811115611416576114156113a7565b5b835b8181101561143f578061142b88826113cd565b845260208401935050602081019050611418565b5050509392505050565b5f82601f83011261145d5761145c6112f0565b5b813561146d8482602086016113e1565b91505092915050565b5f5f5f6060848603121561148d5761148c6112b5565b5b5f61149a868287016112dc565b935050602084013567ffffffffffffffff8111156114bb576114ba6112b9565b5b6114c786828701611449565b925050604084013567ffffffffffffffff8111156114e8576114e76112b9565b5b6114f486828701611449565b9150509250925092565b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f611527826114fe565b9050919050565b6115378161151d565b8114611541575f5ffd5b50565b5f813590506115528161152e565b92915050565b5f6020828403121561156d5761156c6112b5565b5b5f61157a84828501611544565b91505092915050565b61158c816112bd565b82525050565b5f6020820190506115a55f830184611583565b92915050565b5f61ffff82169050919050565b6115c1816115ab565b81146115cb575f5ffd5b50565b5f813590506115dc816115b8565b92915050565b5f602082840312156115f7576115f66112b5565b5b5f611604848285016115ce565b91505092915050565b5f67ffffffffffffffff82169050919050565b6116298161160d565b82525050565b5f6020820190506116425f830184611620565b92915050565b5f63ffffffff82169050919050565b61166081611648565b811461166a575f5ffd5b50565b5f8135905061167b81611657565b92915050565b5f5f5f60608486031215611698576116976112b5565b5b5f6116a5868287016112dc565b93505060206116b6868287016113cd565b92505060406116c78682870161166d565b9150509250925092565b6116da816115ab565b82525050565b5f6020820190506116f35f8301846116d1565b92915050565b5f819050919050565b61170b816116f9565b8114611715575f5ffd5b50565b5f8135905061172681611702565b92915050565b5f5f5f60608486031215611743576117426112b5565b5b5f611750868287016112dc565b935050602061176186828701611718565b925050604061177286828701611718565b9150509250925092565b5f5f60408385031215611792576117916112b5565b5b5f61179f8582860161166d565b92505060206117b0858286016112dc565b9150509250929050565b5f6fffffffffffffffffffffffffffffffff82169050919050565b6117de816117ba565b82525050565b5f6040820190506117f75f8301856117d5565b61180460208301846117d5565b9392505050565b5f5f60408385031215611821576118206112b5565b5b5f61182e858286016115ce565b925050602061183f858286016112dc565b9150509250929050565b5f819050919050565b5f61186c611867611862846114fe565b611849565b6114fe565b9050919050565b5f61187d82611852565b9050919050565b5f61188e82611873565b9050919050565b61189e81611884565b82525050565b5f6020820190506118b75f830184611895565b92915050565b5f6118c782611873565b9050919050565b6118d7816118bd565b82525050565b5f6020820190506118f05f8301846118ce565b92915050565b5f61190082611873565b9050919050565b611910816118f6565b82525050565b5f6020820190506119295f830184611907565b92915050565b5f5ffd5b5f67ffffffffffffffff82111561194d5761194c611304565b5b611956826112f4565b9050602081019050919050565b828183375f83830152505050565b5f61198361197e84611933565b611362565b90508281526020810184848401111561199f5761199e61192f565b5b6119aa848285611963565b509392505050565b5f82601f8301126119c6576119c56112f0565b5b81356119d6848260208601611971565b91505092915050565b5f5f5f5f5f5f5f5f610100898b0312156119fc576119fb6112b5565b5b5f611a098b828c016112dc565b985050602089013567ffffffffffffffff811115611a2a57611a296112b9565b5b611a368b828c016119b2565b975050604089013567ffffffffffffffff811115611a5757611a566112b9565b5b611a638b828c016119b2565b965050606089013567ffffffffffffffff811115611a8457611a836112b9565b5b611a908b828c016119b2565b955050608089013567ffffffffffffffff811115611ab157611ab06112b9565b5b611abd8b828c016119b2565b94505060a089013567ffffffffffffffff811115611ade57611add6112b9565b5b611aea8b828c016119b2565b93505060c089013567ffffffffffffffff811115611b0b57611b0a6112b9565b5b611b178b828c016119b2565b92505060e089013567ffffffffffffffff811115611b3857611b376112b9565b5b611b448b828c016119b2565b9150509295985092959890939650565b611b5d816116f9565b82525050565b5f602082019050611b765f830184611b54565b92915050565b611b858161160d565b8114611b8f575f5ffd5b50565b5f81359050611ba081611b7c565b92915050565b5f5f5f5f5f60a08688031215611bbf57611bbe6112b5565b5b5f611bcc88828901611b92565b9550506020611bdd88828901611b92565b9450506040611bee88828901611b92565b9350506060611bff8882890161166d565b9250506080611c1088828901611544565b9150509295509295909350565b5f60208284031215611c3257611c316112b5565b5b5f611c3f8482850161166d565b91505092915050565b611c51816112bd565b82525050565b611c608161160d565b82525050565b611c6f81611648565b82525050565b5f8115159050919050565b611c8981611c75565b82525050565b61016082015f820151611ca45f850182611c48565b506020820151611cb76020850182611c57565b506040820151611cca6040850182611c57565b506060820151611cdd6060850182611c66565b506080820151611cf06080850182611c57565b5060a0820151611d0360a0850182611c48565b5060c0820151611d1660c0850182611c57565b5060e0820151611d2960e0850182611c80565b50610100820151611d3e610100850182611c48565b50610120820151611d53610120850182611c80565b50610140820151611d68610140850182611c66565b50505050565b5f61016082019050611d825f830184611c8f565b92915050565b5f60208284031215611d9d57611d9c6112b5565b5b5f611daa848285016112dc565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b611de5816116f9565b82525050565b606082015f820151611dff5f850182611c48565b506020820151611e126020850182611ddc565b506040820151611e256040850182611ddc565b50505050565b5f611e368383611deb565b60608301905092915050565b5f602082019050919050565b5f611e5882611db3565b611e628185611dbd565b9350611e6d83611dcd565b805f5b83811015611e9d578151611e848882611e2b565b9750611e8f83611e42565b925050600181019050611e70565b5085935050505092915050565b5f6020820190508181035f830152611ec28184611e4e565b905092915050565b5f611ed482611873565b9050919050565b611ee481611eca565b82525050565b5f602082019050611efd5f830184611edb565b92915050565b5f611f0d82611873565b9050919050565b611f1d81611f03565b82525050565b5f602082019050611f365f830184611f14565b92915050565b611f4581611c75565b8114611f4f575f5ffd5b50565b5f81359050611f6081611f3c565b92915050565b5f5f5f5f5f5f5f60e0888a031215611f8157611f806112b5565b5b5f611f8e8a828b01611b92565b9750506020611f9f8a828b01611b92565b9650506040611fb08a828b01611b92565b9550506060611fc18a828b0161166d565b9450506080611fd28a828b016113cd565b93505060a0611fe38a828b01611f52565b92505060c0611ff48a828b0161166d565b91505092959891949750929550565b5f5f5f6060848603121561201a576120196112b5565b5b5f612027868287016115ce565b935050602061203886828701611544565b9250506040612049868287016115ce565b9150509250925092565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b612085816115ab565b82525050565b604082015f82015161209f5f85018261207c565b5060208201516120b26020850182611c57565b50505050565b5f6120c3838361208b565b60408301905092915050565b5f602082019050919050565b5f6120e582612053565b6120ef818561205d565b93506120fa8361206d565b805f5b8381101561212a57815161211188826120b8565b975061211c836120cf565b9250506001810190506120fd565b5085935050505092915050565b5f6020820190508181035f83015261214f81846120db565b905092915050565b5f61216182611873565b9050919050565b61217181612157565b82525050565b5f60208201905061218a5f830184612168565b92915050565b5f61219a82611873565b9050919050565b6121aa81612190565b82525050565b5f6020820190506121c35f8301846121a1565b92915050565b5f6121d382611873565b9050919050565b6121e3816121c9565b82525050565b5f6020820190506121fc5f8301846121da565b92915050565b5f61220c82611873565b9050919050565b61221c81612202565b82525050565b5f6020820190506122355f830184612213565b92915050565b5f61224582611873565b9050919050565b6122558161223b565b82525050565b5f60208201905061226e5f83018461224c565b92915050565b5f61227e82611873565b9050919050565b61228e81612274565b82525050565b5f6020820190506122a75f830184612285565b92915050565b5f81519050919050565b5f82825260208201905092915050565b5f819050602082019050919050565b6122df816113ab565b82525050565b5f6122f083836122d6565b60208301905092915050565b5f602082019050919050565b5f612312826122ad565b61231c81856122b7565b9350612327836122c7565b805f5b8381101561235757815161233e88826122e5565b9750612349836122fc565b92505060018101905061232a565b5085935050505092915050565b5f6060820190506123775f830186611583565b81810360208301526123898185612308565b9050818103604083015261239d8184612308565b9050949350505050565b6123b08161151d565b82525050565b5f6020820190506123c95f8301846123a7565b92915050565b5f815190506123dd816112c6565b92915050565b5f602082840312156123f8576123f76112b5565b5b5f612405848285016123cf565b91505092915050565b5f8151905061241c81611b7c565b92915050565b5f60208284031215612437576124366112b5565b5b5f6124448482850161240e565b91505092915050565b612456816113ab565b82525050565b61246581611648565b82525050565b5f60608201905061247e5f830186611583565b61248b602083018561244d565b612498604083018461245c565b949350505050565b5f815190506124ae816115b8565b92915050565b5f602082840312156124c9576124c86112b5565b5b5f6124d6848285016124a0565b91505092915050565b5f6060820190506124f25f830186611583565b6124ff6020830185611b54565b61250c6040830184611b54565b949350505050565b5f6040820190506125275f83018561245c565b6125346020830184611583565b9392505050565b612544816117ba565b811461254e575f5ffd5b50565b5f8151905061255f8161253b565b92915050565b5f5f6040838503121561257b5761257a6112b5565b5b5f61258885828601612551565b925050602061259985828601612551565b9150509250929050565b5f6040820190506125b65f8301856116d1565b6125c36020830184611583565b9392505050565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f6125fc826125ca565b61260681856125d4565b93506126168185602086016125e4565b61261f816112f4565b840191505092915050565b5f6101008201905061263e5f83018b611583565b8181036020830152612650818a6125f2565b9050818103604083015261266481896125f2565b9050818103606083015261267881886125f2565b9050818103608083015261268c81876125f2565b905081810360a08301526126a081866125f2565b905081810360c08301526126b481856125f2565b905081810360e08301526126c881846125f2565b90509998505050505050505050565b5f815190506126e581611702565b92915050565b5f60208284031215612700576126ff6112b5565b5b5f61270d848285016126d7565b91505092915050565b5f60a0820190506127295f830188611620565b6127366020830187611620565b6127436040830186611620565b612750606083018561245c565b61275d60808301846123a7565b9695505050505050565b5f60208201905061277a5f83018461245c565b92915050565b5f5ffd5b5f8151905061279281611657565b92915050565b5f815190506127a681611f3c565b92915050565b5f61016082840312156127c2576127c1612780565b5b6127cd610160611362565b90505f6127dc848285016123cf565b5f8301525060206127ef8482850161240e565b60208301525060406128038482850161240e565b604083015250606061281784828501612784565b606083015250608061282b8482850161240e565b60808301525060a061283f848285016123cf565b60a08301525060c06128538482850161240e565b60c08301525060e061286784828501612798565b60e08301525061010061287c848285016123cf565b6101008301525061012061289284828501612798565b610120830152506101406128a884828501612784565b6101408301525092915050565b5f61016082840312156128cb576128ca6112b5565b5b5f6128d8848285016127ac565b91505092915050565b5f67ffffffffffffffff8211156128fb576128fa611304565b5b602082029050602081019050919050565b5f6060828403121561292157612920612780565b5b61292b6060611362565b90505f61293a848285016123cf565b5f83015250602061294d848285016126d7565b6020830152506040612961848285016126d7565b60408301525092915050565b5f61297f61297a846128e1565b611362565b905080838252602082019050606084028301858111156129a2576129a16113a7565b5b835b818110156129cb57806129b7888261290c565b8452602084019350506060810190506129a4565b5050509392505050565b5f82601f8301126129e9576129e86112f0565b5b81516129f984826020860161296d565b91505092915050565b5f60208284031215612a1757612a166112b5565b5b5f82015167ffffffffffffffff811115612a3457612a336112b9565b5b612a40848285016129d5565b91505092915050565b612a5281611c75565b82525050565b5f60e082019050612a6b5f83018a611620565b612a786020830189611620565b612a856040830188611620565b612a92606083018761245c565b612a9f608083018661244d565b612aac60a0830185612a49565b612ab960c083018461245c565b98975050505050505050565b5f606082019050612ad85f8301866116d1565b612ae560208301856123a7565b612af260408301846116d1565b949350505050565b5f67ffffffffffffffff821115612b1457612b13611304565b5b602082029050602081019050919050565b5f60408284031215612b3a57612b39612780565b5b612b446040611362565b90505f612b53848285016124a0565b5f830152506020612b668482850161240e565b60208301525092915050565b5f612b84612b7f84612afa565b611362565b90508083825260208201905060408402830185811115612ba757612ba66113a7565b5b835b81811015612bd05780612bbc8882612b25565b845260208401935050604081019050612ba9565b5050509392505050565b5f82601f830112612bee57612bed6112f0565b5b8151612bfe848260208601612b72565b91505092915050565b5f60208284031215612c1c57612c1b6112b5565b5b5f82015167ffffffffffffffff811115612c3957612c386112b9565b5b612c4584828501612bda565b9150509291505056fea2646970667358221220a2cc2a9c8dfdc11158aae6437dbe7c5bcd4cc87d88a338d0b3f1218b26b81b6b64736f6c63430008230033";
