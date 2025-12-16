import * as assert from "assert";

import { PublicClient } from "viem";
import { ETH_LOCAL_URL } from "../src/config";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ethers } from "ethers";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { Binary, TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { getAliceSigner, getDevnetApi, waitForFinalizedBlock } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";
import { decodeAddress } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";
import { convertH160ToSS58 } from "../src/address-utils";

describe("Test Crowdloan precompile", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>

    const alice = getAliceSigner();
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();
    const wallet4 = generateRandomEthersWallet();

    const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1);

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet1.address)
        await forceSetBalanceToEthAddress(api, wallet2.address)
        await forceSetBalanceToEthAddress(api, wallet3.address)
        await forceSetBalanceToEthAddress(api, wallet4.address)
    })

    it("gets an existing crowdloan created on substrate side", async () => {
        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const end = await api.query.System.Number.getValue() + 100;

        await api.tx.Crowdloan.create({
            deposit: BigInt(15_000_000_000), // 15 TAO
            min_contribution: BigInt(1_000_000_000), // 1 TAO
            cap: BigInt(100_000_000_000), // 100 TAO
            end,
            target_address: undefined,
            call: api.tx.System.remark({ remark: Binary.fromText("foo") }).decodedCall
        }).signAndSubmit(alice);

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        const crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);

        assert.ok(crowdloan);
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
        const end = await api.query.System.Number.getValue() + 100;
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
        assert.ok(crowdloan);
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

    it("contributes/withdraws to a crowdloan created on substrate side", async () => {
        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const deposit = BigInt(15_000_000_000); // 15 TAO
        const end = await api.query.System.Number.getValue() + 100;

        await api.tx.Crowdloan.create({
            deposit,
            min_contribution: BigInt(1_000_000_000), // 1 TAO
            cap: BigInt(100_000_000_000), // 100 TAO
            end,
            target_address: undefined,
            call: api.tx.System.remark({ remark: Binary.fromText("foo") }).decodedCall
        }).signAndSubmit(alice);

        let crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.contributors_count, 1);

        let crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit);
        assert.equal(crowdloanInfo[10], 1);

        let balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));

        const contribution = BigInt(5_000_000_000);
        const tx = await crowdloanContract.contribute(nextId, contribution);
        await tx.wait();

        let balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));
        assert.ok(Number(balanceBefore.data.free - balanceAfter.data.free) - Number(contribution) < 1_000_000);

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit + contribution);
        assert.equal(crowdloan.contributors_count, 2);

        crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit + contribution);
        assert.equal(crowdloanInfo[10], 2);

        balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));

        const tx2 = await crowdloanContract.withdraw(nextId);
        await tx2.wait();

        balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));
        assert.ok(Number(balanceAfter.data.free) - Number(balanceBefore.data.free + contribution) < 1_000_000);

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.contributors_count, 1);

        crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit);
        assert.equal(crowdloanInfo[10], 1);
    });

    it("contributes/withdraws to a crowdloan", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(2_000_000_000); // 2 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = await api.query.System.Number.getValue() + 100;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        let balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));

        let tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        let balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));
        assert.ok(Number(balanceBefore.data.free - balanceAfter.data.free) - Number(deposit) < 1_000_000);

        let crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.contributors_count, 1);

        let crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit);
        assert.equal(crowdloanInfo[10], 1);

        balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));

        const contribution = BigInt(3_000_000_000);
        const crowdloanContract2 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet2);
        tx = await crowdloanContract2.contribute(nextId, contribution);
        await tx.wait();

        balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));
        assert.ok(Number(balanceBefore.data.free - balanceAfter.data.free) - Number(contribution) < 1_000_000);

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit + contribution);
        assert.equal(crowdloan.contributors_count, 2);

        crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit + contribution);
        assert.equal(crowdloanInfo[10], 2);

        balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));

        const tx2 = await crowdloanContract2.withdraw(nextId);
        await tx2.wait();

        balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));
        assert.ok(Number(balanceAfter.data.free) - Number(balanceBefore.data.free + contribution) < 1_000_000);

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.contributors_count, 1);

        crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit);
        assert.equal(crowdloanInfo[10], 1);
    });

    it("finalizes a crowdloan", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(2_000_000_000); // 2 TAO
        const cap = BigInt(100_000_000_000); // 200 TAO
        const end = await api.query.System.Number.getValue() + 100;
        const targetAddress = generateRandomEthersWallet();

        const balanceBefore = await api.query.System.Account.getValue(convertH160ToSS58(targetAddress.address));
        assert.equal(balanceBefore.data.free, BigInt(0));

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        let tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait()

        const contribution = cap - deposit;
        const crowdloanContract2 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet2);
        tx = await crowdloanContract2.contribute(nextId, contribution);
        await tx.wait();

        await waitForFinalizedBlock(api, end);

        tx = await crowdloanContract.finalize(nextId);
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.finalized, true);

        const crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[9], true);

        const balanceAfter = await api.query.System.Account.getValue(convertH160ToSS58(targetAddress.address));
        assert.equal(balanceAfter.data.free, cap);
    });

    it("refunds/dissolves a crowdloan", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(2_000_000_000); // 2 TAO
        const cap = BigInt(100_000_000_000); // 100 TAO
        const end = await api.query.System.Number.getValue() + 100;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        const balanceBefore1 = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));
        let tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait()

        const balanceBefore2 = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));
        const contribution = BigInt(20_000_000_000); // 20 TAO
        const crowdloanContract2 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet2);
        tx = await crowdloanContract2.contribute(nextId, contribution);
        await tx.wait();

        const balanceBefore3 = await api.query.System.Account.getValue(convertH160ToSS58(wallet3.address));
        const crowdloanContract3 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet3);
        tx = await crowdloanContract3.contribute(nextId, contribution);
        await tx.wait();

        const balanceBefore4 = await api.query.System.Account.getValue(convertH160ToSS58(wallet4.address));
        const crowdloanContract4 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet4);
        tx = await crowdloanContract4.contribute(nextId, contribution);
        await tx.wait();

        await waitForFinalizedBlock(api, end);

        let crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit + contribution * BigInt(3));
        assert.equal(crowdloan.contributors_count, 4);

        let crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit + contribution * BigInt(3));
        assert.equal(crowdloanInfo[10], 4);

        tx = await crowdloanContract.refund(nextId);
        await tx.wait();

        const balanceAfter2 = await api.query.System.Account.getValue(convertH160ToSS58(wallet2.address));
        assert.ok(Number(balanceAfter2.data.free) - Number(balanceBefore2.data.free) < 1_000_000);
        const balanceAfter3 = await api.query.System.Account.getValue(convertH160ToSS58(wallet3.address));
        assert.ok(Number(balanceAfter3.data.free) - Number(balanceBefore3.data.free) < 1_000_000);
        const balanceAfter4 = await api.query.System.Account.getValue(convertH160ToSS58(wallet4.address));
        assert.ok(Number(balanceAfter4.data.free) - Number(balanceBefore4.data.free) < 1_000_000);

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.raised, deposit);
        assert.equal(crowdloan.contributors_count, 1);

        crowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(crowdloanInfo[6], deposit);
        assert.equal(crowdloanInfo[10], 1);

        tx = await crowdloanContract.dissolve(nextId);
        await tx.wait();

        crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.equal(crowdloan, undefined);

        const balanceAfter1 = await api.query.System.Account.getValue(convertH160ToSS58(wallet1.address));
        assert.ok(Number(balanceAfter1.data.free) - Number(balanceBefore1.data.free) < 2_000_000);
    });

    it("updates the min contribution", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = await api.query.System.Number.getValue() + 100;
        const targetAddress = generateRandomEthersWallet();

        const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

        let tx = await crowdloanContract.create(
            deposit,
            minContribution,
            cap,
            end,
            targetAddress
        );
        await tx.wait();

        const crowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(crowdloan);
        assert.equal(crowdloan.min_contribution, BigInt(1_000_000_000));

        const newMinContribution = BigInt(2_000_000_000);
        tx = await crowdloanContract.updateMinContribution(nextId, newMinContribution);
        await tx.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(updatedCrowdloan);
        assert.equal(updatedCrowdloan.min_contribution, newMinContribution);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[2], newMinContribution);
    });

    it("updates the end", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = await api.query.System.Number.getValue() + 100;
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
        assert.ok(crowdloan);
        assert.equal(crowdloan.end, end);

        const newEnd = end + 200;
        const tx2 = await crowdloanContract.updateEnd(nextId, newEnd);
        await tx2.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(updatedCrowdloan);
        assert.equal(updatedCrowdloan.end, newEnd);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[3], newEnd);
    });

    it("updates the cap", async () => {
        const deposit = BigInt(20_000_000_000); // 20 TAO
        const minContribution = BigInt(1_000_000_000); // 1 TAO
        const cap = BigInt(200_000_000_000); // 200 TAO
        const end = await api.query.System.Number.getValue() + 100;
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
        assert.ok(crowdloan);
        assert.equal(crowdloan.cap, BigInt(200_000_000_000));

        const newCap = BigInt(300_000_000_000);
        const tx2 = await crowdloanContract.updateCap(nextId, newCap);
        await tx2.wait();

        const updatedCrowdloan = await api.query.Crowdloan.Crowdloans.getValue(nextId);
        assert.ok(updatedCrowdloan);
        assert.equal(updatedCrowdloan.cap, newCap);

        const updatedCrowdloanInfo = await crowdloanContract.getCrowdloan(nextId);
        assert.equal(updatedCrowdloanInfo[4], newCap);
    });
});