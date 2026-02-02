use crate::RuntimeCall;
use crate::Vec;
use crate::pallet_proxy;
use crate::pallet_utility;
use frame_support::traits::Contains;
use sp_std::boxed::Box;
use sp_std::vec;
pub struct NoNestingCallFilter;

impl Contains<RuntimeCall> for NoNestingCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        let calls = match call {
            RuntimeCall::Utility(inner) => {
                let calls = match inner {
                    pallet_utility::Call::force_batch { calls } => calls,
                    pallet_utility::Call::batch { calls } => calls,
                    pallet_utility::Call::batch_all { calls } => calls,
                    _ => return true,
                };

                calls
                    .iter()
                    .map(|call| Box::new(call.clone()))
                    .collect::<Vec<_>>()
            }
            RuntimeCall::Proxy(inner) => {
                let call = match inner {
                    pallet_proxy::Call::proxy { call, .. } => call,
                    pallet_proxy::Call::proxy_announced { call, .. } => call,
                    _ => return true,
                };

                vec![call.clone()]
            }
            RuntimeCall::Multisig(inner) => {
                let call = match inner {
                    pallet_multisig::Call::as_multi { call, .. } => call,
                    pallet_multisig::Call::as_multi_threshold_1 { call, .. } => call,
                    _ => return true,
                };

                vec![call.clone()]
            }
            RuntimeCall::Crowdloan(inner) => {
                let call = match inner {
                    pallet_crowdloan::Call::create {
                        call: Some(call), ..
                    } => call,
                    _ => return true,
                };

                vec![call.clone()]
            }
            RuntimeCall::Scheduler(inner) => {
                let call = match inner {
                    pallet_scheduler::Call::schedule { call, .. } => call,
                    pallet_scheduler::Call::schedule_after { call, .. } => call,
                    pallet_scheduler::Call::schedule_named { call, .. } => call,
                    pallet_scheduler::Call::schedule_named_after { call, .. } => call,
                    _ => return true,
                };

                vec![call.clone()]
            }
            _ => return true,
        };

        !calls.iter().any(|call| {
            matches!(&**call, RuntimeCall::Utility(inner) if matches!(inner, pallet_utility::Call::force_batch { .. } | pallet_utility::Call::batch_all { .. } | pallet_utility::Call::batch { .. })) ||
            matches!(&**call, RuntimeCall::Proxy(inner) if matches!(inner, pallet_proxy::Call::proxy { .. } | pallet_proxy::Call::proxy_announced { .. })) ||
            matches!(&**call, RuntimeCall::Multisig(inner) if matches!(inner, pallet_multisig::Call::as_multi { .. } | pallet_multisig::Call::as_multi_threshold_1 { .. })) ||
            matches!(&**call, RuntimeCall::Crowdloan(inner) if matches!(inner, pallet_crowdloan::Call::create { .. } )) ||
            matches!(&**call, RuntimeCall::Scheduler(inner) if matches!(inner, pallet_scheduler::Call::schedule {..} | pallet_scheduler::Call::schedule_after { .. } | pallet_scheduler::Call::schedule_named {.. } | pallet_scheduler::Call::schedule_named_after { .. } )) ||
            matches!(&**call, RuntimeCall::Sudo(inner) if matches!(inner, pallet_sudo::Call::sudo {..} | pallet_sudo::Call::sudo_as { .. } | pallet_sudo::Call::sudo_unchecked_weight { .. } ))
        })
    }
}

pub struct SafeModeWhitelistedCalls;
impl Contains<RuntimeCall> for SafeModeWhitelistedCalls {
    fn contains(call: &RuntimeCall) -> bool {
        matches!(
            call,
            RuntimeCall::Sudo(_)
                | RuntimeCall::Multisig(_)
                | RuntimeCall::System(_)
                | RuntimeCall::SafeMode(_)
                | RuntimeCall::Timestamp(_)
                | RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::set_weights { .. }
                        | pallet_subtensor::Call::serve_axon { .. }
                )
                | RuntimeCall::Commitments(pallet_commitments::Call::set_commitment { .. })
        )
    }
}
