use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the events for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod errors {
    #[pallet::error]
    pub enum Error<T> {
        /// the network does not exist.
        NetworkDoesNotExist,
        /// the network already exists.
        NetworkExist,
        /// an invalid modality attempted on serve.
        InvalidModality,
        /// the user tries to serve an axon which is not of type 4 (IPv4) or 6 (IPv6).
        InvalidIpType,
        /// an invalid IP address is passed to the serve function.
        InvalidIpAddress,
        /// an invalid port is passed to the serve function.
        InvalidPort,
        /// the caller requests setting or removing data from a neuron which does not exist in the active set.
        NotRegistered,
        /// stake, unstake or subscribe request is made by a coldkey which is not associated with the hotkey account.
        NonAssociatedColdKey,
        /// the caller requests removing more stake than there exists in the staking account. See: fn remove_stake.
        NotEnoughStaketoWithdraw,
        /// the caller requests to set weights but has less than WeightsMinStake
        NotEnoughStakeToSetWeights,
        /// the caller requests adding more stake than there exists in the cold key account. See: fn add_stake
        NotEnoughBalanceToStake,
        /// the caller tries to add stake, but for some reason the requested amount could not be withdrawn from the coldkey account.
        BalanceWithdrawalError,
        /// the caller attempts to set non-self weights without being a permitted validator.
        NoValidatorPermit,
        /// the caller attempts to set the weight keys and values but these vectors have different size.
        WeightVecNotEqualSize,
        /// the caller attempts to set weights with duplicate uids in the weight matrix.
        DuplicateUids,
        /// the caller attempts to set weight to at least one uid that does not exist in the metagraph.
        InvalidUid,
        /// the dispatch attempts to set weights on chain with fewer elements than are allowed.
        NotSettingEnoughWeights,
        /// registrations this block exceeds allowed number.
        TooManyRegistrationsThisBlock,
        /// the caller requests registering a neuron which already exists in the active set.
        AlreadyRegistered,
        /// the supplied pow hash block is in the future or negative.
        InvalidWorkBlock,
        /// the supplied pow hash block does not meet the network difficulty.
        InvalidDifficulty,
        /// the supplied pow hash seal does not match the supplied work.
        InvalidSeal,
        /// the value is invalid for MaxAllowedUids.
        MaxAllowedUIdsNotAllowed,
        /// the dispatch attempts to convert between a u64 and T::balance but the call fails.
        CouldNotConvertToBalance,
        /// the dispatch attempts to convert from a T::Balance to a u64 but the call fails.
        CouldNotConvertToU64,
        /// the caller requests adding stake for a hotkey to the total stake which already added.
        StakeAlreadyAdded,
        /// the dispatch attempts to set weights on chain with where any normalized weight is more than MaxWeightLimit.
        MaxWeightExceeded,
        /// the caller attempts to set a storage value outside of its allowed range.
        StorageValueOutOfRange,
        /// tempo has not set.
        TempoHasNotSet,
        /// tempo is not valid.
        InvalidTempo,
        /// number or received emission rates does not match number of networks.
        EmissionValuesDoesNotMatchNetworks,
        /// emission ratios are not valid (did not sum up to 10^9).
        InvalidEmissionValues,
        /// the hotkey attempts to become delegate when they are already.
        AlreadyDelegate,
        /// the hotkey attempts to set weights twice within net_tempo/2 blocks.
        SettingWeightsTooFast,
        /// a validator attempts to set weights from a validator with incorrect code base key.
        IncorrectNetworkVersionKey,
        /// an axon or prometheus serving exceeds the rate limit for a registered neuron.
        ServingRateLimitExceeded,
        /// an error occurs while setting a balance.
        BalanceSetError,
        /// number of accounts going to be registered exceeds MaxAllowedUids for the network.
        MaxAllowedUidsExceeded,
        /// the caller attempts to set weights with more uids than allowed.
        TooManyUids,
        /// a transactor exceeds the rate limit for transactions.
        TxRateLimitExceeded,
        /// a transactor exceeds the rate limit for stakes.
        StakeRateLimitExceeded,
        /// a transactor exceeds the rate limit for unstakes.
        UnstakeRateLimitExceeded,
        /// registration is disabled
        RegistrationDisabled,
        /// registration attempt exceeds allowed in interval
        TooManyRegistrationsThisInterval,
        /// a function is only available for benchmarking
        BenchmarkingOnly,
        /// the hotkey passed is not the origin, but it should be
        HotkeyOriginMismatch,
        /// attempting to do something to a senate member that is limited
        SenateMember,
        /// a hotkey attempts to do something only senate members can do
        NotSenateMember,
        /// a hotkey attempts to join the senate while already being a member
        AlreadySenateMember,
        /// a hotkey attempts to join the senate without enough stake
        BelowStakeThreshold,
        /// a hotkey attempts to join the senate without being a delegate first
        NotDelegate,
        /// an incorrect amount of Netuids are passed as input
        IncorrectNetuidsLength,
        /// the faucet is disabled
        FaucetDisabled,
        /// not subnet owner
        NotSubnetOwner,
        /// operation not permitted on root subnet
        OperationNotPermittedOnRootSubnet,
        /// a hotkey attempts to join the root subnet with too little stake
        StakeTooLowForRoot,
        /// all subnets are in the immunity period
        AllNetworksInImmunity,
        /// not enough balance
        NotEnoughBalance,
        /// a stake would be below the minimum threshold for nominator validations
        NotRootSubnet,
        /// netuid is not the root network
        IsRoot,
        /// no neuron id is available
        NoNeuronIdAvailable,
        /// Thrown a stake would be below the minimum threshold for nominator validations
        NomStakeBelowMinimumThreshold,
        /// delegate take is being set out of bounds
        InvalidTake,
    }
}
