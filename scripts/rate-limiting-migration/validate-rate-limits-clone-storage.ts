import {
  installConnectionLogFilter,
  verifyCloneStorageAudit,
  waitForLocalRpc,
} from "./rate-limits-clone-lib.ts";

async function main() {
  installConnectionLogFilter();
  await waitForLocalRpc();
  await verifyCloneStorageAudit();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
