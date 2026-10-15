//! Shared admission at the runtime execution boundary, including raw RPC calls.
//!
//! Pool admission alone cannot bound `state_call`, chain-head/archive calls,
//! dry-run, or simulations carrying detached ML-DSA order proofs. Executor clones
//! share per-service budgets. Unknown exports cannot consume simulation or
//! transaction-validation capacity. A permit lives until synchronous Wasm
//! execution returns, including when the original RPC future is cancelled.
//! These are local resource limits, never consensus checks: block import and
//! authoring use `Onchain`.

use sc_executor::{RuntimeVersion, RuntimeVersionOf, error};
use sp_core::traits::{CallContext, CodeExecutor, Externalities, ReadRuntimeVersion, RuntimeCode};
use sp_runtime::codec::{Compact, Decode};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const MAX_ACTIVE: usize = 2;
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
// Reserve an independent 16 MiB input budget for each of four services.
const MAX_ACTIVE_BYTES: usize = MAX_INPUT_BYTES;
const BURST: u32 = 16;
const REFILL: Duration = Duration::from_millis(125);
const MLDSA_ADDRESS_BYTES: u32 = 5367;

fn busy() -> error::Error {
    error::Error::Other(fp_rpc::LOCAL_EXECUTION_CAPACITY_ERROR.into())
}

