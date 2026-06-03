import { installConnectionLogFilter, prepareCloneSpec } from "./rate-limits-clone-lib.ts";

async function main() {
  installConnectionLogFilter();
  await prepareCloneSpec();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
