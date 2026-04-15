import {
  installConnectionLogFilter,
  verifyCloneConfig,
  waitForLocalRpc,
} from "./rate-limits-clone-lib.ts";

async function main() {
  installConnectionLogFilter();
  await waitForLocalRpc();
  await verifyCloneConfig();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