/// Explicitly enumerated read/conversion APIs. New exports are budgeted until
/// reviewed; an entire API family must not silently inherit this exemption.
fn read_only(method: &str) -> bool {
    matches!(
        method,
        "Core_version"
            // Inherent construction/checks decode the block but neither check
            // account signatures nor dispatch calls. Aura/Babe pre-import
            // checks use the default Offchain context and must remain available.
            | "BlockBuilder_inherent_extrinsics"
            | "BlockBuilder_check_inherents"
            // The current Drand worker performs no ML-DSA authorization and
            // runs in the SDK's default Offchain context on genuine imports.
            | "OffchainWorkerApi_offchain_worker"
            | "Metadata_metadata"
            | "Metadata_metadata_at_version"
            | "Metadata_metadata_versions"
            | "GenesisBuilder_get_preset"
            | "GenesisBuilder_preset_names"
            | "AuraApi_slot_duration"
            | "AuraApi_authorities"
            | "SessionKeys_decode_session_keys"
            | "GrandpaApi_grandpa_authorities"
            | "GrandpaApi_current_set_id"
            | "GrandpaApi_generate_key_ownership_proof"
            | "BabeApi_configuration"
            | "BabeApi_current_epoch_start"
            | "BabeApi_current_epoch"
            | "BabeApi_next_epoch"
            | "BabeApi_generate_key_ownership_proof"
            | "AccountNonceApi_account_nonce"
            | "TransactionPaymentApi_query_info"
            | "TransactionPaymentApi_query_fee_details"
            | "TransactionPaymentApi_query_weight_to_fee"
            | "TransactionPaymentApi_query_length_to_fee"
            | "TransactionPaymentCallApi_query_call_info"
            | "TransactionPaymentCallApi_query_call_fee_details"
            | "TransactionPaymentCallApi_query_weight_to_fee"
            | "TransactionPaymentCallApi_query_length_to_fee"
            | "EthereumRuntimeRPCApi_chain_id"
            | "EthereumRuntimeRPCApi_account_basic"
            | "EthereumRuntimeRPCApi_gas_price"
            | "EthereumRuntimeRPCApi_account_code_at"
            | "EthereumRuntimeRPCApi_author"
            | "EthereumRuntimeRPCApi_storage_at"
            | "EthereumRuntimeRPCApi_current_transaction_statuses"
            | "EthereumRuntimeRPCApi_current_block"
            | "EthereumRuntimeRPCApi_current_receipts"
            | "EthereumRuntimeRPCApi_current_all"
            | "EthereumRuntimeRPCApi_extrinsic_filter"
            | "EthereumRuntimeRPCApi_elasticity"
            | "EthereumRuntimeRPCApi_gas_limit_multiplier_support"
            | "ConvertTransactionRuntimeApi_convert_transaction"
            | "ContractsApi_get_storage"
            | "Benchmark_benchmark_metadata"
            | "DelegateInfoRuntimeApi_get_delegates"
            | "DelegateInfoRuntimeApi_get_delegate"
            | "DelegateInfoRuntimeApi_get_delegated"
            | "NeuronInfoRuntimeApi_get_neurons_lite"
            | "NeuronInfoRuntimeApi_get_neuron_lite"
            | "NeuronInfoRuntimeApi_get_neurons"
            | "NeuronInfoRuntimeApi_get_neuron"
            | "SubnetInfoRuntimeApi_get_subnet_info"
            | "SubnetInfoRuntimeApi_get_subnets_info"
            | "SubnetInfoRuntimeApi_get_subnet_info_v2"
            | "SubnetInfoRuntimeApi_get_subnets_info_v2"
            | "SubnetInfoRuntimeApi_get_subnet_hyperparams"
            | "SubnetInfoRuntimeApi_get_subnet_hyperparams_v2"
            | "SubnetInfoRuntimeApi_get_subnet_hyperparams_v3"
            | "SubnetInfoRuntimeApi_get_dynamic_info"
            | "SubnetInfoRuntimeApi_get_metagraph"
            | "SubnetInfoRuntimeApi_get_mechagraph"
            | "SubnetInfoRuntimeApi_get_subnet_state"
            | "SubnetInfoRuntimeApi_get_all_metagraphs"
            | "SubnetInfoRuntimeApi_get_all_mechagraphs"
            | "SubnetInfoRuntimeApi_get_all_dynamic_info"
            | "SubnetInfoRuntimeApi_get_selective_metagraph"
            | "SubnetInfoRuntimeApi_get_subnet_to_prune"
            | "SubnetInfoRuntimeApi_get_coldkey_auto_stake_hotkey"
            | "SubnetInfoRuntimeApi_get_selective_mechagraph"
            | "SubnetInfoRuntimeApi_get_subnet_account_id"
            | "SubnetInfoRuntimeApi_get_next_epoch_start_block"
            | "SubnetInfoRuntimeApi_get_block_emission"
            | "StakeInfoRuntimeApi_get_stake_info_for_coldkey"
            | "StakeInfoRuntimeApi_get_stake_info_for_coldkeys"
            | "StakeInfoRuntimeApi_get_stake_info_for_hotkey_coldkey_netuid"
            | "StakeInfoRuntimeApi_get_stake_availability_for_coldkeys"
            | "StakeInfoRuntimeApi_get_stake_fee"
            | "StakeInfoRuntimeApi_get_coldkey_lock"
            | "StakeInfoRuntimeApi_get_hotkey_conviction"
            | "StakeInfoRuntimeApi_get_most_convicted_hotkey_on_subnet"
            | "SubnetRegistrationRuntimeApi_get_network_registration_cost"
            | "BetaBasketRuntimeApi_get_root_basket_owed"
            | "BetaBasketRuntimeApi_get_basket_payout"
            | "BetaBasketRuntimeApi_get_validator_basket_nav"
            | "BetaBasketRuntimeApi_get_validator_basket"
            | "BetaBasketRuntimeApi_get_root_basket_total_nav"
            | "BetaBasketRuntimeApi_get_validator_basket_summary"
            | "BetaBasketRuntimeApi_get_all_validator_baskets"
            | "BetaBasketRuntimeApi_get_root_basket_positions"
            | "BetaBasketRuntimeApi_get_basket_position"
            | "BetaBasketRuntimeApi_get_root_basket_portfolio"
            | "BetaBasketRuntimeApi_get_beta_pricing"
            | "BetaBasketRuntimeApi_get_all_beta_pricing"
            | "BetaBasketRuntimeApi_get_beta_index"
            | "BetaBasketRuntimeApi_get_beta_position"
            | "BetaBasketRuntimeApi_get_beta_portfolio"
            | "BetaBasketRuntimeApi_get_basket_trading_status"
            | "BetaBasketRuntimeApi_get_basket_claim_preview"
            | "BetaBasketRuntimeApi_get_root_basket_claim_previews"
            | "ProxyFilterRuntimeApi_get_proxy_types"
            | "ProxyFilterRuntimeApi_get_proxy_filters"
            | "SwapRuntimeApi_current_alpha_price"
            | "SwapRuntimeApi_current_alpha_price_all"
            | "SwapRuntimeApi_sim_swap_tao_for_alpha"
            | "SwapRuntimeApi_sim_swap_alpha_for_tao"
            // The runtime's proposer recognizer is syntactic, not its generic
            // pallet helper that checks signatures. Decryption also only
            // decodes; Executive authenticates the outer and inner separately.
            | "ShieldApi_try_decode_shielded_tx"
            | "ShieldApi_try_unshield_tx"
            | "ShieldApi_is_shielded_using_current_key"
    )
}

