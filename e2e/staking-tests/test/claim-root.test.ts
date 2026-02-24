import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  getRootClaimType,
  setRootClaimType,
  log,
} from "shared";

describe("▶ set_root_claim_type extrinsic", () => {
  it("should set root claim type to Keep", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // Check initial claim type (default is "Swap")
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${claimTypeBefore}`);

    // Set root claim type to Keep
    await setRootClaimType(api, coldkey, "Keep");

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${claimTypeAfter}`);

    assert.strictEqual(claimTypeAfter, "Keep", `Expected claim type to be Keep, got ${claimTypeAfter}`);

    log.info("✅ Successfully set root claim type to Keep.");
  });

  it("should set root claim type to Swap", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // First set to Keep so we can verify the change to Swap
    await setRootClaimType(api, coldkey, "Keep");
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${claimTypeBefore}`);
    assert.strictEqual(claimTypeBefore, "Keep", "Should be Keep before changing to Swap");

    // Set root claim type to Swap
    await setRootClaimType(api, coldkey, "Swap");

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${claimTypeAfter}`);

    assert.strictEqual(claimTypeAfter, "Swap", `Expected claim type to be Swap, got ${claimTypeAfter}`);

    log.info("✅ Successfully set root claim type to Swap.");
  });

  it("should set root claim type to KeepSubnets", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // Check initial claim type (default is "Swap")
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${JSON.stringify(claimTypeBefore)}`);

    // Set root claim type to KeepSubnets with specific subnets
    const subnetsToKeep = [1, 2];
    await setRootClaimType(api, coldkey, { type: "KeepSubnets", subnets: subnetsToKeep });

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${JSON.stringify(claimTypeAfter)}`);

    assert.strictEqual(typeof claimTypeAfter, "object", "Expected claim type to be an object");
    assert.strictEqual((claimTypeAfter as { type: string }).type, "KeepSubnets", "Expected type to be KeepSubnets");
    assert.deepStrictEqual((claimTypeAfter as { subnets: number[] }).subnets, subnetsToKeep, "Expected subnets to match");

    log.info("✅ Successfully set root claim type to KeepSubnets.");
  });
});
