export const IADDRESS_MAPPING_ADDRESS = "0x000000000000000000000000000000000000080c";

export const IAddressMappingABI = [
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
    }
];