fn mldsa_validation(data: &[u8]) -> bool {
    // Runtime API v3 encodes TransactionSource, the extrinsic, then block hash.
    // Inspect only its prefix, including malformed v2 profiles/carriers. The
    // current validation pipeline checks extensions but does not dispatch the
    // call; legacy validation therefore cannot execute detached order proofs.
    let Some(mut input) = data.get(1..) else {
        return false;
    };
    Compact::<u32>::decode(&mut input).is_ok()
        && u8::decode(&mut input).ok() == Some(0x84)
        && u8::decode(&mut input).ok() == Some(2)
        && Compact::<u32>::decode(&mut input).ok().map(|x| x.0) == Some(MLDSA_ADDRESS_BYTES)
}

fn needs_admission(method: &str, data: &[u8], context: CallContext) -> bool {
    context == CallContext::Offchain
        && !read_only(method)
        && (method != "TaggedTransactionQueue_validate_transaction" || mldsa_validation(data))
}

/// Keep independent workloads available even if another API is flooded with
/// malformed requests. The executor boundary has no authenticated caller ID.
/// Each service retains its own bounded concurrency, bytes and token bucket;
/// no idle quota is borrowed across services. Four services permit at most
/// eight concurrent executions (and 64 MiB of input) across executor clones.
#[derive(Clone, Copy)]
enum Service {
    Validation = 0,
    Evm = 1,
    Contracts = 2,
    Other = 3,
}

fn service(method: &str) -> Service {
    match method {
        "TaggedTransactionQueue_validate_transaction" => Service::Validation,
        "EthereumRuntimeRPCApi_call"
        | "EthereumRuntimeRPCApi_create"
        | "EthereumRuntimeRPCApi_pending_block" => Service::Evm,
        "ContractsApi_call" | "ContractsApi_instantiate" | "ContractsApi_upload_code" => {
            Service::Contracts
        }
        _ => Service::Other,
    }
}

struct State {
    active: usize,
    bytes: usize,
    tokens: u32,
    refilled: Instant,
}

impl State {
    fn new(now: Instant) -> Self {
        Self {
            active: 0,
            bytes: 0,
            tokens: BURST,
            refilled: now,
        }
    }

    fn acquire(&mut self, bytes: usize, now: Instant) -> Result<(), error::Error> {
        let next_bytes = self.bytes.checked_add(bytes).ok_or_else(busy)?;
        if self.active >= MAX_ACTIVE || bytes > MAX_INPUT_BYTES || next_bytes > MAX_ACTIVE_BYTES {
            return Err(busy());
        }
        if let Some(elapsed) = now.checked_duration_since(self.refilled) {
            let added = elapsed
                .as_nanos()
                .checked_div(REFILL.as_nanos())
                .unwrap_or(0);
            if added != 0 {
                self.tokens = self
                    .tokens
                    .saturating_add(added.min(u128::from(BURST)) as u32)
                    .min(BURST);
                // Discard fractional credit conservatively; never move backwards.
                self.refilled = now;
            }
        }
        if self.tokens == 0 {
            return Err(busy());
        }
        self.tokens = self.tokens.saturating_sub(1);
        self.active = self.active.saturating_add(1);
        self.bytes = next_bytes;
        Ok(())
    }
}

