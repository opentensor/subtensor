# Subtensor Governance: Modular Design

## Problem

The current governance pallet is a monolith. It bundles:

- Referendum lifecycle (propose, schedule, execute)
- Triumvirate signed voting
- Collective anonymous voting (bLSAG ring signatures)
- Collective membership and rotation
- Track configuration (thresholds, delays)

This makes it hard to:

- Add new voting tracks without modifying the core pallet
- Change voting mechanisms (e.g., stake-weighted anonymous voting)
- Add new collective types
- Reuse voting primitives for non-governance use cases (e.g., elections)

## Architecture

Four pallets with clear boundaries, connected through traits:

```
┌─────────────────────────────────────────────┐
│         pallet-multi-collective             │
│  Membership management for all collectives  │
│  No voting, no proposals                    │
│                                             │
│  Exposes: CollectiveInspect trait           │
│  Hooks: OnMembersChanged, OnNewTerm         │
└──────────────┬──────────────────────────────┘
               │
       "who is in what group"
               │
    ┌──────────┴──────────┐
    ▼                     ▼
┌──────────────┐  ┌───────────────────┐
│pallet-signed │  │ pallet-anonymous  │
│   -voting    │  │    -voting        │
│              │  │                   │
│ Eligibility  │  │ Ring snapshot     │
│ check via    │  │ from collective   │
│ voter set    │  │ members           │
│              │  │                   │
│ Signed votes │  │ bLSAG + PoW      │
│ by AccountId │  │ Key image tracking│
│              │  │                   │
│ Pushes tally │  │ Pushes tally      │
│ to referenda │  │ to referenda      │
└──────┬───────┘  └────────┬──────────┘
       │                   │
       └─────────┬─────────┘
                 │ Polls trait (query + notify)
                 ▼
┌─────────────────────────────────────────────┐
│         pallet-referenda                    │
│  Proposal lifecycle + multi-track engine    │
│                                             │
│  Tracks define: voting scheme, voter set,   │
│  proposer set, decision strategy            │
│                                             │
│  Two proposal types:                        │
│  Action(call) — pass/fail, execute on pass  │
│  Review(task) — adjust scheduled task timing│
│                                             │
│  On each tally update:                      │
│  evaluate strategy → noop / approve /       │
│                       reject / adjust delay │
│                                             │
│  Implements Polls trait for voting pallets   │
│  Calls PollHooks on voting pallets          │
└─────────────────────────────────────────────┘
```

Key design principles:

- **Referenda never knows how votes are cast.** It receives tally updates (approval/rejection as `Perbill`) and applies track decision strategy.
- **Voting pallets never know what's being voted on.** They validate votes, record them, and push tally updates to referenda via the `Polls` trait.
- **Multi-collective never knows about proposals or voting.** It manages membership and fires hooks.
- **Track configuration lives in the runtime**, not hardcoded in any pallet.
- **Communication is push-based.** Voting pallets push tally updates to referenda. Referenda pushes poll lifecycle events to voting pallets for setup/cleanup. The state machine reacts to votes in real time — no scheduler nudges needed for vote evaluation.
- **Types are abstract inside pallets.** `CollectiveId`, `VoterSet`, `VotingScheme` are all associated types or generics — pallets don't know the concrete types. Only the runtime wiring resolves them.

---

## Shared Types

These live in a shared crate (e.g., `subtensor-runtime-common`) so all pallets can reference them without circular dependencies.

### VoteTally

The boundary struct between voting pallets and referenda. Voting pallets compute these values from their internal tally and push them to referenda. Referenda only sees percentages.

```rust
#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, TypeInfo, Debug, Default)]
pub struct VoteTally {
    pub approval: Perbill,   // ayes / total_eligible
    pub rejection: Perbill,  // nays / total_eligible
}
```

`approval + rejection + abstention = 100%`. Abstention is implicit (non-voters).

Each voting pallet has its own internal tally struct (e.g., `SignedVoteTally { ayes, nays, total }`) and converts to `VoteTally` before notifying referenda.

### SetLike

Generic trait for voter/proposer set eligibility checks:

