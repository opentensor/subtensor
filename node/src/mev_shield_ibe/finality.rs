use std::sync::Arc;

use mev_shield_ibe_runtime_api::MevShieldIbeApi;
use sc_client_api::{BlockchainEvents, HeaderBackend};
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
    Client: BlockchainEvents<Block>
        + HeaderBackend<Block>
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
                let finalized_hash = notification.hash;
                let finalized_number: u64 = notification.header.number().saturated_into();

                let best_number: u64 = client.info().best_number.saturated_into();

                let Ok(identities) = client
                    .runtime_api()
                    .pending_ibe_identities(finalized_hash, 512)
                else {
                    continue;
                };

                for identity in identities {
                    pool.mark_finalized_identity_unlocked(
                        identity,
                        finalized_number,
                        finalized_hash.into(),
                        best_number,
                    );
                }
            }
        }),
    );
}
