//! Local unsigned submission of finalized epoch DKG public outputs.

use std::{sync::Arc, time::Duration};

use codec::Encode;
use node_subtensor_runtime::{
    RuntimeCall, UncheckedExtrinsic,
    opaque::{Block, UncheckedExtrinsic as OpaqueExtrinsic},
};
use sc_client_api::HeaderBackend;
use sc_service::SpawnTaskHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_runtime::OpaqueExtrinsic as SpOpaqueExtrinsic;

use crate::client::FullClient;
use mev_shield_ibe_runtime_api::EpochDkgPublication;

fn build_unsigned_publish_epoch_key_extrinsic(
    publication: EpochDkgPublication,
) -> Option<OpaqueExtrinsic> {
    let call =
        RuntimeCall::MevShield(pallet_shield::Call::publish_ibe_epoch_public_key { publication });
    let unchecked = UncheckedExtrinsic::new_bare(call);
    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

pub fn spawn_dkg_publication_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    mut rx: futures::channel::mpsc::UnboundedReceiver<EpochDkgPublication>,
) {
    use futures::StreamExt;

    spawn.spawn(
        "mev-shield-ibe-dkg-publication-submitter",
        None,
        Box::pin(async move {
            while let Some(publication) = rx.next().await {
                let Some(xt) = build_unsigned_publish_epoch_key_extrinsic(publication) else {
                    continue;
                };
                let at_hash = client.info().best_hash;
                if let Err(err) = transaction_pool
                    .submit_one(at_hash, TransactionSource::Local, xt)
                    .await
                {
                    log::debug!(
                        target: "mev-shield-ibe",
                        "failed to submit local epoch DKG public-key publication: {err:?}",
                    );
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }),
    );
}