```rust
pub trait SetLike<T> {
    fn contains(&self, item: &T) -> bool;
    fn len(&self) -> u32;
}
```

Used by both `VoterSet` and `ProposerSet` types in the track config. The concrete implementation reads from `pallet-multi-collective` storage.

### Polls

The interface between voting pallets and referenda. Referenda implements it; voting pallets consume it. Combines read-only queries and tally notification in one trait:

```rust
pub trait Polls<AccountId> {
    type Index: Parameter + Copy;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    /// Check if a poll is still ongoing.
    fn is_ongoing(index: Self::Index) -> bool;

    /// Get the voting scheme for a poll (voting pallets check this matches their scheme).
    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme>;

    /// Get the voter set for a poll (voting pallets check eligibility against this).
    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet>;

    /// Notify referenda that a vote changed the tally. Infallible — vote recording
    /// must not fail because referenda couldn't reschedule.
    fn on_tally_updated(index: Self::Index, tally: VoteTally);
}
```

### PollHooks

Referenda calls these on voting pallets for lifecycle events:

```rust
pub trait PollHooks<PollIndex> {
    /// A new poll was started. Voting pallets initialize their tally,
    /// snapshot rings (anonymous), etc.
    fn on_started(poll_index: PollIndex);

    /// A poll has concluded. Voting pallets clean up their storage.
    fn on_completed(poll_index: PollIndex);
}
```

Runtime wires both voting pallets as a tuple:
```rust
type PollHooks = (SignedVoting, AnonymousVoting);
```

Each pallet checks the poll's `VotingScheme` and only acts if it matches their scheme.

### bLSAG Primitives

Already implemented in `stp-crypto` (`primitives/crypto/`). Provides:

- `sign()`, `verify()`, `generate_key_image()`, `link()`
- `BlsagSignature`, `BlsagError`
- 35 unit tests covering round-trip, tampering, linkability, edge cases

No changes needed. Used directly by pallet-anonymous-voting.

---

## pallet-multi-collective

Membership management for all collectives. No voting, no proposals. Inspired by `pallet-membership` but uses `StorageMap<CollectiveId, ...>` instead of separate pallet instances.

### Config

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// The collective identifier type. Opaque to the pallet.
    /// Concrete enum defined in runtime primitives.
    type CollectiveId: Parameter + MaxEncodedLen + Copy;

    /// Provides per-collective information (name, min/max members, term duration).
    /// Implemented in the runtime. No storage — compiled-in constants.
    type Collectives: CollectivesInfo<BlockNumberFor<Self>, CollectiveName,
        Id = Self::CollectiveId>;

    /// Required origins for member management (per collective via EnsureOriginWithArg).
    type AddOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;
    type RemoveOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;
    type SwapOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;
    type ResetOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

    /// Called when a collective's membership has changed.
    type OnMembersChanged: OnMembersChanged<Self::CollectiveId, Self::AccountId>;

    /// Called when a collective's term expires.
    type OnNewTerm: OnNewTerm<Self::CollectiveId>;

    /// Maximum members per collective (used for BoundedVec storage bound).
    #[pallet::constant]
    type MaxMembers: Get<u32>;
}
```

### CollectivesInfo trait

Provides static configuration per collective. The pallet iterates this in `on_initialize` for term expiry checks:

```rust
pub trait CollectivesInfo<Moment, Name> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord;

    /// Return all known collectives with their configuration.
    fn collectives() -> impl Iterator<Item = Collective<Self::Id, Moment, Name>>;

    /// Lookup info for a specific collective.
    fn info(id: Self::Id) -> Option<CollectiveInfo<Moment, Name>>;
}

pub struct CollectiveInfo<Moment, Name> {
    pub name: Name,
    pub min_members: u32,
    pub max_members: Option<u32>,
    pub term_duration: Option<Moment>,
}
```

Implemented in the runtime as a static list — adding a `CollectiveId` variant forces handling in the exhaustive match.

### Storage

```rust
/// Members of each collective. The only storage this pallet needs.
pub type Members<T: Config> = StorageMap<
    _, Blake2_128Concat, T::CollectiveId,
    BoundedVec<T::AccountId, T::MaxMembers>, ValueQuery>;
