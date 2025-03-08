import * as assert from "assert";
import * as chai from "chai";

import { getDevnetApi } from "../src/substrate"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { wagmiContract } from "../src/bridgeToken";
import { toViemAddress } from "../src/address-utils";
import { forceSetBalanceToEthAddress, disableWhiteListCheck } from "../src/subtensor";
import { ethers } from "ethers"
describe("bridge token contract deployment", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();
    let publicClient: PublicClient;

    // init substrate part
    let api: TypedApi<typeof devnet>

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet.address)
        await disableWhiteListCheck(api, true)
    });

    it("Can deploy bridge token smart contract", async () => {
        const contractFactory = new ethers.ContractFactory(wagmiContract.abi, wagmiContract.bytecode, wallet)
        const contract = await contractFactory.deploy("name",
            "symbol", wallet.address)
        await contract.waitForDeployment()
        assert.notEqual(contract.target, undefined)

        const contractAddress = contract.target.toString()

        const code = await publicClient.getCode({ address: toViemAddress(contractAddress) })
        if (code === undefined) {
            throw new Error("code not available")
        }
        assert.ok(code.length > 100)
        assert.ok(code.includes("0x60806040523480156"))
    });

    it("Can deploy bridge token contract with gas limit", async () => {
        const contractFactory = new ethers.ContractFactory(wagmiContract.abi, wagmiContract.bytecode, wallet)
        const successful_gas_limit = "12345678";
        const contract = await contractFactory.deploy("name",
            "symbol", wallet.address,
            {
                gasLimit: successful_gas_limit,
            }
        )
        await contract.waitForDeployment()
        assert.notEqual(contract.target, undefined)

        const contractAddress = contract.target.toString()

        const code = await publicClient.getCode({ address: toViemAddress(contractAddress) })
        if (code === undefined) {
            throw new Error("code not available")
        }
        assert.ok(code.length > 100)
        assert.ok(code.includes("0x60806040523480156"))
    });
});