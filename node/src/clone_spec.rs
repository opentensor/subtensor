use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use serde_json::{Value, json};
use sp_runtime::codec::Encode;

use crate::cli::{CloneHistoryBackfill, CloneStateCmd};

type CloneResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const RPC_POLL_INTERVAL: Duration = Duration::from_secs(2);
const GRANDPA_AUTHORITIES_WELL_KNOWN_KEY: &[u8] = b":grandpa_authorities";

#[derive(Clone, Copy)]
struct Validator {
    name: &'static str,
    sr25519_hex: &'static str,
    ed25519_hex: &'static str,
}

static VALIDATORS: &[Validator] = &[
    Validator {
        name: "alice",
        sr25519_hex: "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
        ed25519_hex: "88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee",
    },
    Validator {
        name: "bob",
        sr25519_hex: "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
        ed25519_hex: "d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae69",
    },
    Validator {
        name: "charlie",
        sr25519_hex: "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
        ed25519_hex: "439660b36c6c03afafca027b910b4fecf99801834c62a5e6006f27d978de234f",
    },
];

/// Execute `build-test-clone`: sync network state, export raw chainspec, apply clone patch.
pub fn run(cmd: &CloneStateCmd) -> sc_cli::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|err| sc_cli::Error::Application(Box::new(err)))?;

    runtime
        .block_on(async_run(cmd))
        .map_err(sc_cli::Error::Application)
}

async fn async_run(cmd: &CloneStateCmd) -> CloneResult<()> {
    let validators = selected_validators(cmd);
    let selected_names = validators
        .iter()
        .map(|v| v.name)
        .collect::<Vec<_>>()
        .join(",");

    fs::create_dir_all(&cmd.base_path)?;

    if let Some(parent) = cmd.output.parent() {
        fs::create_dir_all(parent)?;
    }

    let current_exe = std::env::current_exe()?;
    let database_arg = cmd.database.as_ref();
    let sync_arg = cmd.sync.as_ref();
    let skip_backfill = matches!(cmd.history_backfill, CloneHistoryBackfill::Skip);

    log::info!("build-test-clone: validators={selected_names}");

    let mut sync_args = vec![
        "--base-path".to_string(),
        cmd.base_path.display().to_string(),
        "--chain".to_string(),
        cmd.chain.clone(),
        "--sync".to_string(),
        sync_arg.to_string(),
        "--database".to_string(),
        database_arg.to_string(),
        "--rpc-port".to_string(),
        cmd.rpc_port.to_string(),
        "--port".to_string(),
        cmd.port.to_string(),
        "--rpc-methods".to_string(),
        "unsafe".to_string(),
        "--no-telemetry".to_string(),
        "--no-prometheus".to_string(),
        "--no-mdns".to_string(),
        "--name".to_string(),
        "build-test-clone-sync".to_string(),
    ];

    for bootnode in &cmd.bootnodes {
        sync_args.push("--bootnodes".to_string());
        sync_args.push(bootnode.clone());
    }

    if skip_backfill {
        sync_args.push("--skip-history-backfill".to_string());
    }

    log::info!("build-test-clone: starting sync node");

    let mut sync_child = Command::new(&current_exe)
        .args(&sync_args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let sync_wait_result = wait_for_sync_completion(&mut sync_child, cmd).await;
    let stop_result = stop_child_gracefully(&mut sync_child).await;

    sync_wait_result?;
    stop_result?;

    let raw_tmp = temp_raw_path()?;

    log::info!("build-test-clone: exporting raw state");

    export_raw_state(&current_exe, cmd, database_arg, &raw_tmp)?;

    log::info!("build-test-clone: applying clone patch");

    patch_raw_chainspec_file(&raw_tmp, &cmd.output, &validators)?;

    if let Err(err) = fs::remove_file(&raw_tmp) {
        log::warn!(
            "build-test-clone: warning: failed to remove temp file {}: {err}",
            raw_tmp.display()
        );
    }

    log::info!("build-test-clone: wrote {}", cmd.output.display());

    Ok(())
}

async fn wait_for_sync_completion(sync_child: &mut Child, cmd: &CloneStateCmd) -> CloneResult<()> {
    let timeout = Duration::from_secs(cmd.sync_timeout_sec);
    let start = Instant::now();
    let mut stable_ready_checks = 0u8;
    let rpc_url = format!("http://127.0.0.1:{}", cmd.rpc_port);
    let rpc_client = HttpClientBuilder::default()
        .request_timeout(Duration::from_secs(10))
        .build(rpc_url)?;

    log::info!(
        "build-test-clone: waiting for sync completion (timeout={}s)",
        cmd.sync_timeout_sec
    );

    while let None = sync_child
        .try_wait()
        .map_err(|err| std::io::Error::other(format!("Failed to poll sync node process: {err}")))?
    {
        if start.elapsed() > timeout {
            return Err(format!(
                "Timed out waiting for sync completion after {} seconds",
                cmd.sync_timeout_sec
            )
            .into());
        }

        match query_sync_status(&rpc_client).await {
            Ok(status) => {
                let is_ready = !status.is_syncing
                    && status.peers > 0
                    && status.current > 0
                    && status.highest > 0
                    && status.current.saturating_add(cmd.sync_lag_blocks) >= status.highest;

                if is_ready {
                    stable_ready_checks = stable_ready_checks.saturating_add(1);
                    if stable_ready_checks >= 3 {
                        log::info!("build-test-clone: sync target reached");
                        return Ok(());
                    }
                } else {
                    stable_ready_checks = 0;
                }
            }
            Err(_) => {
                // RPC may not be ready yet.
                stable_ready_checks = 0;
            }
        }

        tokio::time::sleep(RPC_POLL_INTERVAL).await;
    }

    let status = sync_child
        .try_wait()
        .map_err(|err| std::io::Error::other(format!("Failed to poll sync node process: {err}")))?
        .ok_or_else(|| std::io::Error::other("Sync node status became unavailable"))?;

    Err(format!("Sync node exited unexpectedly: {status}").into())
}

async fn stop_child_gracefully(child: &mut Child) -> CloneResult<()> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }

    Command::new("kill")
        .arg("-INT")
        .arg(child.id().to_string())
        .status()?;

    for _ in 0..30 {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    child.kill()?;

    child.wait()?;

    Ok(())
}

