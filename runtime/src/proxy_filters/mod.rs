mod call_groups;

use alloc::{format, vec::Vec};

use call_groups::*;
use frame_support::traits::{Contains, InstanceFilter};
use subtensor_runtime_common::{
    CallFilterMetadata, FilterMode, ProxyFilterInfo, ProxyType, ProxyTypeInfo,
};

use crate::RuntimeCall;

// ============================================================================
// Per-proxy allow-lists
//
// Each proxy type's permission set is an *additive* union of whole call groups
// from `call_groups`. A call a proxy does not list is denied. `Any` allows
// everything; the deprecated governance proxies allow nothing.
//
// `Contains` for a tuple is logical OR (any member matches), so these aliases
// read as "allow if the call is in any of these groups".
// ============================================================================

/// Infrastructure pallets every broad proxy may use. Excludes `SudoCalls`
/// (granted only to `NonTransfer`/`NonFungible`).
type InfraCommon = (
    SystemCalls,
    TimestampCalls,
    GrandpaCalls,
    UtilityCalls,
    MultisigCalls,
    PreimageCalls,
    SchedulerCalls,
    ProxyCalls,
    CommitmentsCalls,
    SafeModeCalls,
    EthereumCalls,
    EvmCalls,
    BaseFeeCalls,
    DrandCalls,
    CrowdloanCalls,
    SwapCalls,
    ContractsCalls,
    MevShieldCalls,
    LimitOrdersCalls,
);

/// All admin-utils configuration. Every broad proxy historically allowed every
/// admin call; owner-key rotation is still gated by the dispatch's own origin
/// check.
type AdminAll = (LeaseConfigCalls, OwnerConfigCalls, OwnerKeyCalls);

/// `Transfer`: liquid value movement.
type TransferAllowed = (BalanceTransferCalls, StakeTransferCalls);

/// `Staking`: stake position management plus root-claim mode selection.
type StakingAllowed = (StakeManagementCalls, RootClaimTypeCalls);

/// `Registration`: acquire a slot (POW or by burn).
type RegistrationAllowed = (PowRegistrationCalls, BurnedRegistrationCalls);

/// `Owner`: run a subnet you own (identity + all owner-settable admin config).
type OwnerAllowed = (SubnetIdentityCalls, LeaseConfigCalls, OwnerConfigCalls);

/// `SubnetLeaseBeneficiary`: operate a leased subnet (activation, identity, and
/// the lease-tunable admin config).
type SubnetLeaseAllowed = (SubnetActivationCalls, SubnetIdentityCalls, LeaseConfigCalls);

/// `NonTransfer`: everything except liquid value movement and coldkey swaps.
type NonTransferAllowed = (
    InfraCommon,
    AdminAll,
    SudoCalls,
    StakeManagementCalls,
    PowRegistrationCalls,
    BurnedRegistrationCalls,
    RootRegistrationCalls,
    HotkeySwapCalls,
    CriticalNetworkCalls,
    ChildKeyCalls,
    RootClaimCalls,
    RootClaimTypeCalls,
    SubnetIdentityCalls,
    SubnetActivationCalls,
    SubtensorCommonCalls,
);

/// `NonFungible`: nothing that moves TAO/alpha and no key swaps.
type NonFungibleAllowed = (
    InfraCommon,
    AdminAll,
    SudoCalls,
    PowRegistrationCalls,
    CriticalNetworkCalls,
    ChildKeyCalls,
    RootClaimCalls,
    RootClaimTypeCalls,
    SubnetIdentityCalls,
    SubnetActivationCalls,
    SubtensorCommonCalls,
);

/// `NonCritical`: day-to-day operations including value movement, but no sudo,
/// network dissolution, root/burned registration, or coldkey swaps.
type NonCriticalAllowed = (
    InfraCommon,
    AdminAll,
    BalanceTransferCalls,
    BalanceMaintenanceCalls,
    StakeManagementCalls,
    StakeTransferCalls,
    PowRegistrationCalls,
    HotkeySwapCalls,
    ChildKeyCalls,
    RootClaimCalls,
    RootClaimTypeCalls,
    SubnetIdentityCalls,
    SubnetActivationCalls,
    SubtensorCommonCalls,
);

