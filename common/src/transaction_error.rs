use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

#[derive(Debug, PartialEq)]
pub enum CustomTransactionError {
    ColdkeySwapAnnounced,
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
}

impl From<CustomTransactionError> for u8 {
    fn from(variant: CustomTransactionError) -> u8 {
        match variant {
            CustomTransactionError::ColdkeySwapAnnounced => 0,
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
        }
    }
}

impl From<CustomTransactionError> for TransactionValidityError {
    fn from(variant: CustomTransactionError) -> Self {
        TransactionValidityError::Invalid(InvalidTransaction::Custom(variant.into()))
    }
}
