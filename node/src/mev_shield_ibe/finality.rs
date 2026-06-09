use std::sync::Arc;

use mev_shield_ibe_runtime_api::MevShieldIbeApi;
use sc_client_api::{BlockBackend, BlockchainEvents, HeaderBackend};
use sc_service::SpawnTaskHandle;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, SaturatedConversion};

use super::MevShieldIbeSharePool;

pub fn spawn_finality_gate<Block, Client>(
    spawn: &SpawnTaskHandle,
    client: Arc<Client>,
    pool: MevShieldIbeSharePool,
) where
    Block: BlockT,
    Block::Hash: Into<sp_core::H256>,
    sp_runtime::traits::NumberFor<Block>: core::convert::TryFrom<u64>,
    Client: BlockchainEvents<Block>
        + HeaderBackend<Block>
        + BlockBackend<Block>
        + ProvideRuntimeApi<Block>
        + Send
        + Sync
        + 'static,
    Client::Api: MevShieldIbeApi<Block>,
{
    spawn.spawn(
        "mev-shield-ibe-finality-gate",
        None,
        Box::pin(async move {
            use futures::StreamExt;

            let mut finality_stream = client.finality_notification_stream();

            while let Some(notification) = finality_stream.next().await {
                let finalized_head_hash = notification.hash;
                let finalized_head_number: u64 = (*notification.header.number()).saturated_into();
                let best_number: u64 = client.info().best_number.saturated_into();
                let Ok(identities) = client
                    .runtime_api()
                    .pending_ibe_identities(finalized_head_hash, pool.max_pending_identities())
                else {
                    continue;
                };

                for identity in identities {
                    let Some(ordering_block_number) = identity.target_block.checked_sub(1) else {
                        continue;
                    };
                    if ordering_block_number > finalized_head_number {
                        continue;
                    }
                    let Ok(ordering_number) = <sp_runtime::traits::NumberFor<Block> as core::convert::TryFrom<u64>>::try_from(ordering_block_number) else {
                        continue;
                    };
                    let Ok(Some(ordering_hash)) = client.hash(ordering_number) else {
                        continue;
                    };
                    pool.mark_finalized_identity_unlocked(
                        identity,
                        ordering_block_number,
                        ordering_hash.into(),
                        best_number,
                    );
                }
            }
        }),
    );
}
