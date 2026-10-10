//! Exercise native EVM calls through the full signed extension pipeline.
use super::*;
use frame_support::{
    assert_ok,
    dispatch::Pays,
    storage::{TransactionOutcome, with_transaction_unchecked},
};
use pallet_evm::{AddressMapping, FeeCalculator, runner::Runner};
use sp_core::{H160, U256, ed25519};
use subtensor_hashed::fips204::{
    ml_dsa_65,
    traits::{KeyGen, SerDes, Signer},
};

#[derive(Clone, Copy, Debug)]
enum Mode {
    Sr,
    Ed,
    HashedSr,
    HashedEd,
    HashedMs,
    Ms,
}
const MODES: [Mode; 6] = [
    Mode::Sr,
    Mode::Ed,
    Mode::HashedSr,
    Mode::HashedEd,
    Mode::HashedMs,
    Mode::Ms,
];
struct Wallet {
    mode: Mode,
    account: AccountId,
}
impl Wallet {
    fn new(mode: Mode) -> Self {
        let account = match mode {
            Mode::Sr => AccountId::from(pair(10).public()),
            Mode::Ed => AccountId::new(ed25519::Pair::from_seed(&[10; 32]).public().0),
            _ => composable_accounts::register(Descriptor {
                version: if matches!(mode, Mode::Ms) { 2 } else { 1 },
                scheme: Self::scheme(mode),
                initial_commitment: Self::commitment(mode, 0),
            }),
        };
        System::set_block_number(1);
        hashed_auth::TestVerificationWeight::set(Weight::from_parts(10_000_000, 0));
        let _ = Balances::make_free_balance_be(&account, TaoBalance::new(1_000_000_000_000));
        let source = H160::from_slice(&AsRef::<[u8; 32]>::as_ref(&account)[..20]);
        let backing = evm_origin::HashedAddressMapping::into_account_id(source);
        let _ = Balances::make_free_balance_be(&backing, TaoBalance::new(1_000_000_000_000));
        pallet_evm::DisableWhitelistCheck::<Runtime>::put(true);
        Self { mode, account }
    }
    fn scheme(mode: Mode) -> Scheme {
        match mode {
            Mode::Ed | Mode::HashedEd => Scheme::Ed25519,
            Mode::Ms | Mode::HashedMs => Scheme::MlDsa65,
            _ => Scheme::Sr25519,
        }
    }
    fn public(mode: Mode, generation: u8) -> Vec<u8> {
        let seed = [10 + generation; 32];
        match Self::scheme(mode) {
            Scheme::Sr25519 => sr25519::Pair::from_seed(&seed).public().0.to_vec(),
            Scheme::Ed25519 => ed25519::Pair::from_seed(&seed).public().0.to_vec(),
            Scheme::MlDsa65 => ml_dsa_65::KG::keygen_from_seed(&seed)
                .0
                .into_bytes()
                .to_vec(),
        }
    }
    fn commitment(mode: Mode, generation: u8) -> [u8; 32] {
        let public = Self::public(mode, generation);
        if Self::scheme(mode) == Scheme::MlDsa65 {
            subtensor_hashed::key_commitment(
                Self::scheme(mode),
                <&[u8; 1952]>::try_from(public.as_slice()).unwrap(),
            )
        } else {
            subtensor_hashed::key_commitment(
                Self::scheme(mode),
                <&[u8; 32]>::try_from(public.as_slice()).unwrap(),
            )
        }
    }
    fn source(&self) -> H160 {
        H160::from_slice(&AsRef::<[u8; 32]>::as_ref(&self.account)[..20])
    }
    fn sign(&self, call: RuntimeCall) -> UncheckedExtrinsic {
        let extra = extra(System::account_nonce(&self.account));
        if matches!(self.mode, Mode::Sr | Mode::Ed) {
            let payload = SignedPayload::new(call.clone(), extra.clone()).unwrap();
            let signature = if matches!(self.mode, Mode::Sr) {
                Signature::Sr25519(payload.using_encoded(|m| pair(10).sign(m)))
            } else {
                Signature::Ed25519(
                    payload.using_encoded(|m| ed25519::Pair::from_seed(&[10; 32]).sign(m)),
                )
            };
            return UncheckedExtrinsic::new_signed(
                call,
                self.account.clone().into(),
                signature,
                extra,
            );
        }
        let generation = HashedAccounts::accounts(&self.account).unwrap().generation;
        let key_generation = if matches!(self.mode, Mode::Ms) {
            0
        } else {
            u8::try_from(generation).unwrap()
        };
        let next = if matches!(self.mode, Mode::Ms) {
            0
        } else {
            key_generation + 1
        };
        let next_commitment = Self::commitment(self.mode, next);
        let scheme = Self::scheme(self.mode);
        let version = if scheme == Scheme::MlDsa65 { 2u8 } else { 1u8 };
        let implication = (version, &call, &extra, extra.implicit().unwrap()).encode();
        let payload = subtensor_hashed::transaction_payload(
            self.account.as_ref(),
            scheme,
            generation,
            &next_commitment,
            &implication,
        );
        let seed = [10 + key_generation; 32];
        if scheme == Scheme::MlDsa65 {
            let (public, private) = ml_dsa_65::KG::keygen_from_seed(&seed);
            UncheckedExtrinsic::new_mldsa(
                call,
                hashed_auth::AuthorizeMlDsa {
                    account: self.account.clone(),
                    proof: subtensor_hashed::MlDsaProof {
                        generation,
                        public_key: public.into_bytes(),
                        next_commitment,
                        signature: private
                            .try_sign_with_seed(
                                &[0; 32],
                                &payload,
                                subtensor_hashed::MLDSA_TRANSACTION_CONTEXT,
                            )
                            .unwrap(),
                    },
                },
                extra,
            )
        } else {
            let signature = if scheme == Scheme::Sr25519 {
                sr25519::Pair::from_seed(&seed).sign(&payload).0
            } else {
                ed25519::Pair::from_seed(&seed).sign(&payload).0
            };
            UncheckedExtrinsic::new_hashed(
                call,
                AuthorizeAccount {
                    account: self.account.clone(),
                    proof: Proof {
                        generation,
                        public_key: Self::public(self.mode, key_generation).try_into().unwrap(),
                        next_commitment,
                        signature,
                    },
                },
                extra,
            )
        }
    }
}
fn gas_price() -> U256 {
    <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0
}
fn evm_call(wallet: &Wallet, nonce: Option<U256>) -> RuntimeCall {
    RuntimeCall::EVM(pallet_evm::Call::call {
        source: wallet.source(),
        target: H160::repeat_byte(0x42),
        input: vec![],
        value: U256::zero(),
        gas_limit: 100_000,
        max_fee_per_gas: gas_price(),
        max_priority_fee_per_gas: None,
        nonce,
        access_list: vec![],
        authorization_list: vec![],
    })
}
fn simulate_call(wallet: &Wallet, nonce: Option<U256>) {
    let info = with_transaction_unchecked(|| {
        TransactionOutcome::Rollback(hashed_evm_runner::HashedEvmRunner::call(
            wallet.source(),
            H160::repeat_byte(0x42),
            vec![],
            U256::zero(),
            100_000,
            Some(gas_price()),
            None,
            nonce,
            vec![],
            vec![],
            false,
            true,
            None,
            None,
            <Runtime as pallet_evm::Config>::config(),
        ))
    })
    .unwrap();
    assert!(info.exit_reason.is_succeed());
}