```

### Extrinsics

```rust
fn add_member(origin, collective_id, who) -> DispatchResult;
fn remove_member(origin, collective_id, who) -> DispatchResult;
fn swap_member(origin, collective_id, remove, add) -> DispatchResult;
fn reset_members(origin, collective_id, members: Vec<AccountId>) -> DispatchResult;
```

Each validates the origin via `EnsureOriginWithArg`, checks min/max member bounds from `CollectivesInfo`, and fires `OnMembersChanged` with incoming/outgoing diffs.

### CollectiveInspect trait (exposed to other pallets)

```rust
pub trait CollectiveInspect<AccountId, CollectiveId> {
    fn members_of(collective_id: CollectiveId) -> Vec<AccountId>;
    fn is_member(collective_id: CollectiveId, who: &AccountId) -> bool;
    fn member_count(collective_id: CollectiveId) -> u32;
}
```

### on_initialize

Iterates `CollectivesInfo::collectives()`, checks `term_duration` against the current block, and fires `OnNewTerm` when a term expires. The pallet doesn't know what "new term" means — the hook decides (direct rotation in v1, election referenda in v2).

---

## pallet-signed-voting

Simple signed voting for tracks that don't require anonymity (e.g., triumvirate voting).

### Config

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// The voting scheme this pallet handles. Passed as a constant.
    /// The pallet rejects votes on tracks with a different scheme.
    type Scheme: Get<VotingSchemeOf<Self>>;

    /// The referenda pallet. Provides poll queries and receives tally updates.
    type Polls: Polls<Self::AccountId, Tally = VoteTally>;
}
```

### Storage

```rust
/// Votes keyed by (PollIndex, AccountId) -> vote direction.
pub type VotingFor<T> = StorageDoubleMap<_, _, PollIndex, _, AccountId, bool, OptionQuery>;

/// Tally per poll. Internal representation with raw counts.
/// Converted to VoteTally (Perbill) before pushing to referenda.
pub type TallyOf<T> = StorageMap<_, _, PollIndex, SignedVoteTally, OptionQuery>;
```

`SignedVoteTally` is the internal struct:
```rust
pub struct SignedVoteTally { ayes: u32, nays: u32, total: u32 }
```

### Extrinsics

```rust
/// Cast or change a vote. Errors on duplicate (same direction).
fn vote(origin, poll_index, approve: bool) -> DispatchResult;

/// Remove an existing vote (return to abstain).
fn remove_vote(origin, poll_index) -> DispatchResult;
```

### How It Works

1. Check poll is ongoing via `T::Polls::is_ongoing()`
2. Check `T::Polls::voting_scheme_of()` matches `T::Scheme::get()`
3. Check `T::Polls::voter_set_of()` contains the caller
4. Update `VotingFor` and `TallyOf`
5. Convert `SignedVoteTally` to `VoteTally` (Perbill values)
6. Call `T::Polls::on_tally_updated()` — referenda evaluates and acts

### PollHooks implementation

- `on_started`: Initialize `TallyOf` with `total` from voter set `len()`
- `on_completed`: Clear `VotingFor` prefix and remove `TallyOf` for the poll

---

## pallet-anonymous-voting

Anonymous voting using bLSAG ring signatures. Uses `stp-crypto` for cryptographic primitives.

### Config

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type Scheme: Get<VotingSchemeOf<Self>>;
    type Polls: Polls<Self::AccountId, Tally = VoteTally>;

    /// PoW difficulty for spam prevention on unsigned extrinsics.
    #[pallet::constant]
    type PowDifficulty: Get<u32>;

    /// Maximum ring size.
    #[pallet::constant]
    type MaxRingSize: Get<u32>;
}
```

### Storage

```rust
/// Frozen ring of Ristretto public keys per poll.
pub type PollRing<T> = StorageMap<_, _, PollIndex,
    BoundedVec<[u8; 32], MaxRingSize>, OptionQuery>;

