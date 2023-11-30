use crate as pallet_admin_utils;
use frame_support::traits::{ConstU16, ConstU64};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		AdminUtils: pallet_admin_utils,
		SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>}
	}
);

#[allow(dead_code)]
pub type SubtensorEvent = pallet_subtensor::Event<Test>;

impl pallet_subtensor::Config for Test 
{
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InitialIssuance = InitialIssuance;
    type SudoRuntimeCall = TestRuntimeCall;
    type CouncilOrigin = frame_system::EnsureSigned<AccountId>;
    type SenateMembers = ManageSenateMembers;
    type TriumvirateInterface = TriumvirateVotes;

    type InitialMinAllowedWeights = InitialMinAllowedWeights;
    type InitialEmissionValue = InitialEmissionValue;
    type InitialMaxWeightsLimit = InitialMaxWeightsLimit;
    type InitialTempo = InitialTempo;
    type InitialDifficulty = InitialDifficulty;
    type InitialAdjustmentInterval = InitialAdjustmentInterval;
    type InitialAdjustmentAlpha = InitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = InitialTargetRegistrationsPerInterval;
    type InitialRho = InitialRho;
    type InitialKappa = InitialKappa;
    type InitialMaxAllowedUids = InitialMaxAllowedUids;
    type InitialValidatorPruneLen = InitialValidatorPruneLen;
    type InitialScalingLawPower = InitialScalingLawPower;
    type InitialImmunityPeriod = InitialImmunityPeriod;
    type InitialActivityCutoff = InitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = InitialMaxRegistrationsPerBlock;
    type InitialPruningScore = InitialPruningScore;
    type InitialBondsMovingAverage = InitialBondsMovingAverage;
    type InitialMaxAllowedValidators = InitialMaxAllowedValidators;
    type InitialDefaultTake = InitialDefaultTake;
    type InitialWeightsVersionKey = InitialWeightsVersionKey;
    type InitialMaxDifficulty = InitialMaxDifficulty;
    type InitialMinDifficulty = InitialMinDifficulty;
    type InitialServingRateLimit = InitialServingRateLimit;
    type InitialTxRateLimit = InitialTxRateLimit;
    type InitialBurn = InitialBurn;
    type InitialMaxBurn = InitialMaxBurn;
    type InitialMinBurn = InitialMinBurn;
    type InitialRAORecycledForRegistration = InitialRAORecycledForRegistration;
    type InitialSenateRequiredStakePercentage = InitialSenateRequiredStakePercentage;
    type InitialNetworkImmunityPeriod = InitialNetworkImmunityPeriod;
    type InitialNetworkMinAllowedUids = InitialNetworkMinAllowedUids;
    type InitialNetworkMinLockCost = InitialNetworkMinLockCost;
    type InitialSubnetOwnerCut = InitialSubnetOwnerCut;
    type InitialNetworkLockReductionInterval = InitialNetworkLockReductionInterval;
    type InitialSubnetLimit = InitialSubnetLimit;
    type InitialNetworkRateLimit = InitialNetworkRateLimit;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub struct SubtensorInterface;

impl pallet_admin_utils::SubtensorInterface<AccountId, <pallet_balances::Pallet<Runtime> as frame_support::traits::Currency<AccountId>>::Balance, RuntimeOrigin> for SubtensorInterface
{
    fn set_default_take(default_take: u16)
    {
        SubtensorModule::set_default_take(default_take);
    }

	fn set_tx_rate_limit(rate_limit: u64)
    {
        SubtensorModule::set_tx_rate_limit(rate_limit);
    }

	fn set_serving_rate_limit(netuid: u16, rate_limit: u64)
    {
        SubtensorModule::set_serving_rate_limit(netuid, rate_limit);
    }

	fn set_max_burn(netuid: u16, max_burn: u64)
    {
        SubtensorModule::set_max_burn(netuid, max_burn);
    }

	fn set_min_burn(netuid: u16, min_burn: u64)
    {
        SubtensorModule::set_min_burn(netuid, min_burn);
    }

