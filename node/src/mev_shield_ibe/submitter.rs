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

use super::MevShieldIbeSharePool;

fn build_unsigned_submit_block_decryption_key_extrinsic(
    key: stp_mev_shield_ibe::IbeBlockDecryptionKeyV1,
) -> Option<OpaqueExtrinsic> {
    let call = RuntimeCall::MevShield(pallet_shield::Call::submit_block_decryption_key { key });

    let unchecked = UncheckedExtrinsic::new_bare(call);

    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

pub fn spawn_key_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    pool: MevShieldIbeSharePool,
) {
    spawn.spawn(
        "mev-shield-ibe-key-submitter",
        None,
        Box::pin(async move {
            loop {
                for key in pool.try_combine_ready_keys() {
                    let Some(xt) = build_unsigned_submit_block_decryption_key_extrinsic(key) else {
                        continue;
                    };

                    let at_hash = client.info().best_hash;

                    if let Err(err) = transaction_pool
                        .submit_one(at_hash, TransactionSource::Local, xt)
                        .await
                    {
                        log::debug!(
                            target: "mev-shield-ibe",
                            "failed to submit local block decryption key transaction: {err:?}",
                        );
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }),
    );
}
