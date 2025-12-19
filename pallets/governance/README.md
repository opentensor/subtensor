# On-Chain Governance System

## Abstract

This proposes a comprehensive on-chain governance system to replace the current broken governance implementation that relies on a sudo-based triumvirate multisig. The new system introduces a separation of powers model with three key components: (1) multiple proposer accounts (mostly controlled by OTF) to submit proposals (call executed with root privilege), (2) a three-member Triumvirate that votes on proposals, and (3) two collective bodies (Economic Power and Building Power) that can delay, cancel, or fast-track proposals and vote to replace Triumvirate members. The system will be deployed in two phases: first coexisting with the current sudo implementation for validation, then fully replacing it.

## Motivation

The current governance system in Subtensor is broken and relies entirely on a triumvirate multisig with sudo privileges. The runtime contains dead code related to the original triumvirate collective and senate that no longer functions properly. This centralized approach creates several critical issues:

1. **Single Point of Failure**: The sudo key represents a concentration of power with no on-chain checks or balances (i.e., no blockchain-enforced voting, approval, or oversight mechanisms).
2. **Lack of Transparency**: The governance decision-making process (who voted, when, on what proposal) happens off-chain and is not recorded or auditable on-chain. While the multisig signature itself provides cryptographic proof that the threshold was met, the governance process leading to that decision is opaque.
3. **No Stakeholder Representation**: Major stakeholders (validators and subnet owners) have no formal mechanism to influence protocol upgrades.
4. **Technical Debt**: Dead governance code in the runtime creates maintenance burden and confusion.

This proposal addresses these issues by implementing a proper separation of powers that balances efficiency with stakeholder representation, while maintaining upgrade capability and security.

## Specification

### Overview

The governance system consists of three main actors working together:

1. **Allowed Proposers**: Accounts authorized to submit proposals (mostly controlled by OTF)
2. **Triumvirate**: Approval body of 3 members that vote on proposals
3. **Economic and Building Collectives**: Oversight bodies representing major stakeholders: top 16 validators by total stake and top 16 subnet owners by moving average price respectively

### Actors and Roles

#### Allowed Proposers (mostly OTF-controlled)

- **Purpose**: Authorized to submit proposals (calls executed with root privilege)
- **Assignment**: Allowed proposer account keys are configured in the runtime via governance
- **Permissions**:
  - Can submit proposals to the main governance track (i.e., runtime upgrade proposals or any root extrinsic)
  - Can cancel or withdraw their own proposals anytime before execution (i.e., if they find a bug in the proposal code)
  - Can eject its own key from the allowed proposers list (i.e., if it is lost or compromised)
  - Can propose an update to the allowed proposers list via proposal flow

#### Triumvirate

- **Composition**: 3 distinct accounts (must always maintain 3 members)
- **Role**: Vote on proposals submitted by allowed proposers
- **Voting Threshold**: 2-of-3 approval required for proposals to pass
- **Term**: Indefinite, subject to replacement by collective vote every 6 months (configurable)
- **Accountability**: Each member can be replaced through collective vote process (see Replacement Mechanism)
- **Permissions**:
  - Can vote on proposals submitted by allowed proposers

#### Economic and Building Collectives

- **Economic Collective**: Top 16 validators by total stake (including delegated stake) (configurable)
- **Building Collective**: Top 16 subnet owners by moving average price (with minimum age of 6 months) (configurable)
- **Total Collective Size**: 32 members (16 Economic + 16 Building)
- **Recalculation**: Membership refreshed every 6 months (configurable)
- **Permissions**:
  - Can vote aye/nay on proposals submitted by allowed proposers and approved by Triumvirate
    - Votes are aggregated across both collectives (total of 32 possible votes)
    - More than configured threshold of aye votes (based on total collective size of 32) fast tracks the proposal (next block execution) (threshold configurable)
    - More than configured threshold of nay votes (based on total collective size of 32) cancels the proposal (threshold configurable)
    - Delay is calculated using net score (nays - ayes) and applies exponential delay until cancellation (see Delay Period section)

