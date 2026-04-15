import {
  installConnectionLogFilter,
  verifyCloneBehavior,
  waitForLocalRpc,
} from "./rate-limits-clone-lib.ts";

async function main() {
  installConnectionLogFilter();
  await waitForLocalRpc();
  const filter = process.argv[2] as
    | "serving"
    | "staking"
    | "delegate-take"
    | "weights"
    | "swap-keys"
    | "owner-hparams"
    | undefined;
  await verifyCloneBehavior(filter);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
