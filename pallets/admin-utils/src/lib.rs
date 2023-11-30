#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_runtime::{
	traits::Member,
	RuntimeAppPublic,
	DispatchResult
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::BoundedVec;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_subtensor::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Aura: crate::AuraInterface<Self::AuthorityId, Self::MaxAuthorities>;

		/// The identifier type for an authority.
		type AuthorityId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;
		/// The maximum number of authorities that the pallet can hold.
		type MaxAuthorities: Get<u32>;

		type Subtensor: crate::SubtensorInterface;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> 
	{}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn swap_authorities(origin: OriginFor<T>, new_authorities: BoundedVec<T::AuthorityId, T::MaxAuthorities>) -> DispatchResult {
			ensure_root(origin)?;

			T::Aura::change_authorities(new_authorities);
			
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_default_take(origin: OriginFor<T>, default_take: u16) -> DispatchResult
		{
			ensure_root(origin)?;
			T::Subtensor::set_default_take(default_take);
			log::info!("DefaultTakeSet( default_take: {:?} ) ", default_take);
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_tx_rate_limit(origin: OriginFor<T>, tx_rate_limit: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
			T::Subtensor::set_tx_rate_limit(tx_rate_limit);
			log::info!("TxRateLimitSet( tx_rate_limit: {:?} ) ", tx_rate_limit);
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_serving_rate_limit(origin: OriginFor<T>, netuid: u16, serving_rate_limit: u64) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			T::Subtensor::set_serving_rate_limit(netuid, serving_rate_limit);
			log::info!(
				"ServingRateLimitSet( serving_rate_limit: {:?} ) ",
				serving_rate_limit
			);
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_min_difficulty(origin: OriginFor<T>, netuid: u16, min_difficulty: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_min_difficulty(netuid, min_difficulty);
			log::info!(
				"MinDifficultySet( netuid: {:?} min_difficulty: {:?} ) ",
				netuid,
				min_difficulty
			);
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_difficulty(origin: OriginFor<T>, netuid: u16, max_difficulty: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_max_difficulty(netuid, max_difficulty);
			log::info!(
				"MaxDifficultySet( netuid: {:?} max_difficulty: {:?} ) ",
				netuid,
				max_difficulty
			);
			Ok(())
		}

		
		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_weights_version_key(origin: OriginFor<T>, netuid: u16, weights_version_key: u64) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_weights_version_key(netuid, weights_version_key);
			log::info!(
				"WeightsVersionKeySet( netuid: {:?} weights_version_key: {:?} ) ",
				netuid,
				weights_version_key
			);
			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_weights_set_rate_limit(origin: OriginFor<T>, netuid: u16, weights_set_rate_limit: u64) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_weights_set_rate_limit(netuid, weights_set_rate_limit);
			log::info!(
				"WeightsSetRateLimitSet( netuid: {:?} weights_set_rate_limit: {:?} ) ",
				netuid,
				weights_set_rate_limit
			);
			T::Subtensor::deposit_event(Event::WeightsSetRateLimitSet(
				netuid,
				weights_set_rate_limit,
			));
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_adjustment_interval(origin: OriginFor<T>, netuid: u16, adjustment_interval: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_adjustment_interval(netuid, adjustment_interval);
			log::info!(
				"AdjustmentIntervalSet( netuid: {:?} adjustment_interval: {:?} ) ",
				netuid,
				adjustment_interval
			);
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_adjustment_alpha(origin: OriginFor<T>, netuid: u16, adjustment_alpha: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_adjustment_alpha(netuid, adjustment_alpha);
			log::info!(
				"AdjustmentAlphaSet( adjustment_alpha: {:?} ) ",
				adjustment_alpha
			);
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_validator_prune_len(origin: OriginFor<T>, netuid: u16, validator_prune_len: u64) -> DispatchResult {
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_validator_prune_len(netuid, validator_prune_len);
			log::info!(
				"ValidatorPruneLenSet( netuid: {:?} validator_prune_len: {:?} ) ",
				netuid,
				validator_prune_len
			);
			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_scaling_law_power(origin: OriginFor<T>, netuid: u16, scaling_law_power: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			ensure!(scaling_law_power <= 100, Error::<T>::StorageValueOutOfRange); // The scaling law power must be between 0 and 100 => 0% and 100%
			T::Subtensor::set_scaling_law_power(netuid, scaling_law_power);
			log::info!(
				"ScalingLawPowerSet( netuid: {:?} scaling_law_power: {:?} ) ",
				netuid,
				scaling_law_power
			);
			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_weight_limit(origin: OriginFor<T>, netuid: u16, max_weight_limit: u16) -> DispatchResult {
		   T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_max_weight_limit(netuid, max_weight_limit);
			log::info!(
				"MaxWeightLimitSet( netuid: {:?} max_weight_limit: {:?} ) ",
				netuid,
				max_weight_limit
			);
			Ok(())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_immunity_period(origin: OriginFor<T>, netuid: u16, immunity_period: u16) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
	
			T::Subtensor::set_immunity_period(netuid, immunity_period);
			log::info!(
				"ImmunityPeriodSet( netuid: {:?} immunity_period: {:?} ) ",
				netuid,
				immunity_period
			);
			Ok(())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_min_allowed_weights(origin: OriginFor<T>, netuid: u16, min_allowed_weights: u16) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_min_allowed_weights(netuid, min_allowed_weights);
			log::info!(
				"MinAllowedWeightSet( netuid: {:?} min_allowed_weights: {:?} ) ",
				netuid,
				min_allowed_weights
			);
			Ok(())
		}

		#[pallet::call_index(15)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_allowed_uids(origin: OriginFor<T>, netuid: u16, max_allowed_uids: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			ensure!(
				T::Subtensor::get_subnetwork_n(netuid) < max_allowed_uids,
				Error::<T>::MaxAllowedUIdsNotAllowed
			);
			T::Subtensor::set_max_allowed_uids(netuid, max_allowed_uids);
			log::info!(
				"MaxAllowedUidsSet( netuid: {:?} max_allowed_uids: {:?} ) ",
				netuid,
				max_allowed_uids
			);
			Ok(())
		}

		#[pallet::call_index(16)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_kappa(origin: OriginFor<T>, netuid: u16, kappa: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_kappa(netuid, kappa);
			log::info!("KappaSet( netuid: {:?} kappa: {:?} ) ", netuid, kappa);
			Ok(())
		}

		#[pallet::call_index(17)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_rho(origin: OriginFor<T>, netuid: u16, rho: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_rho(netuid, rho);
			log::info!("RhoSet( netuid: {:?} rho: {:?} ) ", netuid, rho);
			Ok(())
		}

		#[pallet::call_index(18)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_activity_cutoff(origin: OriginFor<T>, netuid: u16, activity_cutoff: u16) -> DispatchResult 
		{
			T::Subtensor::ensure_subnet_owner_or_root(origin, netuid)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_activity_cutoff(netuid, activity_cutoff);
			log::info!(
				"ActivityCutoffSet( netuid: {:?} activity_cutoff: {:?} ) ",
				netuid,
				activity_cutoff
			);
			Ok(())
		}

		#[pallet::call_index(19)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_network_registration_allowed(origin: OriginFor<T>, netuid: u16, registration_allowed: bool) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			T::Subtensor::set_network_registration_allowed(netuid, registration_allowed);
			log::info!(
				"NetworkRegistrationAllowed( registration_allowed: {:?} ) ",
				registration_allowed
			);
			Ok(())
		}	

		#[pallet::call_index(20)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_network_pow_registration_allowed(origin: OriginFor<T>, netuid: u16, registration_allowed: bool) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			T::Subtensor::set_network_pow_registration_allowed(netuid, registration_allowed);
			log::info!(
				"NetworkPowRegistrationAllowed( registration_allowed: {:?} ) ",
				registration_allowed
			);
			Ok(())
		}

		#[pallet::call_index(21)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_target_registrations_per_interval(origin: OriginFor<T>, netuid: u16, target_registrations_per_interval: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_target_registrations_per_interval(netuid, target_registrations_per_interval);
			log::info!(
				"RegistrationPerIntervalSet( netuid: {:?} target_registrations_per_interval: {:?} ) ",
				netuid,
				target_registrations_per_interval
			);
			Ok(())
		}

		#[pallet::call_index(22)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_min_burn(origin: OriginFor<T>, netuid: u16, min_burn: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_min_burn(netuid, min_burn);
			log::info!(
				"MinBurnSet( netuid: {:?} min_burn: {:?} ) ",
				netuid,
				min_burn
			);
			Ok(())
		}

		#[pallet::call_index(23)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_burn(origin: OriginFor<T>, netuid: u16, max_burn: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_max_burn(netuid, max_burn);
			log::info!(
				"MaxBurnSet( netuid: {:?} max_burn: {:?} ) ",
				netuid,
				max_burn
			);
			Ok(())
		}

		#[pallet::call_index(24)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_difficulty(origin: OriginFor<T>, netuid: u16, difficulty: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_difficulty(netuid, difficulty);
			log::info!(
				"DifficultySet( netuid: {:?} difficulty: {:?} ) ",
				netuid,
				difficulty
			);
			Ok(())
		}

		#[pallet::call_index(25)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_allowed_validators(origin: OriginFor<T>, netuid: u16, max_allowed_validators: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			ensure!(
				max_allowed_validators <= T::Subtensor::get_max_allowed_uids(netuid),
				Error::<T>::StorageValueOutOfRange
			);
	
	
			T::Subtensor::set_max_allowed_validators(netuid, max_allowed_validators);
			log::info!(
				"MaxAllowedValidatorsSet( netuid: {:?} max_allowed_validators: {:?} ) ",
				netuid,
				max_allowed_validators
			);
			Ok(())
		}

		#[pallet::call_index(26)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_bonds_moving_average(origin: OriginFor<T>, netuid: u16, bonds_moving_average: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
	
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_bonds_moving_average(netuid, bonds_moving_average);
			log::info!(
				"BondsMovingAverageSet( netuid: {:?} bonds_moving_average: {:?} ) ",
				netuid,
				bonds_moving_average
			);
			Ok(())
		}

		#[pallet::call_index(27)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_max_registrations_per_block(origin: OriginFor<T>, netuid: u16, max_registrations_per_block: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_max_registrations_per_block(netuid, max_registrations_per_block);
			log::info!(
				"MaxRegistrationsPerBlock( netuid: {:?} max_registrations_per_block: {:?} ) ",
				netuid,
				max_registrations_per_block
			);
			Ok(())
		}

		#[pallet::call_index(28)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_subnet_owner_cut(origin: OriginFor<T>, subnet_owner_cut: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			T::Subtensor::set_subnet_owner_cut( subnet_owner_cut );
			log::info!(
				"SubnetOwnerCut( subnet_owner_cut: {:?} ) ",
				subnet_owner_cut
			);
			Ok(())
		}

		#[pallet::call_index(29)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_network_rate_limit(origin: OriginFor<T>, rate_limit: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
			T::Subtensor::set_network_rate_limit( rate_limit );
			log::info!(
				"NetworkRateLimit( rate_limit: {:?} ) ",
				rate_limit
			);
			Ok(())
		}
	
		#[pallet::call_index(30)]
		#[pallet::weight(0)]
		pub fn do_sudo_set_tempo(origin: OriginFor<T>, netuid: u16, tempo: u16) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			T::Subtensor::set_tempo(netuid, tempo);
			log::info!("TempoSet( netuid: {:?} tempo: {:?} ) ", netuid, tempo);
			Ok(())
		}

		#[pallet::call_index(31)]
        #[pallet::weight((Weight::from_ref_time(81_000_000)))]
		pub fn do_sudo_registration(origin: OriginFor<T>, netuid: u16, hotkey: T::AccountId, coldkey: T::AccountId, stake: u64, balance: u64) -> DispatchResult 
		{
			ensure_root(origin)?;
			ensure!(
				netuid != T::Subtensor::get_root_netuid(),
				Error::<T>::OperationNotPermittedonRootSubnet
			);
			ensure!(
				T::Subtensor::if_subnet_exist(netuid),
				Error::<T>::NetworkDoesNotExist
			);
			ensure!(
				!Uids::<T>::contains_key(netuid, &hotkey),
				Error::<T>::AlreadyRegistered
			);
	
			T::Subtensor::create_account_if_non_existent(&coldkey, &hotkey);
			ensure!(
				T::Subtensor::coldkey_owns_hotkey(&coldkey, &hotkey),
				Error::<T>::NonAssociatedColdKey
			);
			T::Subtensor::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake);
	
			let balance_to_be_added_as_balance = T::Subtensor::u64_to_balance(balance);
			ensure!(
				balance_to_be_added_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
			T::Subtensor::add_balance_to_coldkey_account(&coldkey, balance_to_be_added_as_balance.unwrap());
	
			let subnetwork_uid: u16;
			let current_block_number: u64 = T::Subtensor::get_current_block_as_u64();
			let current_subnetwork_n: u16 = T::Subtensor::get_subnetwork_n(netuid);
			if current_subnetwork_n < T::Subtensor::get_max_allowed_uids(netuid) {
				// --- 12.1.1 No replacement required, the uid appends the subnetwork.
				// We increment the subnetwork count here but not below.
				subnetwork_uid = current_subnetwork_n;
	
				// --- 12.1.2 Expand subnetwork with new account.
				T::Subtensor::append_neuron(netuid, &hotkey, current_block_number);
				log::info!("add new neuron account");
			} else {
				// --- 12.1.1 Replacement required.
				// We take the neuron with the lowest pruning score here.
				subnetwork_uid = T::Subtensor::get_neuron_to_prune(netuid);
	
				// --- 12.1.1 Replace the neuron account with the new info.
				T::Subtensor::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
				log::info!("prune neuron");
			}
	
			log::info!(
				"NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
				netuid,
				subnetwork_uid,
				hotkey
			);
			Ok(())
		}

		#[pallet::call_index(32)]
        #[pallet::weight((Weight::from_ref_time(81_000_000)
		.saturating_add(T::DbWeight::get().reads(21))
		.saturating_add(T::DbWeight::get().writes(23)), DispatchClass::Operational, Pays::No))]
        pub fn sudo_register(origin: OriginFor<T>, netuid: u16, hotkey: T::AccountId, coldkey: T::AccountId, stake: u64, balance: u64) -> DispatchResult 
		{
            Self::do_sudo_registration(origin, netuid, hotkey, coldkey, stake, balance)
        }

		#[pallet::call_index(33)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_total_issuance(origin: OriginFor<T>, total_issuance: u64) -> DispatchResult 
		{
            T::Subtensor::do_set_total_issuance(origin, total_issuance)
        }

        #[pallet::call_index(34)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_rao_recycled(origin: OriginFor<T>, netuid: u16, rao_recycled: u64) -> DispatchResult 
		{
            T::Subtensor::do_set_rao_recycled(origin, netuid, rao_recycled)
        }

		#[pallet::call_index(35)]
        #[pallet::weight((Weight::from_ref_time(14_000_000)
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_network_immunity_period(origin: OriginFor<T>, immunity_period: u64) -> DispatchResult 
		{
            ensure_root(origin)?;

            T::Subtensor::set_network_immunity_period( immunity_period );

            log::info!(
                "NetworkImmunityPeriod( period: {:?} ) ",
                immunity_period
            );

            Ok(())
        }

        #[pallet::call_index(36)]
        #[pallet::weight((Weight::from_ref_time(14_000_000)
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_network_min_lock_cost(origin: OriginFor<T>, lock_cost: u64) -> DispatchResult 
		{
            ensure_root(origin)?;

            T::Subtensor::set_network_min_lock( lock_cost );

            log::info!(
                "NetworkMinLockCost( lock_cost: {:?} ) ",
                lock_cost
            );

            Ok(())
        }

        #[pallet::call_index(37)]
        #[pallet::weight((Weight::from_ref_time(14_000_000)
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_subnet_limit(origin: OriginFor<T>, max_subnets: u16) -> DispatchResult 
		{
            ensure_root(origin)?;

            SubnetLimit::<T>::set(max_subnets);

            log::info!(
                "SubnetLimit( max_subnets: {:?} ) ",
                max_subnets
            );

            Ok(())
        }

        #[pallet::call_index(38)]
        #[pallet::weight((Weight::from_ref_time(14_000_000)
		.saturating_add(T::DbWeight::get().writes(1)), DispatchClass::Operational, Pays::No))]
        pub fn sudo_set_lock_reduction_interval(origin: OriginFor<T>, interval: u64) -> DispatchResult 
		{
            ensure_root(origin)?;

            T::Subtensor::set_lock_reduction_interval(interval);

            log::info!(
                "NetworkLockReductionInterval( interval: {:?} ) ",
                interval
            );

            Ok(())
        }
    }
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

