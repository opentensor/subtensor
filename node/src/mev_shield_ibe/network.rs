use std::{collections::BTreeSet, sync::Arc};

use codec::{Decode, Encode};
use futures::{FutureExt, StreamExt, channel::mpsc};
use sc_network::{
    PeerId,
    config::{
        NotificationHandshake, NotificationMetrics, PeerStoreProvider, ProtocolName, SetConfig,
    },
    service::traits::{NetworkBackend, NotificationEvent, NotificationService, ValidationResult},
};
use sc_service::SpawnTaskHandle;
use sp_runtime::traits::Block as BlockT;
use stp_mev_shield_ibe::IbePartialDecryptionKeyShareV1;

use super::dkg_protocol::{DkgAcceptanceVoteV1, DkgDealerCommitmentV1, DkgOutputAttestationV1};

const PROTOCOL: &str = "/subtensor/mev-shield-ibe/3";
const MAX_NOTIFICATION_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Encode, Decode)]
pub enum WireMessage {
    PartialDecryptionKeyShareV1(IbePartialDecryptionKeyShareV1),
    DkgDealerCommitmentV1(DkgDealerCommitmentV1),
    DkgAcceptanceVoteV1(DkgAcceptanceVoteV1),
    DkgOutputAttestationV1(DkgOutputAttestationV1),
}

#[derive(Clone)]
pub struct WireRouter {
    share_pool_tx: mpsc::UnboundedSender<WireMessage>,
    dkg_tx: mpsc::UnboundedSender<WireMessage>,
}

impl WireRouter {
    pub fn new(
        share_pool_tx: mpsc::UnboundedSender<WireMessage>,
        dkg_tx: mpsc::UnboundedSender<WireMessage>,
    ) -> Self {
        Self {
            share_pool_tx,
            dkg_tx,
        }
    }

    fn route(&self, msg: WireMessage) {
        match &msg {
            WireMessage::PartialDecryptionKeyShareV1(_) => {
                let _ = self.share_pool_tx.unbounded_send(msg.clone());
            }
            WireMessage::DkgDealerCommitmentV1(_)
            | WireMessage::DkgAcceptanceVoteV1(_)
            | WireMessage::DkgOutputAttestationV1(_) => {
                let _ = self.dkg_tx.unbounded_send(msg);
            }
        }
    }
}

pub fn protocol_config<NB, Block>(
    metrics: NotificationMetrics,
    peer_store_handle: Arc<dyn PeerStoreProvider>,
) -> (NB::NotificationProtocolConfig, Box<dyn NotificationService>)
where
    Block: BlockT,
    NB: NetworkBackend<Block, <Block as BlockT>::Hash>,
{
    let mut set_config = SetConfig::default();
    set_config.in_peers = 25;
    set_config.out_peers = 75;
    NB::notification_config(
        ProtocolName::from(PROTOCOL),
        Vec::new(),
        MAX_NOTIFICATION_SIZE,
        Some(NotificationHandshake::from_bytes(vec![3])),
        set_config,
        metrics,
        peer_store_handle,
    )
}

pub fn spawn_network_task(
    spawn_handle: &SpawnTaskHandle,
    mut service: Box<dyn NotificationService>,
    router: WireRouter,
    mut outbound: mpsc::UnboundedReceiver<WireMessage>,
) {
    spawn_handle.spawn(
        "mev-shield-ibe-network",
        None,
        Box::pin(async move {
            let mut peers = BTreeSet::<PeerId>::new();
            loop {
                futures::select! {
                    event = service.next_event().fuse() => {
                        match event {
                            Some(NotificationEvent::ValidateInboundSubstream { result_tx, .. }) => {
                                let _ = result_tx.send(ValidationResult::Accept);
                            }
                            Some(NotificationEvent::NotificationStreamOpened { peer, .. }) => {
                                peers.insert(peer);
                            }
                            Some(NotificationEvent::NotificationStreamClosed { peer }) => {
                                peers.remove(&peer);
                            }
                            Some(NotificationEvent::NotificationReceived { notification, .. }) => {
                                if let Ok(msg) = WireMessage::decode(&mut &notification[..]) {
                                    router.route(msg);
                                }
                            }
                            None => break,
                        }
                    }
                    msg = outbound.next().fuse() => {
                        let Some(msg) = msg else { break; };
                        let bytes = msg.encode();
                        for peer in peers.iter() {
                            service.send_sync_notification(peer, bytes.clone());
                        }
                    }
                }
            }
        }),
    );
}
