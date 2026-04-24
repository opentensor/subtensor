//! Benchmarks for Pallet-Referenda.
//!
//! Each benchmark uses the first track declared by `T::Tracks` and a trivial
//! `frame_system::remark` call. `SubmitOrigin` is exercised via
//! `T::SubmitOrigin::try_successful_origin`, which runtimes provide under the
//! `runtime-benchmarks` feature.
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use super::*;
#[allow(unused)]
use crate::Pallet as Referenda;
use alloc::{boxed::Box, vec};
use frame_benchmarking::v2::*;
use frame_support::traits::EnsureOriginWithArg;
use frame_system::RawOrigin;

/// First track declared by the runtime. Real runtimes return their track 0.
fn first_track_id<T: Config>() -> TrackIdOf<T> {
    T::Tracks::track_ids()
        .next()
        .expect("runtime must declare at least one track for benchmarks")
}

fn remark_call<T: Config>() -> CallOf<T> {
    frame_system::Call::<T>::remark { remark: vec![] }.into()
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit() {
        let track = first_track_id::<T>();
        let origin = T::SubmitOrigin::try_successful_origin(&track)
            .expect("SubmitOrigin must supply a successful origin for benchmarks");
        let call = Box::new(remark_call::<T>());

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, track, call);
    }

    #[benchmark]
    fn cancel() {
        let track = first_track_id::<T>();
        let submit_origin = T::SubmitOrigin::try_successful_origin(&track).unwrap();
        Pallet::<T>::submit(submit_origin, track, Box::new(remark_call::<T>())).unwrap();
        let index: ReferendumIndex = 0;

        #[extrinsic_call]
        _(RawOrigin::Root, index);
    }

    #[benchmark]
    fn nudge_referendum() {
        let track = first_track_id::<T>();
        let submit_origin = T::SubmitOrigin::try_successful_origin(&track).unwrap();
        Pallet::<T>::submit(submit_origin, track, Box::new(remark_call::<T>())).unwrap();
        let index: ReferendumIndex = 0;

        #[extrinsic_call]
        _(RawOrigin::Root, index);
    }
}
