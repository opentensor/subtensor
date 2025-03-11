

import * as assert from "assert";
import * as chai from "chai";

import { getDevnetApi } from "../src/substrate"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { INCREMENTAL_CONTRACT_ABI, INCREMENTAL_CONTRACT_BYTECODE } from "../src/contracts/incremental";
import { toViemAddress } from "../src/address-utils";
import { ethers } from "ethers"
import { disableWhiteListCheck, forceSetBalanceToEthAddress } from "../src/subtensor";

describe("bridge token contract deployment", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();
    let publicClient: PublicClient;

    // init substrate part
    let api: TypedApi<typeof devnet>

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet.address)
        await disableWhiteListCheck(api, true)
    });

    it("Can deploy incremental smart contract", async () => {
        const contractFactory = new ethers.ContractFactory(INCREMENTAL_CONTRACT_ABI, INCREMENTAL_CONTRACT_BYTECODE, wallet)
        const contract = await contractFactory.deploy()
        await contract.waitForDeployment()

        const value = await publicClient.readContract({
            abi: INCREMENTAL_CONTRACT_ABI,
            address: toViemAddress(contract.target.toString()),
            functionName: "retrieve",
            args: []
        })
        assert.equal(value, 0)

        const newValue = 1234

        const deployContract = new ethers.Contract(contract.target.toString(), INCREMENTAL_CONTRACT_ABI, wallet)
        const storeTx = await deployContract.store(newValue)
        await storeTx.wait()

        const newValueAfterStore = await publicClient.readContract({
            abi: INCREMENTAL_CONTRACT_ABI,
            address: toViemAddress(contract.target.toString()),
            functionName: "retrieve",
            args: []
        })

        assert.equal(newValue, newValueAfterStore)
    });
});
