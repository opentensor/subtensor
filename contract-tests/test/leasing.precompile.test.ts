import * as assert from "assert";

import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { ILEASING_ADDRESS, ILeasingABI } from "../src/contracts/leasing";
import { INEURON_ADDRESS, INeuronABI } from "../src/contracts/neuron";
import {
  convertH160ToPublicKey,
  convertH160ToSS58,
} from "../src/address-utils";
import { generateRandomEthersWallet } from "../src/utils";
import { getDevnetApi, waitForFinalizedBlock } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";

describe("Leasing precompile E2E smoke", () => {
  let api: TypedApi<typeof devnet>;
  let wallet1: ethers.Wallet;
  let wallet2: ethers.Wallet;
  let leaseContract: ethers.Contract;
  let crowdloanContract: ethers.Contract;
  let neuronContract: ethers.Contract;

  beforeEach(async () => {
    api = await getDevnetApi();

    wallet1 = generateRandomEthersWallet();
    wallet2 = generateRandomEthersWallet();
    leaseContract = new ethers.Contract(ILEASING_ADDRESS, ILeasingABI, wallet1);
    crowdloanContract = new ethers.Contract(
      ICROWDLOAN_ADDRESS,
      ICrowdloanABI,
      wallet1,
    );
    neuronContract = new ethers.Contract(INEURON_ADDRESS, INeuronABI, wallet1);

    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);
  });

  it("creates, reads, and terminates a lease through RPC", async () => {
    const hotkey = generateRandomEthersWallet();
    let tx = await neuronContract.burnedRegister(
      1,
      convertH160ToPublicKey(hotkey.address),
    );
    await tx.wait();

    const nextCrowdloanId =
      await api.query.Crowdloan.NextCrowdloanId.getValue();
    const crowdloanDeposit = BigInt(100_000_000_000);
    const crowdloanMinContribution = BigInt(1_000_000_000);
    const crowdloanCap =
      (await api.query.SubtensorModule.NetworkLastLockCost.getValue()) *
      BigInt(2);
    const crowdloanEnd = (await api.query.System.Number.getValue()) + 100;
    const leasingEmissionsShare = 15;
    const leasingEndBlock = (await api.query.System.Number.getValue()) + 200;

    tx = await leaseContract.createLeaseCrowdloan(
      crowdloanDeposit,
      crowdloanMinContribution,
      crowdloanCap,
      crowdloanEnd,
      leasingEmissionsShare,
      true,
      leasingEndBlock,
    );
    await tx.wait();

    const crowdloanContract2 = new ethers.Contract(
      ICROWDLOAN_ADDRESS,
      ICrowdloanABI,
      wallet2,
    );
    tx = await crowdloanContract2.contribute(
      nextCrowdloanId,
      crowdloanCap - crowdloanDeposit,
    );
    await tx.wait();

    await waitForFinalizedBlock(api, crowdloanEnd);

    const nextLeaseId =
      await api.query.SubtensorModule.NextSubnetLeaseId.getValue();
    tx = await crowdloanContract.finalize(nextCrowdloanId);
    await tx.wait();

    const lease =
      await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
    assert.ok(lease);
    assert.equal(lease.beneficiary, convertH160ToSS58(wallet1.address));
    assert.equal(lease.emissions_share, leasingEmissionsShare);
    assert.equal(lease.end_block, leasingEndBlock);

    const leaseInfo = await leaseContract.getLease(nextLeaseId);
    assert.equal(leaseInfo[3], lease.emissions_share);
    assert.equal(leaseInfo[4], true);
    assert.equal(leaseInfo[5], lease.end_block);
    assert.equal(leaseInfo[6], lease.netuid);
    assert.equal(leaseInfo[7], lease.cost);

    const leaseId = await leaseContract.getLeaseIdForSubnet(lease.netuid);
    assert.equal(leaseId, nextLeaseId);

    const beneficiaryShare = await leaseContract.getContributorShare(
      nextLeaseId,
      convertH160ToPublicKey(wallet1.address),
    );
    assert.deepEqual(beneficiaryShare, [BigInt(0), BigInt(0)]);

    const contributorShare = await leaseContract.getContributorShare(
      nextLeaseId,
      convertH160ToPublicKey(wallet2.address),
    );
    assert.notDeepEqual(contributorShare, [BigInt(0), BigInt(0)]);

    await waitForFinalizedBlock(api, leasingEndBlock);

    tx = await leaseContract.terminateLease(
      nextLeaseId,
      convertH160ToPublicKey(hotkey.address),
    );
    await tx.wait();

    const terminatedLease =
      await api.query.SubtensorModule.SubnetLeases.getValue(nextLeaseId);
    assert.equal(terminatedLease, undefined);

    const ownerColdkey = await api.query.SubtensorModule.SubnetOwner.getValue(
      lease.netuid,
    );
    const ownerHotkey =
      await api.query.SubtensorModule.SubnetOwnerHotkey.getValue(lease.netuid);
    assert.equal(ownerColdkey, convertH160ToSS58(wallet1.address));
    assert.equal(ownerHotkey, convertH160ToSS58(hotkey.address));
  });
});
