//! Fan-out bridge from local MeV Shield messages into the shared notification wire protocol.

use futures::{StreamExt, channel::mpsc};
use sc_service::SpawnTaskHandle;

use super::network::WireMessage;

pub fn spawn_outbound_fanout(
    spawn_handle: &SpawnTaskHandle,
    mut local_outbound: mpsc::UnboundedReceiver<WireMessage>,
    wire_outbound: mpsc::UnboundedSender<WireMessage>,
) {
    spawn_handle.spawn(
        "mev-shield-ibe-outbound-fanout",
        None,
        Box::pin(async move {
            while let Some(msg) = local_outbound.next().await {
                if wire_outbound.unbounded_send(msg).is_err() {
                    break;
                }
            }
        }),
    );
}
