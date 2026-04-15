import {
  installConnectionLogFilter,
  upgradeCloneRuntime,
  waitForLocalRpc,
} from "./rate-limits-clone-lib.ts";

async function main() {
  installConnectionLogFilter();
  await waitForLocalRpc();
  await upgradeCloneRuntime();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
