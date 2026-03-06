import * as assert from "assert";

import { PublicClient } from "viem";
import { ETH_LOCAL_URL } from "../src/config";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { getAliceSigner, getBobSigner, getDevnetApi, waitForFinalizedBlock } from "../src/substrate";
import { forceSetBalanceToEthAddress, resetNetworkLastLockCost } from "../src/subtensor";
import { decodeAddress } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";
import { ILEASING_ADDRESS, ILeasingABI } from "../src/contracts/leasing";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron";
import { convertH160ToPublicKey, convertH160ToSS58 } from "../src/address-utils";

describe("Test Leasing precompile", () => {
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>;

    let wallet1: ethers.Wallet;
    let wallet2: ethers.Wallet;
    let leaseContract: ethers.Contract;
    let crowdloanContract: ethers.Contract;
    let neuronContract: ethers.Contract;
    const crowdloanDeposit = BigInt(100_000_000_000);
    let crowdloanCap: bigint;

    const alice = getAliceSigner();
    const bob = getBobSigner();

    beforeEach(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL);
        api = await getDevnetApi();

        wallet1 = generateRandomEthersWallet();
        wallet2 = generateRandomEthersWallet();
        leaseContract = new ethers.Contract(ILEASING_ADDRESS, ILeasingABI, wallet1);
        crowdloanContract = new ethers.Contract(ICROWDLOAN_ADDRESS, ICrowdloanABI, wallet1);
        neuronContract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet1);

        await forceSetBalanceToEthAddress(api, wallet1.address);
        await forceSetBalanceToEthAddress(api, wallet2.address);

        await resetNetworkLastLockCost(api);
        const minLockCost = await api.query.SubtensorModule.NetworkMinLockCost.getValue();
        // guarantee that the crowdloan cap is larger than the deposit
        if (minLockCost > crowdloanDeposit) {
            crowdloanCap = minLockCost * BigInt(2);
        } else {
            crowdloanCap = crowdloanDeposit * BigInt(2);
        }
    });

    it("gets an existing lease created on substrate side, its subnet id and its contributor shares", async () => {
        const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const crowdloanEnd = await api.query.System.Number.getValue() + 100;
        const leaseEmissionsShare = 15;
        const leaseEnd = await api.query.System.Number.getValue() + 300;

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

        assert.ok(lease);
        assert.equal(leaseInfo[0], u8aToHex(decodeAddress(lease.beneficiary)));
        assert.equal(leaseInfo[1], u8aToHex(decodeAddress(lease.coldkey)));
        assert.equal(leaseInfo[2], u8aToHex(decodeAddress(lease.hotkey)));
        assert.equal(leaseInfo[3], lease.emissions_share);
        assert.equal(leaseInfo[4], true); //has_end_block
        assert.equal(leaseInfo[5], lease.end_block);
        assert.equal(leaseInfo[6], lease.netuid);
        assert.equal(leaseInfo[7], lease.cost);

        const leaseId = await leaseContract.getLeaseIdForSubnet(lease.netuid);
        assert.equal(leaseId, nextLeaseId);

        // Bob has some share and alice share is 0 because she is the beneficiary
        // and beneficiary share is dynamic based on other contributors shares
        const aliceShare = await leaseContract.getContributorShare(nextLeaseId, alice.publicKey)
        assert.deepEqual(aliceShare, [BigInt(0), BigInt(0)]);
        const bobShare = await leaseContract.getContributorShare(nextLeaseId, bob.publicKey)
        assert.notDeepEqual(bobShare, [BigInt(0), BigInt(0)]);
    });

    it("registers a new leased network through a crowdloan and retrieves the lease", async () => {
        const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const crowdloanMinContribution = BigInt(1_000_000_000); // 1 TAO
        const crowdloanEnd = await api.query.System.Number.getValue() + 100;
        const leasingEmissionsShare = 15;
        const leasingEndBlock = await api.query.System.Number.getValue() + 300;

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
        assert.ok(lease);
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

        const leaseId = await leaseContract.getLeaseIdForSubnet(lease.netuid);
        assert.equal(leaseId, nextLeaseId);

        // Bob has some share and alice share is 0 because she is the beneficiary
        // and beneficiary share is dynamic based on other contributors shares
        const contributor1 = await leaseContract.getContributorShare(nextLeaseId, convertH160ToPublicKey(wallet1.address))
        assert.deepEqual(contributor1, [BigInt(0), BigInt(0)]);
        const contributor2 = await leaseContract.getContributorShare(nextLeaseId, convertH160ToPublicKey(wallet2.address))
        assert.notDeepEqual(contributor2, [BigInt(0), BigInt(0)]);
    });

    it("terminates a lease", async () => {
        const hotkey = generateRandomEthersWallet();
        let tx = await neuronContract.burnedRegister(1, convertH160ToPublicKey(hotkey.address));
        await tx.wait();

        const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
        const crowdloanMinContribution = BigInt(1_000_000_000); // 1 TAO
        const crowdloanEnd = await api.query.System.Number.getValue() + 100;
        const leasingEmissionsShare = 15;
        const leasingEndBlock = await api.query.System.Number.getValue() + 200;

        tx = await leaseContract.createLeaseCrowdloan(
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

        await waitForFinalizedBlock(api, leasingEndBlock);

        let lease = await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
        assert.ok(lease);
        const netuid = lease.netuid;

        tx = await leaseContract.terminateLease(nextLeaseId, convertH160ToPublicKey(hotkey.address));
        await tx.wait();

        lease = await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
        assert.strictEqual(lease, undefined);

        // Ensure that the subnet ownership has been transferred
        const ownerColdkey = await api.query.SubtensorModule.SubnetOwner.getValue(netuid);
        const ownerHotkey = await api.query.SubtensorModule.SubnetOwnerHotkey.getValue(netuid);
        assert.equal(ownerColdkey, convertH160ToSS58(wallet1.address));
        assert.equal(ownerHotkey, convertH160ToSS58(hotkey.address));
    });
})