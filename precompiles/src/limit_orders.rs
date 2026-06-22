use core::marker::PhantomData;

use alloc::string::String;
use fp_evm::{ExitError, PrecompileFailure};
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::ConstU32;
use frame_support::{BoundedVec, traits::IsSubType};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, PrecompileHandle};
use pallet_limit_orders::{Order, OrderStatus, OrderType, SignedOrder, VersionedOrder};
use precompile_utils::prelude::{Address, UnboundedBytes};
use precompile_utils::{EvmResult, solidity::Codec};
use sp_core::{ByteArray, H256, sr25519};
use sp_runtime::{MultiSignature, Perbill, traits::AsSystemOriginSigner, traits::Dispatchable};
use subtensor_runtime_common::NetUid;

use crate::{PrecompileExt, PrecompileHandleExt};

pub struct LimitOrdersPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for LimitOrdersPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_limit_orders::Config
        + pallet_subtensor::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_limit_orders::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    const INDEX: u64 = 2062;
}

#[precompile_utils::precompile]
impl<R> LimitOrdersPrecompile<R>
where
    R: frame_system::Config
        + pallet_balances::Config
        + pallet_evm::Config
        + pallet_limit_orders::Config
        + pallet_subtensor::Config
        + pallet_shield::Config
        + pallet_subtensor_proxy::Config
        + Send
        + Sync
        + scale_info::TypeInfo,
    R::AccountId: From<[u8; 32]> + ByteArray,
    <R as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<R::AccountId> + Clone,
    <R as frame_system::Config>::RuntimeCall: From<pallet_limit_orders::Call<R>>
        + GetDispatchInfo
        + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<pallet_balances::Call<R>>
        + IsSubType<pallet_subtensor::Call<R>>
        + IsSubType<pallet_shield::Call<R>>
        + IsSubType<pallet_subtensor_proxy::Call<R>>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    #[precompile::public("getLimitOrdersEnabled()")]
    #[precompile::view]
    fn get_limit_orders_enabled(handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
        handle.record_db_reads::<R>(1)?;
        Ok(pallet_limit_orders::LimitOrdersEnabled::<R>::get())
    }

    #[precompile::public("getOrderStatus(bytes32)")]
    #[precompile::view]
    fn get_order_status(handle: &mut impl PrecompileHandle, order_id: H256) -> EvmResult<u8> {
        handle.record_db_reads::<R>(1)?;
        Ok(order_status_to_u8(pallet_limit_orders::Orders::<R>::get(
            order_id,
        )))
    }

    #[precompile::public(
        "deriveOrderId((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool))"
    )]
    #[precompile::view]
    fn derive_order_id(_handle: &mut impl PrecompileHandle, order: OrderInput) -> EvmResult<H256> {
        let versioned = versioned_order_from_input::<R>(order)?;
        Ok(pallet_limit_orders::Pallet::<R>::derive_order_id(
            &versioned,
        ))
    }

    #[precompile::public(
        "executeOrders(((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool),bytes,bool,uint64)[],bool)"
    )]
    #[precompile::payable]
    fn execute_orders(
        handle: &mut impl PrecompileHandle,
        orders: alloc::vec::Vec<SignedOrderInput>,
        should_fail: bool,
    ) -> EvmResult<()> {
        let batch = signed_orders_batch::<R>(orders)?;
        let call = pallet_limit_orders::Call::<R>::execute_orders {
            orders: batch,
            should_fail,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public(
        "executeBatchedOrders(uint16,((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool),bytes,bool,uint64)[])"
    )]
    #[precompile::payable]
    fn execute_batched_orders(
        handle: &mut impl PrecompileHandle,
        netuid: u16,
        orders: alloc::vec::Vec<SignedOrderInput>,
    ) -> EvmResult<()> {
        let batch = signed_orders_batch::<R>(orders)?;
        let call = pallet_limit_orders::Call::<R>::execute_batched_orders {
            netuid: netuid.into(),
            orders: batch,
        };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }

    #[precompile::public(
        "cancelOrder((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool))"
    )]
    #[precompile::payable]
    fn cancel_order(handle: &mut impl PrecompileHandle, order: OrderInput) -> EvmResult<()> {
        let versioned = versioned_order_from_input::<R>(order)?;
        let call = pallet_limit_orders::Call::<R>::cancel_order { order: versioned };

        handle.try_dispatch_runtime_call::<R, _>(
            call,
            RawOrigin::Signed(handle.caller_account_id::<R>()),
        )
    }
}

