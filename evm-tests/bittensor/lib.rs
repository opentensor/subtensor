#![cfg_attr(not(feature = "std"), no_std, no_main)]

use parity_scale_codec::{Compact, CompactAs, Error as CodecError};

#[derive(Debug, Clone)]
pub struct CustomEnvironment;

// pub enum FunctionId {
//     GetStakeInfoForHotkeyColdkeyNetuidV1 = 0,
//     AddStakeV1 = 1,
//     RemoveStakeV1 = 2,
//     UnstakeAllV1 = 3,
//     UnstakeAllAlphaV1 = 4,
//     MoveStakeV1 = 5,
//     TransferStakeV1 = 6,
//     SwapStakeV1 = 7,
//     AddStakeLimitV1 = 8,
//     RemoveStakeLimitV1 = 9,
//     SwapStakeLimitV1 = 10,
//     RemoveStakeFullLimitV1 = 11,
//     SetColdkeyAutoStakeHotkeyV1 = 12,
//     AddProxyV1 = 13,
//     RemoveProxyV1 = 14,
// }

#[ink::chain_extension(extension = 0x1000)]
pub trait RuntimeReadWrite {
    type ErrorCode = ReadWriteErrorCode;

    #[ink(function = 0)]
    fn get_stake_info_for_hotkey_coldkey_netuid(
        hotkey: ink::primitives::AccountId,
        coldkey: ink::primitives::AccountId,
        netuid: u16,
    ) -> Option<StakeInfo<ink::primitives::AccountId>>;

    // #[ink(function = 1)]
    // fn add_stake(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 2)]
    // fn remove_stake(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 3)]
    // fn unstake_all(hotkey: &[u8], netuid: &[u8]);

    // #[ink(function = 4)]
    // fn unstake_all_alpha(hotkey: &[u8], netuid: &[u8]);

    // #[ink(function = 5)]
    // fn move_stake(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 6)]
    // fn transfer_stake(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 7)]
    // fn swap_stake(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 8)]
    // fn add_stake_limit(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 9)]
    // fn remove_stake_limit(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 10)]
    // fn swap_stake_limit(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 11)]
    // fn remove_stake_full_limit(hotkey: &[u8], netuid: &[u8], amount: &[u8]);

    // #[ink(function = 12)]
    // fn set_coldkey_auto_stake_hotkey(coldkey: &[u8], hotkey: &[u8]);

    // #[ink(function = 13)]
    // fn add_proxy(hotkey: &[u8], netuid: &[u8]);

    // #[ink(function = 14)]
    // fn remove_proxy(hotkey: &[u8], netuid: &[u8]);
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

pub struct NetUid(u16);
impl CompactAs for NetUid {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<NetUid>> for NetUid {
    fn from(c: Compact<NetUid>) -> Self {
        c.0
    }
}

impl From<NetUid> for u16 {
    fn from(val: NetUid) -> Self {
        val.0
    }
}

impl From<u16> for NetUid {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct AlphaCurrency(u64);
impl CompactAs for AlphaCurrency {
    type As = u64;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}
impl From<Compact<AlphaCurrency>> for AlphaCurrency {
    fn from(c: Compact<AlphaCurrency>) -> Self {
        c.0
    }
}

impl From<AlphaCurrency> for u64 {
    fn from(val: AlphaCurrency) -> Self {
        val.0
    }
}

impl From<u64> for AlphaCurrency {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct TaoCurrency(u64);
impl CompactAs for TaoCurrency {
    type As = u64;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<TaoCurrency>> for TaoCurrency {
    fn from(c: Compact<TaoCurrency>) -> Self {
        c.0
    }
}

impl From<TaoCurrency> for u64 {
    fn from(val: TaoCurrency) -> Self {
        val.0
    }
}

impl From<u64> for TaoCurrency {
    fn from(value: u64) -> Self {
        Self(value)
    }
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
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn a(&self) -> bool {
            true
        }

        // pub fn get_stake_info_for_hotkey_coldkey_netuid(
        //     &mut self,
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
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let bittensor = Bittensor::default();
            assert_eq!(bittensor.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut bittensor = Bittensor::new(false);
            assert_eq!(bittensor.get(), false);
            bittensor.flip();
            assert_eq!(bittensor.get(), true);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = BittensorRef::default();

            // When
            let contract = client
                .instantiate("bittensor", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Bittensor>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = BittensorRef::new(false);
            let contract = client
                .instantiate("bittensor", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Bittensor>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
