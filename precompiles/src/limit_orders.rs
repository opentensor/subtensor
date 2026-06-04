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
    fn get_limit_orders_enabled(_handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
        Ok(pallet_limit_orders::LimitOrdersEnabled::<R>::get())
    }

    #[precompile::public("getOrderStatus(bytes32)")]
    #[precompile::view]
    fn get_order_status(_handle: &mut impl PrecompileHandle, order_id: H256) -> EvmResult<u8> {
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

#[derive(Codec)]
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

#[derive(Codec)]
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