/// Anonymous votes keyed by (PollIndex, KeyImage) -> vote direction.
pub type AnonymousVotes<T> = StorageDoubleMap<_, _, PollIndex,
    _, [u8; 32], bool, OptionQuery>;

/// Internal tally per poll.
pub type TallyOf<T> = StorageMap<_, _, PollIndex, AnonymousVoteTally, OptionQuery>;
```

### Extrinsics

```rust
/// Cast an anonymous vote using a bLSAG ring signature.
/// Unsigned extrinsic guarded by PoW.
/// Vote action: Aye, Nay, or Remove.
fn anonymous_vote(
    origin,  // must be none (unsigned)
    poll_index: PollIndex,
    vote: AnonymousVoteAction,  // Aye, Nay, Remove
    signature: stp_crypto::BlsagSignature,
    pow_nonce: u64,
) -> DispatchResult;
```

### Ring Lifecycle

- **Creation (`on_started`):** Snapshot the ring from the voter set's collective members. AccountId bytes are the ring members (Sr25519 keys = compressed Ristretto points). Non-Ristretto keys filtered via `stp_crypto::verify_point_valid()`.
- **Frozen:** Ring does not change during the poll's lifetime, even if the collective rotates.
- **Cleanup (`on_completed`):** Clear `PollRing`, `AnonymousVotes`, `TallyOf`.

### ValidateUnsigned

```rust
fn validate_unsigned(source, call) -> TransactionValidity {
    // 1. PoW check (cheapest filter)
    // 2. Poll must exist and be ongoing
    // 3. Ring must exist
    // 4. Structural check (response count == ring size)
    // 5. Full bLSAG signature verification
}
```

### How It Works

1. Check poll is ongoing, voting scheme matches
2. Verify bLSAG signature against frozen ring
3. Validate PoW
4. Check key image for double-voting (allows direction change and removal)
5. Update `AnonymousVotes` and `TallyOf`
6. Convert to `VoteTally`, call `T::Polls::on_tally_updated()`

---

## pallet-referenda

The proposal lifecycle engine. Two extrinsics: `submit` and `cancel`.

### Config

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeCall: Parameter + Dispatchable + ...;

    /// Track definitions. All track config (voter set, voting scheme, proposer set,
    /// decision strategy) comes from here. Referenda stores and passes through the
    /// opaque types without inspecting them.
    type Tracks: TracksInfo<...>;

    /// Origin allowed to cancel a referendum.
    type CancelOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// Scheduler for execution and timeouts.
    type Scheduler: ScheduleNamed<...> + ScheduleAnon<...>;

    /// Preimage provider for call storage.
    type Preimages: QueryPreimage + StorePreimage;

    /// Lifecycle hooks for voting pallets.
    type PollHooks: PollHooks<ReferendumIndex>;

    /// Block number provider.
    type BlockNumberProvider: BlockNumberProvider;
}
```

### TracksInfo trait

Defined in the referenda pallet, implemented in the runtime. The associated types are opaque to referenda — voting pallets constrain them to the concrete types they need:

```rust
pub trait TracksInfo<Name, AccountId, Moment> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord;
    type ProposerSet: SetLike<AccountId>;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn tracks() -> impl Iterator<Item = Track<...>>;
    fn info(id: Self::Id) -> Option<TrackInfo<...>>;

    /// Optional per-track call validation. Default allows all.
    fn authorize_proposal(id: Self::Id, proposal: &Call) -> bool { true }
}
```

### TrackInfo

```rust
pub struct TrackInfo<Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub name: Name,
    pub proposer_set: ProposerSet,
    pub voter_set: VoterSet,
    pub voting_scheme: VotingScheme,
    pub decision_strategy: DecisionStrategy<Moment>,
}
```

### Proposal types

```rust
pub enum Proposal<Call> {
    /// A call to execute if approved.
    Action(Call),
    /// A reference to an existing scheduled task. Votes adjust its timing.
    Review(TaskName),
}
```

### DecisionStrategy

