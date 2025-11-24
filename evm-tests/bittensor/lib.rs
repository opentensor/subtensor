#![cfg_attr(not(feature = "std"), no_std, no_main)]

use parity_scale_codec::Compact;

#[derive(Debug, Clone)]
pub struct CustomEnvironment;

use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};
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
    AddProxyV1 = 13,
    RemoveProxyV1 = 14,
}

#[ink::chain_extension(extension = 0x1000)]
pub trait RuntimeReadWrite {
    type ErrorCode = ReadWriteErrorCode;

    #[ink(function = 0)]
    fn get_stake_info_for_hotkey_coldkey_netuid(
        hotkey: ink::primitives::AccountId,
        coldkey: ink::primitives::AccountId,
        netuid: u16,
    ) -> Option<StakeInfo<ink::primitives::AccountId>>;

    #[ink(function = 1)]
    fn add_stake(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        netuid: NetUid,
        amount: AlphaCurrency,
    );

    #[ink(function = 2)]
    fn remove_stake(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        netuid: NetUid,
        amount: AlphaCurrency,
    );

    #[ink(function = 3)]
    fn unstake_all(hotkey: <CustomEnvironment as ink::env::Environment>::AccountId);

    #[ink(function = 4)]
    fn unstake_all_alpha(hotkey: <CustomEnvironment as ink::env::Environment>::AccountId);

    #[ink(function = 5)]
    fn move_stake(
        origin_hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        destination_hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        amount: AlphaCurrency,
    );

    #[ink(function = 6)]
    fn transfer_stake(
        destination_coldkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        amount: AlphaCurrency,
    );

    #[ink(function = 7)]
    fn swap_stake(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        amount: AlphaCurrency,
    );

    #[ink(function = 8)]
    fn add_stake_limit(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        netuid: NetUid,
        amount: TaoCurrency,
        limit_price: TaoCurrency,
        allow_partial: bool,
    );

    #[ink(function = 9)]
    fn remove_stake_limit(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        netuid: NetUid,
        amount: TaoCurrency,
        limit_price: TaoCurrency,
        allow_partial: bool,
    );

    #[ink(function = 10)]
    fn swap_stake_limit(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        amount: AlphaCurrency,
        limit_price: TaoCurrency,
        allow_partial: bool,
    );

    #[ink(function = 11)]
    fn remove_stake_full_limit(
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
        netuid: NetUid,
        limit_price: TaoCurrency,
    );

    #[ink(function = 12)]
    fn set_coldkey_auto_stake_hotkey(
        netuid: NetUid,
        hotkey: <CustomEnvironment as ink::env::Environment>::AccountId,
    );

    #[ink(function = 13)]
    fn add_proxy(delegate: <CustomEnvironment as ink::env::Environment>::AccountId);

    #[ink(function = 14)]
    fn remove_proxy(delegate: <CustomEnvironment as ink::env::Environment>::AccountId);
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum ReadWriteErrorCode {
    ReadFailed,
    WriteFailed,
}

impl ink::env::chain_extension::FromStatusCode for ReadWriteErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(ReadWriteErrorCode::ReadFailed),
            2 => Err(ReadWriteErrorCode::WriteFailed),
            _ => Err(ReadWriteErrorCode::ReadFailed),
        }
    }
}

impl ink::env::Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = 4;

    type AccountId = ink::primitives::AccountId;
    type Balance = u64;
    type Hash = ink::primitives::Hash;
    type BlockNumber = u32;
    type Timestamp = u64;

    type ChainExtension = RuntimeReadWrite;
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StakeInfo<AccountId> {
    hotkey: AccountId,
    coldkey: AccountId,
    netuid: Compact<NetUid>,
    stake: Compact<AlphaCurrency>,
    locked: Compact<u64>,
    emission: Compact<AlphaCurrency>,
    tao_emission: Compact<TaoCurrency>,
    drain: Compact<u64>,
    is_registered: bool,
}

#[ink::contract(env = crate::CustomEnvironment)]
mod bittensor {
    use super::*;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Bittensor {}