fn export_raw_state(
    current_exe: &Path,
    cmd: &CloneStateCmd,
    database_arg: &str,
    raw_tmp: &Path,
) -> CloneResult<()> {
    let stdout = File::create(raw_tmp)?;
    let status = Command::new(current_exe)
        .args([
            "export-state",
            "--chain",
            &cmd.chain,
            "--base-path",
            &cmd.base_path.display().to_string(),
            "--database",
            database_arg,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(format!("export-state failed with status {status}").into());
    }

    Ok(())
}

struct SyncStatus {
    current: u64,
    highest: u64,
    peers: u64,
    is_syncing: bool,
}

async fn query_sync_status(rpc_client: &HttpClient) -> CloneResult<SyncStatus> {
    let sync = rpc_call(rpc_client, "system_syncState").await?;
    let health = rpc_call(rpc_client, "system_health").await?;

    let current = parse_u64_field(&sync, "currentBlock")
        .ok_or_else(|| "system_syncState.currentBlock missing".to_string())?;
    let highest = parse_u64_field(&sync, "highestBlock")
        .ok_or_else(|| "system_syncState.highestBlock missing".to_string())?;
    let peers = parse_u64_field(&health, "peers")
        .ok_or_else(|| "system_health.peers missing".to_string())?;
    let is_syncing = health
        .get("isSyncing")
        .and_then(Value::as_bool)
        .ok_or_else(|| "system_health.isSyncing missing".to_string())?;

    Ok(SyncStatus {
        current,
        highest,
        peers,
        is_syncing,
    })
}

async fn rpc_call(rpc_client: &HttpClient, method: &str) -> CloneResult<Value> {
    rpc_client
        .request(method, rpc_params![])
        .await
        .map_err(Into::into)
}

fn parse_u64_field(value: &Value, field: &str) -> Option<u64> {
    let field_value = value.get(field)?;

    if let Some(n) = field_value.as_u64() {
        return Some(n);
    }

    let s = field_value.as_str()?;

    s.parse::<u64>()
        .ok()
        .or_else(|| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
}

fn temp_raw_path() -> CloneResult<PathBuf> {
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(std::env::temp_dir().join(format!("subtensor-clone-export-{epoch}.json")))
}

fn selected_validators(cmd: &CloneStateCmd) -> Vec<Validator> {
    let explicit = cmd.alice || cmd.bob || cmd.charlie;
    let mut selected = Vec::new();

    if explicit {
        if cmd.alice {
            selected.push(VALIDATORS[0]);
        }
        if cmd.bob {
            selected.push(VALIDATORS[1]);
        }
        if cmd.charlie {
            selected.push(VALIDATORS[2]);
        }
    } else {
        selected.push(VALIDATORS[0]); // only alice be default
    }

    selected
}

fn patch_raw_chainspec_file(
    input: &Path,
    output: &Path,
    validators: &[Validator],
) -> CloneResult<()> {
    let file = File::open(input)?;
    let reader = BufReader::with_capacity(64 * 1024 * 1024, file);
    let mut spec: Value = serde_json::from_reader(reader)?;
    patch_raw_spec(&mut spec, validators)?;

    let out = File::create(output)?;
    let writer = BufWriter::with_capacity(64 * 1024 * 1024, out);
    serde_json::to_writer(writer, &spec)?;
    Ok(())
}

fn patch_raw_spec(spec: &mut Value, validators: &[Validator]) -> CloneResult<()> {
    let top = spec
        .pointer_mut("/genesis/raw/top")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "missing or invalid genesis.raw.top".to_string())?;

    let aura_keys: Vec<[u8; 32]> = validators
        .iter()
        .map(|v| decode_hex_32(v.sr25519_hex))
        .collect::<CloneResult<_>>()?;
    top.insert(
        storage_key("Aura", "Authorities"),
        Value::String(to_hex(&aura_keys.encode())),
    );

    let grandpa_entries: Vec<([u8; 32], u64)> = validators
        .iter()
        .map(|v| Ok((decode_hex_32(v.ed25519_hex)?, 1u64)))
        .collect::<CloneResult<_>>()?;
    let grandpa_encoded = grandpa_entries.encode();

    top.insert(
        storage_key("Grandpa", "Authorities"),
        Value::String(to_hex(&grandpa_encoded)),
    );

    let mut well_known = vec![0x01u8];
    well_known.extend_from_slice(&grandpa_encoded);
    top.insert(
        to_hex(GRANDPA_AUTHORITIES_WELL_KNOWN_KEY),
        Value::String(to_hex(&well_known)),
    );

    top.insert(
        storage_key("Grandpa", "CurrentSetId"),
        Value::String(to_hex(&0u64.to_le_bytes())),
    );
    top.insert(
        storage_key("Grandpa", "State"),
        Value::String("0x00".into()),
    );
    top.remove(&storage_key("Grandpa", "PendingChange"));
    top.remove(&storage_key("Grandpa", "NextForced"));
    top.remove(&storage_key("Grandpa", "Stalled"));
    remove_by_prefix(top, &storage_key("Grandpa", "SetIdSession"));

    top.insert(
        storage_key("Sudo", "Key"),
        Value::String(to_hex(&hex::decode(validators[0].sr25519_hex)?)),
    );

    remove_by_prefix(top, &storage_prefix("Session"));
    clear_top_level(spec);
    Ok(())
}

fn remove_by_prefix(map: &mut serde_json::Map<String, Value>, prefix: &str) {
    let mut keys_to_remove = VecDeque::new();
    for key in map.keys() {
        if key.starts_with(prefix) {
            keys_to_remove.push_back(key.clone());
        }
    }
    while let Some(key) = keys_to_remove.pop_front() {
        map.remove(&key);
    }
}

fn clear_top_level(spec: &mut Value) {
    if let Some(object) = spec.as_object_mut() {
        object.insert("bootNodes".into(), json!([]));
        object.insert("codeSubstitutes".into(), json!({}));
        object.insert("chainType".into(), json!("Local"));
    }
}

fn storage_key(pallet: &str, item: &str) -> String {
    let key = frame_support::storage::storage_prefix(pallet.as_bytes(), item.as_bytes());
    to_hex(&key)
}

fn storage_prefix(pallet: &str) -> String {
    format!(
        "0x{}",
        hex::encode(sp_io::hashing::twox_128(pallet.as_bytes()))
    )
}

fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

fn decode_hex_32(value: &str) -> CloneResult<[u8; 32]> {
    let bytes = hex::decode(value)?;
    let len = bytes.len();
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("expected 32-byte hex value, got {len} bytes"),
        )
    })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CloneDatabase, CloneHistoryBackfill, CloneSyncMode};

    fn target_artifact_path(name: &str) -> PathBuf {
        let target_dir = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target"));
        target_dir.join("clone-spec-tests").join(name)
    }

    fn default_cmd() -> CloneStateCmd {
        CloneStateCmd {
            chain: "finney".to_string(),
            base_path: target_artifact_path("base"),
            output: target_artifact_path("out.json"),
            sync: CloneSyncMode::Warp,
            database: CloneDatabase::ParityDb,
            history_backfill: CloneHistoryBackfill::Skip,
            rpc_port: 9966,
            port: 30466,
            sync_timeout_sec: 10,
            sync_lag_blocks: 8,
            bootnodes: Vec::new(),
            alice: false,
            bob: false,
            charlie: false,
        }
    }

    fn make_minimal_spec() -> Value {
        let mut top = serde_json::Map::new();
        top.insert(storage_key("Grandpa", "PendingChange"), json!("0x01"));
        top.insert(storage_key("Grandpa", "NextForced"), json!("0x02"));
        top.insert(storage_key("Grandpa", "Stalled"), json!("0x03"));
        top.insert(
            format!("{}{}", storage_key("Grandpa", "SetIdSession"), "deadbeef"),
            json!("0x04"),
        );
        top.insert(format!("{}abcd", storage_prefix("Session")), json!("0x05"));
        top.insert(storage_key("Balances", "TotalIssuance"), json!("0x06"));

        json!({
            "genesis": { "raw": { "top": top } },
            "bootNodes": ["/dns4/example.com/tcp/30333/p2p/12D3KooW..."],
            "codeSubstitutes": { "0x01": "0x02" },
            "chainType": "Live"
        })
    }

    #[test]
    fn selected_validators_defaults_to_alice() {
        let cmd = default_cmd();
        let selected = selected_validators(&cmd);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "alice");
    }

    #[test]
    fn selected_validators_respects_explicit_flags() {
        let mut cmd = default_cmd();
        cmd.bob = true;
        cmd.charlie = true;

        let selected = selected_validators(&cmd);
        let names = selected.into_iter().map(|v| v.name).collect::<Vec<_>>();
        assert_eq!(names, vec!["bob", "charlie"]);
    }

    #[test]
    fn parse_u64_field_supports_u64_decimal_and_hex_string() {
        let value = json!({
            "a": 42,
            "b": "123",
            "c": "0x2a"
        });

        assert_eq!(parse_u64_field(&value, "a"), Some(42));
        assert_eq!(parse_u64_field(&value, "b"), Some(123));
        assert_eq!(parse_u64_field(&value, "c"), Some(42));
        assert_eq!(parse_u64_field(&value, "missing"), None);
    }

    #[test]
    fn patch_raw_spec_updates_authorities_sudo_and_top_level() {
        let mut spec = make_minimal_spec();
        let validators = vec![VALIDATORS[0], VALIDATORS[1]];
        patch_raw_spec(&mut spec, &validators).expect("patch should succeed");

        let top = spec
            .pointer("/genesis/raw/top")
            .and_then(Value::as_object)
            .expect("top should be object");

        let aura_hex = top
            .get(&storage_key("Aura", "Authorities"))
            .and_then(Value::as_str)
            .expect("aura authorities key should exist");
        let aura_raw = hex::decode(aura_hex.trim_start_matches("0x")).expect("hex decode aura");
        let expected_aura = vec![
            decode_hex_32(VALIDATORS[0].sr25519_hex).expect("decode"),
            decode_hex_32(VALIDATORS[1].sr25519_hex).expect("decode"),
        ]
        .encode();
        assert_eq!(aura_raw, expected_aura);

        let sudo_hex = top
            .get(&storage_key("Sudo", "Key"))
            .and_then(Value::as_str)
            .expect("sudo key should exist");
        assert_eq!(
            sudo_hex,
            to_hex(&hex::decode(VALIDATORS[0].sr25519_hex).expect("decode")).as_str()
        );

        assert!(!top.contains_key(&storage_key("Grandpa", "PendingChange")));
        assert!(!top.contains_key(&storage_key("Grandpa", "NextForced")));
        assert!(!top.contains_key(&storage_key("Grandpa", "Stalled")));
        assert!(
            top.keys()
                .all(|k| !k.starts_with(&storage_prefix("Session")))
        );

        assert_eq!(spec.get("chainType"), Some(&json!("Local")));
        assert_eq!(spec.get("bootNodes"), Some(&json!([])));
        assert_eq!(spec.get("codeSubstitutes"), Some(&json!({})));
    }

    #[test]
    fn patch_raw_spec_fails_when_top_missing() {
        let mut spec = json!({});
        let err = patch_raw_spec(&mut spec, &[VALIDATORS[0]]).expect_err("must fail");
        assert!(
            err.to_string()
                .contains("missing or invalid genesis.raw.top"),
            "unexpected error: {err}"
        );
    }
}
