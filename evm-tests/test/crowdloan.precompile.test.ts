import { PublicClient } from "viem";
import { ETH_LOCAL_URL } from "../src/config";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ethers } from "ethers";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { Binary, TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { getAliceSigner, getDevnetApi } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";
import { decodeAddress } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";
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

    it("gets an existing crowdloan", async () => {
        const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1)
        const alice = getAliceSigner();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        await api.tx.Crowdloan.create({
            deposit: BigInt(15_000_000_000),
            min_contribution: BigInt(1_000_000_000),
            cap: BigInt(20_000_000_000),
            end: 1000,
            target_address: undefined,
            call: api.tx.System.remark({ remark: Binary.fromText("foo") }).decodedCall
        }).signAndSubmit(alice);

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        const crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);

        assert.equal(crowdloanInfo[0], u8aToHex(decodeAddress(crowdloan?.creator)));
        assert.equal(crowdloanInfo[1], crowdloan?.deposit);
        assert.equal(crowdloanInfo[2], crowdloan?.min_contribution);
        assert.equal(crowdloanInfo[3], crowdloan?.end);
        assert.equal(crowdloanInfo[4], crowdloan?.cap);
        assert.equal(crowdloanInfo[5], u8aToHex(decodeAddress(crowdloan?.funds_account)));
        assert.equal(crowdloanInfo[6], crowdloan?.raised);
        assert.equal(crowdloanInfo[7], false); // has_target_address
        assert.equal(crowdloanInfo[8], u8aToHex(Uint8Array.from(Array(32).fill(0)))); // target_address
        assert.equal(crowdloanInfo[9], false); // finalized
        assert.equal(crowdloanInfo[10], 1); // contributors_count
    });


    // test("creates a new crowdloan", async () => {
    //     const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1)

    //     const deposit = 15_000_000_000; // 15 TAO
    //     const minContribution = 1_000_000_000; // 1 TAO
    //     const cap = 20_000_000_000; // 20 TAO
    //     const currentBlock = await publicClient.getBlockNumber();
    //     const end = Number(currentBlock) + 200;
    //     const targetAddress = generateRandomEthersWallet();

    //     const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
    //     const tx = await crowdloanContract.create(deposit, minContribution, cap, end, targetAddress.address);
    //     await tx.wait();

    //     const crowdloanId = nextId;
    //     const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(crowdloanId);

    //     assert.isDefined(crowdloan);
    //     assert.equal(crowdloan.creator, convertH160ToSS58(wallet1.address));
    //     assert.equal(crowdloan.deposit, BigInt(deposit));
    //     assert.equal(crowdloan.min_contribution, BigInt(minContribution));
    //     assert.equal(crowdloan.cap, BigInt(cap));
    //     assert.equal(crowdloan.end, end);
    //     assert.equal(crowdloan.target_address, convertH160ToSS58(targetAddress.address));
    // })
});