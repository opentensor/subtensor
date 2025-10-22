#![cfg(test)]
use super::*;
use crate::mock::*;
use sp_core::U256;

#[test]
fn environment_works() {
    TestState::default().build_and_execute(|| {
        assert_eq!(
            AllowedProposers::<Test>::get(),
            vec![U256::from(1), U256::from(2), U256::from(3)]
        );
        assert_eq!(
            Triumvirate::<Test>::get(),
            vec![U256::from(1001), U256::from(1002), U256::from(1003)]
        );
    });
}

#[test]
#[should_panic(expected = "Allowed proposers cannot contain duplicate accounts.")]
fn environment_with_duplicate_allowed_proposers_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(2)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers length cannot exceed MaxAllowedProposers.")]
fn environment_with_too_many_allowed_proposers_panics() {
    let max_allowed_proposers = <Test as pallet::Config>::MaxAllowedProposers::get() as usize;
    let allowed_proposers = (0..=max_allowed_proposers).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_allowed_proposers(allowed_proposers)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate cannot contain duplicate accounts.")]
fn environment_with_duplicate_triumvirate_panics() {
    TestState::default()
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1002)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate length cannot exceed 3.")]
fn environment_with_too_many_triumvirate_panics() {
    let triumvirate = (0..=3).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_triumvirate(triumvirate)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers and triumvirate must be disjoint.")]
fn environment_with_overlapping_allowed_proposers_and_triumvirate_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(3)])
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1)])
        .build_and_execute(|| {});
}