struct Permit {
    state: Arc<Mutex<State>>,
    bytes: usize,
}

impl Drop for Permit {
    fn drop(&mut self) {
        // A poisoned admission lock fails closed on subsequent acquisition.
        if let Ok(mut state) = self.state.lock() {
            state.active = state.active.saturating_sub(1);
            state.bytes = state.bytes.saturating_sub(self.bytes);
        }
    }
}

/// Runtime executor with isolated service budgets shared by all its clones.
#[derive(Clone)]
pub struct AdmissionExecutor<E> {
    inner: E,
    states: [Arc<Mutex<State>>; 4],
}

impl<E> AdmissionExecutor<E> {
    /// Wrap an executor with fresh service budgets shared by subsequent clones.
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            states: std::array::from_fn(|_| Arc::new(Mutex::new(State::new(Instant::now())))),
        }
    }

    fn acquire(&self, service: Service, bytes: usize) -> Result<Permit, error::Error> {
        let [validation, evm, contracts, other] = &self.states;
        let state = match service {
            Service::Validation => validation,
            Service::Evm => evm,
            Service::Contracts => contracts,
            Service::Other => other,
        };
        state
            .lock()
            .map_err(|_| busy())?
            .acquire(bytes, Instant::now())?;
        Ok(Permit {
            state: state.clone(),
            bytes,
        })
    }
}

impl<E: CodeExecutor<Error = error::Error>> CodeExecutor for AdmissionExecutor<E> {
    type Error = error::Error;

    fn call(
        &self,
        ext: &mut dyn Externalities,
        runtime_code: &RuntimeCode,
        method: &str,
        data: &[u8],
        context: CallContext,
    ) -> (Result<Vec<u8>, Self::Error>, bool) {
        let _permit = if needs_admission(method, data, context) {
            match self.acquire(service(method), data.len()) {
                Ok(permit) => Some(permit),
                Err(error) => return (Err(error), false),
            }
        } else {
            None
        };
        self.inner.call(ext, runtime_code, method, data, context)
    }
}

impl<E: ReadRuntimeVersion> ReadRuntimeVersion for AdmissionExecutor<E> {
    fn read_runtime_version(
        &self,
        wasm_code: &[u8],
        ext: &mut dyn Externalities,
    ) -> Result<Vec<u8>, String> {
        self.inner.read_runtime_version(wasm_code, ext)
    }
}

