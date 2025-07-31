// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Tests for Proxy Pallet

#![cfg(test)]

use super::*;

use crate as proxy;
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64, Contains},
};
use sp_core::{H160, H256};
use sp_runtime::{traits::BlakeTwo256, BuildStorage, DispatchError, RuntimeDebug};

type Block = frame_system::mocking::MockBlock<Test>;

pub struct DummyAddressMap;

impl pallet_evm::AddressMapping<u64> for DummyAddressMap {
    fn into_account_id(address: sp_core::H160) -> u64 {
        let account = pallet_evm::HashedAddressMapping::<BlakeTwo256>::into_account_id(address);
        let account_id: &[u8; 32] = account.as_ref();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&account_id[0..8]);
        u64::from_be_bytes(bytes)
    }
}

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system = 1,
        Balances: pallet_balances = 2,
        Proxy: proxy = 3,
        Utility: pallet_utility = 4,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type BaseCallFilter = BaseFilter;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type ReserveIdentifier = [u8; 8];
    type AccountStore = System;
}

impl pallet_utility::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = ();
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum ProxyType {
    Any,
    JustTransfer,
    JustUtility,
}
impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::JustTransfer => {
                matches!(
                    c,
                    RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death { .. })
                )
            }
            ProxyType::JustUtility => matches!(c, RuntimeCall::Utility { .. }),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        self == &ProxyType::Any || self == o
    }
}
pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
    fn contains(c: &RuntimeCall) -> bool {
        match *c {
            // Remark is used as a no-op call in the benchmarking
            RuntimeCall::System(SystemCall::remark { .. }) => true,
            RuntimeCall::System(_) => false,
            _ => true,
        }
    }
}
impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ConstU64<1>;
    type ProxyDepositFactor = ConstU64<1>;
    type MaxProxies = ConstU32<4>;
    type WeightInfo = ();
    type CallHasher = BlakeTwo256;
    type MaxPending = ConstU32<2>;
    type AnnouncementDepositBase = ConstU64<1>;
    type AnnouncementDepositFactor = ConstU64<1>;
    type AddressMapping = DummyAddressMap;
}

use super::{Call as ProxyCall, Event as ProxyEvent};
use frame_system::Call as SystemCall;
use pallet_balances::{Call as BalancesCall, Event as BalancesEvent};
use pallet_utility::{Call as UtilityCall, Event as UtilityEvent};

type SystemError = frame_system::Error<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("Expected to not panic");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10), (2, 10), (3, 10), (4, 10), (5, 3)],
    }
    .assimilate_storage(&mut t)
    .expect("Expected to not panic");
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn last_events(n: usize) -> Vec<RuntimeEvent> {
    system::Pallet::<Test>::events()
        .into_iter()
        .rev()
        .take(n)
        .rev()
        .map(|e| e.event)
        .collect()
}

fn expect_events(e: Vec<RuntimeEvent>) {
    assert_eq!(last_events(e.len()), e);
}

fn call_transfer(dest: u64, value: u64) -> RuntimeCall {
    RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest, value })
}

#[test]
fn announcement_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            1
        ));
        System::assert_last_event(
            ProxyEvent::ProxyAdded {
                delegator: 1,
                delegatee: 3,
                proxy_type: ProxyType::Any,
                delay: 1,
            }
            .into(),
        );
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(2),
            3,
            ProxyType::Any,
            1
        ));
        assert_eq!(Balances::reserved_balance(3), 0);

        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 1, [1; 32].into()));
        let announcements = Announcements::<Test>::get(3);
        assert_eq!(
            announcements.0,
            vec![Announcement {
                real: 1,
                call_hash: [1; 32].into(),
                height: 1
            }]
        );
        assert_eq!(Balances::reserved_balance(3), announcements.1);

        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 2, [2; 32].into()));
        let announcements = Announcements::<Test>::get(3);
        assert_eq!(
            announcements.0,
            vec![
                Announcement {
                    real: 1,
                    call_hash: [1; 32].into(),
                    height: 1
                },
                Announcement {
                    real: 2,
                    call_hash: [2; 32].into(),
                    height: 1
                },
            ]
        );
        assert_eq!(Balances::reserved_balance(3), announcements.1);

        assert_noop!(
            Proxy::announce(RuntimeOrigin::signed(3), 2, [3; 32].into()),
            Error::<Test>::TooMany
        );
    });
}

