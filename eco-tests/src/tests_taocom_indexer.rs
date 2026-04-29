//! Indexer-contract tests for the TAO.com / ecosystem indexer.
//! Any modification in these tests will notify the member responsible
//! for the communication between protocol and the indexer team.

#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::traits::OnInitialize;
use pallet_subtensor::*;
use pallet_subtensor_swap as swap;
use sp_core::U256;
use subtensor_runtime_common::{MechId, NetUid, NetUidStorageIndex, TaoBalance};

use super::helpers::*;
use super::mock::*;

#[test]
fn indexer_neuron_per_subnet_vectors() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let netuid_idx = NetUidStorageIndex::from(netuid);

        let _: Vec<bool> = Active::<Test>::get(netuid);
        let _: Vec<u16> = Consensus::<Test>::get(netuid);
        let _: Vec<u16> = Dividends::<Test>::get(netuid);
        let _: Vec<u16> = Incentive::<Test>::get(netuid_idx);
        let _: Vec<u64> = LastUpdate::<Test>::get(netuid_idx);
        let _: Vec<bool> = ValidatorPermit::<Test>::get(netuid);
        let _: Vec<u16> = ValidatorTrust::<Test>::get(netuid);
        let _ = Emission::<Test>::get(netuid);
    });
}

#[test]
fn indexer_neuron_uid_maps() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let netuid_idx = NetUidStorageIndex::from(netuid);
        let hotkey = U256::from(1);
        let uid: u16 = 0;

        let _: Option<u16> = Uids::<Test>::get(netuid, hotkey);
        let _: U256 = Keys::<Test>::get(netuid, uid);
        let _: Vec<(u16, u16)> = Weights::<Test>::get(netuid_idx, uid);
        let _: Vec<(u16, u16)> = Bonds::<Test>::get(netuid_idx, uid);
        let _: Option<AxonInfoOf> = Axons::<Test>::get(netuid, hotkey);
    });
}

#[test]
fn indexer_ownership_and_childkey_graph() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let key = U256::from(42);

        let _: U256 = Owner::<Test>::get(key);
        let _: U256 = SubnetOwner::<Test>::get(netuid);
        let _: U256 = SubnetOwnerHotkey::<Test>::get(netuid);
        let _: Vec<(u64, U256)> = ChildKeys::<Test>::get(key, netuid);
        let _: Vec<(u64, U256)> = ParentKeys::<Test>::get(key, netuid);
    });
}

#[test]
fn indexer_stake_and_alpha_shares() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        let _ = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let _ = TotalHotkeyShares::<Test>::get(hotkey, netuid);
        let _ = TotalHotkeySharesV2::<Test>::get(hotkey, netuid);
        let _ = Alpha::<Test>::get((hotkey, coldkey, netuid));
        let _ = AlphaV2::<Test>::get((hotkey, coldkey, netuid));
    });
}

#[test]
fn indexer_subnet_metadata() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let coldkey = U256::from(7);

        let _: u16 = TotalNetworks::<Test>::get();
        let _: Vec<u8> = TokenSymbol::<Test>::get(netuid);
        let _ = IdentitiesV2::<Test>::get(coldkey);
        let _ = SubnetIdentitiesV3::<Test>::get(netuid);
        let _: MechId = MechanismCountCurrent::<Test>::get(netuid);
        let _: Option<u64> = FirstEmissionBlockNumber::<Test>::get(netuid);
    });
}

#[test]
fn indexer_subnet_pool_and_emissions() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);

        let _ = SubnetMovingPrice::<Test>::get(netuid);
        let _: u128 = SubnetVolume::<Test>::get(netuid);
        let _ = SubnetTAO::<Test>::get(netuid);
        let _ = SubnetAlphaIn::<Test>::get(netuid);
        let _ = SubnetAlphaOut::<Test>::get(netuid);
        let _ = SubnetTaoInEmission::<Test>::get(netuid);
        let _ = SubnetAlphaInEmission::<Test>::get(netuid);
        let _ = SubnetAlphaOutEmission::<Test>::get(netuid);
        let _ = PendingValidatorEmission::<Test>::get(netuid);
        let _ = PendingServerEmission::<Test>::get(netuid);

        let _ = swap::AlphaSqrtPrice::<Test>::get(netuid);
    });
}

