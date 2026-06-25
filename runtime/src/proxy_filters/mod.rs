mod call_groups;

use alloc::{format, vec, vec::Vec};

use call_groups::*;
use frame_support::traits::InstanceFilter;
use frame_system::Call as SystemCall;
use pallet_admin_utils::Call as AdminUtilsCall;
use pallet_balances::Call as BalancesCall;
use pallet_subtensor::Call as SubtensorCall;
use pallet_sudo::Call as SudoCall;
use subtensor_runtime_common::{
    CallConstraint, CallFilterMetadata, CallInfo, FilterMode, ProxyFilterInfo, ProxyType,
    ProxyTypeInfo, SMALL_ALPHA_TRANSFER_LIMIT, SMALL_TRANSFER_LIMIT,
};

use crate::RuntimeCall;

pub(crate) fn proxy_type_filter(proxy_type: &ProxyType, call: &RuntimeCall) -> bool {
    true
    // match proxy_type {
    //     ProxyType::Any => true,
    //     ProxyType::NonTransfer => non_transfer_filter(call),
    //     ProxyType::NonFungible => non_fungible_filter(call),
    //     ProxyType::Transfer => transfer_filter(call),
    //     ProxyType::SmallTransfer => small_transfer_filter(call),
    //     ProxyType::Owner => owner_filter(call),
    //     ProxyType::NonCritical => non_critical_filter(call),
    //     ProxyType::Triumvirate
    //     | ProxyType::Senate
    //     | ProxyType::Governance
    //     | ProxyType::RootWeights => false,
    //     ProxyType::Staking => staking_filter(call),
    //     ProxyType::Registration => registration_filter(call),
    //     ProxyType::ChildKeys => child_keys_filter(call),
    //     ProxyType::SudoUncheckedSetCode => sudo_unchecked_set_code_filter(call),
    //     ProxyType::SwapHotkey => hotkey_swap_filter(call),
    //     ProxyType::SubnetLeaseBeneficiary => subnet_lease_beneficiary_filter(call),
    //     ProxyType::RootClaim => root_claim_filter(call),
    // }
}

impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, call: &RuntimeCall) -> bool {
        true
    }
}