pub(crate) fn proxy_type_filter(proxy_type: &ProxyType, call: &RuntimeCall) -> bool {
    match proxy_type {
        ProxyType::Any => true,
        ProxyType::Owner => OwnerAllowed::contains(call),
        ProxyType::NonCritical => NonCriticalAllowed::contains(call),
        ProxyType::NonTransfer => NonTransferAllowed::contains(call),
        ProxyType::NonFungible => NonFungibleAllowed::contains(call),
        ProxyType::Staking => StakingAllowed::contains(call),
        ProxyType::Registration => RegistrationAllowed::contains(call),
        ProxyType::Transfer => TransferAllowed::contains(call),
        ProxyType::SmallTransfer => SmallTransferCalls::contains(call),
        ProxyType::ChildKeys => ChildKeyCalls::contains(call),
        ProxyType::SwapHotkey => HotkeySwapCalls::contains(call),
        ProxyType::SubnetLeaseBeneficiary => SubnetLeaseAllowed::contains(call),
        ProxyType::RootClaim => RootClaimCalls::contains(call),
        ProxyType::SudoUncheckedSetCode => SudoSetCodeCalls::contains(call),
        ProxyType::Triumvirate
        | ProxyType::Senate
        | ProxyType::Governance
        | ProxyType::RootWeights => false,
    }
}

impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, call: &RuntimeCall) -> bool {
        proxy_type_filter(self, call)
    }

    fn is_superset(&self, other: &Self) -> bool {
        match (self, other) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => {
                !matches!(other, ProxyType::Transfer | ProxyType::SmallTransfer)
            }
            (ProxyType::Transfer, ProxyType::SmallTransfer) => true,
            _ => false,
        }
    }
}

// ============================================================================
// Runtime API metadata
//
// The client-facing allowlist view is derived from the same call groups the
// filter uses, so the two cannot drift.
// ============================================================================

/// The filter mode (allow-all or an explicit allowlist) for one proxy type.
fn proxy_filter_mode(proxy_type: ProxyType) -> FilterMode {
    match proxy_type {
        ProxyType::Any => FilterMode::AllowAll,
        ProxyType::Owner => FilterMode::Allow(OwnerAllowed::call_infos()),
        ProxyType::NonCritical => FilterMode::Allow(NonCriticalAllowed::call_infos()),
        ProxyType::NonTransfer => FilterMode::Allow(NonTransferAllowed::call_infos()),
        ProxyType::NonFungible => FilterMode::Allow(NonFungibleAllowed::call_infos()),
        ProxyType::Staking => FilterMode::Allow(StakingAllowed::call_infos()),
        ProxyType::Registration => FilterMode::Allow(RegistrationAllowed::call_infos()),
        ProxyType::Transfer => FilterMode::Allow(TransferAllowed::call_infos()),
        ProxyType::SmallTransfer => FilterMode::Allow(SmallTransferCalls::call_infos()),
        ProxyType::ChildKeys => FilterMode::Allow(ChildKeyCalls::call_infos()),
        ProxyType::SwapHotkey => FilterMode::Allow(HotkeySwapCalls::call_infos()),
        ProxyType::SubnetLeaseBeneficiary => FilterMode::Allow(SubnetLeaseAllowed::call_infos()),
        ProxyType::RootClaim => FilterMode::Allow(RootClaimCalls::call_infos()),
        ProxyType::SudoUncheckedSetCode => FilterMode::Allow(SudoSetCodeCalls::call_infos()),
        ProxyType::Triumvirate
        | ProxyType::Senate
        | ProxyType::Governance
        | ProxyType::RootWeights => FilterMode::Allow(Vec::new()),
    }
}

/// Every proxy type with its on-chain index and deprecation flag.
pub fn get_all_proxy_type_infos() -> Vec<ProxyTypeInfo> {
    (0u8..=u8::MAX)
        .filter_map(|index| {
            ProxyType::try_from(index).ok().map(|proxy_type| ProxyTypeInfo {
                name: format!("{:?}", proxy_type).into_bytes(),
                index,
                deprecated: proxy_type.is_deprecated(),
            })
        })
        .collect()
}

