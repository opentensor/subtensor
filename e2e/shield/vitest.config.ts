import { defineConfig } from "vitest/config";
import AlphabeticalSequencer from "e2e-shared/sequencer.js";

export default defineConfig({
  test: {
    globals: true,
    testTimeout: 120_000,
    hookTimeout: 300_000,
    fileParallelism: false,
    globalSetup: "./setup.ts",
    include: ["tests/**/*.test.ts"],
    sequence: {
      sequencer: AlphabeticalSequencer,
    },
  },
});
