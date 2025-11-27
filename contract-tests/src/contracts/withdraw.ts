export const WITHDRAW_CONTRACT_ABI = [
    {
        "inputs": [],
        "stateMutability": "nonpayable",
        "type": "constructor"
    },
    {
        "inputs": [
            {
                "internalType": "uint256",
                "name": "value",
                "type": "uint256"
            }
        ],
        "name": "withdraw",
        "outputs": [],
        "stateMutability": "payable",
        "type": "function"
    },
    {
        "stateMutability": "payable",
        "type": "receive"
    }
];

// "compiler": {
//     "version": "0.8.26+commit.8a97fa7a"
// },

export const WITHDRAW_CONTRACT_BYTECODE = "6080604052348015600e575f80fd5b506101148061001c5f395ff3fe608060405260043610601e575f3560e01c80632e1a7d4d146028576024565b36602457005b5f80fd5b603e6004803603810190603a919060b8565b6040565b005b3373ffffffffffffffffffffffffffffffffffffffff166108fc8290811502906040515f60405180830381858888f193505050501580156082573d5f803e3d5ffd5b5050565b5f80fd5b5f819050919050565b609a81608a565b811460a3575f80fd5b50565b5f8135905060b2816093565b92915050565b5f6020828403121560ca5760c96086565b5b5f60d58482850160a6565b9150509291505056fea2646970667358221220f43400858bfe4fcc0bf3c1e2e06d3a9e6ced86454a00bd7e4866b3d4d64e46bb64736f6c634300081a0033"

