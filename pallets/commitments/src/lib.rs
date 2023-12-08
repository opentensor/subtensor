#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub mod types;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use sp_std::boxed::Box;
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::{*, BlockNumberFor};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Currency type that will be used to place deposits on neurons
		type Currency: ReservableCurrency<Self::AccountId> + Send + Sync;

		// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Interface to access-limit metadata commitments
		type CanCommit: CanCommit<Self::AccountId>;

		#[pallet::constant]
		type MaxFields: Get<u32>;

		/// The amount held on deposit for a registered identity
		#[pallet::constant]
		type InitialDeposit: Get<BalanceOf<Self>>;

		/// The amount held on deposit per additional field for a registered identity.
		#[pallet::constant]
		type FieldDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type RateLimit: Get<BlockNumberFor<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Commitment{netuid: u16, who: T::AccountId}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account passed too many additional fields to their commitment
		TooManyFields,
		/// Account isn't allow to make commitments to the chain
		CannotCommit,
		/// Account is trying to commit data too fast
		RateLimitExceeded
	}

	/// Identity data by account
	#[pallet::storage]
	#[pallet::getter(fn commitment_of)]
	pub(super) type CommitmentOf<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u16,
		Twox64Concat,
		T::AccountId,
		Registration<BalanceOf<T>, T::MaxFields, BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn last_commitment)]
	pub(super) type LastCommitment<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u16,
		Twox64Concat,
		T::AccountId,
		BlockNumberFor<T>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight((
			T::WeightInfo::set_commitment(), 
			DispatchClass::Operational
		))]
		pub fn set_commitment(origin: OriginFor<T>, netuid: u16, info: Box<CommitmentInfo<T::MaxFields>>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(T::CanCommit::can_commit(netuid, &who), Error::<T>::CannotCommit);
			
			let extra_fields = info.fields.len() as u32;
			ensure!(extra_fields <= T::MaxFields::get(), Error::<T>::TooManyFields);

			let cur_block = <frame_system::Pallet<T>>::block_number();
			if let Some(last_commit) = <LastCommitment<T>>::get(netuid, &who) {
				ensure!(cur_block >= last_commit + T::RateLimit::get(), Error::<T>::RateLimitExceeded);
			}

			let fd = <BalanceOf<T>>::from(extra_fields) * T::FieldDeposit::get();
			let mut id = match <CommitmentOf<T>>::get(netuid, &who) {
				Some(mut id) => {
					id.info = *info;
					id.block = cur_block;
					id
				},
				None => Registration {
					info: *info,
					block: cur_block,
					deposit: Zero::zero(),
				},
			};

			let old_deposit = id.deposit;
			id.deposit = T::InitialDeposit::get() + fd;
			if id.deposit > old_deposit {
				T::Currency::reserve(&who, id.deposit - old_deposit)?;
			}
			if old_deposit > id.deposit {
				let err_amount = T::Currency::unreserve(&who, old_deposit - id.deposit);
				debug_assert!(err_amount.is_zero());
			}

			<CommitmentOf<T>>::insert(netuid, &who, id);
			<LastCommitment<T>>::insert(netuid, &who, cur_block);
			Self::deposit_event(Event::Commitment { netuid, who });

			Ok(().into())
		}
	}
}

// Interfaces to interact with other pallets
pub trait CanCommit<AccountId> {
    fn can_commit(netuid: u16, who: &AccountId) -> bool;
}

impl<A> CanCommit<A> for () {
	fn can_commit(_: u16, _: &A) -> bool {false}
}