	fn set_burn(netuid: u16, burn: u64)
    {
        SubtensorModule::set_burn(netuid, burn);
    }

	fn set_max_difficulty(netuid: u16, max_diff: u64)
    {
        SubtensorModule::set_max_difficulty(netuid, max_diff);
    }

	fn set_min_difficulty(netuid: u16, min_diff: u64)
    {
        SubtensorModule::set_min_difficulty(netuid, min_diff);
    }

	fn set_difficulty(netuid: u16, diff: u64)
    {
        SubtensorModule::set_difficulty(netuid, diff);
    }

	fn set_weights_rate_limit(netuid: u16, rate_limit: u64)
    {
        SubtensorModule::set_weights_set_rate_limit(netuid, rate_limit);
    }

	fn set_weights_version_key(netuid: u16, version: u64)
    {
        SubtensorModule::set_weights_version_key(netuid, version);
    }

	fn set_bonds_moving_average(netuid: u16, moving_average: u64)
    {
        SubtensorModule::set_bonds_moving_average(netuid, moving_average);
    }

	fn set_max_allowed_validators(netuid: u16, max_validators: u16)
    {
        SubtensorModule::set_max_allowed_validators(netuid, max_validators);
    }

	fn get_root_netuid() -> u16
    {
        return SubtensorModule::get_root_netuid();
    }

	fn if_subnet_exist(netuid: u16) -> bool
    {
        return SubtensorModule::if_subnet_exist(netuid);
    }

	fn create_account_if_non_existent(coldkey: &AccountId, hotkey: &AccountId)
    {
        return SubtensorModule::create_account_if_non_existent(coldkey, hotkey);
    }

	fn coldkey_owns_hotkey(coldkey: &AccountId, hotkey: &AccountId) -> bool
    {
        return SubtensorModule::coldkey_owns_hotkey(coldkey, hotkey);
    }

	fn increase_stake_on_coldkey_hotkey_account(coldkey: &AccountId, hotkey: &AccountId, increment: u64)
    {
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(coldkey, hotkey, increment);
    }

	fn u64_to_balance(input: u64) -> Option<Balance>
    {
        return SubtensorModule::u64_to_balance(input);
    }

	fn add_balance_to_coldkey_account(coldkey: &AccountId, amount: Balance)
    {
        SubtensorModule::add_balance_to_coldkey_account(coldkey, amount);
    }

	fn get_current_block_as_u64() -> u64
    {
        return SubtensorModule::get_current_block_as_u64();
    }

	fn get_subnetwork_n(netuid: u16) -> u16
    {
        return SubtensorModule::get_subnetwork_n(netuid);
    }

	fn get_max_allowed_uids(netuid: u16) -> u16
    {
        return SubtensorModule::get_max_allowed_uids(netuid);
    }

	fn append_neuron(netuid: u16, new_hotkey: &AccountId, block_number: u64)
    {
        return SubtensorModule::append_neuron(netuid, new_hotkey, block_number);
    }

	fn get_neuron_to_prune(netuid: u16) -> u16
    {
        return SubtensorModule::get_neuron_to_prune(netuid);
    }

	fn replace_neuron(netuid: u16, uid_to_replace: u16, new_hotkey: &AccountId, block_number: u64)
    {
        SubtensorModule::replace_neuron(netuid, uid_to_replace, new_hotkey, block_number);
    }

	fn do_set_total_issuance(origin: RuntimeOrigin, total_issuance: u64)
    {
        SubtensorModule::do_set_total_issuance(origin, total_issuance);
    }

	fn set_network_immunity_period(net_immunity_period: u64)
    {
        SubtensorModule::set_network_immunity_period(net_immunity_period);
    }

	fn set_network_min_lock(net_min_lock: u64)
    {
        SubtensorModule::set_network_min_lock(net_min_lock);
    }

    fn set_subnet_limit(limit: u16)
    {
        SubtensorModule::set_max_subnets(limit);
    }

    fn set_lock_reduction_interval(interval: u64)
    {
        SubtensorModule::set_lock_reduction_interval(interval);
    }

