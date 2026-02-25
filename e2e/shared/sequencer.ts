import { BaseSequencer } from "vitest/node";
import type { TestSpecification } from "vitest/node";

/**
 * Sorts test files alphabetically by their module path.
 *
 * Vitest's default sequencer orders files by cached duration/failure history,
 * which does not respect numeric prefixes (00-, 01-, 02-, ...). This sequencer
 * ensures files always run in the order they are named, which matters for
 * multi-file suites where later files depend on state set up by earlier ones
 * (e.g. a scaling test that adds nodes for subsequent edge-case tests).
 */
export default class AlphabeticalSequencer extends BaseSequencer {
  async shard(files: TestSpecification[]): Promise<TestSpecification[]> {
    return super.shard(files);
  }

  async sort(files: TestSpecification[]): Promise<TestSpecification[]> {
    return files.sort((a, b) => a.moduleId.localeCompare(b.moduleId));
  }
}
