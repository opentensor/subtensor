//! Benchmarking setup for pallet-rate-limiting
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::expect_used)]

use codec::Decode;
use frame_benchmarking::v2::*;
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_runtime::traits::{One, Saturating};
use sp_std::boxed::Box;

use super::*;
use crate::CallReadOnly;

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

fn seed_group<T: Config>(name: &[u8], sharing: GroupSharing) -> <T as Config>::GroupId {
    Pallet::<T, ()>::create_group(RawOrigin::Root.into(), name.to_vec(), sharing)
        .expect("group created");
    Pallet::<T, ()>::next_group_id().saturating_sub(<T as Config>::GroupId::one())
}

fn register_call_with_group<T: Config>(
    group: Option<<T as Config>::GroupId>,
) -> TransactionIdentifier {
    let call = sample_call::<T>();
    let identifier = TransactionIdentifier::from_call(call.as_ref()).expect("id");
    Pallet::<T, ()>::register_call(RawOrigin::Root.into(), call, group).expect("registered");
    identifier
}

#[benchmarks]
mod benchmarks {
    use super::*;
    use sp_std::vec::Vec;

    #[benchmark]
    fn register_call() {
        let call = sample_call::<T>();
        let identifier = TransactionIdentifier::from_call(call.as_ref()).expect("id");
        let target = RateLimitTarget::Transaction(identifier);

        #[extrinsic_call]
        _(RawOrigin::Root, call, None);

        assert!(Limits::<T, ()>::contains_key(target));
    }

    #[benchmark]
    fn set_rate_limit() {
        let call = sample_call::<T>();
        let identifier = TransactionIdentifier::from_call(call.as_ref()).expect("id");
        let target = RateLimitTarget::Transaction(identifier);
        Limits::<T, ()>::insert(target, RateLimit::global(RateLimitKind::Default));

        let limit = RateLimitKind::<BlockNumberFor<T>>::Exact(BlockNumberFor::<T>::from(10u32));

        #[extrinsic_call]
        _(RawOrigin::Root, target, None, limit);

        let stored = Limits::<T, ()>::get(target).expect("limit stored");
        assert!(
            matches!(stored, RateLimit::Global(RateLimitKind::Exact(span)) if span == BlockNumberFor::<T>::from(10u32))
        );
    }

    #[benchmark]
    fn assign_call_to_group() {
        let group = seed_group::<T>(b"grp", GroupSharing::UsageOnly);
        let identifier = register_call_with_group::<T>(None);

        #[extrinsic_call]
        _(RawOrigin::Root, identifier, group, false);

        assert_eq!(CallGroups::<T, ()>::get(identifier), Some(group));
        assert_eq!(CallReadOnly::<T, ()>::get(identifier), Some(false));
        assert!(GroupMembers::<T, ()>::get(group).contains(&identifier));
    }

    #[benchmark]
    fn remove_call_from_group() {
        let group = seed_group::<T>(b"team", GroupSharing::ConfigOnly);
        let identifier = register_call_with_group::<T>(Some(group));

        #[extrinsic_call]
        _(RawOrigin::Root, identifier);

        assert!(CallGroups::<T, ()>::get(identifier).is_none());
        assert!(!GroupMembers::<T, ()>::get(group).contains(&identifier));
    }

    #[benchmark]
    fn create_group() {
        let name = b"bench".to_vec();
        let sharing = GroupSharing::ConfigAndUsage;

        #[extrinsic_call]
        _(RawOrigin::Root, name.clone(), sharing);

        let group = Pallet::<T, ()>::next_group_id().saturating_sub(<T as Config>::GroupId::one());
        let details = Groups::<T, ()>::get(group).expect("group stored");
        let stored: Vec<u8> = details.name.into();
        assert_eq!(stored, name);
        assert_eq!(details.sharing, sharing);
    }

    #[benchmark]
    fn update_group() {
        let group = seed_group::<T>(b"old", GroupSharing::UsageOnly);
        let new_name = b"new".to_vec();
        let new_sharing = GroupSharing::ConfigAndUsage;

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            group,
            Some(new_name.clone()),
            Some(new_sharing),
        );

        let details = Groups::<T, ()>::get(group).expect("group exists");
        let stored: Vec<u8> = details.name.into();
        assert_eq!(stored, new_name);
        assert_eq!(details.sharing, new_sharing);
    }

    #[benchmark]
    fn delete_group() {
        let group = seed_group::<T>(b"delete", GroupSharing::UsageOnly);

        #[extrinsic_call]
        _(RawOrigin::Root, group);

        assert!(Groups::<T, ()>::get(group).is_none());
    }

    #[benchmark]
    fn deregister_call() {
        let group = seed_group::<T>(b"dreg", GroupSharing::ConfigAndUsage);
        let identifier = register_call_with_group::<T>(Some(group));
        let target = RateLimitTarget::Transaction(identifier);
        let usage_target = Pallet::<T, ()>::usage_target(&identifier).expect("usage target");
        LastSeen::<T, ()>::insert(
            usage_target,
            None::<T::UsageKey>,
            BlockNumberFor::<T>::from(1u32),
        );

        #[extrinsic_call]
        _(RawOrigin::Root, identifier, None, true);

        assert!(Limits::<T, ()>::get(target).is_none());
        assert!(LastSeen::<T, ()>::get(usage_target, None::<T::UsageKey>).is_none());
        assert!(CallGroups::<T, ()>::get(identifier).is_none());
        assert!(!GroupMembers::<T, ()>::get(group).contains(&identifier));
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
