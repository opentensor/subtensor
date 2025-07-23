import { PublicClient } from "viem";
import { ETH_LOCAL_URL } from "../src/config";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { getAliceSigner, getBobSigner, getDevnetApi, waitForFinalizedBlock } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";
import { decodeAddress } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";
import { ILEASING_ADDRESS, ILeasingABI } from "../src/contracts/leasing";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { assert } from "chai";
import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils";

describe("Test Leasing precompile", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>;

    const alice = getAliceSigner();
    const bob = getBobSigner();
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();

    const crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1);
    const leaseContract = new ethers.Contract(ILEASING_ADDRESS, ILeasingABI, wallet1);

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL);
        api = await getDevnetApi();

        await forceSetBalanceToEthAddress(api, wallet1.address);
        await forceSetBalanceToEthAddress(api, wallet2.address);
    });

    it("gets an existing lease created on substrate side", async () => {
        const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const crowdloanDeposit = BigInt(100_000_000_000); // 100 TAO
        const crowdloanCap = BigInt(2_000_000_000_000); // 2000 TAO
        const crowdloanEnd = await api.query.System.Number.getValue() + 100;
        const leaseEmissionsShare = 15;
        const leaseEnd = await api.query.System.Number.getValue() + 5000;

        await api.tx.Crowdloan.create({
            deposit: crowdloanDeposit,
            min_contribution: BigInt(1_000_000_000), // 1 TAO
            cap: crowdloanCap,
            end: crowdloanEnd,
            target_address: undefined,
            call: api.tx.SubtensorModule.register_leased_network({
                emissions_share: leaseEmissionsShare,
                end_block: leaseEnd,
            }).decodedCall
        }).signAndSubmit(alice);

        await api.tx.Crowdloan.contribute({
            crowdloan_id: nextCrowdloanId,
            amount: crowdloanCap - crowdloanDeposit,
        }).signAndSubmit(bob);

        await waitForFinalizedBlock(api, crowdloanEnd);

        const nextLeaseId = await api.query.SubtensorModule.NextSubnetLeaseId.getValue();
        await api.tx.Crowdloan.finalize({ crowdloan_id: nextCrowdloanId }).signAndSubmit(alice);

        const lease = await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
        const leaseInfo = await leaseContract.getLease(nextLeaseId);

        assert.isDefined(lease);
        assert.equal(leaseInfo[0], u8aToHex(decodeAddress(lease.beneficiary)));
        assert.equal(leaseInfo[1], u8aToHex(decodeAddress(lease.coldkey)));
        assert.equal(leaseInfo[2], u8aToHex(decodeAddress(lease.hotkey)));
        assert.equal(leaseInfo[3], lease.emissions_share);
        assert.equal(leaseInfo[4], true); //has_end_block
        assert.equal(leaseInfo[5], lease.end_block);
        assert.equal(leaseInfo[6], lease.netuid);
        assert.equal(leaseInfo[7], lease.cost);
    })

    it("registers a new leased network through a crowdloan and retrieves the lease", async () => {
        const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const crowdloanDeposit = BigInt(100_000_000_000); // 100 TAO
        const crowdloanMinContribution = BigInt(1_000_000_000); // 1 TAO
        const crowdloanCap = BigInt(2_000_000_000_000); // 2000 TAO
        const crowdloanEnd = await api.query.System.Number.getValue() + 100;
        const leasingEmissionsShare = 15;
        const leasingEndBlock = await api.query.System.Number.getValue() + 5000;

        let tx = await leaseContract.createLeaseCrowdloan(
            crowdloanDeposit,
            crowdloanMinContribution,
            crowdloanCap,
            crowdloanEnd,
            leasingEmissionsShare,
            true, // has_leasing_end_block
            leasingEndBlock
        );
        await tx.wait();

        const crowdloanContract2 = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet2);
        tx = await crowdloanContract2.contribute(nextCrowdloanId, crowdloanCap - crowdloanDeposit);
        await tx.wait();

        await waitForFinalizedBlock(api, crowdloanEnd);

        const nextLeaseId = await api.query.SubtensorModule.NextSubnetLeaseId.getValue();
        tx = await crowdloanContract.finalize(nextCrowdloanId);
        await tx.wait();

        const lease = await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
        assert.isDefined(lease);
        assert.equal(lease.beneficiary, convertH160ToSS58(wallet1.address));
        assert.equal(lease.emissions_share, leasingEmissionsShare);
        assert.equal(lease.end_block, leasingEndBlock);

        const leaseInfo = await leaseContract.getLease(nextLeaseId);
        assert.equal(leaseInfo[0], u8aToHex(decodeAddress(lease.beneficiary)));
        assert.equal(leaseInfo[1], u8aToHex(decodeAddress(lease.coldkey)));
        assert.equal(leaseInfo[2], u8aToHex(decodeAddress(lease.hotkey)));
        assert.equal(leaseInfo[3], lease.emissions_share);
        assert.equal(leaseInfo[4], true); // has_end_block
        assert.equal(leaseInfo[5], lease.end_block);
        assert.equal(leaseInfo[6], lease.netuid);
        assert.equal(leaseInfo[7], lease.cost);
    });
})