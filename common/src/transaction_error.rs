use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

#[derive(Debug, PartialEq)]
pub enum CustomTransactionError {
    /// Deprecated: coldkey swap now uses announcements and check moved to DispatchGuard
    #[deprecated]
    ColdkeyInSwapSchedule,
    StakeAmountTooLow,
    BalanceTooLow,
    SubnetNotExists,
    HotkeyAccountDoesntExist,
    NotEnoughStakeToWithdraw,
    RateLimitExceeded,
    InsufficientLiquidity,
    SlippageTooHigh,
    TransferDisallowed,
    HotKeyNotRegisteredInNetwork,
    InvalidIpAddress,
    ServingRateLimitExceeded,
    InvalidPort,
    BadRequest,
    ZeroMaxAmount,
    InvalidRevealRound,
    CommitNotFound,
    CommitBlockNotInRevealRange,
    InputLengthsUnequal,
    UidNotFound,
    EvmKeyAssociateRateLimitExceeded,
    ColdkeySwapDisputed,
    InvalidRealAccount,
    FailedShieldedTxParsing,
    InvalidShieldedTxPubKeyHash,
    CommitRevealEnabled,
    CommitRevealDisabled,
    NonAssociatedColdKey,
    InvalidIpType,
    IncorrectCommitRevealVersion,
    CommittingWeightsTooFast,
    TooManyUnrevealedCommits,
    NewHotKeyIsSameWithOld,
    HotKeyAlreadyRegisteredInSubNet,
    NotEnoughBalanceToPaySwapHotKey,
    HotKeySwapOnSubnetIntervalNotPassed,
    DelegateTakeTooLow,
    DelegateTakeTooHigh,
    InvalidWorkBlock,
    InvalidDifficulty,
    InvalidSeal,
}

impl From<CustomTransactionError> for u8 {
    fn from(variant: CustomTransactionError) -> u8 {
        match variant {
            #[allow(deprecated)]
            CustomTransactionError::ColdkeyInSwapSchedule => 0,
            CustomTransactionError::StakeAmountTooLow => 1,
            CustomTransactionError::BalanceTooLow => 2,
            CustomTransactionError::SubnetNotExists => 3,
            CustomTransactionError::HotkeyAccountDoesntExist => 4,
            CustomTransactionError::NotEnoughStakeToWithdraw => 5,
            CustomTransactionError::RateLimitExceeded => 6,
            CustomTransactionError::InsufficientLiquidity => 7,
            CustomTransactionError::SlippageTooHigh => 8,
            CustomTransactionError::TransferDisallowed => 9,
            CustomTransactionError::HotKeyNotRegisteredInNetwork => 10,
            CustomTransactionError::InvalidIpAddress => 11,
            CustomTransactionError::ServingRateLimitExceeded => 12,
            CustomTransactionError::InvalidPort => 13,
            CustomTransactionError::BadRequest => 255,
            CustomTransactionError::ZeroMaxAmount => 14,
            CustomTransactionError::InvalidRevealRound => 15,
            CustomTransactionError::CommitNotFound => 16,
            CustomTransactionError::CommitBlockNotInRevealRange => 17,
            CustomTransactionError::InputLengthsUnequal => 18,
            CustomTransactionError::UidNotFound => 19,
            CustomTransactionError::EvmKeyAssociateRateLimitExceeded => 20,
            CustomTransactionError::ColdkeySwapDisputed => 21,
            CustomTransactionError::InvalidRealAccount => 22,
            CustomTransactionError::FailedShieldedTxParsing => 23,
            CustomTransactionError::InvalidShieldedTxPubKeyHash => 24,
            CustomTransactionError::CommitRevealEnabled => 25,
            CustomTransactionError::CommitRevealDisabled => 26,
            CustomTransactionError::NonAssociatedColdKey => 27,
            CustomTransactionError::InvalidIpType => 28,
            CustomTransactionError::IncorrectCommitRevealVersion => 29,
            CustomTransactionError::CommittingWeightsTooFast => 30,
            CustomTransactionError::TooManyUnrevealedCommits => 31,
            CustomTransactionError::NewHotKeyIsSameWithOld => 32,
            CustomTransactionError::HotKeyAlreadyRegisteredInSubNet => 33,
            CustomTransactionError::NotEnoughBalanceToPaySwapHotKey => 34,
            CustomTransactionError::HotKeySwapOnSubnetIntervalNotPassed => 35,
            CustomTransactionError::DelegateTakeTooLow => 36,
            CustomTransactionError::DelegateTakeTooHigh => 37,
            CustomTransactionError::InvalidWorkBlock => 38,
            CustomTransactionError::InvalidDifficulty => 39,
            CustomTransactionError::InvalidSeal => 40,
        }
    }
}

impl From<CustomTransactionError> for TransactionValidityError {
    fn from(variant: CustomTransactionError) -> Self {
        TransactionValidityError::Invalid(InvalidTransaction::Custom(variant.into()))
    }
}
