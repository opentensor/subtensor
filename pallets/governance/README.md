# On-Chain Governance System

## Abstract

This proposes a comprehensive on-chain governance system to replace the current broken governance implementation that relies on a sudo-based triumvirate multisig. The new system introduces a separation of powers model with three key components: (1) an Opentensor Foundation (OTF) account authorized to propose runtime upgrades, (2) a three-member Triumvirate that votes on proposals, and (3) two collective bodies (Economic Power and Building Power) that can delay or cancel proposals and replace Triumvirate members through a removal and appointment process. The system will be deployed in two phases: first coexisting with the current sudo implementation for validation, then fully replacing it.

## Motivation

The current governance system in Subtensor is broken and relies entirely on a triumvirate multisig with sudo privileges. The runtime contains dead code related to the original triumvirate collective and senate that no longer functions properly. This centralized approach creates several critical issues:

1. **Single Point of Failure**: The sudo key represents a concentration of power with no on-chain checks or balances.
2. **Lack of Transparency**: Off-chain multisig decisions are not recorded or auditable on-chain.
3. **No Stakeholder Representation**: Major stakeholders (validators and subnet owners) have no formal mechanism to influence protocol upgrades.
4. **Technical Debt**: Dead governance code in the runtime creates maintenance burden and confusion.
5. **Trust Requirements**: The community must trust the multisig holders without cryptographic guarantees or accountability.

This proposal addresses these issues by implementing a proper separation of powers that balances efficiency with stakeholder representation, while maintaining upgrade capability and security.

## Specification

### Overview

The governance system consists of three main components working together:

1. **Proposal Origin**: OTF-authorized account(s)
2. **Approval Body**: Triumvirate (3 members)
3. **Oversight Bodies**: Economic Power Collective (top 16 validators by total stake) and Building Power Collective (top 16 subnet owners by moving average price)

### Actors and Roles

#### Opentensor Foundation (OTF) Accounts

- **Purpose**: Authorized to create runtime upgrade proposals
- **Assignment**: OTF account key(s) are configured in the runtime via governance
- **Permissions**: Can submit proposals to the main governance track
- **Constraints**: Cannot approve proposals; only the Triumvirate can approve

**Open Questions:**
- Q1: How many OTF accounts should be authorized initially? Single account or multiple? **Multiple because safe, no power except to make proposal, one for Sam and one for other team member.**
- Q2: What happens if OTF account is compromised/lost? Can it be revoked immediately or requires full governance process? **Full governance process**
- Q3: Only one proposal active at a time? Or multiple? Different track for upgrade? **Multiple proposal at the same time but only one get through, other are cancelled**
- Q4: Who can add/remove OTF accounts? Only governance or should Triumvirate have emergency powers?
- Q5: What types of proposals can OTF submit? Only runtime upgrades or any root extrinsic? **All type of calls**
- Q6: Who validates that proposal code matches stated intent before Triumvirate votes? Share runtime WASM hash like Polkadot fellowship does?
- Q7: Would it make sense to have an extrinsic to kick the calling OTF key to avoid compromised key to submit proposals?

#### Triumvirate

- **Composition**: 3 distinct accounts/seats (must always maintain 3 members)
- **Role**: Vote on proposals submitted by OTF accounts
- **Voting Threshold**: 2-of-3 approval required for proposals to pass
- **Term**: Indefinite, subject to replacement by collective vote
- **Accountability**: Each seat can be replaced through collective vote process (see Replacement Mechanism)

**Open Questions:**
- Q8: How are initial Triumvirate members selected? **Current triumvirate**
- Q9: When a member is being replaced, how is the new member selected? List of on-chain potential candidates? **Randomly from economic power collective or building power collective**
- Q10: Should Triumvirate members be known/doxxed or can they be anonymous?
- Q11: What happens if a Triumvirate member goes inactive for extended periods? **They need to accept the nomination or we rerun the nomination**
- Q12: Can Triumvirate members also be in collectives (conflict of interest)?
- Q13: What's the deadline for Triumvirate to vote? Can proposals expire?