impl<E: RuntimeVersionOf> RuntimeVersionOf for AdmissionExecutor<E> {
    fn runtime_version(
        &self,
        ext: &mut dyn Externalities,
        runtime_code: &RuntimeCode,
    ) -> error::Result<RuntimeVersion> {
        self.inner.runtime_version(ext, runtime_code)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use sp_runtime::codec::Encode;
    use std::sync::{
        Condvar,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    };

    #[derive(Clone)]
    struct MockExecutor {
        calls: Arc<AtomicUsize>,
        entered: mpsc::Sender<()>,
        resume: Arc<(Mutex<bool>, Condvar)>,
    }

    impl ReadRuntimeVersion for MockExecutor {
        fn read_runtime_version(
            &self,
            _: &[u8],
            _: &mut dyn Externalities,
        ) -> Result<Vec<u8>, String> {
            Ok(vec![1])
        }
    }

    impl CodeExecutor for MockExecutor {
        type Error = error::Error;

        fn call(
            &self,
            _: &mut dyn Externalities,
            _: &RuntimeCode,
            _: &str,
            data: &[u8],
            _: CallContext,
        ) -> (Result<Vec<u8>, Self::Error>, bool) {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if data == b"wait" {
                self.entered.send(()).unwrap();
                let (lock, wake) = &*self.resume;
                let mut resumed = lock.lock().unwrap();
                while !*resumed {
                    resumed = wake.wait(resumed).unwrap();
                }
            }
            (
                if data == b"fail" {
                    Err(error::Error::Other("invalid proof".into()))
                } else {
                    Ok(vec![])
                },
                false,
            )
        }
    }

    fn executor() -> (AdmissionExecutor<MockExecutor>, mpsc::Receiver<()>) {
        let (entered, receiver) = mpsc::channel();
        (
            AdmissionExecutor::new(MockExecutor {
                calls: Arc::new(AtomicUsize::new(0)),
                entered,
                resume: Arc::new((Mutex::new(false), Condvar::new())),
            }),
            receiver,
        )
    }

    fn execute(
        executor: &AdmissionExecutor<MockExecutor>,
        method: &str,
        data: &[u8],
        context: CallContext,
    ) -> Result<Vec<u8>, error::Error> {
        let mut ext = sp_io::TestExternalities::default();
        executor
            .call(&mut ext.ext(), &RuntimeCode::empty(), method, data, context)
            .0
    }

    fn validation(raw_length: Option<u32>) -> Vec<u8> {
        let mut data = vec![2]; // External source.
        Compact(6000u32).encode_to(&mut data);
        data.push(0x84);
        if let Some(length) = raw_length {
            data.push(2);
            Compact(length).encode_to(&mut data);
        } else {
            data.push(0); // MultiAddress::Id.
        }
        data
    }

    #[test]
    fn only_legacy_validation_and_reviewed_queries_bypass_offchain_admission() {
        let queue = "TaggedTransactionQueue_validate_transaction";
        assert!(!needs_admission(
            queue,
            &validation(None),
            CallContext::Offchain
        ));
        assert!(!needs_admission(
            queue,
            &validation(Some(169)),
            CallContext::Offchain
        ));
        assert!(needs_admission(
            queue,
            &validation(Some(5367)),
            CallContext::Offchain
        ));
        // Execution can contain detached proofs even with a legacy outer signer.
        for method in [
            "BlockBuilder_apply_extrinsic",
            "Core_execute_block",
            "Core_initialize_block",
            "EthereumRuntimeRPCApi_pending_block",
            "EthereumRuntimeRPCApi_call",
            "EthereumRuntimeRPCApi_create",
            "ContractsApi_call",
            "TryRuntime_execute_block",
            "Benchmark_dispatch_benchmark",
            "FutureApi_unknown",
        ] {
            assert!(
                needs_admission(method, &validation(None), CallContext::Offchain),
                "{method}"
            );
            assert!(
                !needs_admission(method, &validation(Some(5367)), CallContext::Onchain),
                "{method}"
            );
        }
        for method in [
            "Core_version",
            "BlockBuilder_check_inherents",
            "BlockBuilder_inherent_extrinsics",
            "OffchainWorkerApi_offchain_worker",
            "ShieldApi_try_decode_shielded_tx",
            "ShieldApi_try_unshield_tx",
            "Metadata_metadata_at_version",
            "TransactionPaymentApi_query_info",
            "TransactionPaymentCallApi_query_call_fee_details",
            "ContractsApi_get_storage",
            "SubnetInfoRuntimeApi_get_metagraph",
        ] {
            assert!(!needs_admission(
                method,
                &validation(Some(5367)),
                CallContext::Offchain
            ));
        }
    }

    #[test]
    fn overload_precedes_execution_and_does_not_block_queries_or_consensus() {
        use sc_transaction_pool_api::error::IntoPoolError;
        let (executor, _) = executor();
        let first = executor.acquire(Service::Other, 1).unwrap();
        let second = executor.clone().acquire(Service::Other, 1).unwrap();
        let error =
            execute(&executor, "Core_execute_block", &[], CallContext::Offchain).unwrap_err();
        assert_eq!(executor.inner.calls.load(Ordering::SeqCst), 0);
        // FullChainApi maps executor/API errors to RuntimeApi; the service maps
        // errors outside IntoPoolError to KnownGood, not a bad peer transaction.
        type PoolError = <sc_transaction_pool::TransactionPoolHandle<
            node_subtensor_runtime::opaque::Block,
            crate::client::FullClient,
        > as sc_transaction_pool_api::TransactionPool>::Error;
        let pool_error = PoolError::RuntimeApi(error.to_string());
        assert!(pool_error.into_pool_error().is_err());
        execute(&executor, "Metadata_metadata", &[], CallContext::Offchain).unwrap();
        execute(&executor, "Core_execute_block", &[], CallContext::Onchain).unwrap();
        execute(
            &executor,
            "TaggedTransactionQueue_validate_transaction",
            &validation(None),
            CallContext::Offchain,
        )
        .unwrap();
        execute(
            &executor,
            "OffchainWorkerApi_offchain_worker",
            &[],
            CallContext::Offchain,
        )
        .unwrap();
        assert_eq!(executor.inner.calls.load(Ordering::SeqCst), 4);
        drop((first, second));
        execute(&executor, "Core_execute_block", &[], CallContext::Offchain).unwrap();
        assert_eq!(
            executor
                .states
                .get(Service::Other as usize)
                .unwrap()
                .lock()
                .unwrap()
                .active,
            0
        );
    }

    #[test]
    fn failed_execution_releases_capacity_without_refunding_rate_budget() {
        let (executor, _) = executor();
        assert!(
            execute(
                &executor,
                "Core_execute_block",
                b"fail",
                CallContext::Offchain
            )
            .is_err()
        );
        let state = executor
            .states
            .get(Service::Other as usize)
            .unwrap()
            .lock()
            .unwrap();
        assert_eq!(state.active, 0);
        assert_eq!(state.bytes, 0);
        assert_eq!(state.tokens, BURST - 1);
    }

    #[test]
    fn cancelled_callers_cannot_release_running_executors() {
        let (executor, entered) = executor();
        let (finished, finished_rx) = mpsc::channel();
        for _ in 0..MAX_ACTIVE {
            let clone = executor.clone();
            let finished = finished.clone();
            // Dropping a caller's JoinHandle does not stop synchronous Wasm or
            // drop the permit held in the execution frame on the worker thread.
            drop(std::thread::spawn(move || {
                execute(&clone, "Core_execute_block", b"wait", CallContext::Offchain).unwrap();
                finished.send(()).unwrap();
            }));
        }
        for _ in 0..MAX_ACTIVE {
            entered.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        assert!(execute(&executor, "Core_execute_block", &[], CallContext::Offchain).is_err());
        assert_eq!(executor.inner.calls.load(Ordering::SeqCst), MAX_ACTIVE);
        let (lock, wake) = &*executor.inner.resume;
        *lock.lock().unwrap() = true;
        wake.notify_all();
        for _ in 0..MAX_ACTIVE {
            finished_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        assert_eq!(
            executor
                .states
                .get(Service::Other as usize)
                .unwrap()
                .lock()
                .unwrap()
                .active,
            0
        );
        execute(&executor, "Core_execute_block", &[], CallContext::Offchain).unwrap();
    }

    #[test]
    fn malformed_requests_cannot_exhaust_other_services() {
        let (executor, _) = executor();
        // Freeze refill so the test does not depend on machine speed.
        for state in &executor.states {
            state.lock().unwrap().refilled = Instant::now() + Duration::from_secs(60);
        }
        for method in [
            "NonexistentApi_call",
            "EthereumRuntimeRPCApi_call",
            "ContractsApi_call",
        ] {
            for _ in 0..BURST {
                assert!(execute(&executor, method, b"fail", CallContext::Offchain).is_err());
            }
            let calls = executor.inner.calls.load(Ordering::SeqCst);
            assert!(execute(&executor, method, b"fail", CallContext::Offchain).is_err());
            assert_eq!(executor.inner.calls.load(Ordering::SeqCst), calls);
            // Neither bad names, EVM failures nor contract failures spend the
            // reserved ML-DSA validation tokens, including through a clone.
            execute(
                &executor.clone(),
                "TaggedTransactionQueue_validate_transaction",
                &validation(Some(5367)),
                CallContext::Offchain,
            )
            .unwrap();
        }
    }

    #[test]
    fn active_calls_do_not_occupy_other_service_slots() {
        let (executor, _) = executor();
        let first = executor
            .acquire(Service::Other, MAX_INPUT_BYTES / 2)
            .unwrap();
        let second = executor
            .acquire(Service::Other, MAX_INPUT_BYTES / 2)
            .unwrap();
        assert!(execute(&executor, "UnknownApi_call", &[], CallContext::Offchain).is_err());
        for method in ["EthereumRuntimeRPCApi_call", "ContractsApi_call"] {
            execute(&executor.clone(), method, &[], CallContext::Offchain).unwrap();
        }
        execute(
            &executor,
            "TaggedTransactionQueue_validate_transaction",
            &validation(Some(5367)),
            CallContext::Offchain,
        )
        .unwrap();
        drop((first, second));
    }

    #[test]
    fn bytes_overflow_rate_limit_and_clock_rollback_fail_closed() {
        let now = Instant::now();
        let mut state = State::new(now);
        assert!(state.acquire(MAX_INPUT_BYTES + 1, now).is_err());
        state.bytes = usize::MAX;
        assert!(state.acquire(1, now).is_err());
        state.bytes = MAX_ACTIVE_BYTES;
        assert!(state.acquire(1, now).is_err());
        state.bytes = 0;
        for _ in 0..BURST {
            state.acquire(0, now).unwrap();
            state.active = 0;
        }
        assert!(state.acquire(0, now).is_err());
        assert!(state.acquire(0, now - REFILL).is_err());
        assert_eq!(state.refilled, now);
        state.acquire(0, now + REFILL).unwrap();
        assert_eq!(state.tokens, 0);
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod admission_tests {
    use fc_rpc::{execution_admission::retry_execution_capacity, internal_err};
    use jsonrpsee::types::error::CALL_EXECUTION_FAILED_CODE;
    use std::{cell::Cell, time::Duration};

    fn capacity_error() -> jsonrpsee::types::ErrorObjectOwned {
        internal_err(format!(
            "runtime error: Execution failed: Other: {}",
            fp_rpc::LOCAL_EXECUTION_CAPACITY_ERROR
        ))
    }

    #[tokio::test]
    async fn retries_only_temporary_execution_capacity_failures() {
        let attempts = Cell::new(0);
        let result = retry_execution_capacity(
            || async {
                attempts.set(attempts.get() + 1);
                if attempts.get() < 3 {
                    Err(capacity_error())
                } else {
                    Ok(42)
                }
            },
            tokio::time::Instant::now() + Duration::from_secs(2),
        )
        .await;
        assert_eq!(result, Ok(42));
        assert_eq!(attempts.get(), 3);
    }

    #[tokio::test]
    async fn execution_errors_are_returned_without_retry() {
        for error in [
            internal_err("VM Exception while processing transaction: revert"),
            internal_err("runtime error: Execution failed: Other: unrelated error"),
            jsonrpsee::types::ErrorObjectOwned::owned(
                CALL_EXECUTION_FAILED_CODE,
                capacity_error().message(),
                None::<()>,
            ),
        ] {
            let attempts = Cell::new(0);
            let result = retry_execution_capacity::<(), _>(
                || async {
                    attempts.set(attempts.get() + 1);
                    Err(error.clone())
                },
                tokio::time::Instant::now() + Duration::from_secs(2),
            )
            .await;
            assert_eq!(result, Err(error));
            assert_eq!(attempts.get(), 1);
        }
    }

    #[tokio::test]
    async fn expired_deadline_does_not_retry() {
        let attempts = Cell::new(0);
        let result = retry_execution_capacity::<(), _>(
            || async {
                attempts.set(attempts.get() + 1);
                Err(capacity_error())
            },
            tokio::time::Instant::now(),
        )
        .await;
        assert_eq!(result, Err(capacity_error()));
        assert_eq!(attempts.get(), 1);
    }

    #[tokio::test]
    async fn cancellation_drops_pending_retry() {
        let attempts = Cell::new(0);
        let mut pending = Box::pin(retry_execution_capacity::<(), _>(
            || async {
                attempts.set(attempts.get() + 1);
                Err(capacity_error())
            },
            tokio::time::Instant::now() + Duration::from_secs(2),
        ));
        assert!(futures::poll!(&mut pending).is_pending());
        drop(pending);
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(attempts.get(), 1);
    }
}
