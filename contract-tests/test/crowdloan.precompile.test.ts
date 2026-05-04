import * as assert from "assert";

import { ethers } from "ethers";
import { Binary, TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { ICROWDLOAN_ADDRESS, ICrowdloanABI } from "../src/contracts/crowdloan";
import { convertH160ToSS58 } from "../src/address-utils";
import { generateRandomEthersWallet } from "../src/utils";
import {
  getAliceSigner,
  getDevnetApi,
  waitForFinalizedBlock,
} from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";

describe("Crowdloan precompile E2E balance smoke", () => {
  let api: TypedApi<typeof devnet>;

  const alice = getAliceSigner();
  const wallet1 = generateRandomEthersWallet();
  const wallet2 = generateRandomEthersWallet();
  const wallet3 = generateRandomEthersWallet();
  const wallet4 = generateRandomEthersWallet();

  const crowdloanContract = new ethers.Contract(
    ICROWDLOAN_ADDRESS,
    ICrowdloanABI,
    wallet1,
  );

  before(async () => {
    api = await getDevnetApi();

    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);
    await forceSetBalanceToEthAddress(api, wallet3.address);
    await forceSetBalanceToEthAddress(api, wallet4.address);
  });

  it("charges and refunds balances through create, contribute, withdraw, refund, and dissolve", async () => {
    const deposit = BigInt(20_000_000_000);
    const minContribution = BigInt(2_000_000_000);
    const cap = BigInt(100_000_000_000);
    const end = (await api.query.System.Number.getValue()) + 100;
    const targetAddress = generateRandomEthersWallet();
    const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();

    const creatorBalanceBeforeCreate = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );
    let tx = await crowdloanContract.create(
      deposit,
      minContribution,
      cap,
      end,
      targetAddress,
    );
    await tx.wait();

    const creatorBalanceAfterCreate = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );
    assert.ok(
      Number(
        creatorBalanceBeforeCreate.data.free -
          creatorBalanceAfterCreate.data.free,
      ) -
        Number(deposit) <
        1_000_000,
    );

    const contribution = BigInt(20_000_000_000);
    const crowdloanContract2 = new ethers.Contract(
      ICROWDLOAN_ADDRESS,
      ICrowdloanABI,
      wallet2,
    );
    const contributorBalanceBefore = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet2.address),
    );
    tx = await crowdloanContract2.contribute(nextId, contribution);
    await tx.wait();

    let contributorBalanceAfter = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet2.address),
    );
    assert.ok(
      Number(
        contributorBalanceBefore.data.free - contributorBalanceAfter.data.free,
      ) -
        Number(contribution) <
        1_000_000,
    );

    tx = await crowdloanContract2.withdraw(nextId);
    await tx.wait();

    contributorBalanceAfter = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet2.address),
    );
    assert.ok(
      Number(
        contributorBalanceBefore.data.free - contributorBalanceAfter.data.free,
      ) < 1_000_000,
    );

    const crowdloanContract3 = new ethers.Contract(
      ICROWDLOAN_ADDRESS,
      ICrowdloanABI,
      wallet3,
    );
    const crowdloanContract4 = new ethers.Contract(
      ICROWDLOAN_ADDRESS,
      ICrowdloanABI,
      wallet4,
    );
    const refundBalanceBefore3 = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet3.address),
    );
    const refundBalanceBefore4 = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet4.address),
    );
    tx = await crowdloanContract3.contribute(nextId, contribution);
    await tx.wait();
    tx = await crowdloanContract4.contribute(nextId, contribution);
    await tx.wait();

    await waitForFinalizedBlock(api, end);

    tx = await crowdloanContract.refund(nextId);
    await tx.wait();

    const refundBalanceAfter3 = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet3.address),
    );
    const refundBalanceAfter4 = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet4.address),
    );
    assert.ok(
      Number(refundBalanceBefore3.data.free - refundBalanceAfter3.data.free) <
        1_000_000,
    );
    assert.ok(
      Number(refundBalanceBefore4.data.free - refundBalanceAfter4.data.free) <
        1_000_000,
    );

    tx = await crowdloanContract.dissolve(nextId);
    await tx.wait();

    const creatorBalanceAfterDissolve = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );
    assert.ok(
      Number(
        creatorBalanceBeforeCreate.data.free -
          creatorBalanceAfterDissolve.data.free,
      ) < 2_000_000,
    );
  });

  it("contributes and withdraws against a crowdloan created on substrate side", async () => {
    const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
    const deposit = BigInt(15_000_000_000);
    const end = (await api.query.System.Number.getValue()) + 100;

    await api.tx.Crowdloan.create({
      deposit,
      min_contribution: BigInt(1_000_000_000),
      cap: BigInt(100_000_000_000),
      end,
      target_address: undefined,
      call: api.tx.System.remark({ remark: Binary.fromText("foo") })
        .decodedCall,
    }).signAndSubmit(alice);

    const balanceBefore = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );

    const contribution = BigInt(5_000_000_000);
    const tx = await crowdloanContract.contribute(nextId, contribution);
    await tx.wait();

    let balanceAfter = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );
    assert.ok(
      Number(balanceBefore.data.free - balanceAfter.data.free) -
        Number(contribution) <
        1_000_000,
    );

    const tx2 = await crowdloanContract.withdraw(nextId);
    await tx2.wait();

    balanceAfter = await api.query.System.Account.getValue(
      convertH160ToSS58(wallet1.address),
    );
    assert.ok(
      Number(balanceBefore.data.free - balanceAfter.data.free) < 1_000_000,
    );
  });
});
