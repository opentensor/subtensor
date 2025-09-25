#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

// Run all tests
// cargo test --package pallet-subtensor --lib -- tests::epoch_logs --show-output

use serial_test::serial;
use sp_core::U256;
use subtensor_runtime_common::{AlphaCurrency, MechId};

use std::io::{Result as IoResult, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use super::mock::*;
use crate::*;

const NETUID: u16 = 1;
const TEMPO: u16 = 10;
const ACTIVITY_CUTOFF: u16 = 10;
// Coldkey is irrelevant for epoch because it operates only hotkeys. Nominators' stake is distributed downstream
// We can use a single coldkey for all tests here.
const COLDKEY: u16 = 9876;

#[derive(Clone)]
struct Neuron {
    uid: u16,
    hotkey: U256,
    validator: bool,
    alpha_stake: u64,
    registration_block: u64,
    last_update: u64,
}

impl Neuron {
    fn new(
        uid: u16,
        hotkey: u16,
        validator: bool,
        alpha_stake: u64,
        registration_block: u64,
        last_update: u64,
    ) -> Self {
        Neuron {
            uid,
            hotkey: U256::from(hotkey),
            validator,
            alpha_stake,
            registration_block,
            last_update,
        }
    }
}

fn setup_epoch(neurons: Vec<Neuron>, mechanism_count: u8) {
    let netuid = NetUid::from(NETUID);

    // Setup subnet parameters
    let network_n = neurons.len() as u16;
    SubnetworkN::<Test>::insert(netuid, network_n);
    ActivityCutoff::<Test>::insert(netuid, ACTIVITY_CUTOFF);
    Tempo::<Test>::insert(netuid, TEMPO);

    // Setup neurons
    let mut last_update_vec: Vec<u64> = Vec::new();
    let mut permit_vec: Vec<bool> = Vec::new();
    neurons.iter().for_each(|neuron| {
        let hotkey = U256::from(neuron.hotkey);

        Keys::<Test>::insert(netuid, neuron.uid, hotkey);
        BlockAtRegistration::<Test>::insert(netuid, neuron.uid, neuron.registration_block);
        last_update_vec.push(neuron.last_update);
        permit_vec.push(neuron.validator);

        // Setup stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &U256::from(COLDKEY),
            netuid,
            AlphaCurrency::from(neuron.alpha_stake),
        );
    });

    ValidatorPermit::<Test>::insert(netuid, permit_vec);
    for m in 0..mechanism_count {
        let netuid_index = SubtensorModule::get_mechanism_storage_index(netuid, m.into());
        LastUpdate::<Test>::insert(netuid_index, last_update_vec.clone());
    }
}

/// Run `f` with a per-thread subscriber configured by `spec` and
/// return the captured log text.
pub fn with_log_capture<F, R>(spec: &str, f: F) -> String
where
    F: FnOnce() -> R,
{
    // ensure log::... is bridged to tracing (no-op if already set)
    let _ = tracing_log::LogTracer::init();

    // Shared buffer we'll write logs into
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    struct Buf(Arc<Mutex<Vec<u8>>>);
    impl Write for Buf {
        fn write(&mut self, b: &[u8]) -> IoResult<usize> {
            let mut g = self.0.lock().unwrap();
            g.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    // Formatting layer that writes into our buffer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .without_time()
        .with_writer({
            let buf = buf.clone();
            move || Buf(buf.clone())
        });

    // Parse filter spec (full RUST_LOG syntax). Fall back to "off" on error.
    let filter = EnvFilter::try_new(spec).unwrap_or_else(|_| EnvFilter::new("off"));

    // Build the per-thread subscriber
    let sub = tracing_subscriber::registry().with(filter).with(fmt_layer);

    // Activate just for this thread/scope and run the code
    tracing::subscriber::with_default(sub, f);

    // Collect captured text
    let mut g = buf.lock().unwrap();
    String::from_utf8(std::mem::take(&mut *g)).unwrap_or_default()
}

#[test]
#[serial]
fn test_simple() {
    new_test_ext(1).execute_with(|| {
        let logs = with_log_capture("trace", || {
            #[rustfmt::skip]
            let neurons = [
                //          uid  hotkey   vali            stake    reg   upd
                Neuron::new(  0,      1,  true,   1_000_000_000,     0,    1 ),
                Neuron::new(  1,      2,  true,   1_000_000_000,     0,    1 ),
            ];
            setup_epoch(neurons.to_vec(), 1);

            // Run epoch, watch logs
            let emission = AlphaCurrency::from(1_000_000_000);
            SubtensorModule::epoch_mechanism(NetUid::from(NETUID), MechId::from(0), emission);
        });

        let has = |s: &str| logs.contains(s);

        assert!(has("Number of Neurons in Network: 2"));
        assert!(has("Inactive: [false, false]"));
        assert!(has("Block at registration: [0, 0]"));
        assert!(has("alpha_stake: [1000000000, 1000000000]"));
        assert!(has("Filtered stake: [1000000000, 1000000000]"));
        assert!(has("Normalised Stake: [0.5, 0.5]"));
        assert!(has("validator_permits: [true, true]"));
        assert!(has("new_validator_permits: [true, true]"));
        assert!(has("Active Stake: [0.5, 0.5]"));
        assert!(has("Consensus: [0, 0]"));
        assert!(has("Normalized Validator Emission: [0.5, 0.5]"));
        assert!(has("Normalized Combined Emission: [0.5, 0.5]"));
        assert!(has(
            "Combined Emission: [AlphaCurrency(500000000), AlphaCurrency(500000000)]"
        ));
        assert!(has("Pruning Scores: [0.5, 0.5]"));
        assert!(!has("math error:"));
    });
}

#[test]
#[serial]
fn test_bad_permit_vector() {
    new_test_ext(1).execute_with(|| {
        let logs = with_log_capture("trace", || {
            #[rustfmt::skip]
            let neurons = [
                //          uid  hotkey   vali            stake    reg   upd
                Neuron::new(  0,      1,  true,   1_000_000_000,     0,    1 ),
                Neuron::new(  1,      2,  true,   1_000_000_000,     0,    1 ),
            ];
            setup_epoch(neurons.to_vec(), 1);
            ValidatorPermit::<Test>::insert(NetUid::from(NETUID), vec![true]);

            // Run epoch, watch logs
            let emission = AlphaCurrency::from(1_000_000_000);
            SubtensorModule::epoch_mechanism(NetUid::from(NETUID), MechId::from(0), emission);
        });

        let has = |s: &str| logs.contains(s);

        assert!(has(
            "math error: inplace_mask_vector input lengths are not equal"
        ));
    });
}
