//! Benchmarking setup for pallet-rate-limiting
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects)]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};

use super::*;

pub trait BenchmarkHelper<Call> {
    fn sample_call() -> Call;
}

impl<Call> BenchmarkHelper<Call> for ()
where
    Call: Decode,
{
    fn sample_call() -> Call {
        Decode::decode(&mut &[][..]).expect("Provide a call via BenchmarkHelper::sample_call")
    }
}

fn sample_call<T: Config>() -> Box<<T as Config>::RuntimeCall>
where
    T::BenchmarkHelper: BenchmarkHelper<<T as Config>::RuntimeCall>,
{
    Box::new(T::BenchmarkHelper::sample_call())
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_rate_limit() {
        let call = sample_call::<T>();
        let limit = RateLimit::<BlockNumberFor<T>>::Exact(BlockNumberFor::<T>::from(10u32));
        let identifier = TransactionIdentifier::from_call::<T>(call.as_ref()).expect("identifier");
        let context = <T as Config>::ContextResolver::context(call.as_ref());

        #[extrinsic_call]
        _(RawOrigin::Root, call, limit.clone(), None);

        assert_eq!(Limits::<T>::get(&identifier, context), Some(limit));
    }

    #[benchmark]
    fn clear_rate_limit() {
        let call = sample_call::<T>();
        let limit = RateLimit::<BlockNumberFor<T>>::Exact(BlockNumberFor::<T>::from(10u32));

        // Pre-populate limit for benchmark call
        let identifier = TransactionIdentifier::from_call::<T>(call.as_ref()).expect("identifier");
        let context = <T as Config>::ContextResolver::context(call.as_ref());
        Limits::<T>::insert(identifier, context.clone(), limit);

        #[extrinsic_call]
        _(RawOrigin::Root, call, None);

        assert!(Limits::<T>::get(identifier, context).is_none());
    }

    #[benchmark]
    fn set_default_rate_limit() {
        let block_span = BlockNumberFor::<T>::from(10u32);

        #[extrinsic_call]
        _(RawOrigin::Root, block_span);

        assert_eq!(DefaultLimit::<T>::get(), block_span);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