#[test]
fn remove_announcement_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            1
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(2),
            3,
            ProxyType::Any,
            1
        ));
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 1, [1; 32].into()));
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 2, [2; 32].into()));
        let e = Error::<Test>::NotFound;
        assert_noop!(
            Proxy::remove_announcement(RuntimeOrigin::signed(3), 1, [0; 32].into()),
            e
        );
        assert_ok!(Proxy::remove_announcement(
            RuntimeOrigin::signed(3),
            1,
            [1; 32].into()
        ));
        let announcements = Announcements::<Test>::get(3);
        assert_eq!(
            announcements.0,
            vec![Announcement {
                real: 2,
                call_hash: [2; 32].into(),
                height: 1
            }]
        );
        assert_eq!(Balances::reserved_balance(3), announcements.1);
    });
}

#[test]
fn reject_announcement_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            1
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(2),
            3,
            ProxyType::Any,
            1
        ));
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 1, [1; 32].into()));
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 2, [2; 32].into()));
        let e = Error::<Test>::NotFound;
        assert_noop!(
            Proxy::reject_announcement(RuntimeOrigin::signed(1), 3, [0; 32].into()),
            e
        );
        let e = Error::<Test>::NotFound;
        assert_noop!(
            Proxy::reject_announcement(RuntimeOrigin::signed(4), 3, [1; 32].into()),
            e
        );
        assert_ok!(Proxy::reject_announcement(
            RuntimeOrigin::signed(1),
            3,
            [1; 32].into()
        ));
        let announcements = Announcements::<Test>::get(3);
        assert_eq!(
            announcements.0,
            vec![Announcement {
                real: 2,
                call_hash: [2; 32].into(),
                height: 1
            }]
        );
        assert_eq!(Balances::reserved_balance(3), announcements.1);
    });
}

#[test]
fn announcer_must_be_proxy() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proxy::announce(RuntimeOrigin::signed(2), 1, H256::zero()),
            Error::<Test>::NotProxy
        );
    });
}

#[test]
fn calling_proxy_doesnt_remove_announcement() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::Any,
            0
        ));

        let call = Box::new(call_transfer(6, 1));
        let call_hash = BlakeTwo256::hash_of(&call);

        assert_ok!(Proxy::announce(RuntimeOrigin::signed(2), 1, call_hash));
        assert_ok!(Proxy::proxy(RuntimeOrigin::signed(2), 1, None, call));

        // The announcement is not removed by calling proxy.
        let announcements = Announcements::<Test>::get(2);
        assert_eq!(
            announcements.0,
            vec![Announcement {
                real: 1,
                call_hash,
                height: 1
            }]
        );
    });
}

#[test]
fn delayed_requires_pre_announcement() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::Any,
            1
        ));
        let call = Box::new(call_transfer(6, 1));
        let e = Error::<Test>::Unannounced;
        assert_noop!(
            Proxy::proxy(RuntimeOrigin::signed(2), 1, None, call.clone()),
            e
        );
        let e = Error::<Test>::Unannounced;
        assert_noop!(
            Proxy::proxy_announced(RuntimeOrigin::signed(0), 2, 1, None, call.clone()),
            e
        );
        let call_hash = BlakeTwo256::hash_of(&call);
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(2), 1, call_hash));
        system::Pallet::<Test>::set_block_number(2);
        assert_ok!(Proxy::proxy_announced(
            RuntimeOrigin::signed(0),
            2,
            1,
            None,
            call.clone()
        ));
    });
}

#[test]
fn proxy_announced_removes_announcement_and_returns_deposit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            1
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(2),
            3,
            ProxyType::Any,
            1
        ));
        let call = Box::new(call_transfer(6, 1));
        let call_hash = BlakeTwo256::hash_of(&call);
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 1, call_hash));
        assert_ok!(Proxy::announce(RuntimeOrigin::signed(3), 2, call_hash));
        // Too early to execute announced call
        let e = Error::<Test>::Unannounced;
        assert_noop!(
            Proxy::proxy_announced(RuntimeOrigin::signed(0), 3, 1, None, call.clone()),
            e
        );

        system::Pallet::<Test>::set_block_number(2);
        assert_ok!(Proxy::proxy_announced(
            RuntimeOrigin::signed(0),
            3,
            1,
            None,
            call.clone()
        ));
        let announcements = Announcements::<Test>::get(3);
        assert_eq!(
            announcements.0,
            vec![Announcement {
                real: 2,
                call_hash,
                height: 1
            }]
        );
        assert_eq!(Balances::reserved_balance(3), announcements.1);
    });
}