#[derive(Codec, Clone)]
pub struct OrderInput {
    signer: Address,
    hotkey: Address,
    netuid: u16,
    order_type: u8,
    amount: u64,
    limit_price: u64,
    expiry: u64,
    fee_rate: u32,
    fee_recipient: Address,
    relayer: alloc::vec::Vec<Address>,
    has_max_slippage: bool,
    max_slippage: u32,
    chain_id: u64,
    partial_fills_enabled: bool,
}

#[derive(Codec, Clone)]
pub struct SignedOrderInput {
    order: OrderInput,
    signature: UnboundedBytes,
    has_partial_fill: bool,
    partial_fill: u64,
}

fn account_from_address<R>(address: Address) -> R::AccountId
where
    R: frame_system::Config + pallet_evm::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    <R as pallet_evm::Config>::AddressMapping::into_account_id(address.0)
}

fn order_type_from_u8(order_type: u8) -> Result<OrderType, PrecompileFailure> {
    match order_type {
        0 => Ok(OrderType::LimitBuy),
        1 => Ok(OrderType::TakeProfit),
        2 => Ok(OrderType::StopLoss),
        _ => Err(PrecompileFailure::Error {
            exit_status: ExitError::Other("invalid order type".into()),
        }),
    }
}

fn signature_from_bytes(signature: &[u8]) -> Result<MultiSignature, PrecompileFailure> {
    let sig: [u8; 64] = signature.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("sr25519 signature must be 64 bytes".into()),
    })?;
    Ok(MultiSignature::Sr25519(sr25519::Signature::from_raw(sig)))
}

fn relayer_from_input<R>(
    relayer: alloc::vec::Vec<Address>,
) -> Result<Option<BoundedVec<R::AccountId, ConstU32<10>>>, PrecompileFailure>
where
    R: frame_system::Config + pallet_evm::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    if relayer.is_empty() {
        return Ok(None);
    }

    let accounts = relayer
        .into_iter()
        .map(account_from_address::<R>)
        .collect::<alloc::vec::Vec<_>>();

    Ok(Some(BoundedVec::try_from(accounts).map_err(|_| {
        PrecompileFailure::Error {
            exit_status: ExitError::Other("relayer list exceeds maximum of 10".into()),
        }
    })?))
}

fn order_from_input<R>(order: OrderInput) -> Result<Order<R::AccountId>, PrecompileFailure>
where
    R: frame_system::Config + pallet_evm::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    Ok(Order {
        signer: account_from_address::<R>(order.signer),
        hotkey: account_from_address::<R>(order.hotkey),
        netuid: NetUid::from(order.netuid),
        order_type: order_type_from_u8(order.order_type)?,
        amount: order.amount,
        limit_price: order.limit_price,
        expiry: order.expiry,
        fee_rate: Perbill::from_parts(order.fee_rate),
        fee_recipient: account_from_address::<R>(order.fee_recipient),
        relayer: relayer_from_input::<R>(order.relayer)?,
        max_slippage: if order.has_max_slippage {
            Some(Perbill::from_parts(order.max_slippage))
        } else {
            None
        },
        chain_id: order.chain_id,
        partial_fills_enabled: order.partial_fills_enabled,
    })
}

fn versioned_order_from_input<R>(
    order: OrderInput,
) -> Result<VersionedOrder<R::AccountId>, PrecompileFailure>
where
    R: frame_system::Config + pallet_evm::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    Ok(VersionedOrder::V1(order_from_input::<R>(order)?))
}

fn signed_order_from_input<R>(
    input: SignedOrderInput,
) -> Result<SignedOrder<R::AccountId>, PrecompileFailure>
where
    R: frame_system::Config + pallet_evm::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    Ok(SignedOrder {
        order: versioned_order_from_input::<R>(input.order)?,
        signature: signature_from_bytes(input.signature.as_bytes())?,
        partial_fill: if input.has_partial_fill {
            Some(input.partial_fill)
        } else {
            None
        },
    })
}

