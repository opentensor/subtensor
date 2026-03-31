import { setup, teardown } from "../setup.js";

const shutdown = async () => {
  await teardown();
  process.exit(0);
};

process.on("SIGINT", () => {
  void shutdown();
});

process.on("SIGTERM", () => {
  void shutdown();
});

await setup();
console.log("rate-limits localnet is up");
await new Promise(() => {});
