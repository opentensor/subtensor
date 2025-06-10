export const abi = [
    {
        "inputs": [],
        "stateMutability": "nonpayable",
        "type": "constructor"
    },
    {
        "stateMutability": "payable",
        "type": "fallback"
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
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            }
        ],
        "name": "stake",
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
                "name": "netuid",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "limitPrice",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "amount",
                "type": "uint256"
            },
            {
                "internalType": "bool",
                "name": "allowPartial",
                "type": "bool"
            }
        ],
        "name": "stakeLimit",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "stateMutability": "payable",
        "type": "receive"
    }
];

export const bytecode = "60806040526004361061002c575f3560e01c80632daedd521461003757806390b9d5341461005f57610033565b3661003357005b5f80fd5b348015610042575f80fd5b5061005d600480360381019061005891906101e3565b610087565b005b34801561006a575f80fd5b5061008560048036038101906100809190610268565b6100fd565b005b5f61080590508073ffffffffffffffffffffffffffffffffffffffff16631fc9b1418584866040518463ffffffff1660e01b81526004016100ca939291906102fd565b5f604051808303815f87803b1580156100e1575f80fd5b505af11580156100f3573d5f803e3d5ffd5b5050505050505050565b5f61080590508073ffffffffffffffffffffffffffffffffffffffff16635beb6b74878587868a6040518663ffffffff1660e01b8152600401610144959493929190610341565b5f604051808303815f87803b15801561015b575f80fd5b505af115801561016d573d5f803e3d5ffd5b50505050505050505050565b5f80fd5b5f819050919050565b61018f8161017d565b8114610199575f80fd5b50565b5f813590506101aa81610186565b92915050565b5f819050919050565b6101c2816101b0565b81146101cc575f80fd5b50565b5f813590506101dd816101b9565b92915050565b5f805f606084860312156101fa576101f9610179565b5b5f6102078682870161019c565b9350506020610218868287016101cf565b9250506040610229868287016101cf565b9150509250925092565b5f8115159050919050565b61024781610233565b8114610251575f80fd5b50565b5f813590506102628161023e565b92915050565b5f805f805f60a0868803121561028157610280610179565b5b5f61028e8882890161019c565b955050602061029f888289016101cf565b94505060406102b0888289016101cf565b93505060606102c1888289016101cf565b92505060806102d288828901610254565b9150509295509295909350565b6102e88161017d565b82525050565b6102f7816101b0565b82525050565b5f6060820190506103105f8301866102df565b61031d60208301856102ee565b61032a60408301846102ee565b949350505050565b61033b81610233565b82525050565b5f60a0820190506103545f8301886102df565b61036160208301876102ee565b61036e60408301866102ee565b61037b6060830185610332565b61038860808301846102ee565b969550505050505056fea2646970667358221220487c1deb4b7fe3dbbddef0692fc83ed276d4f0be73e1fba13146efe61244da1864736f6c634300081a0033"
