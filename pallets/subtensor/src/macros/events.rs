use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the events for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod events {
    use codec::Compact;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// a new network is added.
        NetworkAdded(NetUid, u16),
        /// a network is removed.
        NetworkRemoved(NetUid),
        /// stake has been transferred from the a coldkey account onto the hotkey staking account.
        StakeAdded(
            T::AccountId,
            T::AccountId,
            TaoCurrency,
            AlphaCurrency,
            NetUid,
            u64,
        ),
        /// stake has been removed from the hotkey staking account onto the coldkey account.
        StakeRemoved(
            T::AccountId,
            T::AccountId,
            TaoCurrency,
            AlphaCurrency,
            NetUid,
            u64,
        ),
        /// stake has been moved from origin (hotkey, subnet ID) to destination (hotkey, subnet ID) of this amount (in TAO).
        StakeMoved(
            T::AccountId,
            T::AccountId,
            NetUid,
            T::AccountId,
            NetUid,
            TaoCurrency,
        ),
        /// a caller successfully sets their weights on a subnetwork.
        WeightsSet(NetUid, u16),
        /// a new neuron account has been registered to the chain.
        NeuronRegistered(NetUid, u16, T::AccountId),
        /// multiple uids have been concurrently registered.
        BulkNeuronsRegistered(u16, u16),
        /// FIXME: Not used yet
        BulkBalancesSet(u16, u16),
        /// max allowed uids has been set for a subnetwork.
        MaxAllowedUidsSet(NetUid, u16),
        /// the max weight limit has been set for a subnetwork.
        MaxWeightLimitSet(NetUid, u16),
        /// the difficulty has been set for a subnet.
        DifficultySet(NetUid, u64),
        /// the adjustment interval is set for a subnet.
        AdjustmentIntervalSet(NetUid, u16),
        /// registration per interval is set for a subnet.
        RegistrationPerIntervalSet(NetUid, u16),
        /// we set max registrations per block.
        MaxRegistrationsPerBlockSet(NetUid, u16),
        /// an activity cutoff is set for a subnet.
        ActivityCutoffSet(NetUid, u16),
        /// Rho value is set.
        RhoSet(NetUid, u16),
        /// steepness of the sigmoid used to compute alpha values.
        AlphaSigmoidSteepnessSet(NetUid, i16),
        /// Kappa is set for a subnet.
        KappaSet(NetUid, u16),
        /// minimum allowed weight is set for a subnet.
        MinAllowedWeightSet(NetUid, u16),
        /// the validator pruning length has been set.
        ValidatorPruneLenSet(NetUid, u64),
        /// the scaling law power has been set for a subnet.
        ScalingLawPowerSet(NetUid, u16),
        /// weights set rate limit has been set for a subnet.
        WeightsSetRateLimitSet(NetUid, u64),
        /// immunity period is set for a subnet.
        ImmunityPeriodSet(NetUid, u16),
        /// bonds moving average is set for a subnet.
        BondsMovingAverageSet(NetUid, u64),
        /// bonds penalty is set for a subnet.
        BondsPenaltySet(NetUid, u16),
        /// bonds reset is set for a subnet.
        BondsResetOnSet(NetUid, bool),
        /// setting the max number of allowed validators on a subnet.
        MaxAllowedValidatorsSet(NetUid, u16),
        /// the axon server information is added to the network.
        AxonServed(NetUid, T::AccountId),
        /// the prometheus server information is added to the network.
        PrometheusServed(NetUid, T::AccountId),
        /// a hotkey has become a delegate.
        DelegateAdded(T::AccountId, T::AccountId, u16),
        /// the default take is set.
        DefaultTakeSet(u16),
        /// weights version key is set for a network.
        WeightsVersionKeySet(NetUid, u64),
        /// setting min difficulty on a network.
        MinDifficultySet(NetUid, u64),
        /// setting max difficulty on a network.
        MaxDifficultySet(NetUid, u64),
        /// setting the prometheus serving rate limit.
        ServingRateLimitSet(NetUid, u64),
        /// setting burn on a network.
        BurnSet(NetUid, TaoCurrency),
        /// setting max burn on a network.
        MaxBurnSet(NetUid, TaoCurrency),
        /// setting min burn on a network.
        MinBurnSet(NetUid, TaoCurrency),
        /// setting the transaction rate limit.
        TxRateLimitSet(u64),
        /// setting the delegate take transaction rate limit.
        TxDelegateTakeRateLimitSet(u64),
        /// setting the childkey take transaction rate limit.
        TxChildKeyTakeRateLimitSet(u64),
        /// minimum childkey take set
        MinChildKeyTakeSet(u16),
        /// maximum childkey take set
        MaxChildKeyTakeSet(u16),
        /// childkey take set
        ChildKeyTakeSet(T::AccountId, u16),
        /// a sudo call is done.
        Sudid(DispatchResult),
        /// registration is allowed/disallowed for a subnet.
        RegistrationAllowed(NetUid, bool),
        /// POW registration is allowed/disallowed for a subnet.
        PowRegistrationAllowed(NetUid, bool),
        /// setting tempo on a network
        TempoSet(NetUid, u16),
        /// setting the RAO recycled for registration.
        RAORecycledForRegistrationSet(NetUid, TaoCurrency),
        /// min stake is set for validators to set weights.
        StakeThresholdSet(u64),
        /// setting the minimum required stake amount for senate registration.
        SenateRequiredStakePercentSet(u64),
        /// setting the adjustment alpha on a subnet.
        AdjustmentAlphaSet(NetUid, u64),
        /// the faucet it called on the test net.
        Faucet(T::AccountId, u64),
        /// the subnet owner cut is set.
        SubnetOwnerCutSet(u16),
        /// the network creation rate limit is set.
        NetworkRateLimitSet(u64),
        /// the network immunity period is set.
        NetworkImmunityPeriodSet(u64),
        /// the network minimum locking cost is set.
        NetworkMinLockCostSet(TaoCurrency),
        /// the maximum number of subnets is set
        // SubnetLimitSet(u16),
        /// the lock cost reduction is set
        NetworkLockCostReductionIntervalSet(u64),
        /// the take for a delegate is decreased.
        TakeDecreased(T::AccountId, T::AccountId, u16),
        /// the take for a delegate is increased.
        TakeIncreased(T::AccountId, T::AccountId, u16),
        /// the hotkey is swapped
        HotkeySwapped {
            /// the account ID of coldkey
            coldkey: T::AccountId,
            /// the account ID of old hotkey
            old_hotkey: T::AccountId,
            /// the account ID of new hotkey
            new_hotkey: T::AccountId,
        },
        /// maximum delegate take is set by sudo/admin transaction
        MaxDelegateTakeSet(u16),
        /// minimum delegate take is set by sudo/admin transaction
        MinDelegateTakeSet(u16),
        /// a member of the senate is adjusted
        SenateAdjusted {
            /// the account ID of the old senate member, if any
            old_member: Option<T::AccountId>,
            /// the account ID of the new senate member
            new_member: T::AccountId,
        },
        /// A coldkey has been swapped
        ColdkeySwapped {
            /// the account ID of old coldkey
            old_coldkey: T::AccountId,
            /// the account ID of new coldkey
            new_coldkey: T::AccountId,
            /// the swap cost
            swap_cost: TaoCurrency,
        },
        /// All balance of a hotkey has been unstaked and transferred to a new coldkey
        AllBalanceUnstakedAndTransferredToNewColdkey {
            /// The account ID of the current coldkey
            current_coldkey: T::AccountId,
            /// The account ID of the new coldkey
            new_coldkey: T::AccountId,
            /// The total balance of the hotkey
            total_balance: <<T as Config>::Currency as fungible::Inspect<
                <T as frame_system::Config>::AccountId,
            >>::Balance,
        },
        /// A coldkey swap has been scheduled
        ColdkeySwapScheduled {
            /// The account ID of the old coldkey
            old_coldkey: T::AccountId,
            /// The account ID of the new coldkey
            new_coldkey: T::AccountId,
            /// The arbitration block for the coldkey swap
            execution_block: BlockNumberFor<T>,
            /// The swap cost
            swap_cost: TaoCurrency,
        },
        /// The arbitration period has been extended
        ArbitrationPeriodExtended {
            /// The account ID of the coldkey
            coldkey: T::AccountId,
        },
        /// Setting of children of a hotkey have been scheduled
        SetChildrenScheduled(T::AccountId, NetUid, u64, Vec<(u64, T::AccountId)>),
        /// The children of a hotkey have been set
        SetChildren(T::AccountId, NetUid, Vec<(u64, T::AccountId)>),
        // /// The hotkey emission tempo has been set
        // HotkeyEmissionTempoSet(u64),
        // /// The network maximum stake has been set
        // NetworkMaxStakeSet(u16, u64),
        /// The identity of a coldkey has been set
        ChainIdentitySet(T::AccountId),
        /// The identity of a subnet has been set
        SubnetIdentitySet(NetUid),
        /// The identity of a subnet has been removed
        SubnetIdentityRemoved(NetUid),
        /// A dissolve network extrinsic scheduled.
        DissolveNetworkScheduled {
            /// The account ID schedule the dissolve network extrisnic
            account: T::AccountId,
            /// network ID will be dissolved
            netuid: NetUid,
            /// extrinsic execution block number
            execution_block: BlockNumberFor<T>,
        },
        /// The duration of schedule coldkey swap has been set
        ColdkeySwapScheduleDurationSet(BlockNumberFor<T>),
        /// The duration of dissolve network has been set
        DissolveNetworkScheduleDurationSet(BlockNumberFor<T>),
        /// Commit-reveal v3 weights have been successfully committed.
        ///
        /// - **who**: The account ID of the user committing the weights.
        /// - **netuid**: The network identifier.
        /// - **commit_hash**: The hash representing the committed weights.
        CRV3WeightsCommitted(T::AccountId, NetUid, H256),
        /// Weights have been successfully committed.
        ///
        /// - **who**: The account ID of the user committing the weights.
        /// - **netuid**: The network identifier.
        /// - **commit_hash**: The hash representing the committed weights.
        WeightsCommitted(T::AccountId, NetUid, H256),

        /// Weights have been successfully revealed.
        ///
        /// - **who**: The account ID of the user revealing the weights.
        /// - **netuid**: The network identifier.
        /// - **commit_hash**: The hash of the revealed weights.
        WeightsRevealed(T::AccountId, NetUid, H256),

        /// Weights have been successfully batch revealed.
        ///
        /// - **who**: The account ID of the user revealing the weights.
        /// - **netuid**: The network identifier.
        /// - **revealed_hashes**: A vector of hashes representing each revealed weight set.
        WeightsBatchRevealed(T::AccountId, NetUid, Vec<H256>),

        /// A batch of weights (or commits) have been force-set.
        ///
        /// - **netuids**: The netuids these weights were successfully set/committed for.
        /// - **who**: The hotkey that set this batch.
        BatchWeightsCompleted(Vec<Compact<NetUid>>, T::AccountId),

        /// A batch extrinsic completed but with some errors.
        BatchCompletedWithErrors(),

        /// A weight set among a batch of weights failed.
        ///
        /// - **error**: The dispatch error emitted by the failed item.
        BatchWeightItemFailed(sp_runtime::DispatchError),

        /// Stake has been transferred from one coldkey to another on the same subnet.
        /// Parameters:
        /// (origin_coldkey, destination_coldkey, hotkey, origin_netuid, destination_netuid, amount)
        StakeTransferred(
            T::AccountId,
            T::AccountId,
            T::AccountId,
            NetUid,
            NetUid,
            TaoCurrency,
        ),

        /// Stake has been swapped from one subnet to another for the same coldkey-hotkey pair.
        ///
        /// Parameters:
        /// (coldkey, hotkey, origin_netuid, destination_netuid, amount)
        StakeSwapped(T::AccountId, T::AccountId, NetUid, NetUid, TaoCurrency),

        /// Event called when transfer is toggled on a subnet.
        ///
        /// Parameters:
        /// (netuid, bool)
        TransferToggle(NetUid, bool),

        /// The owner hotkey for a subnet has been set.
        ///
        /// Parameters:
        /// (netuid, new_hotkey)
        SubnetOwnerHotkeySet(NetUid, T::AccountId),
        /// FirstEmissionBlockNumber is set via start call extrinsic
        ///
        /// Parameters:
        /// netuid
        /// block number
        FirstEmissionBlockNumberSet(NetUid, u64),

        /// Alpha has been recycled, reducing AlphaOut on a subnet.
        ///
        /// Parameters:
        /// (coldkey, hotkey, amount, subnet_id)
        AlphaRecycled(T::AccountId, T::AccountId, AlphaCurrency, NetUid),

        /// Alpha have been burned without reducing AlphaOut.
        ///
        /// Parameters:
        /// (coldkey, hotkey, amount, subnet_id)
        AlphaBurned(T::AccountId, T::AccountId, AlphaCurrency, NetUid),

        /// An EVM key has been associated with a hotkey.
        EvmKeyAssociated {
            /// The subnet that the hotkey belongs to.
            netuid: NetUid,
            /// The hotkey associated with the EVM key.
            hotkey: T::AccountId,
            /// The EVM key being associated with the hotkey.
            evm_key: H160,
            /// The block where the association happened.
            block_associated: u64,
        },

        /// CRV3 Weights have been successfully revealed.
        ///
        /// - **netuid**: The network identifier.
        /// - **who**: The account ID of the user revealing the weights.
        CRV3WeightsRevealed(NetUid, T::AccountId),

        /// Commit-Reveal periods has been successfully set.
        ///
        /// - **netuid**: The network identifier.
        /// - **periods**: The number of epochs before the reveal.
        CommitRevealPeriodsSet(NetUid, u64),

        /// Commit-Reveal has been successfully toggled.
        ///
        /// - **netuid**: The network identifier.
        /// - **Enabled**: Is Commit-Reveal enabled.
        CommitRevealEnabled(NetUid, bool),

        /// the hotkey is swapped
        HotkeySwappedOnSubnet {
            /// the account ID of coldkey
            coldkey: T::AccountId,
            /// the account ID of old hotkey
            old_hotkey: T::AccountId,
            /// the account ID of new hotkey
            new_hotkey: T::AccountId,
            /// the subnet ID
            netuid: NetUid,
        },
        /// A subnet lease has been created.
        SubnetLeaseCreated {
            /// The beneficiary of the lease.
            beneficiary: T::AccountId,
            /// The lease ID
            lease_id: LeaseId,
            /// The subnet ID
            netuid: NetUid,
            /// The end block of the lease
            end_block: Option<BlockNumberFor<T>>,
        },

        /// A subnet lease has been terminated.
        SubnetLeaseTerminated {
            /// The beneficiary of the lease.
            beneficiary: T::AccountId,
            /// The subnet ID
            netuid: NetUid,
        },

        /// The symbol for a subnet has been updated.
        SymbolUpdated {
            /// The subnet ID
            netuid: NetUid,
            /// The symbol that has been updated.
            symbol: Vec<u8>,
        },

        /// Commit Reveal Weights version has been updated.
        ///
        /// - **version**: The required version.
        CommitRevealVersionSet(u16),

        /// Timelocked weights have been successfully committed.
        ///
        /// - **who**: The account ID of the user committing the weights.
        /// - **netuid**: The network identifier.
        /// - **commit_hash**: The hash representing the committed weights.
        /// - **reveal_round**: The round at which weights can be revealed.
        TimelockedWeightsCommitted(T::AccountId, NetUid, H256, u64),

        /// Timelocked Weights have been successfully revealed.
        ///
        /// - **netuid**: The network identifier.
        /// - **who**: The account ID of the user revealing the weights.
        TimelockedWeightsRevealed(NetUid, T::AccountId),

        /// The minimum allowed UIDs for a subnet have been set.
        MinAllowedUidsSet(NetUid, u16),
    }
}
