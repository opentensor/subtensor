import { destroyClient, getDevnetApi, sudoSetLockReductionInterval, log } from "shared";

before(async () => {
  const api = await getDevnetApi();
  log.info("Setup: set lock reduction interval to 1 for instant lock cost decay");

  // Set lock reduction interval to 1 block to make network registration lock cost decay instantly.
  // By default, the lock cost doubles with each subnet registration and decays over 14 days (100,800 blocks).
  // Without this, tests creating multiple subnets would fail with CannotAffordLockCost.
  await sudoSetLockReductionInterval(api, 1);
});

after(() => {
  destroyClient();
});
