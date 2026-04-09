#![cfg_attr(not(feature = "std"), no_std, no_main)]

use parity_scale_codec::Compact;

#[derive(Debug, Clone)]
pub struct CustomEnvironment;

pub enum FunctionId {
    Dummy = 100,
}

#[ink::chain_extension(extension = 0x1000)]
pub trait RuntimeReadWrite {
    type ErrorCode = ReadWriteErrorCode;

    #[ink(function = 100)]
    fn dummy();
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

#[ink::contract(env = crate::CustomEnvironment)]
mod direct_call {
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

        #[ink(message)]
        pub fn dummy(&self) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .dummy()
                .map_err(|_e| ReadWriteErrorCode::WriteFailed)
        }
    }
}
