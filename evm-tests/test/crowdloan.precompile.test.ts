import { PublicClient } from "viem";
import { ETH_LOCAL_URL } from "../src/config";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ethers } from "ethers";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { getDevnetApi } from "../src/substrate";
import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address } from "../src/subtensor";
import { convertH160ToSS58, convertPublicKeyToSs58, ss58ToEthAddress } from "../src/address-utils";
import { assert } from "chai";

describe("Test Crowdloan precompile", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>

    const wallet1 = generateRandomEthersWallet();

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet1.address)
    })

    it("can create a crowdloan", async () => {
        const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1)

        const deposit = 15_000_000_000; // 15 TAO
        const minContribution = 1_000_000_000; // 1 TAO
        const cap = 20_000_000_000; // 20 TAO
        const currentBlock = await publicClient.getBlockNumber();
        const end = Number(currentBlock) + 200;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const tx = await crowdloanContract.create(deposit, minContribution, cap, end, targetAddress.address);
        await tx.wait();

        const crowdloanId = nextId;
        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(crowdloanId);
        
        console.log(crowdloan);

        assert.isDefined(crowdloan);
        assert.equal(crowdloan.creator, convertH160ToSS58(wallet1.address));
        assert.equal(crowdloan.deposit, BigInt(deposit));
        assert.equal(crowdloan.min_contribution, BigInt(minContribution));
        assert.equal(crowdloan.cap, BigInt(cap));
        assert.equal(crowdloan.end, end);
        assert.equal(crowdloan.target_address, convertH160ToSS58(targetAddress.address));
        console.log(crowdloan.target_address)
        console.log(convertH160ToSS58(targetAddress.address))

        // console.log("creator h160");
        // console.log(wallet1.address);
        // console.log("creator ss58");
        // console.log(crowdloan.creator);
    })
});