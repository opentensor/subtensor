//! Replaying the runtime transaction-extension tuple for non-extrinsic dispatch paths.
//!
//! Some entry points dispatch runtime calls *directly* (as plain Rust function calls) instead of
//! submitting a signed extrinsic. The two in-tree examples are the EVM precompiles
//! ([`pallet-evm`]) and the ink! chain extensions ([`pallet-contracts`]). Such calls never run the
//! runtime's `TransactionExtension` tuple, so any check that lives in an extension — most
//! importantly `pallet-rate-limiting` — would be silently bypassed.
//!
//! [`RuntimeTxExtensionProvider`] lets a runtime expose its extension tuple, and
//! [`dispatch_with_tx_extensions`] replays it (`validate` -> `prepare` -> `dispatch` ->
//! `post_dispatch`) around the actual dispatch, exactly like a normal extrinsic would. The EVM
//! precompiles wrap this same logic with gas accounting and an EVM context; the ink! chain
//! extensions use it directly.

use frame_support::dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use sp_runtime::{
    DispatchError, DispatchResult,
    traits::{
        AsSystemOriginSigner, Dispatchable, ExtensionPostDispatchWeightHandler,
        TransactionExtension, TxBaseImplication,
    },
    transaction_validity::{TransactionSource, TransactionValidityError},
};

/// Convenience alias for a runtime's aggregated call type.
pub type RuntimeCallOf<R> = <R as frame_system::Config>::RuntimeCall;

/// Provides the runtime-configured transaction-extension tuple to non-extrinsic dispatch paths
/// (EVM precompiles, ink! chain extensions) so they can enforce extension-layer checks such as
/// rate limiting.
pub trait RuntimeTxExtensionProvider: frame_system::Config
where
    RuntimeCallOf<Self>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    /// Runtime-provided transaction extensions used for directly-dispatched runtime calls.
    type Extensions: TransactionExtension<RuntimeCallOf<Self>>;

    fn tx_extensions() -> Self::Extensions;
}

/// Failure modes of [`dispatch_with_tx_extensions`].
#[derive(Debug)]
pub enum TxExtDispatchError {
    /// A transaction extension (`validate` / `prepare` / `post_dispatch`) rejected the call. For
    /// rate limiting this carries `InvalidTransaction::Custom(_)`.
    Extension(TransactionValidityError),
    /// The dispatch itself returned an error after the extensions accepted it.
    Dispatch(DispatchError),
}

/// Dispatch `call` with `origin`, replaying the runtime's transaction-extension tuple around it.
///
/// This mirrors what a normal extrinsic does — `validate` -> `prepare` -> dispatch ->
/// `post_dispatch` — so callers that bypass the extrinsic pipeline (e.g. chain extensions) still
/// trigger extension-layer checks like rate limiting and still record their usage. Unlike the EVM
/// precompile helper it does not establish an EVM context or perform gas accounting; weight
/// charging is expected to be handled by the caller (the contracts host already meters weight).
pub fn dispatch_with_tx_extensions<R, C>(
    call: C,
    origin: RawOrigin<R::AccountId>,
) -> Result<PostDispatchInfo, TxExtDispatchError>
where
    R: RuntimeTxExtensionProvider + Send + Sync,
    RuntimeCallOf<R>:
        From<C> + GetDispatchInfo + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeOrigin:
        From<RawOrigin<R::AccountId>> + AsSystemOriginSigner<R::AccountId> + Clone,
{
    let call = RuntimeCallOf::<R>::from(call);
    let mut info = GetDispatchInfo::get_dispatch_info(&call);

    let extensions = <R as RuntimeTxExtensionProvider>::tx_extensions();
    info.extension_weight = info
        .extension_weight
        .saturating_add(extensions.weight(&call));

    let origin = <R as frame_system::Config>::RuntimeOrigin::from(origin);
    let implicit = extensions
        .implicit()
        .map_err(TxExtDispatchError::Extension)?;
    let (_, val, origin) = extensions
        .validate(
            origin,
            &call,
            &info,
            0,
            implicit,
            &TxBaseImplication(()),
            TransactionSource::External,
        )
        .map_err(TxExtDispatchError::Extension)?;
    let pre = extensions
        .prepare(val, &origin, &call, &info, 0)
        .map_err(TxExtDispatchError::Extension)?;

    match call.dispatch(origin) {
        Ok(mut post_info) => {
            post_info.set_extension_weight(&info);
            let result: DispatchResult = Ok(());
            <R::Extensions as TransactionExtension<RuntimeCallOf<R>>>::post_dispatch(
                pre,
                &info,
                &mut post_info,
                0,
                &result,
            )
            .map_err(TxExtDispatchError::Extension)?;
            Ok(post_info)
        }
        Err(e) => {
            let mut post_info = e.post_info;
            post_info.set_extension_weight(&info);
            let result: DispatchResult = Err(e.error);
            <R::Extensions as TransactionExtension<RuntimeCallOf<R>>>::post_dispatch(
                pre,
                &info,
                &mut post_info,
                0,
                &result,
            )
            .map_err(TxExtDispatchError::Extension)?;
            Err(TxExtDispatchError::Dispatch(e.error))
        }
    }
}
