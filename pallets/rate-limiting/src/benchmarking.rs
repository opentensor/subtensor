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
        let limit = RateLimitKind::<BlockNumberFor<T>>::Exact(BlockNumberFor::<T>::from(10u32));
        let scope = <T as Config>::LimitScopeResolver::context(call.as_ref());
        let identifier =
            TransactionIdentifier::from_call::<T, ()>(call.as_ref()).expect("identifier");

        #[extrinsic_call]
        _(RawOrigin::Root, call, limit.clone());

        let stored = Limits::<T, ()>::get(&identifier).expect("limit stored");
        match (scope, &stored) {
            (Some(ref sc), RateLimit::Scoped(map)) => {
                assert_eq!(map.get(sc), Some(&limit));
            }
            (None, RateLimit::Global(kind)) | (Some(_), RateLimit::Global(kind)) => {
                assert_eq!(kind, &limit);
            }
            (None, RateLimit::Scoped(map)) => {
                assert!(map.values().any(|k| k == &limit));
            }
        }
    }

    #[benchmark]
    fn clear_rate_limit() {
        let call = sample_call::<T>();
        let limit = RateLimitKind::<BlockNumberFor<T>>::Exact(BlockNumberFor::<T>::from(10u32));
        let scope = <T as Config>::LimitScopeResolver::context(call.as_ref());

        // Pre-populate limit for benchmark call
        let identifier =
            TransactionIdentifier::from_call::<T, ()>(call.as_ref()).expect("identifier");
        match scope.clone() {
            Some(sc) => Limits::<T, ()>::insert(identifier, RateLimit::scoped_single(sc, limit)),
            None => Limits::<T, ()>::insert(identifier, RateLimit::global(limit)),
        }

        #[extrinsic_call]
        _(RawOrigin::Root, call);

        assert!(Limits::<T, ()>::get(identifier).is_none());
    }

    #[benchmark]
    fn set_default_rate_limit() {
        let block_span = BlockNumberFor::<T>::from(10u32);

        #[extrinsic_call]
        _(RawOrigin::Root, block_span);

        assert_eq!(DefaultLimit::<T, ()>::get(), block_span);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
