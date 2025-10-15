use codec::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sp_runtime::{DispatchError, ModuleError};

#[repr(u16)]
#[derive(TryFromPrimitive, IntoPrimitive, Decode, Encode)]
pub enum FunctionId {
    GetStakeInfoForHotkeyColdkeyNetuidV1 = 0,
    AddStakeV1 = 1,
    RemoveStakeV1 = 2,
    UnstakeAllV1 = 3,
    UnstakeAllAlphaV1 = 4,
    MoveStakeV1 = 5,
    TransferStakeV1 = 6,
    SwapStakeV1 = 7,
    AddStakeLimitV1 = 8,
    RemoveStakeLimitV1 = 9,
    SwapStakeLimitV1 = 10,
    RemoveStakeFullLimitV1 = 11,
    SetColdkeyAutoStakeHotkeyV1 = 12,
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Outcome {
    /// Success
    Success = 0,
    /// Unknown error
    RuntimeError = 1,
    /// Not enough balance to stake
    NotEnoughBalanceToStake = 2,
    /// Coldkey is not associated with the hotkey
    NonAssociatedColdKey = 3,
    /// Error withdrawing balance
    BalanceWithdrawalError = 4,
    /// Hotkey is not registered
    NotRegistered = 5,
    /// Not enough stake to withdraw
    NotEnoughStakeToWithdraw = 6,
    /// Transaction rate limit exceeded
    TxRateLimitExceeded = 7,
    /// Slippage is too high for the transaction
    SlippageTooHigh = 8,
    /// Subnet does not exist
    SubnetNotExists = 9,
    /// Hotkey is not registered in subnet
    HotKeyNotRegisteredInSubNet = 10,
    /// Same auto stake hotkey already set
    SameAutoStakeHotkeyAlreadySet = 11,
    /// Insufficient balance
    InsufficientBalance = 12,
    /// Amount is too low
    AmountTooLow = 13,
    /// Insufficient liquidity
    InsufficientLiquidity = 14,
    /// Same netuid
    SameNetuid = 15,
}

impl From<DispatchError> for Outcome {
    fn from(input: DispatchError) -> Self {
        let error_text = match input {
            DispatchError::Module(ModuleError { message, .. }) => message,
            _ => Some("No module error Info"),
        };
        match error_text {
            Some("NotEnoughBalanceToStake") => Outcome::NotEnoughBalanceToStake,
            Some("NonAssociatedColdKey") => Outcome::NonAssociatedColdKey,
            Some("BalanceWithdrawalError") => Outcome::BalanceWithdrawalError,
            Some("HotKeyNotRegisteredInSubNet") => Outcome::NotRegistered,
            Some("HotKeyAccountNotExists") => Outcome::NotRegistered,
            Some("NotEnoughStakeToWithdraw") => Outcome::NotEnoughStakeToWithdraw,
            Some("TxRateLimitExceeded") => Outcome::TxRateLimitExceeded,
            Some("SlippageTooHigh") => Outcome::SlippageTooHigh,
            Some("SubnetNotExists") => Outcome::SubnetNotExists,
            Some("SameAutoStakeHotkeyAlreadySet") => Outcome::SameAutoStakeHotkeyAlreadySet,
            Some("InsufficientBalance") => Outcome::InsufficientBalance,
            Some("AmountTooLow") => Outcome::AmountTooLow,
            Some("InsufficientLiquidity") => Outcome::InsufficientLiquidity,
            Some("SameNetuid") => Outcome::SameNetuid,
            _ => Outcome::RuntimeError,
        }
    }
}
