# Subtensor Contributor Guide

## Lifecycle of a Pull Request

1. Individuals wishing to contribute to subtensor should develop their change/feature/fix in a
   [Pull Request](https://github.com/opentensor/subtensor/compare) (PR) targeting the `main`
   branch of the subtensor GitHub repository. It is recommended to start your pull request as a
   draft initially until you are ready to have other developers actively look at it.
2. Once you have finished developing your change/feature/fix and the Rust portion of the CI is
   passing for your PR (everything prefixed with "CI"), you should mark your PR as "Ready for
   Review" and request review from "Nucleus".
3. Core Nucleus team members will review your PR, possibly requesting changes, and will also
   add appropriate labels to your PR as shown below.
4. Once the required passing reviews have been obtained, you are ready to request that your PR
   be included in the next `devnet` deploy. To do this, you should open a companion PR merging
   your branch into the `devnet-ready` branch. You must include a link to the parent PR in the
   description and preface your PR title with "(Devnet Ready)" or the PR will be
   closed/ignored.
5. A core team administrator will review your "(Devnet Ready)" PR, verifying that it logically
   matches the changes introduced in the parent PR (there will sometimes be minor differences
   due to merge conflicts) and will either request changes or approve the PR and merge it. Once
   your companion PR is merged, the administrator will add the `devnet-ready` label to the
   parent PR, indicating that the PR is on the `devnet-ready` branch and will be included in
   the next deploy to `devnet`.
6. At some point, a core team administrator will open a PR merging the current `devnet-ready`
   branch into `devnet`, and the CI will enforce some additional safety checks on this PR
   including a requirement that the new `spec_version` be greater than the current on-chain
   `spec_version`. The administrator will then need to update all PRs with the `devnet-ready`
   label to instead have the `on-devnet` label (TODO: automate this upon merge). The
   administrator will then run deploy `devnet`.
7. Once the `on-devnet` label appears on your PR, if you are a core team member it is your
   responsibility to verify that the features/changes/fixes introduced by your PR are
   functioning properly on `devnet` by interacting with the live network. If you are an
   external contributor, a core team member will be assigned to test this for you.
8. If your feature/change/fix is confirmed working on `devnet`, the `devnet-pass` label should
   be added. Otherwise if there are issues, the `devnet-fail` label should be added and you
   will need to make changes to your PR and repeat the previous steps in this process. In some
   cases a revert PR will need to be created reverting your changes from the `pre-devnet` and
   `devnet` branches, respectively

## PR Labels

| Name  | Description |
| ----- | ----------- |
| `breaking-change` | PR requires synchronized changes with bittensor. Triggers an automatic bot message so the relevant teams are made aware once it is added |
| `migration` | PR contains one or more migrations |
| `devnet-ready` | PR's branch has been merged into the `devnet-ready` branch and will be included in the next `devnet` deploy |
| `on-devnet` | PR has been deployed to `devnet`. Removes `devnet-ready` |
| `devnet-pass` | PR has passed manual testing on `devnet` |
| `devnet-skip` | Allows a critical hotfix PR to skip required testing on `devnet` |
| `devnet-fail` | PR has failed manual testing on `devnet` and requires modification |
| `testnet-ready` | PR's branch has been merged into the `testnet-ready` branch and will be included in the next `testnet` deploy. Requires `devnet-pass` or `devnet-skip` to be already present |
| `on-testnet` | PR has been deployed to `testnet`. Removes `testnet-ready` |
| `testnet-pass` | PR has passed manual testing on `testnet` |
| `testnet-skip` | Allows a critical hotfix PR to skip required manual testing and SOP on `testnet` |
| `testnet-fail` | PR has failed manual testing on `testnet` and requires modification |

