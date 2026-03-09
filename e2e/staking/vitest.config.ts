import { defineConfig } from "vitest/config";
import { BaseSequencer, type TestSpecification } from "vitest/node";

class AlphabeticalSequencer extends BaseSequencer {
  async sort(files: TestSpecification[]): Promise<TestSpecification[]> {
    return files.sort((a, b) => a.moduleId.localeCompare(b.moduleId));
  }
}

export default defineConfig({
  test: {
    globals: true,
    testTimeout: 120_000,
    hookTimeout: 300_000,
    fileParallelism: false,
    globalSetup: "./setup.ts",
    include: ["test/**/*.test.ts"],
    sequence: {
      sequencer: AlphabeticalSequencer,
    },
  },
});