#[test]
fn filtering_works() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 1000);
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::Any,
            0
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::JustTransfer,
            0
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            4,
            ProxyType::JustUtility,
            0
        ));

        let call = Box::new(call_transfer(6, 1));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(4),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );

        let derivative_id = Utility::derivative_account_id(1, 0);
        Balances::make_free_balance_be(&derivative_id, 1000);
        let inner = Box::new(call_transfer(6, 1));

        let call = Box::new(RuntimeCall::Utility(UtilityCall::as_derivative {
            index: 0,
            call: inner.clone(),
        }));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(4),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );

        let call = Box::new(RuntimeCall::Utility(UtilityCall::batch {
            calls: vec![*inner],
        }));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        expect_events(vec![
            UtilityEvent::BatchCompleted.into(),
            ProxyEvent::ProxyExecuted { result: Ok(()) }.into(),
        ]);
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(4),
            1,
            None,
            call.clone()
        ));
        expect_events(vec![
            UtilityEvent::BatchInterrupted {
                index: 0,
                error: SystemError::CallFiltered.into(),
            }
            .into(),
            ProxyEvent::ProxyExecuted { result: Ok(()) }.into(),
        ]);

        let inner = Box::new(RuntimeCall::Proxy(ProxyCall::new_call_variant_add_proxy(
            5,
            ProxyType::Any,
            0,
        )));
        let call = Box::new(RuntimeCall::Utility(UtilityCall::batch {
            calls: vec![*inner],
        }));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        expect_events(vec![
            UtilityEvent::BatchCompleted.into(),
            ProxyEvent::ProxyExecuted { result: Ok(()) }.into(),
        ]);
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(4),
            1,
            None,
            call.clone()
        ));
        expect_events(vec![
            UtilityEvent::BatchInterrupted {
                index: 0,
                error: SystemError::CallFiltered.into(),
            }
            .into(),
            ProxyEvent::ProxyExecuted { result: Ok(()) }.into(),
        ]);

        let call = Box::new(RuntimeCall::Proxy(ProxyCall::remove_proxies {}));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(4),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        expect_events(vec![
            BalancesEvent::<Test>::Unreserved { who: 1, amount: 5 }.into(),
            ProxyEvent::ProxyExecuted { result: Ok(()) }.into(),
        ]);
    });
}

#[test]
fn add_remove_proxies_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::Any,
            0
        ));
        assert_noop!(
            Proxy::add_proxy(RuntimeOrigin::signed(1), 2, ProxyType::Any, 0),
            Error::<Test>::Duplicate
        );
        assert_eq!(Balances::reserved_balance(1), 2);
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::JustTransfer,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 3);
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 4);
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            4,
            ProxyType::JustUtility,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 5);
        assert_noop!(
            Proxy::add_proxy(RuntimeOrigin::signed(1), 4, ProxyType::Any, 0),
            Error::<Test>::TooMany
        );
        assert_noop!(
            Proxy::remove_proxy(RuntimeOrigin::signed(1), 3, ProxyType::JustTransfer, 0),
            Error::<Test>::NotFound
        );
        assert_ok!(Proxy::remove_proxy(
            RuntimeOrigin::signed(1),
            4,
            ProxyType::JustUtility,
            0
        ));
        System::assert_last_event(
            ProxyEvent::ProxyRemoved {
                delegator: 1,
                delegatee: 4,
                proxy_type: ProxyType::JustUtility,
                delay: 0,
            }
            .into(),
        );
        assert_eq!(Balances::reserved_balance(1), 4);
        assert_ok!(Proxy::remove_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 3);
        System::assert_last_event(
            ProxyEvent::ProxyRemoved {
                delegator: 1,
                delegatee: 3,
                proxy_type: ProxyType::Any,
                delay: 0,
            }
            .into(),
        );
        assert_ok!(Proxy::remove_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::Any,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 2);
        System::assert_last_event(
            ProxyEvent::ProxyRemoved {
                delegator: 1,
                delegatee: 2,
                proxy_type: ProxyType::Any,
                delay: 0,
            }
            .into(),
        );
        assert_ok!(Proxy::remove_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::JustTransfer,
            0
        ));
        assert_eq!(Balances::reserved_balance(1), 0);
        System::assert_last_event(
            ProxyEvent::ProxyRemoved {
                delegator: 1,
                delegatee: 2,
                proxy_type: ProxyType::JustTransfer,
                delay: 0,
            }
            .into(),
        );
        assert_noop!(
            Proxy::add_proxy(RuntimeOrigin::signed(1), 1, ProxyType::Any, 0),
            Error::<Test>::NoSelfProxy
        );
    });
}