```rust
pub enum DecisionStrategy<Moment> {
    /// Binary decision: passes or fails before a deadline.
    /// If `approve_threshold` reached → execute the call.
    /// If `reject_threshold` reached → cancel.
    /// If deadline expires → expired.
    PassOrFail {
        decision_period: Moment,
        approve_threshold: Perbill,
        reject_threshold: Perbill,
    },
    /// Timing adjustment for an already-scheduled task.
    /// Strong approval → fast-track (reschedule to ASAP).
    /// Strong rejection → cancel the task.
    /// In between → linearly interpolate execution delay.
    /// No deadline — lives until the task executes or is cancelled.
    Adjustable {
        fast_track_threshold: Perbill,
        reject_threshold: Perbill,
    },
}
```

### Storage

```rust
/// Global referendum counter, incremented on each submit.
pub type ReferendumCount<T> = StorageValue<_, ReferendumIndex, ValueQuery>;

/// Referendum status per index.
pub type ReferendumStatusFor<T> = StorageMap<_, _, ReferendumIndex,
    ReferendumStatus<...>, OptionQuery>;

/// Tally cache per referendum (updated on each on_tally_updated call).
/// Used for timeout evaluation when no vote triggers the check.
pub type ReferendumTally<T> = StorageMap<_, _, ReferendumIndex,
    VoteTally, OptionQuery>;
```

### Key Types

```rust
pub struct ReferendumInfo<AccountId, TrackId, Call, Moment, ScheduleAddress> {
    pub track: TrackId,
    pub proposal: Proposal<Call>,
    pub submitter: AccountId,
    pub submitted: Moment,
    pub alarm: Option<(Moment, ScheduleAddress)>,
}

pub enum ReferendumStatus<...> {
    Ongoing(ReferendumInfo<...>),
    Approved(Moment),
    Rejected(Moment),
    Cancelled(Moment),
    Expired(Moment),
}
```

Note: the detailed vote tally is NOT stored in the referendum. Voting pallets own their tallies. Referenda caches a `VoteTally` (two `Perbill` values) in `ReferendumTally` for timeout evaluation.

### Extrinsics

```rust
/// Submit a new referendum. Proposal type (Action/Review) determines behavior.
fn submit(origin, track: TrackId, proposal: Proposal<BoundedCallOf<T>>) -> DispatchResult;

/// Cancel an ongoing referendum.
fn cancel(origin, index: ReferendumIndex) -> DispatchResult;
```

### Submit flow

1. Get track info from `TracksInfo`
2. Check proposer is in `track.proposer_set`
3. If `Action(call)`: validate via `TracksInfo::authorize_proposal(track, &call)`
4. If `Review(task_name)`: verify the named task exists in the scheduler
5. Increment `ReferendumCount`, create `ReferendumInfo`
6. For `PassOrFail`: set scheduler alarm for `decision_period` timeout
7. Call `PollHooks::on_started()` — voting pallets initialize tallies, snapshot rings

### State Machine (on_tally_updated)

Event-driven — reacts to each tally update pushed by voting pallets. Referenda implements the `Polls` trait and evaluates the decision strategy inside `on_tally_updated`:

```rust
fn on_tally_updated(index, tally: VoteTally) {
    // Cache the tally for timeout evaluation
    ReferendumTally::insert(index, tally);

    let info = ReferendumStatusFor::get(index);
    let track = Tracks::info(info.track);

    match (&info.proposal, &track.decision_strategy) {
        // Action + PassOrFail: simple approve/reject
        (Action(call), PassOrFail { approve_threshold, reject_threshold, .. }) => {
            if tally.approval >= *approve_threshold {
                schedule_and_approve(index, call);
            } else if tally.rejection >= *reject_threshold {
                reject(index);
            }
        },

        // Review + Adjustable: reschedule the named task
        (Review(task_name), Adjustable { fast_track_threshold, reject_threshold }) => {
            if tally.approval >= *fast_track_threshold {
                reschedule_named(task_name, now + 1);
                approve(index);
            } else if tally.rejection >= *reject_threshold {
                cancel_named(task_name);
                cancel(index);
            } else {
                // Linear interpolation between current approval and thresholds
                // to determine execution delay
                reschedule_named(task_name, computed_delay);
            }
        },
    }
}
```