#[test]
fn queried_nonce_executes_and_advances_independently_for_every_wallet_mode() {
    for mode in MODES {
        ext().execute_with(|| {
            let wallet = Wallet::new(mode);
            let native = System::account_nonce(&wallet.account);
            let nonce = EVM::account_basic(&wallet.source()).0.nonce;
            simulate_call(&wallet, Some(nonce));
            assert_ok!(
                Executive::apply_extrinsic(wallet.sign(evm_call(&wallet, Some(nonce)))).unwrap()
            );
            assert_eq!(
                System::account_nonce(&wallet.account),
                native + 1,
                "{mode:?}"
            );
            assert_eq!(
                EVM::account_basic(&wallet.source()).0.nonce,
                nonce + U256::one(),
                "{mode:?}"
            );
        });
    }
}

#[test]
fn create_prediction_matches_dispatch_with_and_without_explicit_nonce() {
    for mode in MODES {
        for explicit in [false, true] {
            ext().execute_with(|| {
                let wallet = Wallet::new(mode);
                let nonce = explicit.then(|| EVM::account_basic(&wallet.source()).0.nonce);
                let init = hex::decode("6001600c60003960016000f300").unwrap();
                let simulated = with_transaction_unchecked(|| {
                    TransactionOutcome::Rollback(hashed_evm_runner::HashedEvmRunner::create(
                        wallet.source(),
                        init.clone(),
                        U256::zero(),
                        200_000,
                        Some(gas_price()),
                        None,
                        nonce,
                        vec![],
                        vec![],
                        true,
                        vec![],
                        false,
                        true,
                        None,
                        None,
                        <Runtime as pallet_evm::Config>::config(),
                    ))
                })
                .unwrap();
                assert!(simulated.exit_reason.is_succeed());
                let call = RuntimeCall::EVM(pallet_evm::Call::create {
                    source: wallet.source(),
                    init,
                    value: U256::zero(),
                    gas_limit: 200_000,
                    max_fee_per_gas: gas_price(),
                    max_priority_fee_per_gas: None,
                    nonce,
                    access_list: vec![],
                    authorization_list: vec![],
                });
                assert_ok!(Executive::apply_extrinsic(wallet.sign(call)).unwrap());
                let created = System::events()
                    .iter()
                    .rev()
                    .find_map(|event| match event.event {
                        RuntimeEvent::EVM(pallet_evm::Event::Created { address }) => Some(address),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(
                    created, simulated.value,
                    "{mode:?}, explicit nonce={explicit}"
                );
            });
        }
    }
}

#[test]
fn authorization_fee_is_not_refunded_when_gas_pays_for_execution() {
    for mode in MODES {
        let mut fees = Vec::new();
        let mut charges = Vec::new();
        for factor in [1, 100] {
            ext().execute_with(|| {
                let wallet = Wallet::new(mode);
                hashed_auth::TestVerificationWeight::set(Weight::from_parts(
                    10_000_000 * factor,
                    0,
                ));
                let xt = wallet.sign(evm_call(&wallet, None));
                if matches!(mode, Mode::HashedMs | Mode::Ms) {
                    assert!(xt.encoded_size() > 5_000);
                }
                let mut envelope = xt.get_dispatch_info();
                envelope.call_weight = Weight::zero();
                let expected = TransactionPayment::compute_fee(
                    u32::try_from(xt.encoded_size()).unwrap(),
                    &envelope,
                    TaoBalance::ZERO,
                );
                // Exercise the wire decoder as well as the local constructor.
                let decoded = UncheckedExtrinsic::decode(&mut xt.encode().as_slice()).unwrap();
                assert_eq!(decoded.encode(), xt.encode());
                let before = Balances::free_balance(&wallet.account);
                assert_ok!(Executive::apply_extrinsic(decoded).unwrap());
                charges.push(before - Balances::free_balance(&wallet.account));
                let fee = System::events()
                    .iter()
                    .rev()
                    .find_map(|event| match event.event {
                        RuntimeEvent::TransactionPayment(
                            pallet_transaction_payment::Event::TransactionFeePaid {
                                actual_fee,
                                ..
                            },
                        ) => Some(actual_fee),
                        _ => None,
                    })
                    .unwrap();
                if !matches!(mode, Mode::Sr | Mode::Ed) {
                    assert_eq!(fee, expected, "{mode:?}");
                }
                fees.push(fee);
            });
        }
        if matches!(mode, Mode::Sr | Mode::Ed) {
            assert_eq!(fees, vec![TaoBalance::ZERO; 2]);
        } else {
            assert!(
                fees[1] > fees[0] && fees[0] > TaoBalance::ZERO,
                "{mode:?}: authorization fees {fees:?}"
            );
            // Gas is unchanged: increasing authorization work must increase
            // the real balance debit by precisely the added native fee.
            assert!(charges[0] > fees[0]);
            assert_eq!(charges[1] - charges[0], fees[1] - fees[0], "{mode:?}");
        }
    }
}

#[test]
fn reverted_and_invalid_evm_calls_keep_authorization_fees_and_reject_replays() {
    for mode in MODES {
        for invalid in [false, true] {
            ext().execute_with(|| {
                let wallet = Wallet::new(mode);
                let nonce = EVM::account_basic(&wallet.source()).0.nonce;
                // REVERT(0, 0): a completed EVM execution still pays gas.
                pallet_evm::AccountCodes::<Runtime>::insert(
                    H160::repeat_byte(0x42),
                    hex::decode("60006000fd").unwrap(),
                );
                let xt = wallet.sign(evm_call(
                    &wallet,
                    Some(nonce + U256::from(<u8 as From<bool>>::from(invalid))),
                ));
                let replay = xt.clone();
                let native = System::account_nonce(&wallet.account);
                let generation = HashedAccounts::accounts(&wallet.account).map(|a| a.generation);
                let result = Executive::apply_extrinsic(xt).unwrap();
                if invalid {
                    assert_eq!(
                        result,
                        Err(pallet_evm::Error::<Runtime>::InvalidNonce.into())
                    );
                } else {
                    assert_ok!(result);
                    assert!(System::events().iter().any(|e| matches!(
                        e.event,
                        RuntimeEvent::EVM(pallet_evm::Event::ExecutedFailed { .. })
                    )));
                }
                assert_eq!(System::account_nonce(&wallet.account), native + 1);
                assert_eq!(
                    EVM::account_basic(&wallet.source()).0.nonce,
                    nonce + U256::from(<u8 as From<bool>>::from(!invalid))
                );
                if let Some(generation) = generation {
                    assert_eq!(
                        HashedAccounts::accounts(&wallet.account)
                            .unwrap()
                            .generation,
                        generation + 1
                    );
                }
                let paid = System::events()
                    .iter()
                    .rev()
                    .find_map(|e| match e.event {
                        RuntimeEvent::TransactionPayment(
                            pallet_transaction_payment::Event::TransactionFeePaid {
                                actual_fee,
                                ..
                            },
                        ) => Some(actual_fee),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(
                    paid > TaoBalance::ZERO,
                    invalid || !matches!(mode, Mode::Sr | Mode::Ed)
                );
                assert!(Executive::apply_extrinsic(replay).is_err());
            });
        }
    }
}

#[test]
fn alias_nonce_survives_balance_reaping_and_native_transactions_when_disabled() {
    for mode in [Mode::HashedSr, Mode::HashedEd, Mode::HashedMs, Mode::Ms] {
        ext().execute_with(|| {
            let wallet = Wallet::new(mode);
            assert_ok!(
                Executive::apply_extrinsic(wallet.sign(evm_call(&wallet, Some(U256::zero()))))
                    .unwrap()
            );
            let backing =
                evm_origin::HashedAddressMapping::nonce_account_id(wallet.source()).unwrap();
            assert_ne!(backing, wallet.account);
            let _ = Balances::make_free_balance_be(&backing, TaoBalance::new(1_000_000_000));
            let _ = Balances::make_free_balance_be(&backing, TaoBalance::ZERO);
            assert_eq!(System::account_nonce(&backing), 1);
            assert_eq!(System::account(&backing).sufficients, 1);
            pallet_admin_utils::HashedAccountsEnabled::<Runtime>::put(false);
            assert_eq!(EVM::account_basic(&wallet.source()).0.nonce, U256::one());
            simulate_call(&wallet, Some(U256::one()));
            assert!(Executive::apply_extrinsic(wallet.sign(call())).is_err());
            pallet_admin_utils::HashedAccountsEnabled::<Runtime>::put(true);
            assert_ok!(Executive::apply_extrinsic(wallet.sign(call())).unwrap());
            assert_eq!(EVM::account_basic(&wallet.source()).0.nonce, U256::one());
            simulate_call(&wallet, Some(U256::one()));
            assert_ok!(
                Executive::apply_extrinsic(wallet.sign(evm_call(&wallet, Some(U256::one()))))
                    .unwrap()
            );
            assert_eq!(EVM::account_basic(&wallet.source()).0.nonce, U256::from(2));
        });
    }
}

#[test]
fn create2_prediction_and_envelope_fee_match_dispatch() {
    for mode in MODES {
        ext().execute_with(|| {
            let wallet = Wallet::new(mode);
            let nonce = Some(EVM::account_basic(&wallet.source()).0.nonce);
            let init = hex::decode("6001600c60003960016000f300").unwrap();
            let salt = H256::repeat_byte(0x17);
            let simulated = with_transaction_unchecked(|| {
                TransactionOutcome::Rollback(hashed_evm_runner::HashedEvmRunner::create2(
                    wallet.source(),
                    init.clone(),
                    salt,
                    U256::zero(),
                    200_000,
                    Some(gas_price()),
                    None,
                    nonce,
                    vec![],
                    vec![],
                    true,
                    vec![],
                    false,
                    true,
                    None,
                    None,
                    <Runtime as pallet_evm::Config>::config(),
                ))
            })
            .unwrap();
            assert!(simulated.exit_reason.is_succeed());
            let call = RuntimeCall::EVM(pallet_evm::Call::create2 {
                source: wallet.source(),
                init,
                salt,
                value: U256::zero(),
                gas_limit: 200_000,
                max_fee_per_gas: gas_price(),
                max_priority_fee_per_gas: None,
                nonce,
                access_list: vec![],
                authorization_list: vec![],
            });
            let xt = wallet.sign(call);
            let mut envelope = xt.get_dispatch_info();
            envelope.call_weight = Weight::zero();
            envelope.pays_fee = Pays::Yes;
            let expected = TransactionPayment::compute_fee(
                u32::try_from(xt.encoded_size()).unwrap(),
                &envelope,
                TaoBalance::ZERO,
            );
            assert_ok!(Executive::apply_extrinsic(xt).unwrap());
            let events = System::events();
            let created = events
                .iter()
                .rev()
                .find_map(|event| match event.event {
                    RuntimeEvent::EVM(pallet_evm::Event::Created { address }) => Some(address),
                    _ => None,
                })
                .unwrap();
            assert_eq!(created, simulated.value, "{mode:?}");
            let fee = events
                .iter()
                .rev()
                .find_map(|event| match event.event {
                    RuntimeEvent::TransactionPayment(
                        pallet_transaction_payment::Event::TransactionFeePaid {
                            actual_fee, ..
                        },
                    ) => Some(actual_fee),
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                fee,
                if matches!(mode, Mode::Sr | Mode::Ed) {
                    TaoBalance::ZERO
                } else {
                    expected
                }
            );
        });
    }
}