#### Economic Power Collective

- **Composition**: Top 20 validators by total stake
- **Recalculation**: Membership refreshed every 2 months (432,000 blocks)
- **Powers**:
  - Delay or cancel proposals approved by Triumvirate
  - Replace one Triumvirate member every 6 months via single atomic vote (remove current holder + install replacement candidate, with rotating seat selection)

**Open Questions:**
- Q14: "Total stake" - does this include delegated stake or only self-bonded? **Includes delegated stake**
- Q15: Should there be a minimum stake threshold to enter collective? **Given we select top N, should be enough to be an implicit minimum**
- Q16: What happens if validator drops out of top 20 mid-term? Immediate removal or wait for refresh? **Keep their spot until next refresh**
- Q18: Can a validator be in both Economic and Building collectives if they also own top subnet? **Yes, although imply a different key**

#### Building Power Collective

- **Composition**: Top 20 subnet owners by moving average (MA) price
- **Recalculation**: Membership refreshed every 2 months (432,000 blocks)
- **Powers**:
  - Delay or cancel proposals approved by Triumvirate
  - Replace one Triumvirate member every 6 months via single atomic vote (remove current holder + install replacement candidate, with rotating seat selection)

**Open Questions:**
- Q19: What if subnet ownership transfers? Does collective seat transfer or recalculated when rotation happens?
- Q20: Should there be minimum subnet age requirement (prevent fresh subnets from voting)? **Maybe 3 or 4 months, or half a year, configurable**
- Q21: What if subnet is deregistered mid-term? Immediate collective removal?
- Q22: Can one entity own multiple subnets and occupy multiple collective seats? If not, how to prevent that? **Unique key only allowed on a collective**

### Governance Process Flow

#### Proposal Submission

1. OTF account creates a proposal containing runtime upgrade or any root extrinsic
2. Proposal enters "Triumvirate Voting" phase
3. Voting period: 7 days (50,400 blocks)

**Open Questions:**
- Q23: Can OTF cancel/withdraw a proposal after submission? What if they find a bug?
- Q24: Is there a queue limit?
- Q25: Who pays for proposal storage/execution? OTF, treasury, or included in proposal?

#### Triumvirate Approval

1. Triumvirate members cast votes (Aye/Nay) on the proposal
2. Requirement: At least 2 of 3 members must approve
3. If approved: Proposal enters "Delay Period"
4. If rejected: Proposal fails and is archived

**Open Questions:**
- Q26: What happens if only 1 of 3 members votes within 7 days? Proposal cancels?
- Q27: Can Triumvirate members change their vote before voting period ends?
- Q28: Should there be a veto power for individual Triumvirate members for emergency stops?

#### Delay Period (Collective Oversight)

1. Initial Duration: 7 days (50,400 blocks)
2. Both collectives can vote to delay/cancel
3. Each collective member can cast a "Delay" vote
4. Delay votes accumulate with cumulative time delays:
   - Vote 1: +12 hours (3,600 blocks at 12s/block)
   - Vote 2: +1 day (7,200 blocks)
   - Vote 3: +2 days (14,400 blocks)
   - Vote 4: +4 days (28,800 blocks)
   - Vote 5: +8 days (57,600 blocks)
   - Vote 6: +16 days (115,200 blocks)
   - Vote 7: +30 days (216,000 blocks)
   - Vote 8: +60 days (432,000 blocks)
5. Cancellation threshold: If 9 delay votes are cast within a single collective
6. If cancelled: Proposal is terminated
7. If delay period expires without cancellation: Proposal executes automatically

**Open Questions:**
- Q29: Are cumulative delays applied per-collective or across both collectives combined?
- Q30: Can collective members change their delay vote during the delay period?
- Q31: Should "Delay" votes require justification/reason on-chain?
- Q32: Can members vote "Support" (opposite of delay) to counter delay votes?
- Q33: Does EITHER collective reaching 9 votes cancel, or BOTH needed?

#### Execution

