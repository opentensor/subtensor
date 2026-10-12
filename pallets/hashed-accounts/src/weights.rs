//! Unmeasured fail-closed weights. Replace using reference-hardware benchmarks
//! before production activation; these are intentionally not operating estimates.
use core::marker::PhantomData;
// Keep the reference generator's imports and implementation shape so CI can
// replace individual methods without leaving an unusable first weight patch.
#[allow(unused_imports)]
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
        Weight::MAX
    }
    fn register() -> Weight {
        Weight::MAX
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
        Weight::MAX
    }
    fn register() -> Weight {
        Weight::MAX
    }
    fn authorize(_: u32) -> Weight {
        Weight::MAX
    }
}
