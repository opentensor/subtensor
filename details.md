# Subtensor Chain - Complete Pallet Documentation

This document provides a comprehensive reference for all pallets, events, extrinsics (dispatchable functions), storage items, and custom types in the Subtensor blockchain. This is intended to help developers build custom blockchain explorers and wallets.

## Table of Contents

1. [Pallet Subtensor](#pallet-subtensor) - Core pallet for network management
2. [Pallet Admin Utils](#pallet-admin-utils) - Administrative utilities
3. [Pallet Commitments](#pallet-commitments) - Data commitment management
4. [Pallet Crowdloan](#pallet-crowdloan) - Crowdfunding functionality
5. [Pallet Drand](#pallet-drand) - Randomness beacon integration
6. [Pallet Proxy](#pallet-proxy) - Account delegation
7. [Pallet Registry](#pallet-registry) - On-chain identity management
8. [Pallet Shield](#pallet-shield) - MEV protection
9. [Pallet Swap](#pallet-swap) - Token swapping (Uniswap V3-like)
10. [Pallet Transaction Fee](#pallet-transaction-fee) - Custom fee handling
11. [Pallet Utility](#pallet-utility) - Batch operations

---

## Pallet Subtensor

The main pallet for the Subtensor blockchain, managing subnets, neurons, staking, weights, and network parameters.

### Core Concepts

- **NetUid**: Network UID - unique identifier for a subnet
- **MechId**: Mechanism ID - identifier for specific mechanisms within subnets
- **Hotkey**: Account used for neuron operations and receiving rewards
- **Coldkey**: Account that controls stake and can be used for cold storage
- **UID**: Unique identifier for a neuron within a subnet
- **TAO**: Native currency of the chain
- **Alpha**: Subnet-specific token that can be earned and swapped

### Events

#### Network Management

**NetworkAdded(NetUid, u16)**
- Emitted when a new subnet is created
- Parameters:
  - `NetUid`: The unique identifier for the new network
  - `u16`: The network registration cost or index

**NetworkRemoved(NetUid)**
- Emitted when a subnet is removed
- Parameters:
  - `NetUid`: The identifier of the removed network

#### Staking Operations

**StakeAdded(T::AccountId, T::AccountId, TaoCurrency, AlphaCurrency, NetUid, u64)**
- Emitted when TAO is staked to a hotkey
- Parameters:
  - `T::AccountId` (1st): The coldkey account adding stake
  - `T::AccountId` (2nd): The hotkey receiving the stake
  - `TaoCurrency`: Amount of TAO staked
  - `AlphaCurrency`: Amount of Alpha associated with this stake
  - `NetUid`: The subnet where stake is added
  - `u64`: Block number when stake was added

**StakeRemoved(T::AccountId, T::AccountId, TaoCurrency, AlphaCurrency, NetUid)**
- Emitted when stake is removed from a hotkey
- Parameters:
  - `T::AccountId` (1st): The coldkey removing stake
  - `T::AccountId` (2nd): The hotkey losing stake
  - `TaoCurrency`: Amount of TAO removed
  - `AlphaCurrency`: Amount of Alpha removed
  - `NetUid`: The subnet from which stake is removed

**StakeTransferred(T::AccountId, T::AccountId, T::AccountId, TaoCurrency, AlphaCurrency, NetUid)**
- Emitted when stake is transferred between hotkeys
- Parameters:
  - `T::AccountId` (1st): The coldkey initiating the transfer
  - `T::AccountId` (2nd): The source hotkey
  - `T::AccountId` (3rd): The destination hotkey
  - `TaoCurrency`: Amount of TAO transferred
  - `AlphaCurrency`: Amount of Alpha transferred
  - `NetUid`: The subnet where transfer occurs

#### Weight Setting

**WeightsSet(NetUidStorageIndex, u16)**
- Emitted when a validator sets weights
- Parameters:
  - `NetUidStorageIndex`: Packed storage index containing NetUid
  - `u16`: The UID of the validator setting weights

**CRV3WeightsCommitted(T::AccountId, NetUidStorageIndex, H256)**
- Emitted when weights are committed (commit-reveal mechanism)
- Parameters:
  - `T::AccountId`: The hotkey committing weights
  - `NetUidStorageIndex`: Packed subnet identifier
  - `H256`: Hash of the committed weights

**CRV3WeightsRevealed(T::AccountId, NetUidStorageIndex)**
- Emitted when committed weights are revealed
- Parameters:
  - `T::AccountId`: The hotkey revealing weights
  - `NetUidStorageIndex`: Packed subnet identifier

#### Neuron Registration

**NeuronRegistered(NetUid, u16, T::AccountId)**
- Emitted when a new neuron is registered
- Parameters:
  - `NetUid`: The subnet where registration occurred
  - `u16`: The UID assigned to the neuron
  - `T::AccountId`: The hotkey of the registered neuron

**BurnedRegistered(NetUid, u16, T::AccountId, TaoCurrency)**
- Emitted when a neuron is registered by burning TAO
- Parameters:
  - `NetUid`: The subnet
  - `u16`: The assigned UID
  - `T::AccountId`: The hotkey
  - `TaoCurrency`: Amount of TAO burned

#### Axon/Prometheus Serving

**AxonServed(NetUid, T::AccountId)**
- Emitted when a neuron updates its axon endpoint information
- Parameters:
  - `NetUid`: The subnet
  - `T::AccountId`: The hotkey serving the axon

**PrometheusServed(NetUid, T::AccountId)**
- Emitted when a neuron updates its Prometheus metrics endpoint
- Parameters:
  - `NetUid`: The subnet
  - `T::AccountId`: The hotkey serving Prometheus

#### Delegation

**DelegateAdded(T::AccountId, T::AccountId, u16)**
- Emitted when a hotkey becomes a delegate
- Parameters:
  - `T::AccountId` (1st): The coldkey owner
  - `T::AccountId` (2nd): The hotkey that became a delegate
  - `u16`: The delegate take rate (percentage in basis points)

**DelegateRemoved(T::AccountId, T::AccountId)**
- Emitted when a hotkey is removed as a delegate
- Parameters:
  - `T::AccountId` (1st): The coldkey owner
  - `T::AccountId` (2nd): The hotkey removed as delegate

#### Key Swapping

**ColdkeySwapped { old_coldkey: T::AccountId, new_coldkey: T::AccountId, swap_cost: TaoCurrency }**
- Emitted when a coldkey is swapped
- Parameters:
  - `old_coldkey`: The previous coldkey
  - `new_coldkey`: The new coldkey taking over
  - `swap_cost`: The TAO cost for this swap operation

**HotKeySwapped { old_hotkey: T::AccountId, new_hotkey: T::AccountId, coldkey: T::AccountId }**
- Emitted when a hotkey is swapped
- Parameters:
  - `old_hotkey`: The previous hotkey
  - `new_hotkey`: The new hotkey taking over
  - `coldkey`: The coldkey controlling this swap

#### Childkey Management

**ChildKeyAdded(T::AccountId, T::AccountId, u64, u64)**
- Emitted when a childkey is added to a hotkey
- Parameters:
  - `T::AccountId` (1st): The parent hotkey
  - `T::AccountId` (2nd): The childkey being added
  - `u64` (1st): The proportion of stake assigned to childkey
  - `u64` (2nd): Block number of addition

**ChildKeyRemoved(T::AccountId, T::AccountId, u64)**
- Emitted when a childkey is removed
- Parameters:
  - `T::AccountId` (1st): The parent hotkey
  - `T::AccountId` (2nd): The childkey being removed
  - `u64`: Block number of removal

#### Subnet Identity

**SubnetIdentitySet(NetUid)**
- Emitted when subnet identity information is set
- Parameters:
  - `NetUid`: The subnet whose identity was updated

**EvmKeyAssociated { netuid: NetUid, hotkey: T::AccountId, evm_key: H160, block_associated: u64 }**
- Emitted when an EVM key is associated with a hotkey
- Parameters:
  - `netuid`: The subnet where association occurs
  - `hotkey`: The hotkey being associated
  - `evm_key`: The Ethereum address (160-bit)
  - `block_associated`: Block number when association was made

#### Token Operations

**AlphaRecycled(T::AccountId, AlphaCurrency, NetUid)**
- Emitted when Alpha tokens are recycled
- Parameters:
  - `T::AccountId`: The hotkey recycling Alpha
  - `AlphaCurrency`: Amount recycled
  - `NetUid`: The subnet

**AlphaBurned(T::AccountId, AlphaCurrency, NetUid)**
- Emitted when Alpha tokens are burned
- Parameters:
  - `T::AccountId`: The hotkey burning Alpha
  - `AlphaCurrency`: Amount burned
  - `NetUid`: The subnet

#### Subnet Leasing

**SubnetLeaseCreated { netuid: NetUid, buyer: T::AccountId, lease_block: u64, end_block: u64 }**
- Emitted when a subnet lease is created
- Parameters:
  - `netuid`: The subnet being leased
  - `buyer`: The account purchasing the lease
  - `lease_block`: Block when lease starts
  - `end_block`: Block when lease ends

### Extrinsics (Dispatchable Functions)

#### Weight Setting

**set_weights(origin, netuid: NetUid, dests: Vec<u16>, weights: Vec<u16>, version_key: u64)**
- Sets validator weights for neurons in a subnet
- Parameters:
  - `origin`: Must be signed by a registered hotkey
  - `netuid`: The subnet to set weights for
  - `dests`: Vector of UIDs to assign weights to
  - `weights`: Vector of weight values (must match length of dests)
  - `version_key`: Version of the weights protocol being used
- Notes: Disabled when commit-reveal is enabled for the subnet

**commit_weights(origin, netuid: NetUid, commit_hash: H256)**
- Commits a hash of weights (first step of commit-reveal)
- Parameters:
  - `origin`: Must be signed by a registered hotkey
  - `netuid`: The subnet
  - `commit_hash`: Hash of the weights, salt, and version key
- Notes: Only available when commit-reveal is enabled

**reveal_weights(origin, netuid: NetUid, uids: Vec<u16>, values: Vec<u16>, salt: Vec<u16>, version_key: u64)**
- Reveals previously committed weights
- Parameters:
  - `origin`: Must be signed by the hotkey that committed
  - `netuid`: The subnet
  - `uids`: Vector of UIDs (must hash with other params to match commitment)
  - `values`: Vector of weight values
  - `salt`: Random salt used in commitment
  - `version_key`: Version key used in commitment

#### Staking Operations

**add_stake(origin, hotkey: T::AccountId, netuid: NetUid, amount_staked: TaoCurrency)**
- Adds stake from coldkey to a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The hotkey to stake to
  - `netuid`: The subnet for staking
  - `amount_staked`: Amount of TAO to stake

**remove_stake(origin, hotkey: T::AccountId, netuid: NetUid, amount_unstaked: TaoCurrency)**
- Removes stake from a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The hotkey to unstake from
  - `netuid`: The subnet
  - `amount_unstaked`: Amount of TAO to remove
- Notes: Fees can be paid in Alpha

**add_stake_multiple(origin, hotkeys: Vec<T::AccountId>, netuids: Vec<NetUid>, amounts_staked: Vec<TaoCurrency>)**
- Adds stake to multiple hotkeys in a single transaction
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkeys`: Vector of hotkeys
  - `netuids`: Vector of subnets (must match length)
  - `amounts_staked`: Vector of amounts (must match length)

**unstake_all(origin, hotkey: T::AccountId, netuid: NetUid)**
- Removes all stake from a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The hotkey to fully unstake
  - `netuid`: The subnet
- Notes: Fees can be paid in Alpha

**move_stake(origin, old_hotkey: T::AccountId, new_hotkey: T::AccountId, netuid: NetUid, amount: TaoCurrency)**
- Moves stake from one hotkey to another
- Parameters:
  - `origin`: Must be signed by coldkey
  - `old_hotkey`: Source hotkey
  - `new_hotkey`: Destination hotkey
  - `netuid`: The subnet
  - `amount`: Amount to move
- Notes: Fees can be paid in Alpha

**transfer_stake(origin, old_hotkey: T::AccountId, new_hotkey: T::AccountId, netuid: NetUid, amount: TaoCurrency)**
- Similar to move_stake but with different semantics
- Parameters: Same as move_stake
- Notes: Fees can be paid in Alpha

**swap_stake(origin, old_hotkey: T::AccountId, new_hotkey: T::AccountId, netuid: NetUid, amount: TaoCurrency)**
- Swaps stake between hotkeys
- Parameters: Same as move_stake
- Notes: Fees can be paid in Alpha

#### Registration

**register(origin, netuid: NetUid, block_number: u64, nonce: u64, work: Vec<u8>, hotkey: T::AccountId, coldkey: T::AccountId)**
- Registers a neuron via Proof of Work
- Parameters:
  - `origin`: Can be signed by anyone
  - `netuid`: The subnet to register in
  - `block_number`: Block number used for PoW
  - `nonce`: Nonce found that satisfies difficulty
  - `work`: The work bytes proving computation
  - `hotkey`: The hotkey to register
  - `coldkey`: The controlling coldkey

**burned_register(origin, netuid: NetUid, hotkey: T::AccountId)**
- Registers a neuron by burning TAO
- Parameters:
  - `origin`: Must be signed by coldkey
  - `netuid`: The subnet to register in
  - `hotkey`: The hotkey to register

#### Serving Information

**serve_axon(origin, netuid: NetUid, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8)**
- Updates axon serving information for a neuron
- Parameters:
  - `origin`: Must be signed by registered hotkey
  - `netuid`: The subnet
  - `version`: Axon version number
  - `ip`: IP address (128-bit for IPv6 compatibility)
  - `port`: Port number
  - `ip_type`: IP type (4 for IPv4, 6 for IPv6)
  - `protocol`: Protocol type
  - `placeholder1`: Reserved for future use
  - `placeholder2`: Reserved for future use

**serve_prometheus(origin, netuid: NetUid, version: u32, ip: u128, port: u16, ip_type: u8)**
- Updates Prometheus serving information
- Parameters:
  - `origin`: Must be signed by registered hotkey
  - `netuid`: The subnet
  - `version`: Prometheus version
  - `ip`: IP address
  - `port`: Port number
  - `ip_type`: IP type

#### Delegation Management

**become_delegate(origin, hotkey: T::AccountId, take: u16)**
- Makes a hotkey a delegate
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The hotkey to become delegate
  - `take`: Percentage taken by delegate (in basis points, e.g., 1000 = 10%)

**remove_delegate(origin, hotkey: T::AccountId)**
- Removes delegate status from a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The delegate hotkey to remove

**increase_take(origin, hotkey: T::AccountId, take: u16)**
- Increases the take rate of a delegate
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The delegate hotkey
  - `take`: New take rate (must be higher than current)

**decrease_take(origin, hotkey: T::AccountId, take: u16)**
- Decreases the take rate of a delegate
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The delegate hotkey
  - `take`: New take rate (must be lower than current)

#### Key Management

**swap_coldkey(origin, old_coldkey: T::AccountId, new_coldkey: T::AccountId, swap_cost: TaoCurrency)**
- Swaps a coldkey (usually sudo-scheduled)
- Parameters:
  - `origin`: Usually Root origin
  - `old_coldkey`: The coldkey to replace
  - `new_coldkey`: The new coldkey
  - `swap_cost`: TAO cost for the swap

**swap_hotkey(origin, old_hotkey: T::AccountId, new_hotkey: T::AccountId)**
- Swaps a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `old_hotkey`: The hotkey to replace
  - `new_hotkey`: The new hotkey

**add_childkey(origin, hotkey: T::AccountId, childkey: T::AccountId, proportion: u64)**
- Adds a childkey to a hotkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The parent hotkey
  - `childkey`: The childkey to add
  - `proportion`: Proportion of stake assigned (u64 representation)

**remove_childkey(origin, hotkey: T::AccountId, childkey: T::AccountId)**
- Removes a childkey
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The parent hotkey
  - `childkey`: The childkey to remove

#### Subnet Management

**register_network(origin)**
- Registers a new subnet
- Parameters:
  - `origin`: Must be signed and pay registration cost

**dissolve_network(origin, netuid: NetUid)**
- Dissolves a subnet
- Parameters:
  - `origin`: Must be signed by subnet owner or root
  - `netuid`: The subnet to dissolve

**set_subnet_identity(origin, netuid: NetUid, identity: SubnetIdentityV3)**
- Sets identity information for a subnet
- Parameters:
  - `origin`: Must be signed by subnet owner
  - `netuid`: The subnet
  - `identity`: Identity information structure

#### EVM Integration

**associate_evm_key(origin, netuid: NetUid, evm_key: H160, block_number: u64, signature: Signature)**
- Associates an EVM address with a hotkey
- Parameters:
  - `origin`: Must be signed by registered hotkey
  - `netuid`: The subnet
  - `evm_key`: Ethereum address to associate
  - `block_number`: Block number for replay protection
  - `signature`: Signature proving ownership of EVM key

#### Token Operations

**recycle_alpha(origin, hotkey: T::AccountId, amount: AlphaCurrency, netuid: NetUid)**
- Recycles Alpha tokens
- Parameters:
  - `origin`: Must be signed
  - `hotkey`: The hotkey recycling
  - `amount`: Amount to recycle
  - `netuid`: The subnet
- Notes: Fees can be paid in Alpha

**burn_alpha(origin, hotkey: T::AccountId, amount: AlphaCurrency, netuid: NetUid)**
- Burns Alpha tokens
- Parameters:
  - `origin`: Must be signed
  - `hotkey`: The hotkey burning
  - `amount`: Amount to burn
  - `netuid`: The subnet
- Notes: Fees can be paid in Alpha

#### Subnet Parameters (Many require sudo or subnet owner)

**sudo_set_tempo(origin, netuid: NetUid, tempo: u16)**
- Sets the tempo (blocks between epochs) for a subnet
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `tempo`: New tempo value

**sudo_set_kappa(origin, netuid: NetUid, kappa: u16)**
- Sets kappa parameter
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `kappa`: New kappa value

**sudo_set_rho(origin, netuid: NetUid, rho: u16)**
- Sets rho parameter
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `rho`: New rho value

**sudo_set_difficulty(origin, netuid: NetUid, difficulty: u64)**
- Sets registration difficulty
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `difficulty`: New difficulty

**sudo_set_max_allowed_uids(origin, netuid: NetUid, max_allowed_uids: u16)**
- Sets maximum UIDs in subnet
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `max_allowed_uids`: New maximum

**sudo_set_min_allowed_weights(origin, netuid: NetUid, min_allowed_weights: u16)**
- Sets minimum allowed weights
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `min_allowed_weights`: New minimum

**sudo_set_max_weight_limit(origin, netuid: NetUid, max_weight_limit: u16)**
- Sets maximum weight limit
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `max_weight_limit`: New maximum

**sudo_set_commit_reveal_enabled(origin, netuid: NetUid, enabled: bool)**
- Enables or disables commit-reveal for weight setting
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `enabled`: True to enable, false to disable

**sudo_set_liquid_alpha_enabled(origin, netuid: NetUid, enabled: bool)**
- Enables or disables liquid alpha
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `enabled`: True to enable, false to disable

### Key Storage Items

- **TotalSubnets**: Number of active subnets
- **NetworksAdded**: Tracking for subnet creation
- **SubnetOwner**: Maps NetUid to owner AccountId
- **Stake**: Maps (Coldkey, Hotkey, NetUid) to staked amount
- **Delegates**: Maps hotkeys to delegate information
- **TotalStake**: Total stake per hotkey
- **Weights**: Weight assignments by validators
- **Uids**: Maps hotkey to UID within subnet
- **Keys**: Maps UID to hotkey within subnet
- **Axons**: Axon serving information
- **Prometheus**: Prometheus serving information

### Custom Types

**AxonInfo**
- `version`: u32
- `ip`: u128
- `port`: u16
- `ip_type`: u8
- `protocol`: u8

**PrometheusInfo**
- `version`: u32
- `ip`: u128
- `port`: u16
- `ip_type`: u8

**SubnetIdentityV3**
- Identity fields for subnet metadata
- Includes name, description, links, etc.

---

## Pallet Admin Utils

Administrative pallet for managing network hyperparameters and configurations. Most extrinsics require root origin or subnet owner permissions.

### Events

**PrecompileUpdated { precompile_id: PrecompileEnum, enabled: bool }**
- Emitted when a precompile contract is enabled/disabled
- Parameters:
  - `precompile_id`: The precompile identifier
  - `enabled`: Whether it's now enabled

**Yuma3EnableToggled { netuid: NetUid, enabled: bool }**
- Emitted when Yuma3 consensus is toggled
- Parameters:
  - `netuid`: The subnet
  - `enabled`: New state

**BondsResetToggled { netuid: NetUid, enabled: bool }**
- Emitted when bonds reset is toggled
- Parameters:
  - `netuid`: The subnet
  - `enabled`: New state

### Extrinsics

**sudo_set_default_take(origin, default_take: u16)**
- Sets global default delegate take rate
- Parameters:
  - `origin`: Must be Root
  - `default_take`: Default take percentage in basis points

**sudo_set_serving_rate_limit(origin, netuid: NetUid, serving_rate_limit: u64)**
- Sets rate limit for serving endpoint updates
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `serving_rate_limit`: Number of blocks between updates

**sudo_set_max_allowed_uids(origin, netuid: NetUid, max_allowed_uids: u16)**
- Sets maximum UIDs allowed in subnet
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `max_allowed_uids`: New maximum (must be reasonable)

**sudo_set_liquid_alpha_enabled(origin, netuid: NetUid, enabled: bool)**
- Toggles liquid alpha feature
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `enabled`: True to enable liquid alpha

**sudo_set_evm_chain_id(origin, chain_id: u64)**
- Sets the EVM chain ID
- Parameters:
  - `origin`: Must be Root
  - `chain_id`: The EVM chain identifier

**sudo_set_weights_set_rate_limit(origin, netuid: NetUid, weights_set_rate_limit: u64)**
- Sets rate limit for weight setting operations
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `weights_set_rate_limit`: Blocks between weight settings

**sudo_set_bonds_moving_average(origin, netuid: NetUid, bonds_moving_average: u64)**
- Sets bonds moving average parameter
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `bonds_moving_average`: New moving average value

**sudo_set_adjustment_alpha(origin, netuid: NetUid, adjustment_alpha: u64)**
- Sets adjustment alpha parameter
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `adjustment_alpha`: New alpha value

**sudo_set_target_registrations_per_interval(origin, netuid: NetUid, target_registrations_per_interval: u16)**
- Sets target registrations per interval
- Parameters:
  - `origin`: Root or subnet owner
  - `netuid`: The subnet
  - `target_registrations_per_interval`: Target number

### Errors

- **SubnetDoesNotExist**: The specified subnet doesn't exist
- **MaxValidatorsLargerThanMaxUIds**: Invalid validator configuration
- **MaxAllowedUIdsLessThanCurrentUIds**: Can't reduce max UIDs below current count
- **BondsMovingAverageMaxReached**: Bonds MA value too high
- **NegativeSigmoidSteepness**: Invalid sigmoid parameter
- **ValueNotInBounds**: Parameter value outside allowed range
- **InvalidValue**: Generic invalid value error

---

## Pallet Commitments

Manages data commitments, including timelock-encrypted commitments that are automatically revealed using Drand randomness.

### Events

**Commitment { netuid: NetUid, who: T::AccountId }**
- Emitted when a regular commitment is made
- Parameters:
  - `netuid`: The subnet
  - `who`: The account making the commitment

**TimelockCommitment { netuid: NetUid, who: T::AccountId, reveal_round: u64 }**
- Emitted when a timelock commitment is made
- Parameters:
  - `netuid`: The subnet
  - `who`: The account making the commitment
  - `reveal_round`: Drand round when it will be revealed

**CommitmentRevealed { netuid: NetUid, who: T::AccountId }**
- Emitted when a timelock commitment is automatically revealed
- Parameters:
  - `netuid`: The subnet
  - `who`: The account whose commitment was revealed

### Extrinsics

**set_commitment(origin, netuid: NetUid, info: Box<CommitmentInfo<T::MaxFields>>)**
- Sets commitment data for an account
- Parameters:
  - `origin`: Must be signed by registered hotkey
  - `netuid`: The subnet
  - `info`: Commitment information structure containing data fields
- Notes: Can include timelock-encrypted data for automatic revelation

**set_max_space(origin, new_limit: u32)**
- Sets maximum storage space for commitments
- Parameters:
  - `origin`: Must be Root
  - `new_limit`: New space limit in bytes

### Storage

- **CommitmentOf**: Maps (NetUid, AccountId) to commitment data
- **LastCommitment**: Tracks when accounts last committed
- **RevealedCommitments**: Stores revealed timelock commitments
- **UsedSpaceOf**: Tracks storage usage per account
- **MaxSpace**: Maximum allowed commitment space

### Custom Types

**CommitmentInfo**
- Contains multiple `Data` fields
- Each field can be:
  - `Raw(BoundedVec)`: Raw data up to 128 bytes
  - `BigRaw(BoundedVec)`: Raw data up to 512 bytes
  - `TimelockEncrypted`: Timelock-encrypted data with reveal round
  - `ResetBondsFlag`: Special flag for resetting bonds
  - Various hash types (Blake2b, Keccak256, etc.)

### Errors

- **TooManyFieldsInCommitmentInfo**: Exceeded field limit
- **AccountNotAllowedCommit**: Account not registered or rate limited
- **SpaceLimitExceeded**: Commitment too large
- **UnexpectedUnreserveLeftover**: Internal accounting error

---

## Pallet Crowdloan

Generic crowdloan functionality for raising funds toward a goal, with automatic execution of a call on success.

### Events

**Created { crowdloan_id: CrowdloanId, creator: T::AccountId, end: BlockNumberFor<T>, cap: BalanceOf<T> }**
- Emitted when a new crowdloan is created
- Parameters:
  - `crowdloan_id`: Unique identifier for the crowdloan
  - `creator`: Account creating the crowdloan
  - `end`: Block number when crowdloan ends
  - `cap`: Funding goal amount

**Contributed { crowdloan_id: CrowdloanId, contributor: T::AccountId, amount: BalanceOf<T> }**
- Emitted when someone contributes
- Parameters:
  - `crowdloan_id`: The crowdloan
  - `contributor`: Account contributing
  - `amount`: Amount contributed

**Withdrew { crowdloan_id: CrowdloanId, contributor: T::AccountId, amount: BalanceOf<T> }**
- Emitted when contribution is withdrawn
- Parameters:
  - `crowdloan_id`: The crowdloan
  - `contributor`: Account withdrawing
  - `amount`: Amount withdrawn

**Finalized { crowdloan_id: CrowdloanId }**
- Emitted when crowdloan successfully reaches goal and executes
- Parameters:
  - `crowdloan_id`: The completed crowdloan

**Dissolved { crowdloan_id: CrowdloanId }**
- Emitted when crowdloan is dissolved and funds returned
- Parameters:
  - `crowdloan_id`: The dissolved crowdloan

**MinContributionUpdated**
**EndUpdated**
**CapUpdated**
- Emitted when crowdloan parameters are updated

### Extrinsics

**create(origin, deposit: BalanceOf<T>, min_contribution: BalanceOf<T>, cap: BalanceOf<T>, end: BlockNumberFor<T>, call: Option<Box<RuntimeCall>>, target_address: Option<T::AccountId>)**
- Creates a new crowdloan
- Parameters:
  - `origin`: Must be signed
  - `deposit`: Deposit amount (refundable)
  - `min_contribution`: Minimum contribution amount
  - `cap`: Funding goal
  - `end`: End block
  - `call`: Optional call to execute on success
  - `target_address`: Optional target address for funds

**contribute(origin, crowdloan_id: CrowdloanId, amount: BalanceOf<T>)**
- Contributes to a crowdloan
- Parameters:
  - `origin`: Must be signed
  - `crowdloan_id`: The crowdloan to contribute to
  - `amount`: Amount to contribute

**withdraw(origin, crowdloan_id: CrowdloanId)**
- Withdraws contribution if crowdloan failed or not yet finalized
- Parameters:
  - `origin`: Must be signed by contributor
  - `crowdloan_id`: The crowdloan

**finalize(origin, crowdloan_id: CrowdloanId)**
- Finalizes a successful crowdloan, executing its call
- Parameters:
  - `origin`: Can be called by anyone
  - `crowdloan_id`: The crowdloan to finalize

**refund(origin, crowdloan_id: CrowdloanId)**
- Refunds contributions from a failed crowdloan
- Parameters:
  - `origin`: Can be called by anyone
  - `crowdloan_id`: The crowdloan to refund

**dissolve(origin, crowdloan_id: CrowdloanId)**
- Dissolves a crowdloan and returns deposit
- Parameters:
  - `origin`: Must be creator or root
  - `crowdloan_id`: The crowdloan to dissolve

**update_min_contribution(origin, crowdloan_id: CrowdloanId, min_contribution: BalanceOf<T>)**
**update_end(origin, crowdloan_id: CrowdloanId, end: BlockNumberFor<T>)**
**update_cap(origin, crowdloan_id: CrowdloanId, cap: BalanceOf<T>)**
- Updates crowdloan parameters
- Must be called by creator before finalization

### Storage

- **Crowdloans**: Maps CrowdloanId to crowdloan details
- **Contributions**: Maps (CrowdloanId, AccountId) to contribution amount
- **NextCrowdloanId**: Counter for next crowdloan ID

### Errors

- **DepositTooLow**: Insufficient deposit
- **CapTooLow**: Cap below minimum
- **CannotEndInPast**: Invalid end block
- **InsufficientBalance**: Not enough balance to contribute
- **InvalidCrowdloanId**: Crowdloan doesn't exist
- **CapRaised**: Already reached goal
- **ContributionPeriodEnded**: Too late to contribute
- **ContributionTooLow**: Below minimum contribution
- **AlreadyFinalized**: Already completed
- **ContributionPeriodNotEnded**: Too early to finalize
- **NoContribution**: No contribution to withdraw
- **MaxContributorsReached**: Too many contributors

---

## Pallet Drand

Integrates Drand distributed randomness beacon. An offchain worker fetches pulses and submits them as unsigned transactions.

### Events

**BeaconConfigChanged**
- Emitted when Drand beacon configuration is updated
- No parameters

**NewPulse { rounds: Vec<RoundNumber> }**
- Emitted when new Drand pulses are stored
- Parameters:
  - `rounds`: Vector of round numbers that were added

**SetOldestStoredRound(u64)**
- Emitted when oldest stored round changes
- Parameters:
  - `u64`: New oldest round number

### Extrinsics

**write_pulse(origin, pulses_payload: PulsesPayload<T::Public, BlockNumberFor<T>>, _signature: Option<T::Signature>)**
- Writes verified Drand pulses to storage (unsigned transaction)
- Parameters:
  - `origin`: Unsigned origin (submitted by offchain worker)
  - `pulses_payload`: Contains pulses and metadata
  - `_signature`: Optional signature (not used)
- Notes: Automatically called by offchain worker

**set_beacon_config(origin, config_payload: BeaconConfigurationPayload<T::Public, BlockNumberFor<T>>, _signature: Option<T::Signature>)**
- Updates Drand beacon configuration
- Parameters:
  - `origin`: Must be Root
  - `config_payload`: New beacon configuration
  - `_signature`: Optional signature

**set_oldest_stored_round(origin, oldest_round: u64)**
- Sets which old rounds to prune
- Parameters:
  - `origin`: Must be Root
  - `oldest_round`: Round number to keep as oldest

### Storage

- **BeaconConfig**: Current Drand beacon configuration
- **Pulses**: Maps round number to pulse data (randomness)
- **LastStoredRound**: Most recent round stored
- **OldestStoredRound**: Oldest round kept in storage
- **NextUnsignedAt**: Rate limiting for unsigned transactions

### Custom Types

**Pulse**
- `round`: Round number
- `randomness`: The random value (32 bytes)
- `signature`: BLS signature proving validity

**BeaconConfigurationPayload**
- Contains public key and genesis time of Drand beacon

### Errors

- **NoneValue**: Missing expected value
- **StorageOverflow**: Too many pulses stored
- **DrandConnectionFailure**: Offchain worker couldn't reach Drand
- **UnverifiedPulse**: Signature verification failed
- **InvalidRoundNumber**: Round number out of sequence
- **PulseVerificationError**: General verification error

---

## Pallet Proxy

Enables accounts to grant limited control to proxy accounts. Supports time delays and "pure" proxy accounts.

### Events

**ProxyExecuted { result: DispatchResult }**
- Emitted when a proxy executes a call
- Parameters:
  - `result`: Success or error of the dispatched call

**PureCreated { pure: T::AccountId, who: T::AccountId, proxy_type: T::ProxyType, disambiguation_index: u16 }**
- Emitted when a pure proxy account is created
- Parameters:
  - `pure`: The new pure proxy account
  - `who`: The creator/controller
  - `proxy_type`: Type of proxy permissions
  - `disambiguation_index`: Index for generating unique address

**PureKilled { pure: T::AccountId, spawner: T::AccountId, proxy_type: T::ProxyType, disambiguation_index: u16 }**
- Emitted when a pure proxy is destroyed
- Parameters:
  - `pure`: The pure proxy account destroyed
  - `spawner`: The original creator
  - `proxy_type`: The proxy type
  - `disambiguation_index`: The index used

**Announced { real: T::AccountId, proxy: T::AccountId, call_hash: CallHashOf<T> }**
- Emitted when a proxy announces a delayed call
- Parameters:
  - `real`: The real account
  - `proxy`: The proxy announcing
  - `call_hash`: Hash of the call to be executed

**ProxyAdded { delegator: T::AccountId, delegatee: T::AccountId, proxy_type: T::ProxyType, delay: BlockNumberFor<T> }**
- Emitted when a proxy relationship is created
- Parameters:
  - `delegator`: The account granting proxy rights
  - `delegatee`: The proxy account
  - `proxy_type`: Type of permissions granted
  - `delay`: Number of blocks of delay for execution

**ProxyRemoved { delegator: T::AccountId, delegatee: T::AccountId, proxy_type: T::ProxyType, delay: BlockNumberFor<T> }**
- Emitted when a proxy relationship is removed
- Parameters: Same as ProxyAdded

**DepositPoked { who: T::AccountId, kind: DepositKind, old_deposit: BalanceOf<T>, new_deposit: BalanceOf<T> }**
- Emitted when deposit is recalculated
- Parameters:
  - `who`: The account
  - `kind`: Type of deposit (proxy or announcement)
  - `old_deposit`: Previous deposit amount
  - `new_deposit`: New deposit amount

### Extrinsics

**proxy(origin, real: AccountIdLookupOf<T>, force_proxy_type: Option<T::ProxyType>, call: Box<RuntimeCall>)**
- Executes a call as a proxy
- Parameters:
  - `origin`: Must be signed by proxy account
  - `real`: The account being proxied
  - `force_proxy_type`: Optional specific proxy type to use
  - `call`: The call to execute

**add_proxy(origin, delegate: AccountIdLookupOf<T>, proxy_type: T::ProxyType, delay: BlockNumberFor<T>)**
- Adds a proxy relationship
- Parameters:
  - `origin`: Must be signed by account granting proxy
  - `delegate`: The proxy account to add
  - `proxy_type`: Type of permissions (e.g., Any, NonTransfer, Staking, etc.)
  - `delay`: Blocks of delay before proxy can execute

**remove_proxy(origin, delegate: AccountIdLookupOf<T>, proxy_type: T::ProxyType, delay: BlockNumberFor<T>)**
- Removes a proxy relationship
- Parameters:
  - `origin`: Must be signed by delegator
  - `delegate`: The proxy to remove
  - `proxy_type`: Must match existing relationship
  - `delay`: Must match existing relationship

**remove_proxies(origin)**
- Removes all proxy relationships for caller
- Parameters:
  - `origin`: Must be signed

**create_pure(origin, proxy_type: T::ProxyType, delay: BlockNumberFor<T>, index: u16)**
- Creates a pure proxy account
- Parameters:
  - `origin`: Must be signed
  - `proxy_type`: Permissions for the pure proxy
  - `delay`: Delay for operations
  - `index`: Disambiguation index for address generation
- Notes: Pure proxies are accounts that can only be controlled through proxy calls

**kill_pure(origin, spawner: AccountIdLookupOf<T>, proxy_type: T::ProxyType, index: u16, height: BlockNumberFor<T>, ext_index: u32)**
- Destroys a pure proxy account
- Parameters:
  - `origin`: Must be signed by controller
  - `spawner`: Original creator account
  - `proxy_type`: Must match creation parameters
  - `index`: Must match creation parameters
  - `height`: Block height of creation
  - `ext_index`: Extrinsic index of creation

**announce(origin, real: AccountIdLookupOf<T>, call_hash: CallHashOf<T>)**
- Announces a delayed proxy call
- Parameters:
  - `origin`: Must be signed by proxy
  - `real`: The real account
  - `call_hash`: Hash of the call to execute later

**remove_announcement(origin, real: AccountIdLookupOf<T>, call_hash: CallHashOf<T>)**
- Removes an announcement
- Parameters:
  - `origin`: Must be signed by proxy
  - `real`: The real account
  - `call_hash`: Hash of announced call

**reject_announcement(origin, delegate: AccountIdLookupOf<T>, call_hash: CallHashOf<T>)**
- Rejects an announced call
- Parameters:
  - `origin`: Must be signed by real account
  - `delegate`: The proxy that announced
  - `call_hash`: Hash of announced call

**proxy_announced(origin, delegate: AccountIdLookupOf<T>, real: AccountIdLookupOf<T>, force_proxy_type: Option<T::ProxyType>, call: Box<RuntimeCall>)**
- Executes a previously announced call
- Parameters:
  - `origin`: Must be signed
  - `delegate`: The proxy account
  - `real`: The real account
  - `force_proxy_type`: Optional proxy type
  - `call`: The call to execute (must match announcement)

**poke_deposit(origin)**
- Recalculates and updates deposits
- Parameters:
  - `origin`: Must be signed

### Storage

- **Proxies**: Maps AccountId to list of proxy relationships
- **Announcements**: Maps AccountId to announced calls

### Custom Types

**ProxyDefinition**
- `delegate`: The proxy account
- `proxy_type`: Type of permissions
- `delay`: Blocks of delay

**ProxyType** (enum, configured per runtime)
- Common types: Any, NonTransfer, Governance, Staking, etc.

### Errors

- **TooMany**: Too many proxies or announcements
- **NotFound**: Proxy relationship doesn't exist
- **NotProxy**: Account is not a proxy
- **Unproxyable**: Call cannot be proxied
- **Duplicate**: Proxy already exists
- **NoPermission**: Proxy type doesn't allow this call
- **Unannounced**: Call hasn't been announced
- **NoSelfProxy**: Cannot proxy to self
- **InvalidDerivedAccountId**: Pure proxy address derivation failed

---

## Pallet Registry

On-chain identity registry for accounts. Allows setting display names, legal names, web links, email, and other metadata.

### Events

**IdentitySet { who: T::AccountId }**
- Emitted when identity information is set
- Parameters:
  - `who`: The account whose identity was set

**IdentityDissolved { who: T::AccountId }**
- Emitted when identity is cleared
- Parameters:
  - `who`: The account whose identity was cleared

### Extrinsics

**set_identity(origin, identified: T::AccountId, info: Box<IdentityInfo<T::MaxAdditionalFields>>)**
- Sets identity information for an account
- Parameters:
  - `origin`: Must be signed (usually by the account or authorized party)
  - `identified`: The account to set identity for
  - `info`: Identity information structure
- Notes: Requires deposit

**clear_identity(origin, identified: T::AccountId)**
- Clears identity information
- Parameters:
  - `origin`: Must be signed by authorized party
  - `identified`: The account to clear identity for
- Notes: Returns deposit

### Storage

- **IdentityOf**: Maps AccountId to identity information

### Custom Types

**IdentityInfo**
- `display`: Display name
- `legal`: Legal name
- `web`: Website URL
- `email`: Email address
- `pgp_fingerprint`: PGP fingerprint
- `image`: Image URL
- `twitter`: Twitter handle
- `github`: GitHub username
- `discord`: Discord username
- `additional`: Additional custom fields

**Data** (used for each field)
- `None`: No data
- `Raw(BoundedVec)`: Raw bytes up to limit
- `BlakeTwo256(H256)`: Blake2 256-bit hash
- `Sha256(H256)`: SHA-256 hash
- `Keccak256(H256)`: Keccak-256 hash
- `ShaThree256(H256)`: SHA3-256 hash

**Registration**
- `info`: The identity information
- `deposit`: Amount deposited

### Errors

- **CannotRegister**: Registration not allowed
- **TooManyFieldsInIdentityInfo**: Exceeded maximum fields
- **NotRegistered**: No identity set for this account

---

## Pallet Shield

MEV protection mechanism using encrypted transactions. Block authors can decrypt and execute transactions.

### Events

**EncryptedSubmitted { id: T::Hash, who: T::AccountId }**
- Emitted when encrypted transaction is submitted
- Parameters:
  - `id`: Hash identifying the submission
  - `who`: Account submitting

**DecryptedExecuted { id: T::Hash, signer: T::AccountId }**
- Emitted when encrypted transaction is successfully decrypted and executed
- Parameters:
  - `id`: Submission hash
  - `signer`: Original signer of the transaction

**DecryptedRejected { id: T::Hash, reason: DispatchErrorWithPostInfo }**
- Emitted when decrypted transaction fails execution
- Parameters:
  - `id`: Submission hash
  - `reason`: Why execution failed

**DecryptionFailed { id: T::Hash, reason: BoundedVec<u8, ConstU32<256>> }**
- Emitted when decryption fails
- Parameters:
  - `id`: Submission hash
  - `reason`: Error message

### Extrinsics

**announce_next_key(origin, public_key: BoundedVec<u8, ConstU32<2048>>)**
- Block author announces next epoch's public key
- Parameters:
  - `origin`: Must be block author
  - `public_key`: RSA or other public key for encryption
- Notes: Usually called automatically

**submit_encrypted(origin, commitment: T::Hash, ciphertext: BoundedVec<u8, ConstU32<8192>>)**
- Submits encrypted transaction
- Parameters:
  - `origin`: Must be signed
  - `commitment`: Hash commitment to the plaintext
  - `ciphertext`: Encrypted transaction data

**mark_decryption_failed(origin, id: T::Hash, reason: BoundedVec<u8, ConstU32<256>>)**
- Block author marks a decryption attempt as failed
- Parameters:
  - `origin`: Must be block author
  - `id`: Submission hash
  - `reason`: Why decryption failed

### Storage

- **CurrentKey**: Current epoch's public key
- **NextKey**: Next epoch's public key
- **Submissions**: Maps submission hash to encrypted data
- **KeyHashByBlock**: Maps block number to key hash

### Errors

- **SubmissionAlreadyExists**: Duplicate submission
- **MissingSubmission**: Submission not found
- **CommitmentMismatch**: Decrypted data doesn't match commitment
- **SignatureInvalid**: Invalid signature on decrypted transaction
- **BadPublicKeyLen**: Public key wrong length
- **KeyExpired**: Encryption key no longer valid
- **KeyHashMismatch**: Key hash doesn't match expected

---

## Pallet Swap

Uniswap V3-style concentrated liquidity AMM for TAO/Alpha token pairs per subnet.

### Events

**FeeRateSet { netuid: NetUid, rate: u16 }**
- Emitted when fee rate is changed
- Parameters:
  - `netuid`: The subnet
  - `rate`: Fee rate in basis points

**UserLiquidityToggled { netuid: NetUid, enable: bool }**
- Emitted when user-provided liquidity is enabled/disabled
- Parameters:
  - `netuid`: The subnet
  - `enable`: New state

**LiquidityAdded { coldkey: T::AccountId, hotkey: T::AccountId, netuid: NetUid, position_id: PositionId, liquidity: u64, tao: TaoCurrency, alpha: AlphaCurrency, tick_low: TickIndex, tick_high: TickIndex }**
- Emitted when liquidity is added
- Parameters:
  - `coldkey`: Account providing liquidity
  - `hotkey`: Associated hotkey
  - `netuid`: The subnet
  - `position_id`: Unique ID for this position
  - `liquidity`: Liquidity amount in abstract units
  - `tao`: TAO amount added
  - `alpha`: Alpha amount added
  - `tick_low`: Lower price bound (tick index)
  - `tick_high`: Upper price bound (tick index)

**LiquidityRemoved { coldkey: T::AccountId, hotkey: T::AccountId, netuid: NetUid, position_id: PositionId, liquidity: u64, tao: TaoCurrency, alpha: AlphaCurrency }**
- Emitted when liquidity is removed
- Parameters:
  - `coldkey`: Account removing liquidity
  - `hotkey`: Associated hotkey
  - `netuid`: The subnet
  - `position_id`: Position being closed
  - `liquidity`: Liquidity amount removed
  - `tao`: TAO amount withdrawn
  - `alpha`: Alpha amount withdrawn

**LiquidityModified { coldkey: T::AccountId, hotkey: T::AccountId, netuid: NetUid, position_id: PositionId, liquidity_delta: i64, tao: TaoCurrency, alpha: AlphaCurrency }**
- Emitted when existing position is modified
- Parameters:
  - `coldkey`: Account modifying
  - `hotkey`: Associated hotkey
  - `netuid`: The subnet
  - `position_id`: Position being modified
  - `liquidity_delta`: Change in liquidity (positive or negative)
  - `tao`: TAO change
  - `alpha`: Alpha change

### Extrinsics

**set_fee_rate(origin, netuid: NetUid, rate: u16)**
- Sets swap fee rate
- Parameters:
  - `origin`: Must be Root or subnet owner
  - `netuid`: The subnet
  - `rate`: Fee rate in basis points (e.g., 30 = 0.3%)

**toggle_user_liquidity(origin, netuid: NetUid, enable: bool)**
- Enables or disables user-provided liquidity
- Parameters:
  - `origin`: Must be Root or subnet owner
  - `netuid`: The subnet
  - `enable`: True to allow user liquidity

**add_liquidity(origin, hotkey: T::AccountId, netuid: NetUid, tick_low: TickIndex, tick_high: TickIndex, liquidity: u64)**
- Adds liquidity to a price range
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The associated hotkey
  - `netuid`: The subnet
  - `tick_low`: Lower price tick (e.g., -887272)
  - `tick_high`: Upper price tick (e.g., 887272)
  - `liquidity`: Amount of liquidity to add
- Notes: Ticks represent prices on a log scale

**remove_liquidity(origin, hotkey: T::AccountId, netuid: NetUid, position_id: PositionId)**
- Removes all liquidity from a position
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The associated hotkey
  - `netuid`: The subnet
  - `position_id`: Position to close

**modify_position(origin, hotkey: T::AccountId, netuid: NetUid, position_id: PositionId, liquidity_delta: i64)**
- Increases or decreases liquidity in existing position
- Parameters:
  - `origin`: Must be signed by coldkey
  - `hotkey`: The associated hotkey
  - `netuid`: The subnet
  - `position_id`: Position to modify
  - `liquidity_delta`: Positive to add, negative to remove

### Storage

- **FeeRate**: Fee rate per subnet
- **FeeGlobalTao**: Accumulated TAO fees
- **FeeGlobalAlpha**: Accumulated Alpha fees
- **Ticks**: Tick state (liquidity, fee growth)
- **AlphaSqrtPrice**: Current sqrt price of Alpha in terms of TAO
- **CurrentTick**: Current price tick
- **CurrentLiquidity**: Active liquidity at current price
- **EnabledUserLiquidity**: Whether users can provide liquidity
- **Positions**: User liquidity positions
- **LastPositionId**: Counter for position IDs
- **TickIndexBitmapWords**: Bitmap for efficient tick lookup
- **ScrapReservoirTao**: Small amounts that couldn't be distributed
- **ScrapReservoirAlpha**: Small amounts that couldn't be distributed

### Custom Types

**TickIndex**
- i32 representing log price
- Range typically -887272 to 887272
- Each tick represents a 0.01% price change

**PositionId**
- u64 unique identifier for a liquidity position

**SwapResult**
- `amount_in`: Amount of input token
- `amount_out`: Amount of output token
- `fee`: Fee charged

### Errors

- **FeeRateTooHigh**: Fee rate exceeds maximum
- **InsufficientInputAmount**: Not enough tokens provided
- **InsufficientLiquidity**: Not enough liquidity for swap
- **PriceLimitExceeded**: Price moved beyond limit
- **InsufficientBalance**: Account doesn't have enough tokens
- **LiquidityNotFound**: Position doesn't exist
- **InvalidTickRange**: tick_low >= tick_high or out of bounds
- **MaxPositionsExceeded**: Too many positions per account
- **TooManySwapSteps**: Swap crosses too many ticks
- **InvalidLiquidityValue**: Liquidity amount invalid
- **ReservesTooLow**: Insufficient reserves
- **MechanismDoesNotExist**: Subnet doesn't exist
- **UserLiquidityDisabled**: User liquidity not allowed
- **SubtokenDisabled**: Liquid alpha not enabled

---

## Pallet Transaction Fee

Custom transaction fee handler that allows certain operations to pay fees in Alpha instead of TAO.

### Key Concepts

This pallet doesn't have its own events or extrinsics. It implements the `OnChargeTransaction` trait to intercept and customize fee payment.

### Fee Payment Rules

**Operations that can pay fees in Alpha:**
- `remove_stake`
- `unstake_all`
- `move_stake`
- `transfer_stake`
- `swap_stake`
- `recycle_alpha`
- `burn_alpha`

**Fee Calculation:**
1. Transaction fee is calculated normally in TAO
2. If the extrinsic is one of the Alpha-fee-eligible calls:
   - Fee is converted to Alpha using current swap rate
   - Alpha is burned from the hotkey's balance
   - Any TAO fee taken is refunded
3. Otherwise, fee is paid in TAO as normal

### Integration

This pallet works with:
- **pallet-subtensor**: To access hotkey balances
- **pallet-swap**: To get Alpha/TAO exchange rates
- **pallet-transaction-payment**: Base fee calculation

---

## Pallet Utility

Provides utility functions for batching calls and dispatching from derived accounts.

### Events

**BatchInterrupted { index: u32, error: DispatchError }**
- Emitted when batch stops on error
- Parameters:
  - `index`: Which call in the batch failed
  - `error`: The error that occurred

**BatchCompleted**
- Emitted when all calls in batch succeed
- No parameters

**BatchCompletedWithErrors**
- Emitted when batch completes but some calls failed
- No parameters

**ItemCompleted**
- Emitted after each successful call in force_batch
- No parameters

**ItemFailed { error: DispatchError }**
- Emitted after each failed call in force_batch
- Parameters:
  - `error`: The error that occurred

**DispatchedAs { result: DispatchResult }**
- Emitted when dispatch_as executes a call
- Parameters:
  - `result`: Success or error of dispatched call

**IfElseMainSuccess**
- Emitted when if_else main branch succeeds
- No parameters

**IfElseFallbackCalled { main_error: DispatchError }**
- Emitted when if_else falls back
- Parameters:
  - `main_error`: Error from main branch

### Extrinsics

**batch(origin, calls: Vec<RuntimeCall>)**
- Executes multiple calls in sequence, stops on first error
- Parameters:
  - `origin`: Must be signed
  - `calls`: Vector of calls to execute
- Notes: All calls use same origin. Stops at first error.

**as_derivative(origin, index: u16, call: Box<RuntimeCall>)**
- Dispatches call from a derived account
- Parameters:
  - `origin`: Must be signed
  - `index`: Derivation index (0-65535)
  - `call`: Call to execute
- Notes: Derived account is deterministically generated from origin + index

**batch_all(origin, calls: Vec<RuntimeCall>)**
- Executes all calls, reverts all if any fail
- Parameters:
  - `origin`: Must be signed
  - `calls`: Vector of calls to execute
- Notes: Atomically executes all or reverts all

**dispatch_as(origin, as_origin: Box<T::PalletsOrigin>, call: Box<RuntimeCall>)**
- Dispatches call as a different origin
- Parameters:
  - `origin`: Must be Root
  - `as_origin`: Origin to dispatch as (can be Root, Signed, None, etc.)
  - `call`: Call to execute
- Notes: Root only, allows dispatching as any origin

**force_batch(origin, calls: Vec<RuntimeCall>)**
- Executes all calls, continues on errors
- Parameters:
  - `origin`: Must be signed
  - `calls`: Vector of calls to execute
- Notes: Executes all calls regardless of failures

**with_weight(origin, call: Box<RuntimeCall>, weight: Weight)**
- Executes call with custom weight limit
- Parameters:
  - `origin`: Must be Root
  - `call`: Call to execute
  - `weight`: Weight limit override
- Notes: Root only, for benchmarking/testing

### Errors

- **TooManyCalls**: Batch size exceeds maximum
- **InvalidDerivedAccount**: as_derivative account derivation failed

---

## Common Types and Concepts

### Account Types

**T::AccountId**
- Generic account identifier
- Usually 32-byte SS58 address
- Can be either coldkey or hotkey depending on context

**H160**
- 160-bit Ethereum address
- Used for EVM key associations

### Currency Types

**TaoCurrency**
- Native token of the chain
- Used for staking, fees, and network security
- Measured in atomic units (1 TAO = 10^9 atomic units)

**AlphaCurrency**
- Subnet-specific token
- Earned through subnet participation
- Can be swapped for TAO
- Measured in atomic units

### Network Identifiers

**NetUid**
- u16 subnet identifier
- Each subnet has unique NetUid
- NetUid 0 is typically root network

**UID**
- u16 neuron identifier within a subnet
- Assigned upon registration
- Reused when neurons are replaced

### Block Numbers

**BlockNumberFor<T>**
- Block number type
- Used for timestamps, delays, rate limiting
- Monotonically increasing

### Hashes

**H256**
- 256-bit hash
- Used for commitments, identifiers
- Usually Blake2 hash

**CallHashOf<T>**
- Hash of an encoded call
- Used in proxy announcements

### Weight and Priority

**Weight**
- Represents computational cost
- Has ref_time (CPU) and proof_size (storage) components
- Used for fee calculation

### Bounded Collections

**BoundedVec<T, S>**
- Vector with maximum length S
- Prevents unbounded storage growth
- Fails if size exceeded

### Results

**DispatchResult**
- Ok(()) or Err(DispatchError)
- Result of executing an extrinsic

**DispatchError**
- Enum of possible errors
- Includes pallet errors, arithmetic errors, etc.

---

## Best Practices for Explorer/Wallet Development

### Querying Events

1. **Subscribe to finalized blocks** to get events reliably
2. **Filter by pallet and event name** for specific functionality
3. **Parse event parameters** according to type definitions
4. **Handle multiple events per block** - operations can emit several events

### Transaction Construction

1. **Get current nonce** from `api.rpc.system.accountNextIndex()`
2. **Estimate fees** before submitting
3. **Check rate limits** (serving, weights, commitments)
4. **Validate parameters** (e.g., UIDs exist, subnets exist)
5. **Sign with correct key** (coldkey vs hotkey)
6. **Watch for finalization** not just inclusion

### State Queries

1. **Use `api.query.<pallet>.<storage>()`** for current state
2. **Use `api.query.<pallet>.<storage>.multi()`** for batch queries
3. **Subscribe to storage changes** for real-time updates
4. **Query at specific block hash** for historical state

### Account Management

1. **Distinguish coldkey from hotkey**
   - Coldkey: Controls funds and can be offline
   - Hotkey: Participates in network, should be online
2. **Track both TAO and Alpha balances** per subnet
3. **Display stake per (coldkey, hotkey, netuid) tuple**
4. **Show delegate status and take rate**

### Subnet-Specific Features

1. **Check if commit-reveal is enabled** before showing weight setting UI
2. **Check if liquid alpha is enabled** before showing Alpha operations
3. **Query subnet hyperparameters** to show rate limits, tempo, etc.
4. **Display current tick and price** for swap functionality

### Error Handling

1. **Parse error types** from DispatchError
2. **Show user-friendly messages** for common errors
3. **Suggest solutions** (e.g., "insufficient stake" → "add more stake")
4. **Retry logic** for rate limit errors

### Fee Estimation

1. **Call `api.tx.<pallet>.<extrinsic>().paymentInfo(account)`** for fee estimate
2. **Check if Alpha fees apply** for eligible extrinsics
3. **Convert Alpha fees** using current swap rate
4. **Account for tip** if user adds one

### Swap Integration

1. **Query current price** from AlphaSqrtPrice storage
2. **Calculate expected output** using Uniswap V3 math
3. **Account for fees** (typically 0.3%)
4. **Set price limits** to prevent front-running
5. **Show price impact** for large swaps

---

## Additional Resources

### Polkadot.js API Usage

```javascript
// Connect to node
const api = await ApiPromise.create({ provider: wsProvider });

// Query storage
const stake = await api.query.subtensorModule.stake(coldkey, hotkey, netuid);

// Listen to events
api.query.system.events((events) => {
  events.forEach((record) => {
    const { event } = record;
    if (event.section === 'subtensorModule' && event.method === 'StakeAdded') {
      const [coldkey, hotkey, tao, alpha, netuid, block] = event.data;
      console.log('Stake added:', { coldkey, hotkey, tao, alpha, netuid, block });
    }
  });
});

// Submit transaction
const tx = api.tx.subtensorModule.addStake(hotkey, netuid, amount);
await tx.signAndSend(coldkey, ({ status, events }) => {
  if (status.isFinalized) {
    console.log('Finalized in block', status.asFinalized.toHex());
  }
});
```

### Type Definitions

All types are available through the chain's metadata:
- `api.registry.createType('NetUid', value)`
- `api.registry.createType('AxonInfo', value)`
- etc.

### Constants

Query on-chain constants:
- `api.consts.<pallet>.<constant>`
- Example: `api.consts.subtensorModule.maxAllowedUids`

---

This document provides a comprehensive reference for all pallets, their events, extrinsics, and parameters. Use this as a guide when building blockchain explorers, wallets, or any tools that interact with the Subtensor chain.




