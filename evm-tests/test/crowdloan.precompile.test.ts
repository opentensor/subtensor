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
import { convertH160ToSS58 } from "../src/address-utils";

describe("Test Crowdloan precompile", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>

    const alice = getAliceSigner();
    const wallet1 = generateRandomEthersWallet();

    const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1)

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet1.address)
    })

    it("gets an existing crowdloan created on substrate side", async () => {
        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        await api.tx.Crowdloan.create({
            deposit: BigInt(15_000_000_000), // 15 TAO
            min_contribution: BigInt(1_000_000_000), // 1 TAO
            cap: BigInt(100_000_000_000), // 100 TAO
            end: 1000,
            target_address: undefined,
            call: api.tx.System.remark({ remark: Binary.fromText("foo") }).decodedCall
        }).signAndSubmit(alice);

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        const crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);

        assert.isDefined(crowdloan);
        assert.equal(crowdloanInfo[0], u8aToHex(decodeAddress(crowdloan.creator)));
        assert.equal(crowdloanInfo[1], crowdloan.deposit);
        assert.equal(crowdloanInfo[2], crowdloan.min_contribution);
        assert.equal(crowdloanInfo[3], crowdloan.end);
        assert.equal(crowdloanInfo[4], crowdloan.cap);
        assert.equal(crowdloanInfo[5], u8aToHex(decodeAddress(crowdloan.funds_account)));
        assert.equal(crowdloanInfo[6], crowdloan.raised);
        assert.equal(crowdloanInfo[7], false); // has_target_address
        assert.equal(crowdloanInfo[8], u8aToHex(Uint8Array.from(Array(32).fill(0)))); // target_address
        assert.equal(crowdloanInfo[9], false); // finalized
        assert.equal(crowdloanInfo[10], 1); // contributors_count
    });

    it("creates a new crowdloan and retrieves it", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(2_000_000_000); // 2 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = 1000;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        const tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(crowdloan);
        assert.equal(crowdloan.creator, convertH160ToSS58(wallet1.address));
        assert.equal(crowdloan.deposit, deposit);
        assert.equal(crowdloan.min_contribution, minContribution);
        assert.equal(crowdloan.cap, cap);
        assert.equal(crowdloan.end, end);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.target_address, convertH160ToSS58(targetAddress.address));
        assert.equal(crowdloan.finalized, false);
        assert.equal(crowdloan.contributors_count, 1);

        const crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[0], u8aToHex(decodeAddress(crowdloan.creator)));
        assert.equal(crowdloanInfo[1], crowdloan.deposit);
        assert.equal(crowdloanInfo[2], crowdloan.min_contribution);
        assert.equal(crowdloanInfo[3], crowdloan.end);
        assert.equal(crowdloanInfo[4], crowdloan.cap);
        assert.equal(crowdloanInfo[5], u8aToHex(decodeAddress(crowdloan.funds_account)));
        assert.equal(crowdloanInfo[6], crowdloan.raised);
        assert.equal(crowdloanInfo[7], true); // has_target_address
        assert.equal(crowdloanInfo[8], u8aToHex(decodeAddress(crowdloan.target_address))); // target_address
        assert.equal(crowdloanInfo[9], crowdloan.finalized);
        assert.equal(crowdloanInfo[10], crowdloan.contributors_count);
    });

    it("updates the min contribution", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = 1000;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        const tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(crowdloan);
        assert.equal(crowdloan.min_contribution, BigInt(1_000_000_000));

        const newMinContribution = BigInt(2_000_000_000);
        const tx2 = await crowdloanContract.updateMinContribution(nextId, newMinContribution);
        await tx2.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(updatedCrowdloan);
        assert.equal(updatedCrowdloan.min_contribution, newMinContribution);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[2], newMinContribution);
    });

    it("updates the end", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = 1000;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        const tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(crowdloan);
        assert.equal(crowdloan.end, 1000);

        const newEnd = 2000;
        const tx2 = await crowdloanContract.updateEnd(nextId, newEnd);
        await tx2.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(updatedCrowdloan);
        assert.equal(updatedCrowdloan.end, newEnd);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[3], newEnd);
    });

    it("updates the cap", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = 1000;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        const tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(crowdloan);
        assert.equal(crowdloan.cap, BigInt(200_000_000_000));

        const newCap = BigInt(300_000_000_000);
        const tx2 = await crowdloanContract.updateCap(nextId, newCap);
        await tx2.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.isDefined(updatedCrowdloan);
        assert.equal(updatedCrowdloan.cap, newCap);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[4], newCap);
    });
});