export const wagmiContract = {
  abi: [
    {
      "inputs": [
        {
          "internalType": "string",
          "name": "name_",
          "type": "string"
        },
        {
          "internalType": "string",
          "name": "symbol_",
          "type": "string"
        },
        {
          "internalType": "address",
          "name": "admin",
          "type": "address"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "constructor"
    },
    {
      "inputs": [],
      "name": "AccessControlBadConfirmation",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        },
        {
          "internalType": "bytes32",
          "name": "neededRole",
          "type": "bytes32"
        }
      ],
      "name": "AccessControlUnauthorizedAccount",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "allowance",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "needed",
          "type": "uint256"
        }
      ],
      "name": "ERC20InsufficientAllowance",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "balance",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "needed",
          "type": "uint256"
        }
      ],
      "name": "ERC20InsufficientBalance",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "approver",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidApprover",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "receiver",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidReceiver",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "sender",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidSender",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidSpender",
      "type": "error"
    },
    {
      "inputs": [],
      "name": "UnauthorizedHandler",
      "type": "error"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "Approval",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "indexed": true,
          "internalType": "bytes32",
          "name": "previousAdminRole",
          "type": "bytes32"
        },
        {
          "indexed": true,
          "internalType": "bytes32",
          "name": "newAdminRole",
          "type": "bytes32"
        }
      ],
      "name": "RoleAdminChanged",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "account",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        }
      ],
      "name": "RoleGranted",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "account",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "sender",
          "type": "address"
        }
      ],
      "name": "RoleRevoked",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "from",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "Transfer",
      "type": "event"
    },
    {
      "inputs": [],
      "name": "DEFAULT_ADMIN_ROLE",
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
      "inputs": [
        {
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        }
      ],
      "name": "allowance",
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
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "approve",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "balanceOf",
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
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "burn",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "from",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        }
      ],
      "name": "burnFrom",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "decimals",
      "outputs": [
        {
          "internalType": "uint8",
          "name": "",
          "type": "uint8"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        }
      ],
      "name": "getRoleAdmin",
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
      "inputs": [
        {
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "grantRole",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "hasRole",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "isAdmin",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        }
      ],
      "name": "mint",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "name",
      "outputs": [
        {
          "internalType": "string",
          "name": "",
          "type": "string"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "internalType": "address",
          "name": "callerConfirmation",
          "type": "address"
        }
      ],
      "name": "renounceRole",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "bytes32",
          "name": "role",
          "type": "bytes32"
        },
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "revokeRole",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "bytes4",
          "name": "interfaceId",
          "type": "bytes4"
        }
      ],
      "name": "supportsInterface",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "symbol",
      "outputs": [
        {
          "internalType": "string",
          "name": "",
          "type": "string"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "totalSupply",
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
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "transfer",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "from",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "transferFrom",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    }
  ],
  bytecode:
    '0x60806040523480156200001157600080fd5b5060405162000fac38038062000fac8339810160408190526200003491620001ea565b8282600362000044838262000308565b50600462000053828262000308565b5062000065915060009050826200006f565b50505050620003d4565b60008281526005602090815260408083206001600160a01b038516845290915281205460ff16620001185760008381526005602090815260408083206001600160a01b03861684529091529020805460ff19166001179055620000cf3390565b6001600160a01b0316826001600160a01b0316847f2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d60405160405180910390a45060016200011c565b5060005b92915050565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126200014a57600080fd5b81516001600160401b038082111562000167576200016762000122565b604051601f8301601f19908116603f0116810190828211818310171562000192576200019262000122565b8160405283815260209250866020858801011115620001b057600080fd5b600091505b83821015620001d45785820183015181830184015290820190620001b5565b6000602085830101528094505050505092915050565b6000806000606084860312156200020057600080fd5b83516001600160401b03808211156200021857600080fd5b620002268783880162000138565b945060208601519150808211156200023d57600080fd5b506200024c8682870162000138565b604086015190935090506001600160a01b03811681146200026c57600080fd5b809150509250925092565b600181811c908216806200028c57607f821691505b602082108103620002ad57634e487b7160e01b600052602260045260246000fd5b50919050565b601f82111562000303576000816000526020600020601f850160051c81016020861015620002de5750805b601f850160051c820191505b81811015620002ff57828155600101620002ea565b5050505b505050565b81516001600160401b0381111562000324576200032462000122565b6200033c8162000335845462000277565b84620002b3565b602080601f8311600181146200037457600084156200035b5750858301515b600019600386901b1c1916600185901b178555620002ff565b600085815260208120601f198616915b82811015620003a55788860151825594840194600190910190840162000384565b5085821015620003c45787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b610bc880620003e46000396000f3fe608060405234801561001057600080fd5b506004361061012c5760003560e01c806340c10f19116100ad57806395d89b411161007157806395d89b4114610288578063a217fddf14610290578063a9059cbb14610298578063d547741f146102ab578063dd62ed3e146102be57600080fd5b806340c10f191461021357806342966c681461022657806370a082311461023957806379cc67901461026257806391d148541461027557600080fd5b8063248a9ca3116100f4578063248a9ca3146101a657806324d7806c146101c95780632f2ff15d146101dc578063313ce567146101f157806336568abe1461020057600080fd5b806301ffc9a71461013157806306fdde0314610159578063095ea7b31461016e57806318160ddd1461018157806323b872dd14610193575b600080fd5b61014461013f3660046109ab565b6102f7565b60405190151581526020015b60405180910390f35b61016161032e565b60405161015091906109dc565b61014461017c366004610a47565b6103c0565b6002545b604051908152602001610150565b6101446101a1366004610a71565b6103d8565b6101856101b4366004610aad565b60009081526005602052604090206001015490565b6101446101d7366004610ac6565b6103fc565b6101ef6101ea366004610ae1565b610408565b005b60405160128152602001610150565b6101ef61020e366004610ae1565b610433565b6101ef610221366004610a47565b61046b565b6101ef610234366004610aad565b610480565b610185610247366004610ac6565b6001600160a01b031660009081526020819052604090205490565b6101ef610270366004610a47565b61048d565b610144610283366004610ae1565b6104a2565b6101616104cd565b610185600081565b6101446102a6366004610a47565b6104dc565b6101ef6102b9366004610ae1565b6104ea565b6101856102cc366004610b0d565b6001600160a01b03918216600090815260016020908152604080832093909416825291909152205490565b60006001600160e01b03198216637965db0b60e01b148061032857506301ffc9a760e01b6001600160e01b03198316145b92915050565b60606003805461033d90610b37565b80601f016020809104026020016040519081016040528092919081815260200182805461036990610b37565b80156103b65780601f1061038b576101008083540402835291602001916103b6565b820191906000526020600020905b81548152906001019060200180831161039957829003601f168201915b5050505050905090565b6000336103ce81858561050f565b5060019392505050565b6000336103e685828561051c565b6103f1858585610599565b506001949350505050565b600061032881836104a2565b600082815260056020526040902060010154610423816105f8565b61042d8383610602565b50505050565b6001600160a01b038116331461045c5760405163334bd91960e11b815260040160405180910390fd5b6104668282610696565b505050565b6000610476816105f8565b6104668383610703565b61048a338261073d565b50565b6000610498816105f8565b610466838361073d565b60009182526005602090815260408084206001600160a01b0393909316845291905290205460ff1690565b60606004805461033d90610b37565b6000336103ce818585610599565b600082815260056020526040902060010154610505816105f8565b61042d8383610696565b6104668383836001610773565b6001600160a01b03838116600090815260016020908152604080832093861683529290522054600019811461042d578181101561058a57604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064015b60405180910390fd5b61042d84848484036000610773565b6001600160a01b0383166105c357604051634b637e8f60e11b815260006004820152602401610581565b6001600160a01b0382166105ed5760405163ec442f0560e01b815260006004820152602401610581565b610466838383610848565b61048a8133610972565b600061060e83836104a2565b61068e5760008381526005602090815260408083206001600160a01b03861684529091529020805460ff191660011790556106463390565b6001600160a01b0316826001600160a01b0316847f2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d60405160405180910390a4506001610328565b506000610328565b60006106a283836104a2565b1561068e5760008381526005602090815260408083206001600160a01b0386168085529252808320805460ff1916905551339286917ff6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b9190a4506001610328565b6001600160a01b03821661072d5760405163ec442f0560e01b815260006004820152602401610581565b61073960008383610848565b5050565b6001600160a01b03821661076757604051634b637e8f60e11b815260006004820152602401610581565b61073982600083610848565b6001600160a01b03841661079d5760405163e602df0560e01b815260006004820152602401610581565b6001600160a01b0383166107c757604051634a1406b160e11b815260006004820152602401610581565b6001600160a01b038085166000908152600160209081526040808320938716835292905220829055801561042d57826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161083a91815260200190565b60405180910390a350505050565b6001600160a01b0383166108735780600260008282546108689190610b71565b909155506108e59050565b6001600160a01b038316600090815260208190526040902054818110156108c65760405163391434e360e21b81526001600160a01b03851660048201526024810182905260448101839052606401610581565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b03821661090157600280548290039055610920565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161096591815260200190565b60405180910390a3505050565b61097c82826104a2565b6107395760405163e2517d3f60e01b81526001600160a01b038216600482015260248101839052604401610581565b6000602082840312156109bd57600080fd5b81356001600160e01b0319811681146109d557600080fd5b9392505050565b60006020808352835180602085015260005b81811015610a0a578581018301518582016040015282016109ee565b506000604082860101526040601f19601f8301168501019250505092915050565b80356001600160a01b0381168114610a4257600080fd5b919050565b60008060408385031215610a5a57600080fd5b610a6383610a2b565b946020939093013593505050565b600080600060608486031215610a8657600080fd5b610a8f84610a2b565b9250610a9d60208501610a2b565b9150604084013590509250925092565b600060208284031215610abf57600080fd5b5035919050565b600060208284031215610ad857600080fd5b6109d582610a2b565b60008060408385031215610af457600080fd5b82359150610b0460208401610a2b565b90509250929050565b60008060408385031215610b2057600080fd5b610b2983610a2b565b9150610b0460208401610a2b565b600181811c90821680610b4b57607f821691505b602082108103610b6b57634e487b7160e01b600052602260045260246000fd5b50919050565b8082018082111561032857634e487b7160e01b600052601160045260246000fdfea2646970667358221220e179fc58c926e64cb6e87416f8ca64c117044e3195b184afe45038857606c15364736f6c63430008160033'
} as const