#[test]
fn cannot_add_proxy_without_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(5),
            3,
            ProxyType::Any,
            0
        ));
        assert_eq!(Balances::reserved_balance(5), 2);
        assert_noop!(
            Proxy::add_proxy(RuntimeOrigin::signed(5), 4, ProxyType::Any, 0),
            DispatchError::ConsumerRemaining,
        );
    });
}

#[test]
fn proxying_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            2,
            ProxyType::JustTransfer,
            0
        ));
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(1),
            3,
            ProxyType::Any,
            0
        ));

        let call = Box::new(call_transfer(6, 1));
        assert_noop!(
            Proxy::proxy(RuntimeOrigin::signed(4), 1, None, call.clone()),
            Error::<Test>::NotProxy
        );
        assert_noop!(
            Proxy::proxy(
                RuntimeOrigin::signed(2),
                1,
                Some(ProxyType::Any),
                call.clone()
            ),
            Error::<Test>::NotProxy
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_eq!(Balances::free_balance(6), 1);

        let call = Box::new(RuntimeCall::System(SystemCall::set_code { code: vec![] }));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );

        let call = Box::new(RuntimeCall::Balances(BalancesCall::transfer_keep_alive {
            dest: 6,
            value: 1,
        }));
        assert_ok!(
            RuntimeCall::Proxy(super::Call::new_call_variant_proxy(1, None, call.clone()))
                .dispatch(RuntimeOrigin::signed(2))
        );
        System::assert_last_event(
            ProxyEvent::ProxyExecuted {
                result: Err(SystemError::CallFiltered.into()),
            }
            .into(),
        );
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(3),
            1,
            None,
            call.clone()
        ));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_eq!(Balances::free_balance(6), 2);
    });
}

#[test]
fn pure_works() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 11); // An extra one for the ED.
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(1),
            ProxyType::Any,
            0,
            0
        ));
        let anon = Proxy::pure_account(&1, &ProxyType::Any, 0, None);
        System::assert_last_event(
            ProxyEvent::PureCreated {
                pure: anon,
                who: 1,
                proxy_type: ProxyType::Any,
                disambiguation_index: 0,
            }
            .into(),
        );

        // other calls to pure allowed as long as they're not exactly the same.
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(1),
            ProxyType::JustTransfer,
            0,
            0
        ));
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(1),
            ProxyType::Any,
            0,
            1
        ));
        let anon2 = Proxy::pure_account(&2, &ProxyType::Any, 0, None);
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(2),
            ProxyType::Any,
            0,
            0
        ));
        assert_noop!(
            Proxy::create_pure(RuntimeOrigin::signed(1), ProxyType::Any, 0, 0),
            Error::<Test>::Duplicate
        );
        System::set_extrinsic_index(1);
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(1),
            ProxyType::Any,
            0,
            0
        ));
        System::set_extrinsic_index(0);
        System::set_block_number(2);
        assert_ok!(Proxy::create_pure(
            RuntimeOrigin::signed(1),
            ProxyType::Any,
            0,
            0
        ));

        let call = Box::new(call_transfer(6, 1));
        assert_ok!(Balances::transfer_allow_death(
            RuntimeOrigin::signed(3),
            anon,
            5
        ));
        assert_eq!(Balances::free_balance(6), 0);
        assert_ok!(Proxy::proxy(RuntimeOrigin::signed(1), anon, None, call));
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_eq!(Balances::free_balance(6), 1);

        let call = Box::new(RuntimeCall::Proxy(ProxyCall::new_call_variant_kill_pure(
            1,
            ProxyType::Any,
            0,
            1,
            0,
        )));
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(2),
            anon2,
            None,
            call.clone()
        ));
        let de: DispatchError = DispatchError::from(Error::<Test>::NoPermission).stripped();
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Err(de) }.into());
        assert_noop!(
            Proxy::kill_pure(RuntimeOrigin::signed(1), 1, ProxyType::Any, 0, 1, 0),
            Error::<Test>::NoPermission
        );
        assert_eq!(Balances::free_balance(1), 1);
        assert_ok!(Proxy::proxy(
            RuntimeOrigin::signed(1),
            anon,
            None,
            call.clone()
        ));
        assert_eq!(Balances::free_balance(1), 3);
        assert_noop!(
            Proxy::proxy(RuntimeOrigin::signed(1), anon, None, call.clone()),
            Error::<Test>::NotProxy
        );

        // Actually kill the pure proxy.
        assert_ok!(Proxy::kill_pure(
            RuntimeOrigin::signed(anon),
            1,
            ProxyType::Any,
            0,
            1,
            0
        ));
        System::assert_last_event(
            ProxyEvent::PureKilled {
                pure: anon,
                spawner: 1,
                proxy_type: ProxyType::Any,
                disambiguation_index: 0,
            }
            .into(),
        );
    });
}