- Successful proposals execute automatically after the delay period
- Execution applies runtime upgrade or execute extrinsic
- Execution event is recorded on-chain

**Open Questions:**
- Q34: What if execution fails due to runtime error? Who is responsible to fix?
- Q35: Can execution be delayed further if critical issue discovered on day 13?
- Q36: Should there be a final "confirm execution" step by OTF or Triumvirate?
- Q37: What if network is congested and execution can't fit in block?

### Triumvirate Replacement Mechanism

Each collective can replace one Triumvirate member every 6 months through a **single atomic vote**: the collective votes to replace the current seat holder with a specific new candidate. If the vote succeeds, the replacement happens immediately. The Triumvirate always maintains exactly 3 active members.

#### Timing

- Each collective can initiate replacement vote every 6 months (1,296,800 blocks)
- Economic and Building collectives have independent 6-month cycles
- Cooldown timer starts after vote completion (whether successful or failed)

**Open Questions:**
- Q38: Does the 6-month timer start from genesis, from last replacement attempt, or last successful replacement?
- Q39: Can replacement be initiated early in emergency situations?
- Q40: Can a replaced member be voted back in immediately, or should there be a cooldown period?
- Q41: Should failed replacement attempts have a shorter cooldown (e.g., 1 month retry)?

#### Rotating Seat Selection

- Triumvirate seats are numbered: Seat 0, Seat 1, Seat 2
- Each collective maintains an automatic rotation index
- Economic Power automatically targets the next seat in rotation:
  - If last removal was Seat 0, next automatically targets Seat 1
  - If last removal was Seat 1, next automatically targets Seat 2
  - If last removal was Seat 2, next automatically targets Seat 0
- Building Power has independent automatic rotation
- Rotation ensures no single seat is disproportionately targeted
- Collective members cannot choose which seat to target - it's determined automatically

**Open Questions:**
- Q42: Should rotation reset if removal fails, or continue regardless?

#### Replacement Process (Single Atomic Vote)

The replacement happens in a single vote where the collective votes **both** to remove the current seat holder **and** to install a specific replacement candidate. This is an atomic operation - either both happen or neither happens.

**Process:**
1. **Proposal Phase**: Any collective member can propose a replacement by submitting:
   - Replacement candidate account
   - Optional: Justification text
   
2. **Voting Phase**: 
   - All collective members vote Aye/Nay on the replacement proposal
   - Threshold: Simple majority (11 of 20 members)
   - Voting period: 7 days (50,400 blocks)

   - **If vote succeeds**: Current seat holder immediately removed, replacement candidate immediately installed
   - **If vote fails**: No change, current member remains, cooldown timer starts
   
4. **Transition**: Atomic swap ensures Triumvirate always has exactly 3 members with no vacancy period

**Open Questions:**
- Q43: From where the candidate is selected?
- Q44: Can multiple replacement proposals be submitted for the same cycle? First-come-first-served or best candidate wins?
- Q45: Can replacement vote be vetoed by OTF in emergency situations?
- Q46: What happens to in-flight proposals where replaced member already voted?
- Q47: Can a replaced member be immediately proposed as replacement for a different seat?
- Q48: Who can propose replacement candidates? Any collective member or requires threshold support?
- Q49: Should there be a minimum vetting period between proposal and voting?

### Implementation Phases

#### Phase 1: Coexistence (Duration: 3-6 months)

1. Remove dead code: triumvirate collective and senate pallets and related code
2. Implement the governance as a new pallet
3. Deploy new governance pallet to runtime
4. Configure initial Triumvirate members
5. Configure OTF account(s)
6. Run new governance system in parallel with existing sudo multisig
7. All governance decisions processed through new system but sudo retains override capability
8. Monitor system performance, voting patterns, and security
9. Community review and feedback period

#### Phase 2: Full Migration

1. Disable sudo pallet via governance vote
2. Remove dead code: triumvirate collective and senate pallets
3. New governance system becomes sole authority
4. Emergency procedures documented and tested

**Open Questions:**
- Q50: What constitutes "emergency" and who decides to invoke emergency procedures?