    fn set_tempo(netuid: u16, tempo: u16)
    {
        SubtensorModule::set_tempo(netuid, tempo);
    }

    fn set_subnet_owner_cut(subnet_owner_cut: u16)
    {
        SubtensorModule::set_subnet_owner_cut(subnet_owner_cut);
    }

    fn set_network_rate_limit(limit: u64)
    {
        SubtensorModule::set_network_rate_limit(limit);
    }

    fn set_max_registrations_per_block(netuid: u16, max_registrations_per_block: u16)
    {
        SubtensorModule::set_max_registrations_per_block(netuid, max_registrations_per_block);
    }

    fn set_adjustment_alpha(netuid: u16, adjustment_alpha: u64)
    {
        SubtensorModule::set_adjustment_alpha(netuid, adjustment_alpha);
    }

    fn set_target_registrations_per_interval(netuid: u16, target_registrations_per_interval: u16)
    {
        SubtensorModule::set_target_registrations_per_interval(netuid, target_registrations_per_interval);
    }

    fn set_network_pow_registration_allowed(netuid: u16, registration_allowed: bool)
    {
        SubtensorModule::set_network_pow_registration_allowed(netuid, registration_allowed);
    }

    fn set_network_registration_allowed(netuid: u16, registration_allowed: bool)
    {
        SubtensorModule::set_network_pow_registration_allowed(netuid, registration_allowed);
    }

    fn set_activity_cutoff(netuid: u16, activity_cutoff: u16)
    {
        SubtensorModule::set_activity_cutoff(netuid, activity_cutoff);
    }

    fn ensure_subnet_owner_or_root(o: RuntimeOrigin, netuid: u16) -> Result<(), DispatchError>
    {
        return SubtensorModule::ensure_subnet_owner_or_root(o, netuid);
    }

    fn set_rho(netuid: u16, rho: u16)
    {
        SubtensorModule::set_rho(netuid, rho);
    }

    fn set_kappa(netuid: u16, kappa: u16)
    {
        SubtensorModule::set_kappa(netuid, kappa);
    }

    fn set_max_allowed_uids(netuid: u16, max_allowed: u16)
    {
        SubtensorModule::set_max_allowed_uids(netuid, max_allowed);
    }

    fn set_min_allowed_weights(netuid: u16, min_allowed_weights: u16)
    {
        SubtensorModule::set_min_allowed_weights(netuid, min_allowed_weights);
    }

    fn set_immunity_period(netuid: u16, immunity_period: u16)
    {
        SubtensorModule::set_immunity_period(netuid, immunity_period);
    }

    fn set_max_weight_limit(netuid: u16, max_weight_limit: u16)
    {
        SubtensorModule::set_max_weight_limit(netuid, max_weight_limit);
    }

    fn set_scaling_law_power(netuid: u16, scaling_law_power: u16)
    {
        SubtensorModule::set_scaling_law_power(netuid, scaling_law_power);
    }

    fn set_validator_prune_len(netuid: u16, validator_prune_len: u64)
    {
        SubtensorModule::set_validator_prune_len(netuid, validator_prune_len);
    }

    fn set_adjustment_interval(netuid: u16, adjustment_interval: u16)
    {
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
    }

    fn set_weights_set_rate_limit(netuid: u16, weights_set_rate_limit: u64)
    {
        SubtensorModule::set_weights_set_rate_limit(netuid, weights_set_rate_limit);
    }

    fn set_rao_recycled(netuid: u16, rao_recycled: u64)
    {
        SubtensorModule::set_rao_recycled(netuid, rao_recycled);
    }

    fn is_hotkey_registered_on_network(netuid: u16, hotkey: &AccountId) -> bool
    {
        return SubtensorModule::is_hotkey_registered_on_network(netuid, hotkey);
    }


}

impl pallet_admin_utils::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = AuraId;
    type MaxAuthorities = ConstU32<32>;
    type Aura = AuraPalletIntrf;
    type Currency = ();
    type Subtensor = SubtensorInterface;
}


#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities 
{
    sp_tracing::try_init_simple();
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}