// Interfaces to interact with other pallets
use sp_runtime::BoundedVec;

pub trait AuraInterface<AuthorityId, MaxAuthorities> {
    fn change_authorities(new: BoundedVec<AuthorityId, MaxAuthorities>);
}

impl<A, M> AuraInterface<A, M> for () {
	fn change_authorities(_: BoundedVec<A, M>) {}
}

///////////////////////////////////////////

pub trait SubtensorInterface 
{
	fn set_default_take(default_take: u16) -> DispatchResult;
	fn set_tx_rate_limit(rate_limit: u64) -> DispatchResult;

	fn set_serving_rate_limit(netuid: u16, rate_limit: u64) -> DispatchResult;

	fn set_max_burn(netuid: u16, max_burn: u64) -> DispatchResult;
	fn set_min_burn(netuid: u16, min_burn: u64) -> DispatchResult;
	fn set_burn(netuid: u16, burn: u64) -> DispatchResult;

	fn set_max_difficulty(netuid: u16, max_diff: u64) -> DispatchResult;
	fn set_min_difficulty(netuid: u16, min_diff: u64) -> DispatchResult;
	fn set_difficulty(netuid: u16, diff: u64) -> DispatchResult;

	fn set_weights_rate_limit(netuid: u16, rate_limit: u64) -> DispatchResult;