#[test]
fn test_create_evm_pure() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_slice(&[1; 20]);
        let owner = DummyAddressMap::into_account_id(evm_address);
        Balances::make_free_balance_be(&owner, 100);

        // Test successful creation of EVM pure proxy
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            0,
            0,
            evm_address
        ));

        // Check that the proxy was added to EVMProxies storage
        let proxies = Proxy::evm_proxies(evm_address);
        assert_eq!(proxies.len(), 1);

        // The pure account should exist in Proxies storage
        let pure_account = proxies.first().unwrap();
        assert!(Proxies::<Test>::contains_key(pure_account));

        // Test creation with different index
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            0,
            1,
            evm_address
        ));

        let proxies_after = Proxy::evm_proxies(evm_address);
        assert_eq!(proxies_after.len(), 2);

        // Test error when non-owner tries to create
        let non_owner = 999u64;
        Balances::make_free_balance_be(&non_owner, 100);
        assert_noop!(
            Proxy::create_evm_pure(
                RuntimeOrigin::signed(non_owner),
                ProxyType::Any,
                0,
                2,
                evm_address
            ),
            Error::<Test>::OriginNotMatchMappedEVM
        );

        let max_proxies: u32 = <Test as crate::Config>::MaxProxies::get();
        for i in 2..(max_proxies as u16) {
            assert_ok!(Proxy::create_evm_pure(
                RuntimeOrigin::signed(owner),
                ProxyType::Any,
                0,
                i,
                evm_address
            ));
        }

        // Should fail when trying to exceed MaxProxies
        assert_noop!(
            Proxy::create_evm_pure(
                RuntimeOrigin::signed(owner),
                ProxyType::Any,
                0,
                99,
                evm_address
            ),
            Error::<Test>::TooMany
        );
    });
}

