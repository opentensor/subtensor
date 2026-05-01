use codec::{Decode, Encode};
use futures::{FutureExt, StreamExt, channel::mpsc};
use sc_network::{
    PeerId,
    config::{NonDefaultSetConfig, NotificationHandshake, ProtocolName, SetConfig},
    service::traits::{NotificationEvent, NotificationService},
};
use sc_service::SpawnTaskHandle;
use std::collections::BTreeSet;
use stp_mev_shield_ibe::IbePartialDecryptionKeyShareV1;

const PROTOCOL: &[u8] = b"/subtensor/mev-shield-ibe/2";
const MAX_NOTIFICATION_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Encode, Decode)]
pub enum WireMessage {
    PartialDecryptionKeyShareV1(IbePartialDecryptionKeyShareV1),
}

pub fn protocol_config() -> (NonDefaultSetConfig, Box<dyn NotificationService>) {
    let mut set_config = SetConfig::default();
    set_config.in_peers = 25;
    set_config.out_peers = 75;

    NonDefaultSetConfig::new(
        ProtocolName::from(PROTOCOL.to_vec()),
        Vec::new(),
        MAX_NOTIFICATION_SIZE,
        Some(NotificationHandshake::from_bytes(vec![2])),
        set_config,
    )
}

pub fn spawn_network_task(
    spawn_handle: &SpawnTaskHandle,
    mut service: Box<dyn NotificationService>,
    inbound: mpsc::UnboundedSender<WireMessage>,
    mut outbound: mpsc::UnboundedReceiver<WireMessage>,
) {
    spawn_handle.spawn(
        "mev-shield-ibe-share-network",
        None,
        Box::pin(async move {
            let mut peers = BTreeSet::<PeerId>::new();

            loop {
                futures::select! {
                    event = service.next_event().fuse() => {
                        match event {
                            Some(NotificationEvent::NotificationStreamOpened { remote, .. }) => {
                                peers.insert(remote);
                            }
                            Some(NotificationEvent::NotificationStreamClosed { remote }) => {
                                peers.remove(&remote);
                            }
                            Some(NotificationEvent::NotificationReceived { notification, .. }) => {
                                if let Ok(msg) = WireMessage::decode(&mut &notification[..]) {
                                    let _ = inbound.unbounded_send(msg);
                                }
                            }
                            None => break,
                        }
                    }

                    msg = outbound.next().fuse() => {
                        let Some(msg) = msg else { break };
                        let bytes = msg.encode();

                        for peer in peers.iter().copied() {
                            service.send_sync_notification(peer, bytes.clone());
                        }
                    }
                }
            }
        }),
    );
}
