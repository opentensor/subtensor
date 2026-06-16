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
        let mut retry_tick = tokio::time::interval(std::time::Duration::from_secs(6));
        loop {
            let notification = tokio::select! {
                notification = finality_stream.next() => notification,
                _ = retry_tick.tick() => None,
            };
            let (finalized_head_hash, finalized_head_number) = if let Some(notification) = notification {
                let finalized_head_hash = notification.hash;
                let finalized_head_number: u64 = (*notification.header.number()).saturated_into();
                (finalized_head_hash, finalized_head_number)
            } else {
                let info = client.info();
                (info.finalized_hash, info.finalized_number.saturated_into())
            };
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
                // This call intentionally republishes even for identities that were
                // already unlocked.  It gives late/rotated proposers and peers a
                // durable catch-up path until the runtime queue entry drains and the
                // identity disappears from pending_ibe_identities().
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

/// Dev/manual-seal finality shim.
///
/// Manual seal does not necessarily produce GRANDPA finality notifications, but
/// local demos still need threshold-IBE partial-key release once the local chain
/// has advanced past `target_block - 1`. This task treats the best local block as
/// finalized for manual-seal only; production authority nodes keep using the
/// normal `spawn_finality_gate` path.
pub fn spawn_best_block_gate<Block, Client>(
    spawn: &sc_service::SpawnTaskHandle,
    client: std::sync::Arc<Client>,
    pool: crate::mev_shield_ibe::MevShieldIbeSharePool,
) where
    Block: sp_runtime::traits::Block,
    <Block as sp_runtime::traits::Block>::Hash: Into<sp_core::H256>,
    sp_runtime::traits::NumberFor<Block>: core::convert::TryFrom<u64>,
    Client: sc_client_api::HeaderBackend<Block>
        + sc_client_api::BlockBackend<Block>
        + sp_api::ProvideRuntimeApi<Block>
        + Send
        + Sync
        + 'static,
    Client::Api: mev_shield_ibe_runtime_api::MevShieldIbeApi<Block>,
{
    spawn.spawn(
        "mev-shield-ibe-manual-finality-gate",
        None,
        Box::pin(async move {
            let mut retry_tick = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                retry_tick.tick().await;
                let info = client.info();
                use sp_runtime::traits::SaturatedConversion as _;
                let best_hash = info.best_hash;
                let best_number: u64 = info.best_number.saturated_into::<u64>();
                let Ok(identities) = client
                    .runtime_api()
                    .pending_ibe_identities(best_hash, pool.max_pending_identities())
                else {
                    continue;
                };
                for identity in identities {
                    let Some(ordering_block_number) = identity.target_block.checked_sub(1) else {
                        continue;
                    };
                    if ordering_block_number > best_number {
                        continue;
                    }
                    let Ok(ordering_number) =
                        <sp_runtime::traits::NumberFor<Block> as core::convert::TryFrom<u64>>::try_from(
                            ordering_block_number,
                        )
                    else {
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
