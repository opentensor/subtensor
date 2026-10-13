//! Fixed-cost weights recovered from reference CI run 38318619464 (50 steps, 20 repeats).
//! Measured at 5fd041a246253c6387ee2db84a5e7555c5dbad06. The run timed out before
//! saving its generated patch, so authorization's missing per-byte coefficients
//! remain fail-closed until a complete reference result is available.
use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{Weight, constants::RocksDbWeight},
};

pub trait WeightInfo {
    fn register() -> Weight;
    fn check_registered() -> Weight;
    fn authorize(call_len: u32) -> Weight;
    fn authorize_ed25519(call_len: u32) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn authorize_ed25519(_: u32) -> Weight {
        Weight::MAX
    }
    fn check_registered() -> Weight {
        Weight::from_parts(16_291_000, 3587).saturating_add(T::DbWeight::get().reads(2_u64))
    }
    fn register() -> Weight {
        Weight::from_parts(154_470_000, 21449)
            .saturating_add(T::DbWeight::get().reads(15_u64))
            .saturating_add(T::DbWeight::get().writes(7_u64))
    }
    fn authorize(_: u32) -> Weight {
        Weight::MAX
    }
}

impl WeightInfo for () {
    fn authorize_ed25519(_: u32) -> Weight {
        Weight::MAX
    }
    fn check_registered() -> Weight {
        Weight::from_parts(16_291_000, 3587).saturating_add(RocksDbWeight::get().reads(2_u64))
    }
    fn register() -> Weight {
        Weight::from_parts(154_470_000, 21449)
            .saturating_add(RocksDbWeight::get().reads(15_u64))
            .saturating_add(RocksDbWeight::get().writes(7_u64))
    }
    fn authorize(_: u32) -> Weight {
        Weight::MAX
    }
}
