#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{PollHooks, Polls};

pub use pallet::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PollIndexOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::Index;
type VotingSchemeOf<T> = <<T as Config>::Polls as Polls<AccountIdOf<T>>>::VotingScheme;

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The voting scheme this pallet handles.
        type Scheme: Get<VotingSchemeOf<Self>>;

        /// The referenda pallet. Provides poll queries and receives tally updates.
        type Polls: Polls<Self::AccountId>;

        #[pallet::constant]
        type PowDifficulty: Get<u32>;

        #[pallet::constant]
        type MaxRingSize: Get<u32>;
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        /// Anonymous voting is not implemented yet.
        NotImplemented,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Placeholder extrinsic. Full bLSAG + PoW implementation pending.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())] // TODO: add benchmarks
        pub fn anonymous_vote(
            origin: OriginFor<T>,
            _poll_index: PollIndexOf<T>,
            _approve: bool,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Err(Error::<T>::NotImplemented.into())
        }
    }
}

impl<T: Config> PollHooks<PollIndexOf<T>> for Pallet<T> {
    fn on_poll_created(_poll_index: PollIndexOf<T>) {}
    fn on_poll_completed(_poll_index: PollIndexOf<T>) {}
}
