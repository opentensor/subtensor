export const IPROXY_ADDRESS = "0x000000000000000000000000000000000000080b";

export const IProxyABI = [
    {
        "inputs": [
          {
            "internalType": "uint8",
            "name": "proxy_type",
            "type": "uint8"
          },
          {
            "internalType": "uint32",
            "name": "delay",
            "type": "uint32"
          },
          {
            "internalType": "uint16",
            "name": "index",
            "type": "uint16"
          }
        ],
        "name": "createPureProxy",
        "outputs": [
            {
                "internalType": "bytes32",
                "name": "proxy",
                "type": "bytes32"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bytes32",
                "name": "spawner",
                "type": "bytes32"
            },
            {
                "internalType": "uint8",
                "name": "proxy_type",
                "type": "uint8"
            },
            {
              "internalType": "uint16",
              "name": "index",
              "type": "uint16"
            },
            {
              "internalType": "uint32",
              "name": "height",
              "type": "uint32"
            },
            {
              "internalType": "uint32",
              "name": "ext_index",
              "type": "uint32"
            }
        ],
        "name": "killPureProxy",
        "outputs": [],
        "stateMutability": "nonpayable",
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
                "name": "force_proxy_type", // optional
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
      "inputs": [],
      "name": "removeProxies",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },{
      "inputs": [],
      "name": "pokeDeposit",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
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
      "name": "removeProxy",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
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
    }
];