### Timeout (PassOrFail only)

When the scheduler alarm fires for a `PassOrFail` referendum:
- Read cached `ReferendumTally`
- If neither threshold reached → mark as Expired
- Call `PollHooks::on_completed()`

`Adjustable` referenda have no timeout — they live until the task executes or is cancelled.

### Membership Changes and Active Polls

Membership changes do NOT affect active polls:

- **Signed voting:** Eligibility checked at vote time against current collective. Rotated-out members can't vote but existing votes remain.
- **Anonymous voting:** Ring frozen at poll creation. Rotation doesn't change it.

Simple and predictable.

---

## Worked Example: Runtime Upgrade Flow

### Setup

- OTF is an allowed proposer for track 0 (triumvirate)
- Triumvirate has 3 members: Alice, Bob, Charlie
- Economic collective has 16 members, Building collective has 16 members

### Step 1: OTF Submits Proposal

OTF submits an `Action` proposal on track 0. The call is a batch that schedules the upgrade AND creates a Review referendum for the collective:

```
OTF → referenda.submit(
    track: 0,
    proposal: Action(batch_all(
        scheduler.schedule_named("upgrade_42", block + 100, set_code(wasm)),
        referenda.submit(track: 1, Review("upgrade_42")),
    )),
)
```

Referenda:
- `proposer_set` for track 0 contains OTF ✓
- `authorize_proposal` validates the call ✓
- Creates poll #0, sets alarm for decision_period timeout
- `PollHooks::on_started(0)` → signed voting initializes tally with total=3

### Step 2: Triumvirate Votes

```
Alice → signed_voting.vote(poll_index: 0, approve: true)
```
- Voting scheme check: track 0 = Signed ✓
- Voter set check: Alice in Triumvirate ✓
- Tally: {ayes: 1, nays: 0, total: 3} → approval = 33%
- Referenda: 33% < 67% → noop

```
Bob → signed_voting.vote(poll_index: 0, approve: true)
```
- Tally: {ayes: 2, nays: 0, total: 3} → approval = 67%
- Referenda: 67% >= 67% → **Approved!**
- Batch executes:
  - Upgrade scheduled as "upgrade_42" at block + 100
  - Review referendum created on track 1
- Poll #0 marked Approved, `PollHooks::on_completed(0)`

### Step 3: Ring Snapshot

`PollHooks::on_started(1)` fires for track 1:
- Anonymous voting checks: track 1 voting_scheme = Anonymous ✓
- Voter set = Union([Economic, Building])
- Snapshots 32 member AccountIds as Ristretto ring
- Initializes tally with total=32

### Step 4: Collective Adjusts Timing

```
??? → anonymous_voting.anonymous_vote(poll: 1, vote: Aye, sig: <bLSAG>, pow: 12345)
```
- bLSAG valid against frozen ring ✓, PoW valid ✓
- Tally updated, pushed to referenda

As votes accumulate:
- **62.5% approval** → delay = linear interpolation between max and 0
- **75%+ approval** → fast-tracked, task rescheduled to now + 1
- **51%+ rejection** → task cancelled

### Step 5: Execution

At the (possibly adjusted) scheduled block, `set_code(wasm)` executes.

---

## Runtime Wiring (v1)

```rust
use primitives::CollectiveId;

impl pallet_multi_collective::Config for Runtime {
    type CollectiveId = CollectiveId;
    type Collectives = SubtensorCollectives;  // static list
    type AddOrigin = EnsureRoot<AccountId>;
    type RemoveOrigin = EnsureRoot<AccountId>;
    type SwapOrigin = EnsureRoot<AccountId>;
    type ResetOrigin = EnsureRoot<AccountId>;
    type OnMembersChanged = ();
    type OnNewTerm = DirectRotation;
    type MaxMembers = ConstU32<32>;
}

impl pallet_signed_voting::Config for Runtime {
    type Scheme = SignedScheme;
    type Polls = Referenda;
}

impl pallet_anonymous_voting::Config for Runtime {
    type Scheme = AnonymousScheme;
    type Polls = Referenda;
    type PowDifficulty = ConstU32<16>;
    type MaxRingSize = ConstU32<64>;
}

impl pallet_referenda::Config for Runtime {
    type Tracks = SubtensorTracks;
    type CancelOrigin = EnsureRoot<AccountId>;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type PollHooks = (SignedVoting, AnonymousVoting);
}
```

