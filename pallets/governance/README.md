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

**Open Questions:**
- Q1: Who can add/remove proposer accounts? Only governance or should Triumvirate have emergency powers?
- Q2: Who validates that proposal code matches stated intent before Triumvirate votes? Share runtime WASM hash like Polkadot fellowship does?

#### Triumvirate

- **Composition**: 3 distinct accounts (must always maintain 3 members)
- **Role**: Vote on proposals submitted by allowed proposers
- **Voting Threshold**: 2-of-3 approval required for proposals to pass
- **Term**: Indefinite, subject to replacement by collective vote every 6 months (configurable)
- **Accountability**: Each member can be replaced through collective vote process (see Replacement Mechanism)
- **Permissions**:
  - Can vote on proposals submitted by allowed proposers

**Open Questions:**
  - Q3: How to allow a triumvirate member to resign?

#### Economic and Building Collectives

- **Economic Collective**: Top 16 validators by total stake (including delegated stake) (configurable)
- **Building Collective**: Top 16 subnet owners by moving average price (with minimum age of 6 months) (configurable)
- **Recalculation**: Membership refreshed every 6 months (configurable)
- **Permissions**:
  - Can vote aye/nay on proposals submitted by allowed proposers and approved by Triumvirate
    - More than 2/3 of aye vote for any collective fast tracks the proposal (next block execution) (threshold configurable)
    - More than 1/2 of nay vote for any collective cancels the proposal (threshold configurable)
    - Nays votes accumulate and delay the proposal execution exponentially until cancellation (see Delay Period section)
  - Can replace a Triumvirate member every 6 months via single atomic vote (remove current holder + install replacement candidate, with rotating seat selection)
  - Can mark himself as eligible for nomination to the Triumvirate
  - Can accept a nomination to the Triumvirate
  
**Open Questions:**
- Q4: How to handle the nomination process?
- Q5: How to incentivize the collective members to vote?

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
  - 2/3 vote aye, proposal is approved: Proposal is scheduled for execution in 7 days (configurable) and enters "Delay Period"
  - 2/3 vote nay, proposal is rejected: Proposal is cleaned up from storage (it was never scheduled for execution).

- Triumvirate members can change their vote during the voting period (before the proposal is scheduled or cancelled).
- There is a queue limit in the number of scheduled proposals and in the delay period (configurable).
- If a triumvirate member is replaced, all his votes are removed from the active proposals.

#### Delay Period (Collective Oversight)

When a proposal has been approved by the Triumvirate, it is scheduled in 7 days (configurable) and enters the "Delay Period" where the Economic and Building Collectives can vote to delay, cancel or fast-track the proposal.

1. Both collectives can vote aye/nay on the proposal
2. Delay is an exponential function of the number of nays votes, set to 1.5^n (configurable).
  - Initial delay is 7 days (configurable).
  - After 1 nays vote, the delay is 1.5^1 * 7 days = 10.5 days.
  - After 2 nays votes, the delay is 1.5^2 * 7 days = ~16 days.
  - After 3 nays votes, the delay is 1.5^3 * 7 days = ~23 days.
  - After 4 nays votes, the delay is 1.5^4 * 7 days = ~35 days.
  - After 5 nays votes, the delay is 1.5^5 * 7 days = ~53 days.
  - After 6 nays votes, the delay is 1.5^6 * 7 days = ~80 days.
  - After 7 nays votes, the delay is 1.5^7 * 7 days = ~120 days.
  - After 8 nays votes, the delay is 1.5^8 * 7 days = ~180 days.
  - After 9 nays votes, proposal is cancelled (given we have a collective size of 16, hence more than 1/2 of the collective votes nay).
3. If the delay period expires without cancellation: Proposal executes automatically

- The delay is calculated based on the collective with the most nays votes (i.e., if Economic has 3 nays and Building has 1 nay, the delay is based on 3 nays = ~23 days).
- More than 2/3 of aye vote for any collective fast tracks the proposal (next block execution) (threshold configurable)
- More than 1/2 of nay vote for any collective cancels the proposal (threshold configurable)
- Collective members can change their vote during the delay period. If changing a nay vote to aye reduces the delay below the time already elapsed, the proposal executes immediately.
  - **Example**: A proposal has 3 nays votes, creating a 23-day delay. After 17 days have elapsed, a collective member changes their nay vote to aye, reducing the delay to 16 days. Since 17 days have already passed (more than the new 16-day delay), the proposal executes immediately.

**Open Questions:**
- Q6: Should the voting be across both collectives or each collective votes independently? What if a collective decide to go rogue and fast track proposals that the other collective is against or vice versa?

#### Execution

- Proposals executed automatically after the delay period if not cancelled or when fast-tracked by the collectives.
- If executing fails, the proposal is not retried and is cleaned up from storage.

### Triumvirate Replacement Mechanism

Each collective can replace one Triumvirate member every 6 months through a **single atomic vote**: the collective votes to replace the current seat holder with a randomly selected new candidate from the eligible candidates. If the vote succeeds, the replacement happens immediately. The Triumvirate always maintains exactly 3 active members.

#### Timing

- Each collective can initiate replacement vote every 6 months (configurable)
- Economic and Building collectives have independent cycles (seat are rotated independently)

**Open Questions:**
- Q7: How to have an emergency replacement vote?
- Q8: Can a replaced member be voted back in immediately, or should there be a cooldown period?

#### Rotating Seat Selection

- Triumvirate seats are numbered: Seat 0, Seat 1, Seat 2
- Each collective maintains an independent rotation index that determines which seat they target:
- Economic Power automatically targets the next seat in rotation:
  - If last removal was Seat 0, next automatically targets Seat 1
  - If last removal was Seat 1, next automatically targets Seat 2
  - If last removal was Seat 2, next automatically targets Seat 0
- Building Power has independent automatic rotation
- Rotation ensures no single seat is disproportionately targeted
- Collective members cannot choose which seat to target: it's determined automatically

#### Replacement Process (Single Atomic Vote)

The replacement happens in a single vote where the collective votes **both** to remove the current seat holder **and** to install a specific replacement candidate. This is an atomic operation: either both happen or neither happens.

**Process:**
1. **Eligibility Phase**: Collective members can mark themselves as eligible for nomination to the Triumvirate.
2. **Voting Phase**: Collective members can vote aye/nay during the voting period to replace the current seat holder.
   - Threshold of more than 1/2 of the collective size (configurable)
   - **If vote succeeds**: Current seat holder immediately removed, replacement candidate immediately installed
   - **If vote fails**: No change, current member remains.
3. **Selection Phase**: The replacement candidate is selected randomly from the eligible candidates.
4. **Validation Phase**: The replacement candidate validates their nomination on-chain to avoid nominating inactive members.
5. **Transition**: Atomic swap ensures Triumvirate always has exactly 3 members with no vacancy period

### Implementation Phases

#### Phase 1: Coexistence (Duration: TBD)

1. Remove dead code: triumvirate collective and senate pallets and related code
2. Implement the governance as a new pallet
3. Deploy new governance pallet to runtime
4. Configure initial Triumvirate members and allowed proposers.
5. Run new governance system in parallel with existing sudo multisig
6. Emergency procedures documented and tested
7. Community review and feedback period

#### Phase 2: Full Migration

1. Disable sudo pallet via governance vote (new runtime)
2. New governance system becomes sole authority