#[test]
fn test_kill_evm_pure() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_slice(&[2; 20]);
        let owner = DummyAddressMap::into_account_id(evm_address);
        Balances::make_free_balance_be(&owner, 100);

        // Create a pure proxy first
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            0,
            0,
            evm_address
        ));

        let proxies = Proxy::evm_proxies(evm_address);
        assert_eq!(proxies.len(), 1);
        let pure_account = proxies[0];

        // Test successful killing of EVM pure proxy
        assert_ok!(Proxy::kill_evm_pure(
            RuntimeOrigin::signed(owner),
            evm_address,
            pure_account
        ));

        // Check that the proxy was removed from EVMProxies storage
        let proxies_after = Proxy::evm_proxies(evm_address);
        assert_eq!(proxies_after.len(), 0);

        // The pure account should no longer exist in Proxies storage
        assert!(!Proxies::<Test>::contains_key(pure_account));

        // Test error when non-owner tries to kill
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            0,
            1,
            evm_address
        ));

        let new_proxies = Proxy::evm_proxies(evm_address);
        let new_pure_account = new_proxies[0];

        let non_owner = 998u64;
        Balances::make_free_balance_be(&non_owner, 100);
        assert_noop!(
            Proxy::kill_evm_pure(
                RuntimeOrigin::signed(non_owner),
                evm_address,
                new_pure_account
            ),
            Error::<Test>::OriginNotMatchMappedEVM
        );

        // Test error when trying to kill non-existent proxy
        let fake_proxy = 777u64;
        assert_noop!(
            Proxy::kill_evm_pure(RuntimeOrigin::signed(owner), evm_address, fake_proxy),
            Error::<Test>::EVMProxyNotFound
        );

        // Clean up - kill the remaining proxy
        assert_ok!(Proxy::kill_evm_pure(
            RuntimeOrigin::signed(owner),
            evm_address,
            new_pure_account
        ));
    });
}

#[test]
fn test_evm_proxy() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_slice(&[3; 20]);
        let owner = DummyAddressMap::into_account_id(evm_address);
        Balances::make_free_balance_be(&owner, 100);

        // Create a pure proxy first
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            0,
            0,
            evm_address
        ));

        let proxies = Proxy::evm_proxies(evm_address);
        let pure_account = proxies[0];

        // Test successful EVM proxy call
        let call = Box::new(call_transfer(6, 1));
        Balances::make_free_balance_be(&pure_account, 10);

        assert_ok!(Proxy::evm_proxy(
            RuntimeOrigin::signed(owner),
            Some(ProxyType::Any),
            call.clone(),
            evm_address,
            pure_account
        ));

        // Verify the call was executed
        System::assert_last_event(ProxyEvent::ProxyExecuted { result: Ok(()) }.into());
        assert_eq!(Balances::free_balance(6), 1);

        // Test error when non-owner tries to use evm_proxy
        let non_owner = 997u64;
        Balances::make_free_balance_be(&non_owner, 100);
        assert_noop!(
            Proxy::evm_proxy(
                RuntimeOrigin::signed(non_owner),
                Some(ProxyType::Any),
                call.clone(),
                evm_address,
                pure_account
            ),
            Error::<Test>::OriginNotMatchMappedEVM
        );

        // Test error when trying to use non-existent EVM proxy
        let fake_proxy = 888u64;
        assert_noop!(
            Proxy::evm_proxy(
                RuntimeOrigin::signed(owner),
                Some(ProxyType::Any),
                call.clone(),
                evm_address,
                fake_proxy
            ),
            Error::<Test>::EVMProxyNotFound
        );

        // Test with different proxy type
        assert_ok!(Proxy::add_proxy(
            RuntimeOrigin::signed(pure_account),
            owner,
            ProxyType::JustTransfer,
            0
        ));

        assert_ok!(Proxy::evm_proxy(
            RuntimeOrigin::signed(owner),
            Some(ProxyType::JustTransfer),
            call.clone(),
            evm_address,
            pure_account
        ));

        // Test without specifying proxy type (should use Any)
        assert_ok!(Proxy::evm_proxy(
            RuntimeOrigin::signed(owner),
            None,
            call,
            evm_address,
            pure_account
        ));
    });
}

#[test]
fn test_evm_proxy_with_delay() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_slice(&[4; 20]);
        let owner = DummyAddressMap::into_account_id(evm_address);
        Balances::make_free_balance_be(&owner, 100);

        // Create a pure proxy with delay
        assert_ok!(Proxy::create_evm_pure(
            RuntimeOrigin::signed(owner),
            ProxyType::Any,
            5, // 5 block delay
            0,
            evm_address
        ));

        let proxies = Proxy::evm_proxies(evm_address);
        let pure_account = proxies[0];

        // Test that immediate call fails due to delay
        let call = Box::new(call_transfer(6, 1));
        Balances::make_free_balance_be(&pure_account, 10);

        assert_noop!(
            Proxy::evm_proxy(
                RuntimeOrigin::signed(owner),
                Some(ProxyType::Any),
                call,
                evm_address,
                pure_account
            ),
            Error::<Test>::Unannounced
        );
    });
}