### Governance Process Flow

#### Proposal Submission

1. An allowed proposer account submits a proposal containing runtime upgrade or any root extrinsic
2. Proposal enters "Triumvirate Voting" phase
3. Voting period: 7 days (configurable), after this period, the proposal is automatically rejected if not approved by the Triumvirate.

- There is a queue limit in the number of proposals that can be submitted at the same time (configurable)
- Proposal can be cancelled by the proposer before the final execution for security reasons (e.g., if they find a bug in the proposal code).
- An allowed proposer can eject its own key from the allowed proposers, removing all its submitted proposals waiting for triumvirate approval from the queue.

#### Triumvirate Approval

1. Triumvirate members cast votes (aye/nay) on the proposal

- 2/3 vote aye, proposal is approved: Proposal is scheduled for execution in 1 hour (configurable) and enters "Delay Period"
- 2/3 vote nay, proposal is rejected: Proposal is cleaned up from storage (it was never scheduled for execution).

- Triumvirate members can change their vote during the voting period (before the proposal is scheduled or cancelled).
- There is a queue limit in the number of scheduled proposals and in the delay period (configurable).
- If a triumvirate member is replaced, all his votes are removed from the active proposals.

#### Delay Period (Collective Oversight)

When a proposal has been approved by the Triumvirate, it is scheduled in 1 hour (configurable) and enters the "Delay Period" where the Economic and Building Collectives can vote to delay, cancel or fast-track the proposal.

1. Both collectives can vote aye/nay on the proposal, with votes aggregated across all 32 collective members
2. Delay is calculated using **net score** (nays - ayes) and applies an exponential function based on a configurable delay factor.

- Initial delay is 1 hour (configurable).
- Net score = (number of nays) - (number of ayes)
- If net score > 0: additional delay = initial_delay × (delay_factor ^ net_score)
- If net score ≤ 0: no additional delay (proposal can be fast-tracked if net score becomes negative)
- **Example with delay_factor = 2**:
  - Net score of 1 (e.g., 1 nay, 0 ayes): delay = 1 hour × 2^1 = 2 hours
  - Net score of 2 (e.g., 2 nays, 0 ayes): delay = 1 hour × 2^2 = 4 hours
  - Net score of 3 (e.g., 3 nays, 0 ayes): delay = 1 hour × 2^3 = 8 hours
  - Net score of 4 (e.g., 4 nays, 0 ayes): delay = 1 hour × 2^4 = 16 hours
  - Net score of 5 (e.g., 5 nays, 0 ayes): delay = 1 hour × 2^5 = 32 hours
  - Net score of 16 (e.g., 16 nays, 0 ayes): delay = 1 hour × 2^16 = 65,536 hours
  - Net score of 17 (e.g., 17 nays, 0 ayes): proposal is cancelled (threshold configurable, typically ≥ 17 nays out of 32 total members)

3. If the delay period expires without cancellation: Proposal executes automatically

- The delay is calculated based on the **net score** across both collectives (total of 32 members), not per collective
- More than configured threshold of aye votes (based on total collective size of 32) fast tracks the proposal (next block execution) (threshold configurable)
- More than configured threshold of nay votes (based on total collective size of 32) cancels the proposal (threshold configurable, typically ≥ 17 nays)
- Collective members can change their vote during the delay period. If changing a nay vote to aye (or vice versa) changes the net score such that the delay is reduced below the time already elapsed, the proposal executes immediately.
  - **Example**: A proposal has net score of 3 (3 nays, 0 ayes), creating an 8 hour delay. After 5 hours have elapsed, a collective member changes their nay vote to aye, reducing the net score to 2 (2 nays, 1 aye) and the delay to 4 hours. Since 5 hours have already passed (more than the new 4 hours delay), the proposal executes immediately.

#### Execution

- Proposals executed automatically after the delay period if not cancelled or when fast-tracked by the collectives.
- If executing fails, the proposal is not retried and is cleaned up from storage.