	fn set_weights_version_key(netuid: u16, version: u64) -> DispatchResult;

	fn set_bonds_moving_average(netuid: u16, moving_average: u64) -> DispatchResult;

	fn set_max_allowed_validators(netuid: u16, max_validators: u16) -> DispatchResult;

	fn get_root_netuid() -> u16;
	fn if_subnet_exist(netuid: u16) -> bool;
	fn create_account_if_non_existent(coldkey: &AccountId, hotkey: &T::AccountId);
	fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool;
	fn increase_stake_on_coldkey_hotkey_account(coldkey: &T::AccountId, hotkey: &<T as frame_system::Config>::AccountId, increment: u64);
	fn u64_to_balance(input: u64) -> Option<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>;
	fn add_balance_to_coldkey_account(coldkey: &T::AccountId, amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance);
	fn get_current_block_as_u64() -> u64;
	fn get_subnetwork_n(netuid: u16) -> u16;
	fn get_max_allowed_uids(netuid: u16) -> u16;
	fn append_neuron(netuid: u16, new_hotkey: &T::AccountId, block_number: u64);
	fn get_neuron_to_prune(netuid: u16) -> u16;
	fn replace_neuron(netuid: u16, uid_to_replace: u16, new_hotkey: &T::AccountId, block_number: u64);
	fn do_set_total_issuance(origin: T::RuntimeOrigin, total_issuance: u64) -> DispatchResult;
	fn do_set_rao_recycled(origin: T::RuntimeOrigin, netuid: u16, rao_recycled: u64) -> DispatchResult;
	fn set_network_immunity_period(net_immunity_period: u64);
	fn set_network_min_lock(net_min_lock: u64);
}