### v1 Tracks

```rust
const TRACKS: &[(TrackId, TrackInfo<...>)] = &[
    (0, TrackInfo {
        name: "triumvirate",
        proposer_set: MemberSet::Single(CollectiveId::Proposers),
        voter_set: MemberSet::Single(CollectiveId::Triumvirate),
        voting_scheme: VotingScheme::Signed,
        decision_strategy: DecisionStrategy::PassOrFail {
            decision_period: 20,
            approve_threshold: Perbill::from_percent(67),
            reject_threshold: Perbill::from_percent(67),
        },
    }),
    (1, TrackInfo {
        name: "collective",
        proposer_set: MemberSet::Single(CollectiveId::Proposers),
        voter_set: MemberSet::Union(vec![CollectiveId::Economic, CollectiveId::Building]),
        voting_scheme: VotingScheme::Anonymous,
        decision_strategy: DecisionStrategy::Adjustable {
            fast_track_threshold: Perbill::from_percent(75),
            reject_threshold: Perbill::from_percent(51),
        },
    }),
];
```

---

## Future Extensions

### Election Mechanism (v2)

Plugs in via `OnNewTerm` hook on pallet-multi-collective:
- Hook creates referenda per seat on an election track
- Members call `declare_candidacy(collective, seat)` (new extrinsic on multi-collective)
- Each candidate gets a binary aye/nay anonymous vote
- Highest approval above threshold wins the seat

No new pallets needed.

### Stake-Weighted Anonymous Voting

bLSAG proves membership but not properties of the member. Options:
- **Bucket rings:** Separate rings per stake bracket, vote carries bucket weight
- **ZK proofs:** Prove stake in zero knowledge alongside ring signature

The `VoteTally` boundary struct handles this — voting pallets compute approval/rejection however they want internally. Referenda only sees `Perbill` values.

### Additional Voting Pallets

Any new voting mechanism (conviction, delegation, ZK) just needs to:
1. Implement `PollHooks` for lifecycle
2. Call `Polls::on_tally_updated()` with a `VoteTally`
3. Add a `VotingScheme` variant

No changes to referenda or existing voting pallets.

### Additional Tracks

Adding a track is a runtime config change — define parameters in `TracksInfo`, no pallet code changes.

---

## Open Issues

1. **Threshold participation.** `PassOrFail` uses `approval = ayes / total_eligible`. One aye out of 3 = 33%, below 67% threshold. This naturally requires participation. Verify during implementation.

2. **VotingScheme as config constant.** Each voting pallet has `type Scheme: Get<VotingScheme>` to self-identify. If the `VotingScheme` enum gains variants, existing pallets are unaffected — they just check `scheme == my_scheme`.

3. **on_tally_updated is infallible.** If referenda's scheduler call fails internally, it should log and continue — not fail the voter's extrinsic.

4. **Batch composition for two-phase flow.** The proposer submits `batch_all(schedule_named(...), referenda.submit(Review(...)))`. Verify this works when dispatched by the scheduler after track 0 approval.

5. **Preimage handling.** Use Polkadot's `pallet-preimage` as-is for storing large proposal calls (e.g., `set_code`).

6. **Benchmarking.** bLSAG verification + PoW in `ValidateUnsigned` is expensive. Need benchmarks for anonymous voting weights, especially with 32-member rings.

---

## Implementation Path

The monolith governance pallet has not been deployed. No migration is needed.

1. Build the four new pallets (multi-collective and voting pallets first, referenda last)
2. Wire them in a mock runtime to verify interfaces compile
3. Remove the old monolith governance pallet

The `stp-crypto` bLSAG primitives are already done and tested (35 tests). They drop directly into pallet-anonymous-voting unchanged.