fn signed_orders_batch<R>(
    orders: alloc::vec::Vec<SignedOrderInput>,
) -> Result<
    BoundedVec<SignedOrder<R::AccountId>, <R as pallet_limit_orders::Config>::MaxOrdersPerBatch>,
    PrecompileFailure,
>
where
    R: frame_system::Config + pallet_evm::Config + pallet_limit_orders::Config,
    R::AccountId: ByteArray,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
{
    orders
        .into_iter()
        .map(signed_order_from_input::<R>)
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .and_then(|converted| {
            BoundedVec::try_from(converted).map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("orders batch exceeds maximum size".into()),
            })
        })
}

fn order_status_to_u8(status: Option<OrderStatus>) -> u8 {
    match status {
        None => 0,
        Some(OrderStatus::Fulfilled) => 1,
        Some(OrderStatus::PartiallyFilled(_)) => 2,
        Some(OrderStatus::Cancelled) => 3,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::expect_used,
        clippy::unwrap_used
    )]

    use alloc::vec;
    use alloc::vec::Vec;

    use codec::Encode;
    use pallet_limit_orders::{
        LimitOrdersEnabled, Order, OrderStatus, OrderType, Orders, SignedOrder, VersionedOrder,
    };
    use precompile_utils::prelude::Address;
    use precompile_utils::prelude::UnboundedBytes;
    use precompile_utils::solidity::{encode_return_value, encode_with_selector};
    use precompile_utils::testing::PrecompileTesterExt;
    use sp_core::{H160, H256, Pair, U256, sr25519};
    use sp_runtime::traits::AccountIdConversion;

    use super::*;
    use crate::PrecompileExt;
    use crate::limit_orders_mock::LimitOrdersMockSwap;
    use crate::mock::{
        AccountId, Runtime, RuntimeOrigin, TEST_HOTKEY_ADDR_INDEX, TEST_SIGNER_ADDR_INDEX,
        addr_from_index, assert_static_call, fund_account, mapped_account, new_test_ext,
        precompiles, selector_u32,
    };

    const CHAIN_ID: u64 = 945;
    const NETUID: u16 = 1;
    const FAR_FUTURE: u64 = u64::MAX;
    const ORDER_AMOUNT: u64 = 1_000_000;
    const LIMIT_PRICE: u64 = 10_000_000_000;
    const ACCOUNT_BALANCE: u64 = 100_000_000_000;

    fn precompile_addr() -> H160 {
        addr_from_index(LimitOrdersPrecompile::<Runtime>::INDEX)
    }

    fn signer_setup() -> (H160, H160, AccountId, sr25519::Pair) {
        let pair = sr25519::Pair::from_string("//Alice", None).expect("valid test seed");
        let signer = AccountId::from(pair.public());
        let signer_address = addr_from_index(TEST_SIGNER_ADDR_INDEX);
        let hotkey_address = addr_from_index(TEST_HOTKEY_ADDR_INDEX);
        assert_eq!(mapped_account(signer_address), signer);
        (signer_address, hotkey_address, signer, pair)
    }

    fn init_limit_orders() {
        LimitOrdersMockSwap::clear();
        let pallet_account: AccountId =
            crate::limit_orders_mock::LimitOrdersPalletId::get().into_account_truncating();
        let pallet_hotkey = crate::limit_orders_mock::LimitOrdersPalletHotkey::get();
        let _ = LimitOrdersMockSwap::register_hotkey(&pallet_account, &pallet_hotkey);
        LimitOrdersEnabled::<Runtime>::set(true);
        LimitOrdersMockSwap::set_price(1.0);
        LimitOrdersMockSwap::set_buy_alpha_return(ORDER_AMOUNT);
    }

    fn order_input(
        signer: H160,
        hotkey: H160,
        order_type: u8,
        relayer: Vec<Address>,
    ) -> OrderInput {
        OrderInput {
            signer: Address(signer),
            hotkey: Address(hotkey),
            netuid: NETUID,
            order_type,
            amount: ORDER_AMOUNT,
            limit_price: LIMIT_PRICE,
            expiry: FAR_FUTURE,
            fee_rate: 0,
            fee_recipient: Address(signer),
            relayer,
            has_max_slippage: false,
            max_slippage: 0,
            chain_id: CHAIN_ID,
            partial_fills_enabled: false,
        }
    }

    fn signed_order_from_pair(
        pair: &sr25519::Pair,
        signer: H160,
        hotkey: H160,
        order_type: u8,
    ) -> SignedOrderInput {
        let input = order_input(signer, hotkey, order_type, vec![]);
        let versioned = versioned_order_from_input::<Runtime>(input.clone()).expect("valid order");
        let signature = pair.sign(versioned.encode().as_slice());
        SignedOrderInput {
            order: input,
            signature: UnboundedBytes::from(&signature.0[..]),
            has_partial_fill: false,
            partial_fill: 0,
        }
    }

    fn pallet_signed_order(
        pair: &sr25519::Pair,
        signer: AccountId,
        hotkey: AccountId,
        order_type: OrderType,
    ) -> SignedOrder<AccountId> {
        let order = VersionedOrder::V1(Order {
            signer: signer.clone(),
            hotkey,
            netuid: NETUID.into(),
            order_type,
            amount: ORDER_AMOUNT,
            limit_price: LIMIT_PRICE,
            expiry: FAR_FUTURE,
            fee_rate: Perbill::zero(),
            fee_recipient: signer.clone(),
            relayer: None,
            max_slippage: None,
            chain_id: CHAIN_ID,
            partial_fills_enabled: false,
        });
        let versioned = order.clone();
        SignedOrder {
            order: versioned.clone(),
            signature: MultiSignature::Sr25519(pair.sign(versioned.encode().as_slice())),
            partial_fill: None,
        }
    }

    // ── helper function unit tests ───────────────────────────────────────────

    #[test]
    fn order_type_from_u8_accepts_known_variants() {
        assert_eq!(order_type_from_u8(0).unwrap(), OrderType::LimitBuy);
        assert_eq!(order_type_from_u8(1).unwrap(), OrderType::TakeProfit);
        assert_eq!(order_type_from_u8(2).unwrap(), OrderType::StopLoss);
    }

    #[test]
    fn order_type_from_u8_rejects_unknown_variant() {
        assert!(order_type_from_u8(9).is_err());
    }

    #[test]
    fn order_status_to_u8_maps_all_variants() {
        assert_eq!(order_status_to_u8(None), 0);
        assert_eq!(order_status_to_u8(Some(OrderStatus::Fulfilled)), 1);
        assert_eq!(
            order_status_to_u8(Some(OrderStatus::PartiallyFilled(42))),
            2
        );
        assert_eq!(order_status_to_u8(Some(OrderStatus::Cancelled)), 3);
    }

    #[test]
    fn signature_from_bytes_accepts_64_byte_sr25519_signature() {
        let bytes = [7u8; 64];
        assert!(matches!(
            signature_from_bytes(&bytes).unwrap(),
            MultiSignature::Sr25519(_)
        ));
    }

    #[test]
    fn signature_from_bytes_rejects_invalid_length() {
        assert!(signature_from_bytes(&[1u8; 63]).is_err());
    }

    #[test]
    fn relayer_from_input_accepts_empty_and_bounded_lists() {
        new_test_ext().execute_with(|| {
            assert!(relayer_from_input::<Runtime>(vec![]).unwrap().is_none());

            let relayers = vec![Address(addr_from_index(1)), Address(addr_from_index(2))];
            let bounded = relayer_from_input::<Runtime>(relayers).unwrap();
            assert_eq!(bounded.expect("some relayers").len(), 2);
        });
    }

    #[test]
    fn relayer_from_input_rejects_more_than_ten_relayers() {
        new_test_ext().execute_with(|| {
            let relayers = (0..11)
                .map(|i| Address(addr_from_index(i)))
                .collect::<Vec<_>>();
            assert!(relayer_from_input::<Runtime>(relayers).is_err());
        });
    }

    #[test]
    fn signed_orders_batch_rejects_oversized_input() {
        new_test_ext().execute_with(|| {
            let (signer, hotkey, _, pair) = signer_setup();
            let order = signed_order_from_pair(&pair, signer, hotkey, 0);
            let orders = vec![order; 65];
            assert!(signed_orders_batch::<Runtime>(orders).is_err());
        });
    }

    // ── precompile view tests ────────────────────────────────────────────────

    #[test]
    fn get_limit_orders_enabled_reads_storage() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let caller = addr_from_index(0x9001);
            assert_static_call(
                &precompiles::<LimitOrdersPrecompile<Runtime>>(),
                caller,
                precompile_addr(),
                encode_with_selector(selector_u32("getLimitOrdersEnabled()"), ()),
                U256::from(1u64),
            );

            LimitOrdersEnabled::<Runtime>::set(false);
            assert_static_call(
                &precompiles::<LimitOrdersPrecompile<Runtime>>(),
                caller,
                precompile_addr(),
                encode_with_selector(selector_u32("getLimitOrdersEnabled()"), ()),
                U256::zero(),
            );
        });
    }

    #[test]
    fn get_order_status_reads_on_chain_status() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let caller = addr_from_index(0x9002);
            let order_id = H256::from_low_u64_be(42);

            assert_static_call(
                &precompiles::<LimitOrdersPrecompile<Runtime>>(),
                caller,
                precompile_addr(),
                encode_with_selector(selector_u32("getOrderStatus(bytes32)"), (order_id,)),
                U256::zero(),
            );

            Orders::<Runtime>::insert(order_id, OrderStatus::Cancelled);
            assert_static_call(
                &precompiles::<LimitOrdersPrecompile<Runtime>>(),
                caller,
                precompile_addr(),
                encode_with_selector(selector_u32("getOrderStatus(bytes32)"), (order_id,)),
                U256::from(3u64),
            );
        });
    }

    #[test]
    fn derive_order_id_matches_pallet_derivation() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let (signer, hotkey, _account, pair) = signer_setup();
            let input = order_input(signer, hotkey, 0, vec![]);
            let versioned =
                versioned_order_from_input::<Runtime>(input.clone()).expect("valid order");
            let expected = pallet_limit_orders::Pallet::<Runtime>::derive_order_id(&versioned);
            let _ = pair;

            precompiles::<LimitOrdersPrecompile<Runtime>>()
                .prepare_test(
                    signer,
                    precompile_addr(),
                    encode_with_selector(
                        selector_u32(
                            "deriveOrderId((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool))",
                        ),
                        (input,),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(encode_return_value(expected));
        });
    }

    // ── precompile dispatch tests ────────────────────────────────────────────

    #[test]
    fn cancel_order_marks_order_cancelled() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let (signer, hotkey, signer_account, pair) = signer_setup();
            LimitOrdersMockSwap::register_hotkey(&signer_account, &mapped_account(hotkey));
            let input = order_input(signer, hotkey, 0, vec![]);
            let order_id =
                pallet_limit_orders::Pallet::<Runtime>::derive_order_id(
                    &versioned_order_from_input::<Runtime>(input.clone()).expect("valid order"),
                );
            let _ = pair;

            precompiles::<LimitOrdersPrecompile<Runtime>>()
                .prepare_test(
                    signer,
                    precompile_addr(),
                    encode_with_selector(
                        selector_u32(
                            "cancelOrder((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool))",
                        ),
                        (input,),
                    ),
                )
                .execute_returns(());

            assert_eq!(
                Orders::<Runtime>::get(order_id),
                Some(OrderStatus::Cancelled)
            );
        });
    }

    #[test]
    fn execute_orders_runs_valid_limit_buy() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let (signer, hotkey, signer_account, pair) = signer_setup();
            let relayer = addr_from_index(0x9003);
            LimitOrdersMockSwap::register_hotkey(&signer_account, &mapped_account(hotkey));
            LimitOrdersMockSwap::set_tao_balance(signer_account, ACCOUNT_BALANCE);
            fund_account(&mapped_account(relayer), ACCOUNT_BALANCE);

            let signed = signed_order_from_pair(&pair, signer, hotkey, 0);
            let order_id = pallet_limit_orders::Pallet::<Runtime>::derive_order_id(
                &versioned_order_from_input::<Runtime>(signed.order.clone()).expect("valid order"),
            );

            precompiles::<LimitOrdersPrecompile<Runtime>>()
                .prepare_test(
                    relayer,
                    precompile_addr(),
                    encode_with_selector(
                        selector_u32(
                            "executeOrders(((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool),bytes,bool,uint64)[],bool)",
                        ),
                        (vec![signed], false),
                    ),
                )
                .execute_returns(());

            assert_eq!(Orders::<Runtime>::get(order_id), Some(OrderStatus::Fulfilled));
        });
    }

    #[test]
    fn execute_batched_orders_runs_valid_limit_buy() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let (signer, hotkey, signer_account, pair) = signer_setup();
            let relayer = addr_from_index(0x9004);
            LimitOrdersMockSwap::register_hotkey(&signer_account, &mapped_account(hotkey));
            LimitOrdersMockSwap::set_tao_balance(signer_account, ACCOUNT_BALANCE);
            fund_account(&mapped_account(relayer), ACCOUNT_BALANCE);

            let signed = signed_order_from_pair(&pair, signer, hotkey, 0);
            let order_id = pallet_limit_orders::Pallet::<Runtime>::derive_order_id(
                &versioned_order_from_input::<Runtime>(signed.order.clone()).expect("valid order"),
            );

            precompiles::<LimitOrdersPrecompile<Runtime>>()
                .prepare_test(
                    relayer,
                    precompile_addr(),
                    encode_with_selector(
                        selector_u32(
                            "executeBatchedOrders(uint16,((address,address,uint16,uint8,uint64,uint64,uint64,uint32,address,address[],bool,uint32,uint64,bool),bytes,bool,uint64)[])",
                        ),
                        (NETUID, vec![signed]),
                    ),
                )
                .execute_returns(());

            assert_eq!(Orders::<Runtime>::get(order_id), Some(OrderStatus::Fulfilled));
        });
    }

    #[test]
    fn signed_order_from_input_converts_valid_signature() {
        new_test_ext().execute_with(|| {
            let (signer, hotkey, _, pair) = signer_setup();
            let signed = signed_order_from_pair(&pair, signer, hotkey, 0);
            let converted = signed_order_from_input::<Runtime>(signed).expect("valid signed order");
            assert!(matches!(converted.signature, MultiSignature::Sr25519(_)));
            assert!(converted.partial_fill.is_none());
        });
    }
    #[test]
    fn order_from_input_maps_fields() {
        new_test_ext().execute_with(|| {
            let (signer, hotkey, signer_account, _) = signer_setup();
            let input = order_input(signer, hotkey, 0, vec![]);
            let order = order_from_input::<Runtime>(input).expect("valid order");
            assert_eq!(order.signer, signer_account);
            assert_eq!(order.order_type, OrderType::LimitBuy);
            assert_eq!(order.chain_id, CHAIN_ID);
        });
    }

    #[test]
    fn account_from_address_uses_runtime_mapping() {
        new_test_ext().execute_with(|| {
            let address = addr_from_index(0x9010);
            assert_eq!(
                account_from_address::<Runtime>(Address(address)),
                mapped_account(address)
            );
        });
    }

    #[test]
    fn versioned_order_from_input_wraps_v1() {
        new_test_ext().execute_with(|| {
            let (signer, hotkey, _, _) = signer_setup();
            let input = order_input(signer, hotkey, 1, vec![]);
            let versioned = versioned_order_from_input::<Runtime>(input).expect("valid order");
            assert_eq!(versioned.inner().order_type, OrderType::TakeProfit);
        });
    }

    #[test]
    fn signed_orders_batch_accepts_valid_batch() {
        new_test_ext().execute_with(|| {
            let (signer, hotkey, _, pair) = signer_setup();
            let signed = signed_order_from_pair(&pair, signer, hotkey, 0);
            let batch = signed_orders_batch::<Runtime>(vec![signed]).expect("valid batch");
            assert_eq!(batch.len(), 1);
        });
    }

    #[test]
    fn pallet_signed_order_can_be_cancelled_directly_for_reference() {
        new_test_ext().execute_with(|| {
            init_limit_orders();
            let (_signer, hotkey, signer_account, pair) = signer_setup();
            LimitOrdersMockSwap::register_hotkey(&signer_account, &mapped_account(hotkey));
            let signed = pallet_signed_order(
                &pair,
                signer_account.clone(),
                mapped_account(hotkey),
                OrderType::LimitBuy,
            );
            let order_id = pallet_limit_orders::Pallet::<Runtime>::derive_order_id(&signed.order);

            pallet_limit_orders::Pallet::<Runtime>::cancel_order(
                RuntimeOrigin::signed(signer_account),
                signed.order,
            )
            .expect("cancel should succeed");

            assert_eq!(
                Orders::<Runtime>::get(order_id),
                Some(OrderStatus::Cancelled)
            );
        });
    }
}