#[test]
fn indexer_subnet_hyperparams() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);

        let _: u16 = Rho::<Test>::get(netuid);
        let _: u16 = Kappa::<Test>::get(netuid);
        let _: u16 = ImmunityPeriod::<Test>::get(netuid);
        let _: u16 = MinAllowedWeights::<Test>::get(netuid);
        let _: u16 = MaxWeightsLimit::<Test>::get(netuid);
        let _: u16 = Tempo::<Test>::get(netuid);
        let _: u64 = MinDifficulty::<Test>::get(netuid);
        let _: u64 = MaxDifficulty::<Test>::get(netuid);
        let _: u64 = WeightsVersionKey::<Test>::get(netuid);
        let _: u64 = WeightsSetRateLimit::<Test>::get(netuid);
        let _: u16 = AdjustmentInterval::<Test>::get(netuid);
        let _: u16 = ActivityCutoff::<Test>::get(netuid);
        let _: bool = NetworkRegistrationAllowed::<Test>::get(netuid);
        let _: u16 = TargetRegistrationsPerInterval::<Test>::get(netuid);
        let _ = MinBurn::<Test>::get(netuid);
        let _ = MaxBurn::<Test>::get(netuid);
        let _: u64 = BondsMovingAverage::<Test>::get(netuid);
        let _: u16 = MaxRegistrationsPerBlock::<Test>::get(netuid);
        let _: u64 = ServingRateLimit::<Test>::get(netuid);
        let _: u16 = MaxAllowedValidators::<Test>::get(netuid);
        let _: u64 = Difficulty::<Test>::get(netuid);
        let _ = AdjustmentAlpha::<Test>::get(netuid);
        let _: u64 = RevealPeriodEpochs::<Test>::get(netuid);
        let _: bool = CommitRevealWeightsEnabled::<Test>::get(netuid);
        let _: bool = LiquidAlphaOn::<Test>::get(netuid);
        let _: i16 = AlphaSigmoidSteepness::<Test>::get(netuid);
        let _: bool = Yuma3On::<Test>::get(netuid);
        let _: bool = BondsResetOn::<Test>::get(netuid);
        let _: (u16, u16) = AlphaValues::<Test>::get(netuid);
        let _: RecycleOrBurnEnum = RecycleOrBurn::<Test>::get(netuid);
    });
}

#[test]
fn indexer_step_and_toggles() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);

        let _: u64 = BlocksSinceLastStep::<Test>::get(netuid);
        let _: u64 = LastMechansimStepBlock::<Test>::get(netuid);
        let _ = LastRateLimitedBlock::<Test>::iter().next();
        let _: bool = TransferToggle::<Test>::get(netuid);
        let _: bool = swap::EnabledUserLiquidity::<Test>::get(netuid);
    });
}

#[test]
fn indexer_network_economics() {
    new_test_ext(1).execute_with(|| {
        let _: TaoBalance = NetworkMinLockCost::<Test>::get();
        let _: TaoBalance = NetworkLastLockCost::<Test>::get();
        let _: u64 = NetworkLockReductionInterval::<Test>::get();
        let _: TaoBalance = TotalIssuance::<Test>::get();
    });
}

#[test]
fn indexer_runtime_api_signatures() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1u16);
        let coldkey = U256::from(3);
        let hotkey = U256::from(4);

        let _ = SubtensorModule::get_delegate(hotkey);

        let _ = SubtensorModule::get_stake_info_for_coldkeys(vec![coldkey]);

        use subtensor_swap_interface::SwapHandler;
        let _ = <Test as pallet_subtensor::Config>::SwapInterface::current_alpha_price(netuid);
    });
}