    impl Bittensor {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Constructor
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        // #[ink(message)]
        // pub fn get_stake_info_for_hotkey_coldkey_netuid(
        //     &self,
        //     hotkey: [u8; 32],
        //     coldkey: [u8; 32],
        //     netuid: u16,
        // ) -> Result<Option<StakeInfo<ink::primitives::AccountId>>, ReadWriteErrorCode> {
        //     self.env()
        //         .extension()
        //         .get_stake_info_for_hotkey_coldkey_netuid(
        //             hotkey.into(),
        //             coldkey.into(),
        //             netuid.into(),
        //         )
        //         .map_err(|_e| ReadWriteErrorCode::ReadFailed)
        // }

        #[ink(message)]
        pub fn add_stake(
            &self,
            hotkey: [u8; 32],
            netuid: u16,
            amount: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .add_stake(hotkey.into(), netuid.into(), amount.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn remove_stake(
            &self,
            hotkey: [u8; 32],
            netuid: u16,
            amount: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .remove_stake(hotkey.into(), netuid.into(), amount.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn unstake_all(&self, hotkey: [u8; 32]) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .unstake_all(hotkey.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn unstake_all_alpha(&self, hotkey: [u8; 32]) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .unstake_all_alpha(hotkey.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn move_stake(
            &self,
            origin_hotkey: [u8; 32],
            destination_hotkey: [u8; 32],
            origin_netuid: u16,
            destination_netuid: u16,
            amount: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .move_stake(
                    origin_hotkey.into(),
                    destination_hotkey.into(),
                    origin_netuid.into(),
                    destination_netuid.into(),
                    amount.into(),
                )
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn transfer_stake(
            &self,
            destination_coldkey: [u8; 32],
            hotkey: [u8; 32],
            origin_netuid: u16,
            destination_netuid: u16,
            amount: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .transfer_stake(
                    destination_coldkey.into(),
                    hotkey.into(),
                    origin_netuid.into(),
                    destination_netuid.into(),
                    amount.into(),
                )
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn swap_stake(
            &self,
            hotkey: [u8; 32],
            origin_netuid: u16,
            destination_netuid: u16,
            amount: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .swap_stake(
                    hotkey.into(),
                    origin_netuid.into(),
                    destination_netuid.into(),
                    amount.into(),
                )
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn add_stake_limit(
            &self,
            hotkey: [u8; 32],
            netuid: u16,
            amount: u64,
            limit_price: u64,
            allow_partial: bool,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .add_stake_limit(
                    hotkey.into(),
                    netuid.into(),
                    amount.into(),
                    limit_price.into(),
                    allow_partial,
                )
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn remove_stake_limit(
            &self,
            hotkey: [u8; 32],
            netuid: u16,
            amount: u64,
            limit_price: u64,
            allow_partial: bool,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env().extension().remove_stake_limit(
                hotkey.into(),
                netuid.into(),
                amount.into(),
                limit_price.into(),
                allow_partial,
            )
        }

        #[ink(message)]
        pub fn swap_stake_limit(
            &self,
            hotkey: [u8; 32],
            origin_netuid: u16,
            destination_netuid: u16,
            amount: u64,
            limit_price: u64,
            allow_partial: bool,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .swap_stake_limit(
                    hotkey.into(),
                    origin_netuid.into(),
                    destination_netuid.into(),
                    amount.into(),
                    limit_price.into(),
                    allow_partial,
                )
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn remove_stake_full_limit(
            &self,
            hotkey: [u8; 32],
            netuid: u16,
            limit_price: u64,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .remove_stake_full_limit(hotkey.into(), netuid.into(), limit_price.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn set_coldkey_auto_stake_hotkey(
            &self,
            netuid: u16,
            hotkey: [u8; 32],
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .set_coldkey_auto_stake_hotkey(netuid.into(), hotkey.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn add_proxy(&self, delegate: [u8; 32]) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .add_proxy(delegate.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }

        #[ink(message)]
        pub fn remove_proxy(&self, delegate: [u8; 32]) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .remove_proxy(delegate.into())
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }
    }
}