/// Filter metadata for the requested proxy types (all of them when `None`).
pub fn get_proxy_filters(proxy_types: Option<Vec<u8>>) -> Vec<ProxyFilterInfo> {
    (0u8..=u8::MAX)
        .filter_map(|index| ProxyType::try_from(index).ok().map(|proxy_type| (index, proxy_type)))
        .filter(|(index, _)| {
            proxy_types
                .as_ref()
                .map_or(true, |selected| selected.contains(index))
        })
        .map(|(index, proxy_type)| ProxyFilterInfo {
            proxy_type: index,
            name: format!("{:?}", proxy_type).into_bytes(),
            deprecated: proxy_type.is_deprecated(),
            filter_mode: proxy_filter_mode(proxy_type),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use alloc::{
        collections::BTreeSet,
        string::{String, ToString},
        vec,
    };
    use frame_support::traits::GetCallMetadata;
    use subtensor_runtime_common::CallInfo;

    fn call_name(info: &CallInfo) -> String {
        format!(
            "{}::{}",
            String::from_utf8_lossy(&info.pallet_name),
            String::from_utf8_lossy(&info.call_name)
        )
    }

    /// All `pallet::call` names in the runtime, straight from `RuntimeCall`
    /// metadata.
    fn all_runtime_calls() -> BTreeSet<String> {
        RuntimeCall::get_module_names()
            .iter()
            .flat_map(|module| {
                RuntimeCall::get_call_names(module)
                    .iter()
                    .map(move |call| format!("{}::{}", module, call))
            })
            .collect()
    }

    fn group_calls<G: CallFilterMetadata>() -> BTreeSet<String> {
        G::call_infos().iter().map(call_name).collect()
    }

    /// The set of calls a proxy type allows, taken from its metadata view.
    fn allowed_calls(proxy_type: ProxyType) -> BTreeSet<String> {
        match proxy_filter_mode(proxy_type) {
            FilterMode::AllowAll => all_runtime_calls(),
            FilterMode::Allow(infos) => infos.iter().map(call_name).collect(),
        }
    }

    fn expected(calls: &[&str]) -> BTreeSet<String> {
        calls.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn any_allows_everything_and_deprecated_allow_nothing() {
        assert_eq!(allowed_calls(ProxyType::Any), all_runtime_calls());
        for deprecated in [
            ProxyType::Triumvirate,
            ProxyType::Senate,
            ProxyType::Governance,
            ProxyType::RootWeights,
        ] {
            assert!(allowed_calls(deprecated).is_empty());
        }
    }

    // Broad proxies are specified subtractively here (all calls minus a few
    // denied groups) and checked against the additive composition in the filter.
    // Because the inventory groups partition every runtime call, the two must
    // agree exactly; a missing or extra group in the filter shows up as a diff.
    #[test]
    fn non_transfer_is_everything_but_transfers_and_coldkey_swaps() {
        let denied = &(&group_calls::<BalanceTransferCalls>()
            | &group_calls::<BalanceMaintenanceCalls>())
            | &(&group_calls::<StakeTransferCalls>() | &group_calls::<ColdkeySwapCalls>());
        assert_eq!(allowed_calls(ProxyType::NonTransfer), &all_runtime_calls() - &denied);
    }

    #[test]
    fn non_fungible_is_everything_but_value_movement_and_key_swaps() {
        let denied = &(&(&group_calls::<BalanceTransferCalls>()
            | &group_calls::<BalanceMaintenanceCalls>())
            | &(&group_calls::<StakeManagementCalls>() | &group_calls::<StakeTransferCalls>()))
            | &(&(&group_calls::<BurnedRegistrationCalls>()
                | &group_calls::<RootRegistrationCalls>())
                | &(&group_calls::<HotkeySwapCalls>() | &group_calls::<ColdkeySwapCalls>()));
        assert_eq!(allowed_calls(ProxyType::NonFungible), &all_runtime_calls() - &denied);
    }

    #[test]
    fn non_critical_is_everything_but_sudo_and_critical_ops() {
        let denied = &(&(&group_calls::<SudoCalls>() | &group_calls::<BurnedRegistrationCalls>())
            | &(&group_calls::<RootRegistrationCalls>() | &group_calls::<CriticalNetworkCalls>()))
            | &group_calls::<ColdkeySwapCalls>();
        assert_eq!(allowed_calls(ProxyType::NonCritical), &all_runtime_calls() - &denied);
    }

    #[test]
    fn owner_is_admin_config_minus_owner_keys_plus_subnet_identity() {
        let admin_utils: BTreeSet<String> = all_runtime_calls()
            .into_iter()
            .filter(|name| name.starts_with("AdminUtils::"))
            .collect();
        let expected = &(&admin_utils - &group_calls::<OwnerKeyCalls>())
            | &group_calls::<SubnetIdentityCalls>();
        assert_eq!(allowed_calls(ProxyType::Owner), expected);
    }

    #[test]
    fn subnet_lease_boundaries() {
        let lease = allowed_calls(ProxyType::SubnetLeaseBeneficiary);
        // Can activate and tune the subnet...
        assert!(lease.contains("SubtensorModule::start_call"));
        assert!(lease.contains("SubtensorModule::set_subnet_identity"));
        assert!(lease.contains("AdminUtils::sudo_set_kappa"));
        // ...but not touch owner-only economics, owner keys, or authorities.
        assert!(!lease.contains("AdminUtils::sudo_set_total_issuance"));
        assert!(!lease.contains("AdminUtils::sudo_set_sn_owner_hotkey"));
        assert!(!lease.contains("AdminUtils::swap_authorities"));
        assert!(!lease.contains("SubtensorModule::terminate_lease"));
    }

    #[test]
    fn narrow_proxies_have_exact_allow_lists() {
        assert_eq!(
            allowed_calls(ProxyType::Transfer),
            expected(&[
                "Balances::transfer_keep_alive",
                "Balances::transfer_allow_death",
                "Balances::transfer_all",
                "SubtensorModule::transfer_stake",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::SmallTransfer),
            expected(&[
                "Balances::transfer_keep_alive",
                "Balances::transfer_allow_death",
                "SubtensorModule::transfer_stake",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::Staking),
            expected(&[
                "SubtensorModule::add_stake",
                "SubtensorModule::add_stake_limit",
                "SubtensorModule::remove_stake",
                "SubtensorModule::remove_stake_limit",
                "SubtensorModule::remove_stake_full_limit",
                "SubtensorModule::unstake_all",
                "SubtensorModule::unstake_all_alpha",
                "SubtensorModule::move_stake",
                "SubtensorModule::swap_stake",
                "SubtensorModule::swap_stake_limit",
                "SubtensorModule::set_root_claim_type",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::Registration),
            expected(&[
                "SubtensorModule::register",
                "SubtensorModule::register_limit",
                "SubtensorModule::burned_register",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::ChildKeys),
            expected(&[
                "SubtensorModule::set_children",
                "SubtensorModule::set_childkey_take",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::SwapHotkey),
            expected(&[
                "SubtensorModule::swap_hotkey",
                "SubtensorModule::swap_hotkey_v2",
            ])
        );
        assert_eq!(
            allowed_calls(ProxyType::RootClaim),
            expected(&["SubtensorModule::claim_root"])
        );
        assert_eq!(
            allowed_calls(ProxyType::SudoUncheckedSetCode),
            expected(&["Sudo::sudo_unchecked_weight"])
        );
    }

    // The newer calls that leaked through `main`'s denylists must stay denied
    // for every broad proxy.
    #[test]
    fn tightened_denylist_leaks_stay_denied() {
        for proxy_type in [
            ProxyType::NonTransfer,
            ProxyType::NonFungible,
            ProxyType::NonCritical,
        ] {
            let allowed = allowed_calls(proxy_type);
            assert!(!allowed.contains("SubtensorModule::reset_coldkey_swap"));
            assert!(!allowed.contains("SubtensorModule::swap_coldkey"));
            assert!(!allowed.contains("SubtensorModule::schedule_swap_coldkey"));
        }
        // `root_dissolve_network` leaked into NonCritical specifically.
        assert!(!allowed_calls(ProxyType::NonCritical).contains("SubtensorModule::root_dissolve_network"));
    }

    // The SmallTransfer / SudoUncheckedSetCode metadata must carry their
    // amount / nested-call constraints.
    #[test]
    fn conditional_proxies_expose_constraints() {
        use subtensor_runtime_common::CallConstraint;

        let small = match proxy_filter_mode(ProxyType::SmallTransfer) {
            FilterMode::Allow(infos) => infos,
            FilterMode::AllowAll => vec![],
        };
        assert!(small.iter().all(|info| matches!(
            info.constraint,
            Some(CallConstraint::ParamLessThan { .. })
        )));

        let set_code = match proxy_filter_mode(ProxyType::SudoUncheckedSetCode) {
            FilterMode::Allow(infos) => infos,
            FilterMode::AllowAll => vec![],
        };
        assert!(set_code.iter().any(|info| matches!(
            &info.constraint,
            Some(CallConstraint::NestedCallMustBe { pallet_name, call_name, .. })
            if pallet_name == b"System" && call_name == b"set_code"
        )));
    }
}
