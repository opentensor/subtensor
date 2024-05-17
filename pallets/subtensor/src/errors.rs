use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod errors {
    #[pallet::error]
    pub enum Error<T> {
        /// the network does not exist.
        SubNetworkDoesNotExist,
        /// the root network does not exist.
        RootNetworkDoesNotExist,
        /// the user tries to serve an axon which is not of type 4 (IPv4) or 6 (IPv6).
        InvalidIpType,
        /// an invalid IP address is passed to the serve function.
        InvalidIpAddress,
        /// an invalid port is passed to the serve function.
        InvalidPort,
        /// the hot key not registered in subnet
        HotKeyNotRegisteredInSubNet,
        /// the hot key not exists
        HotKeyAccountNotExists,
        /// the hot key not registered in any subnet.
        HotKeyNotRegisteredInNetwork,
        /// stake, unstake or subscribe request is made by a coldkey which is not associated with the hotkey account.
        NonAssociatedColdKey,
        /// the hot key is not a delegate and the signer is not the owner of the hot key.
        HotKeyNotDelegateAndSignerNotOwnHotKey,
        /// stake to withdraw amount is zero
        StakeToWithdrawIsZero,
        /// the caller requests removing more stake than there exists in the staking account. See: fn remove_stake.
        NotEnoughStakeToWithdraw,
        /// the caller requests to set weights but has less than WeightsMinStake
        NotEnoughStakeToSetWeights,
        /// the caller requests adding more stake than there exists in the cold key account. See: fn add_stake
        NotEnoughBalanceToStake,
        /// the caller tries to add stake, but for some reason the requested amount could not be withdrawn from the coldkey account.
        BalanceWithdrawalError,
        /// successfully deducted balance for withdraw is zero
        ZeroBalanceWithdrawn,
        /// the caller attempts to set non-self weights without being a permitted validator.
        NeuronNoValidatorPermit,
        /// the caller attempts to set the weight keys and values but these vectors have different size.
        WeightVecNotEqualSize,
        /// the caller attempts to set weights with duplicate uids in the weight matrix.
        DuplicateUids,
        /// the caller attempts to set weight to at least one uid that does not exist in the metagraph.
        UidVecContainInvalidOne,
        /// the dispatch attempts to set weights on chain with fewer elements than are allowed.
        WeightVecLengthIsLow,
        /// registrations this block exceeds allowed number.
        TooManyRegistrationsThisBlock,
        /// the caller requests registering a neuron which already exists in the active set.
        HotKeyAlreadyRegisteredInSubNet,
        /// the new hot key is the same as old one
        NewHotKeyIsSameWithOld,
        /// the supplied pow hash block is in the future or negative.
        InvalidWorkBlock,
        /// the supplied pow hash block does not meet the network difficulty.
        InvalidDifficulty,
        /// the supplied pow hash seal does not match the supplied work.
        InvalidSeal,
        /// the dispatch attempts to set weights on chain with where any normalized weight is more than MaxWeightLimit.
        MaxWeightExceeded,
        /// the hotkey attempts to become delegate when they are already.
        HotKeyAlreadyDelegate,
        /// the hotkey attempts to set weights twice within net_tempo/2 blocks.
        SettingWeightsTooFast,
        /// a validator attempts to set weights from a validator with incorrect code base key.
        IncorrectNetworkVersionKey,
        /// an axon or prometheus serving exceeds the rate limit for a registered neuron.
        ServingRateLimitExceeded,
        /// the caller attempts to set weights with more uids than allowed.
        UidsLengthExceedUidsInSubNet,
        /// a transactor exceeds the rate limit for add network tx.
        NetworkTxRateLimitExceeded,
        /// a transactor exceeds the rate limit for delegate tx.
        DelegateTxRateLimitExceeded,
        /// a transactor exceeds the rate limit for set or swap hot key.
        HotKeySetTxRateLimitExceeded,
        /// a transactor exceeds the rate limit for stakes.
        StakeRateLimitExceeded,
        /// a transactor exceeds the rate limit for unstakes.
        UnstakeRateLimitExceeded,
        /// registration is disabled
        SubNetRegistrationDisabled,
        /// registration attempt exceeds allowed in interval
        TooManyRegistrationsThisInterval,
        /// the hotkey passed is not the origin, but it should be
        TransactorAccountShouldBeHotKey,
        /// a hotkey attempts to do something only senate members can do
        NotSenateMember,
        /// the faucet is disabled
        FaucetDisabled,
        /// not subnet owner
        NotSubnetOwner,
        /// operation not permitted on root subnet
        RegistrationNotPermittedOnRootSubnet,
        /// a hotkey attempts to join the root subnet with too little stake
        StakeTooLowForRoot,
        /// all subnets are in the immunity period
        AllNetworksInImmunity,
        /// not enough balance to pay swap hot key
        NotEnoughBalanceToPaySwapHotKey,
        /// Set root network weights not match net uid
        NotRootSubnet,
        /// can't set weights for root network
        CanNotSetRootNetworkWeights,
        /// no neuron id is available
        NoNeuronIdAvailable,
        /// Thrown a stake would be below the minimum threshold for nominator validations
        NomStakeBelowMinimumThreshold,
        /// delegate take is too low
        DelegateTakeTooLow,
        /// delegate take is too high
        DelegateTakeTooHigh,
        /// Not allowed to commit weights
        CommitNotAllowed,
        /// No commit found for provided hotkey+netuid when attempting to reveal weights
        NoCommitFound,
        /// Not the correct block/range to reveal weights
        InvalidRevealTempo,
        /// Committed hash does not equal the hashed reveal data
        InvalidReveal,
    }
}
