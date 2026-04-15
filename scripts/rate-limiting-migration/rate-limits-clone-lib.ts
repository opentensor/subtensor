import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { constants as fsConstants } from "node:fs";
import { access, mkdir, readFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Binary, Enum, createClient, type PolkadotClient, type TypedApi } from "polkadot-api";
import { getWsProvider } from "polkadot-api/ws-provider/node";
import { getPolkadotSigner } from "polkadot-api/signer";
import { subtensor } from "@polkadot-api/descriptors";
import { Keyring } from "@polkadot/keyring";
import { tao } from "../../ts-tests/utils/balance.ts";
import { generateKeyringPair } from "../../ts-tests/utils/account.ts";
import { addStake, sudoSetAdminFreezeWindow, sudoSetTempo } from "../../ts-tests/utils/staking.ts";
import { waitForBlocks } from "../../ts-tests/utils/staking.ts";
import { waitForFinalizedBlocks, waitForTransactionWithRetry } from "../../ts-tests/utils/transactions.ts";
import {
  expectTransactionFailure,
  forceSetBalancesForRateLimit,
  getCallRateLimit,
  getGroupedResponseGroupId,
  getRateLimitConfig,
  getStakeValueForRateLimit,
  isGlobalConfig,
  isScopedConfig,
  rateLimitKindExact,
  rateLimitTargetGroup,
  submitTransactionBestEffort,
  waitForRateLimitTransactionWithRetry,
} from "../../ts-tests/utils/rate-limiting.ts";
import { burnedRegister } from "../../ts-tests/utils/subnet.ts";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const LOCAL_ROOT = resolve(SCRIPT_DIR, "..");
export const REPO_ROOT = resolve(LOCAL_ROOT, "..");
export const BASE_DIR = process.env.CLONE_BASE_DIR || resolve(REPO_ROOT, "target/rate-limits-test");
export const RUN_DIR = `${BASE_DIR}/run`;
export const RUN_BASE_PATH = process.env.CLONE_RUN_BASE_PATH || `${RUN_DIR}/alice`;
export const SOURCE_CHAIN_SPEC =
  process.env.CLONE_CHAIN_SPEC || resolve(REPO_ROOT, "chainspecs/raw_spec_finney.json");
const SOURCE_CHAIN_SPEC_NAME = basename(SOURCE_CHAIN_SPEC, ".json").replace(/^raw_spec_/, "");
export const PATCHED_CHAIN_SPEC =
  process.env.PATCHED_CHAIN_SPEC || `${BASE_DIR}/patched-${SOURCE_CHAIN_SPEC_NAME}.json`;
export const SOURCE_BASE_PATH = process.env.CLONE_SOURCE_BASE_PATH || `${BASE_DIR}/source`;
export const BINARY_PATH = process.env.BINARY_PATH || resolve(REPO_ROOT, "target/release/node-subtensor");
export const RUNTIME_WASM =
  process.env.RUNTIME_WASM ||
  resolve(REPO_ROOT, "target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm");
export const SYNC_MODE = process.env.CLONE_SYNC_MODE || "warp";
export const SYNC_TIMEOUT_SEC = process.env.CLONE_SYNC_TIMEOUT_SEC || "7200";
export const PREP_RPC_PORT = Number(process.env.CLONE_PREP_RPC_PORT || 9976);
export const PREP_P2P_PORT = Number(process.env.CLONE_PREP_P2P_PORT || 30476);
export const LOCAL_RPC_PORT = Number(process.env.CLONE_NODE_RPC_PORT || 9964);

const GROUP_SERVE = 0;
const GROUP_DELEGATE_TAKE = 1;
const GROUP_WEIGHTS_SET = 2;
const GROUP_REGISTER_NETWORK = 3;
const GROUP_OWNER_HPARAMS = 4;
const GROUP_STAKING_OPS = 5;
const GROUP_SWAP_KEYS = 6;
const SET_CODE_WEIGHT = {
  ref_time: 164_247_810_000n,
  proof_size: 67_035n,
} as const;

export function log(message: string) {
  console.log(message);
}

type BehaviorPhase = "serving" | "staking" | "delegate-take" | "weights" | "swap-keys" | "owner-hparams";

function shouldRunPhase(filter: BehaviorPhase | undefined, phase: BehaviorPhase): boolean {
  return filter === undefined || filter === phase;
}

export function installConnectionLogFilter() {
  const shouldSuppress = (args: unknown[]) => {
    const first = args[0];
    return typeof first === "string" && first.includes("Unable to connect to ws://127.0.0.1:");
  };

  const originalConsoleError = console.error;
  const originalConsoleWarn = console.warn;
  const originalConsoleLog = console.log;

  console.error = (...args: unknown[]) => {
    if (!shouldSuppress(args)) originalConsoleError(...args);
  };
  console.warn = (...args: unknown[]) => {
    if (!shouldSuppress(args)) originalConsoleWarn(...args);
  };
  console.log = (...args: unknown[]) => {
    if (!shouldSuppress(args)) originalConsoleLog(...args);
  };
}

async function runCommand(command: string, args: string[]) {
  await new Promise<void>((resolvePromise, reject) => {
    const child = spawn(command, args, {
      cwd: REPO_ROOT,
      stdio: "inherit",
      env: process.env,
    });

    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`${command} exited with code ${code}`));
    });
  });
}

export async function prepareCloneSpec() {
  await mkdir(BASE_DIR, { recursive: true });

  log("Preparing patched mainnet clone spec");
  await runCommand(BINARY_PATH, [
    "build-patched-spec",
    "--chain",
    SOURCE_CHAIN_SPEC,
    "--base-path",
    SOURCE_BASE_PATH,
    "--output",
    PATCHED_CHAIN_SPEC,
    "--sync",
    SYNC_MODE,
    "--sync-timeout-sec",
    SYNC_TIMEOUT_SEC,
    "--rpc-port",
    String(PREP_RPC_PORT),
    "--port",
    String(PREP_P2P_PORT),
  ]);
}

export async function waitForLocalRpc(timeoutMs = 1_200_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  const url = `ws://127.0.0.1:${LOCAL_RPC_PORT}`;
  let lastProgressLog = 0;

  while (Date.now() < deadline) {
    let client: PolkadotClient | undefined;
    try {
      client = createClient(getWsProvider(url));
      const api = client.getTypedApi(subtensor);
      await api.query.System.Number.getValue();
      try {
        client.destroy();
      } catch {}
      return;
    } catch {
      try {
        client?.destroy();
      } catch {}
      const now = Date.now();
      if (now - lastProgressLog >= 30_000) {
        lastProgressLog = now;
        log(`Waiting for alice RPC at ${url} (genesis import may take several minutes)`);
      }
      await new Promise((resolve) => setTimeout(resolve, 1_000));
    }
  }

  throw new Error(`alice RPC at ${url} did not become ready in time`);
}

export async function waitForLocalFinalization(blocks = 3) {
  const client = await connectLocalClient();
  try {
    const api = client.getTypedApi(subtensor);
    await waitForFinalizedBlocks(api, blocks);
  } finally {
    try {
      client.destroy();
    } catch {}
  }
}

export async function connectLocalClient(): Promise<PolkadotClient> {
  return createClient(getWsProvider(`ws://127.0.0.1:${LOCAL_RPC_PORT}`));
}

export async function waitForRateLimitingRuntimeApi(client: PolkadotClient, timeoutMs = 120_000) {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    try {
      await getCallRateLimit(client as any, "SubtensorModule", "serve_axon");
      return;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (!message.includes("RateLimitingRuntimeApi_get_rate_limit is not found")) {
        throw error;
      }
      await new Promise((resolve) => setTimeout(resolve, 1_000));
    }
  }

  throw new Error("RateLimiting runtime API did not become available in time");
}

export async function upgradeCloneRuntime() {
  const client = await connectLocalClient();
  try {
    const api = client.getTypedApi(subtensor);
    log(`Upgrading clone runtime from ${RUNTIME_WASM}`);

    const runtimeWasm = new Uint8Array(await readFile(RUNTIME_WASM));
    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");

    const setCode = api.tx.System.set_code({
      code: Binary.fromBytes(runtimeWasm),
    });
    const tx = api.tx.Sudo.sudo_unchecked_weight({
      call: setCode.decodedCall,
      weight: SET_CODE_WEIGHT,
    });

    const signer = getPolkadotSigner(alice.publicKey, "Sr25519", alice.sign);
    const account = await api.query.System.Account.getValue(alice.address, { at: "best" });
    const signedHex = await tx.sign(signer, {
      at: "best",
      nonce: account.nonce,
    });
    const txHash = await (client as any)._request("author_submitExtrinsic", [signedHex]);
    log(`Submitted clone runtime upgrade tx ${String(txHash)}`);
    const startHeader = (await (client as any)._request("chain_getHeader", [])) as { number?: string };
    const startNumber = Number.parseInt(startHeader?.number ?? "0x0", 16);
    const deadline = Date.now() + 60_000;

    while (Date.now() < deadline) {
      await new Promise((resolve) => setTimeout(resolve, 1_000));
      try {
        const header = (await (client as any)._request("chain_getHeader", [])) as { number?: string };
        const currentNumber = Number.parseInt(header?.number ?? "0x0", 16);
        if (currentNumber >= startNumber + 2) {
          log("Upgrade transaction submitted; restart the node on the same db now");
          return;
        }
      } catch {
        log("Upgrade submitted and RPC disconnected; restart the node on the same db now");
        return;
      }
    }

    throw new Error("upgrade tx was submitted, but chain did not advance within 60s");
  } finally {
    try {
      client.destroy();
    } catch {}
  }
}

async function expectGroupedCall(client: PolkadotClient, pallet: string, extrinsic: string, groupId: number) {
  const response = await getCallRateLimit(client as any, pallet, extrinsic);
  assert.ok(response, `${pallet}.${extrinsic} returned no rate-limit response`);
  assert.equal(
    getGroupedResponseGroupId(response),
    groupId,
    `${pallet}.${extrinsic} group mismatch: ${JSON.stringify(response)}`,
  );
  return response;
}

function assertScopeKind(response: unknown, scopeKind: "global" | "scoped", label: string) {
  const config = getRateLimitConfig(response as any);
  if (scopeKind === "global") {
    assert.equal(isGlobalConfig(config), true, `${label} expected global config`);
  } else {
    assert.equal(isScopedConfig(config), true, `${label} expected scoped config`);
  }
}

function enumVariantName(value: any): string | undefined {
  if (value && typeof value === "object") {
    if (typeof value.type === "string") return value.type;
    const [key] = Object.entries(value)[0] ?? [];
    if (typeof key === "string") return key;
  }
  return undefined;
}

function enumVariantValue(value: any): any {
  if (value && typeof value === "object") {
    if ("value" in value) return value.value;
    const [, inner] = Object.entries(value)[0] ?? [];
    return inner;
  }
  return undefined;
}

function migrationFlagKey(name: string) {
  return Binary.fromBytes(new TextEncoder().encode(name));
}

function txTarget(pallet_index: number, extrinsic_index: number) {
  return Enum("Transaction", { pallet_index, extrinsic_index });
}

const OWNER_HPARAM_IDENTIFIERS = new Map<string, { pallet_index: number; extrinsic_index: number }>([
  ["ServingRateLimit", { pallet_index: 19, extrinsic_index: 3 }],
  ["MaxDifficulty", { pallet_index: 19, extrinsic_index: 5 }],
  ["AdjustmentAlpha", { pallet_index: 19, extrinsic_index: 9 }],
  ["ImmunityPeriod", { pallet_index: 19, extrinsic_index: 13 }],
  ["MinAllowedWeights", { pallet_index: 19, extrinsic_index: 14 }],
  ["MaxAllowedUids", { pallet_index: 19, extrinsic_index: 15 }],
  ["Rho", { pallet_index: 19, extrinsic_index: 17 }],
  ["ActivityCutoff", { pallet_index: 19, extrinsic_index: 18 }],
  ["PowRegistrationAllowed", { pallet_index: 19, extrinsic_index: 20 }],
  ["MinBurn", { pallet_index: 19, extrinsic_index: 22 }],
  ["MaxBurn", { pallet_index: 19, extrinsic_index: 23 }],
  ["BondsMovingAverage", { pallet_index: 19, extrinsic_index: 26 }],
  ["BondsPenalty", { pallet_index: 19, extrinsic_index: 60 }],
  ["CommitRevealEnabled", { pallet_index: 19, extrinsic_index: 49 }],
  ["LiquidAlphaEnabled", { pallet_index: 19, extrinsic_index: 50 }],
  ["AlphaValues", { pallet_index: 19, extrinsic_index: 51 }],
  ["WeightCommitInterval", { pallet_index: 19, extrinsic_index: 57 }],
  ["TransferEnabled", { pallet_index: 19, extrinsic_index: 61 }],
  ["AlphaSigmoidSteepness", { pallet_index: 19, extrinsic_index: 68 }],
  ["Yuma3Enabled", { pallet_index: 19, extrinsic_index: 69 }],
  ["BondsResetEnabled", { pallet_index: 19, extrinsic_index: 70 }],
  ["ImmuneNeuronLimit", { pallet_index: 19, extrinsic_index: 72 }],
  ["RecycleOrBurn", { pallet_index: 19, extrinsic_index: 80 }],
  ["BurnHalfLife", { pallet_index: 19, extrinsic_index: 89 }],
  ["BurnIncreaseMult", { pallet_index: 19, extrinsic_index: 90 }],
]);

function getResponseKind(response: any): "grouped" | "standalone" | undefined {
  if (response && typeof response === "object") {
    if ("group_id" in response) return "grouped";
    if (response.type === "grouped" || response.type === "Grouped" || "Grouped" in response) return "grouped";
    if (
      response.type === "standalone" ||
      response.type === "Standalone" ||
      "Standalone" in response
    ) {
      return "standalone";
    }
    const [key] = Object.entries(response)[0] ?? [];
    if (typeof key === "string") {
      if (key.toLowerCase() === "grouped") return "grouped";
      if (key.toLowerCase() === "standalone") return "standalone";
    }
  }
  return undefined;
}

function assertResponseKind(response: unknown, kind: "grouped" | "standalone", label: string) {
  assert.equal(getResponseKind(response as any), kind, `${label} expected ${kind} rate-limit response`);
}

export async function verifyCloneConfig() {
  const client = await connectLocalClient();
  try {
    log("Validating migrated clone configuration");
    await waitForRateLimitingRuntimeApi(client);

    const expectedGroupedCalls: Array<{
      pallet: string;
      extrinsic: string;
      groupId: number;
      scopeKind: "global" | "scoped";
    }> = [
      { pallet: "SubtensorModule", extrinsic: "serve_axon", groupId: GROUP_SERVE, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "serve_axon_tls", groupId: GROUP_SERVE, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "serve_prometheus", groupId: GROUP_SERVE, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "increase_take", groupId: GROUP_DELEGATE_TAKE, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "decrease_take", groupId: GROUP_DELEGATE_TAKE, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "set_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "batch_set_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "commit_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "batch_commit_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "commit_timelocked_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "reveal_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "batch_reveal_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "set_mechanism_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "commit_mechanism_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "commit_crv3_mechanism_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "commit_timelocked_mechanism_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "reveal_mechanism_weights", groupId: GROUP_WEIGHTS_SET, scopeKind: "scoped" },
      { pallet: "SubtensorModule", extrinsic: "register_network", groupId: GROUP_REGISTER_NETWORK, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "register_network_with_identity", groupId: GROUP_REGISTER_NETWORK, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_serving_rate_limit", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_max_difficulty", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_adjustment_alpha", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_immunity_period", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_min_allowed_weights", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_max_allowed_uids", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_activity_cutoff", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_rho", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_network_pow_registration_allowed", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_min_burn", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_max_burn", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_bonds_moving_average", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_bonds_penalty", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_commit_reveal_weights_enabled", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_liquid_alpha_enabled", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_alpha_values", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_commit_reveal_weights_interval", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_toggle_transfer", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_alpha_sigmoid_steepness", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_yuma3_enabled", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_bonds_reset_enabled", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_owner_immune_neuron_limit", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_recycle_or_burn", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_burn_half_life", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "AdminUtils", extrinsic: "sudo_set_burn_increase_mult", groupId: GROUP_OWNER_HPARAMS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "add_stake", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "add_stake_limit", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "remove_stake", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "remove_stake_limit", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "remove_stake_full_limit", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "move_stake", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "transfer_stake", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "swap_stake", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "swap_stake_limit", groupId: GROUP_STAKING_OPS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "swap_hotkey", groupId: GROUP_SWAP_KEYS, scopeKind: "global" },
      { pallet: "SubtensorModule", extrinsic: "swap_coldkey", groupId: GROUP_SWAP_KEYS, scopeKind: "global" },
    ];

    for (const { pallet, extrinsic, groupId, scopeKind } of expectedGroupedCalls) {
      const response = await expectGroupedCall(client, pallet, extrinsic, groupId);
      assertResponseKind(response, "grouped", `${pallet}.${extrinsic}`);
      assertScopeKind(response, scopeKind, `${pallet}.${extrinsic}`);
    }
  } finally {
    try {
      client.destroy();
    } catch {}
  }
}

export async function verifyCloneStorageAudit() {
  const client = await connectLocalClient();
  try {
    const api = client.getTypedApi(subtensor);
    log("Auditing migrated clone storage");

    const groupedMarker = await api.query.SubtensorModule.HasMigrationRun.getValue(
      migrationFlagKey("migrate_grouped_rate_limiting"),
    );
    const standaloneMarker = await api.query.SubtensorModule.HasMigrationRun.getValue(
      migrationFlagKey("migrate_standalone_rate_limiting"),
    );
    log(`Migration markers: grouped=${groupedMarker} standalone=${standaloneMarker}`);

    try {
      await api.query.RateLimiting.LastSeen.getValue(
        rateLimitTargetGroup(GROUP_SERVE) as never,
        undefined,
        { at: "best" },
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (message.includes("Storage(RateLimiting.LastSeen) not found")) {
        throw new Error(
          "upgraded rate-limiting runtime is not active on this node; run upgrade-rate-limits-clone.sh, stop the old node, and start it again before running storage audit",
        );
      }
      throw error;
    }

    let checkedServing = 0;
    for (const { keyArgs, value } of await api.query.SubtensorModule.Axons.getEntries({ at: "best" })) {
      if (!value) continue;
      const block = Number((value as any).block ?? 0);
      if (block === 0) continue;
      const [netuid, account] = keyArgs as [number, string];
      const migrated = await api.query.RateLimiting.LastSeen.getValue(
        rateLimitTargetGroup(GROUP_SERVE) as never,
        Enum("AccountSubnetServing", { account, netuid, endpoint: Enum("Axon") }) as never,
        { at: "best" },
      );
      assert.equal(Number(migrated), block, `serving axon last_seen mismatch for ${account} on netuid ${netuid}`);
      checkedServing += 1;
    }
    for (const { keyArgs, value } of await api.query.SubtensorModule.Prometheus.getEntries({ at: "best" })) {
      if (!value) continue;
      const block = Number((value as any).block ?? 0);
      if (block === 0) continue;
      const [netuid, account] = keyArgs as [number, string];
      const migrated = await api.query.RateLimiting.LastSeen.getValue(
        rateLimitTargetGroup(GROUP_SERVE) as never,
        Enum("AccountSubnetServing", { account, netuid, endpoint: Enum("Prometheus") }) as never,
        { at: "best" },
      );
      assert.equal(
        Number(migrated),
        block,
        `serving prometheus last_seen mismatch for ${account} on netuid ${netuid}`,
      );
      checkedServing += 1;
    }

    let checkedWeights = 0;
    for (const { keyArgs, value } of await api.query.SubtensorModule.LastUpdate.getEntries({ at: "best" })) {
      const [netuidIndex] = keyArgs as [number];
      const netuid = netuidIndex % 4096;
      const mecid = Math.floor(netuidIndex / 4096);
      const blocks = value as Array<bigint | number>;
      for (let uid = 0; uid < blocks.length; uid += 1) {
        const block = Number(blocks[uid] ?? 0);
        if (block === 0) continue;
        const migrated = await api.query.RateLimiting.LastSeen.getValue(
          rateLimitTargetGroup(GROUP_WEIGHTS_SET) as never,
          Enum("SubnetMechanismNeuron", { netuid, mecid, uid }) as never,
          { at: "best" },
        );
        assert.equal(
          Number(migrated),
          block,
          `weights last_seen mismatch for netuid ${netuid}, mecid ${mecid}, uid ${uid}`,
        );
        checkedWeights += 1;
      }
    }

    let checkedOwner = 0;
    let checkedRegister = 0;
    let checkedDelegateTake = 0;
    let checkedSwapKeys = 0;
    for (const { keyArgs, value } of await api.query.SubtensorModule.LastRateLimitedBlock.getEntries({ at: "best" })) {
      const [rateLimitKey] = keyArgs as [any];
      const block = Number(value);
      if (block === 0) continue;
      const variant = enumVariantName(rateLimitKey);
      const payload = enumVariantValue(rateLimitKey);

      if (variant === "OwnerHyperparamUpdate") {
        const [netuid, hyper] = payload as [number, any];
        const hyperName = enumVariantName(hyper);
        const identifier = hyperName ? OWNER_HPARAM_IDENTIFIERS.get(hyperName) : undefined;
        if (!identifier) continue;
        const migrated = await api.query.RateLimiting.LastSeen.getValue(
          txTarget(identifier.pallet_index, identifier.extrinsic_index) as never,
          Enum("Subnet", netuid) as never,
          { at: "best" },
        );
        assert.equal(
          Number(migrated),
          block,
          `owner-hparam last_seen mismatch for ${hyperName} on netuid ${netuid}`,
        );
        checkedOwner += 1;
        continue;
      }

      if (variant === "NetworkLastRegistered") {
        const migrated = await api.query.RateLimiting.LastSeen.getValue(
          rateLimitTargetGroup(GROUP_REGISTER_NETWORK) as never,
          undefined,
          { at: "best" },
        );
        assert.equal(Number(migrated), block, "register_network last_seen mismatch");
        checkedRegister += 1;
        continue;
      }

      if (variant === "LastTxBlockDelegateTake") {
        const account = payload as string;
        const migrated = await api.query.RateLimiting.LastSeen.getValue(
          rateLimitTargetGroup(GROUP_DELEGATE_TAKE) as never,
          Enum("Account", account) as never,
          { at: "best" },
        );
        assert.equal(Number(migrated), block, `delegate_take last_seen mismatch for ${account}`);
        checkedDelegateTake += 1;
        continue;
      }

      if (variant === "LastTxBlock") {
        const account = payload as string;
        const migrated = await api.query.RateLimiting.LastSeen.getValue(
          rateLimitTargetGroup(GROUP_SWAP_KEYS) as never,
          Enum("Account", account) as never,
          { at: "best" },
        );
        assert.equal(Number(migrated), block, `swap_keys last_seen mismatch for ${account}`);
        checkedSwapKeys += 1;
      }
    }

    log(
      `Storage audit checked serving=${checkedServing}, weights=${checkedWeights}, owner=${checkedOwner}, register=${checkedRegister}, delegate_take=${checkedDelegateTake}, swap_keys=${checkedSwapKeys}`,
    );
  } finally {
    try {
      client.destroy();
    } catch {}
  }
}

function getAlice() {
  const keyring = new Keyring({ type: "sr25519" });
  return keyring.addFromUri("//Alice");
}

function asSudoTx(api: TypedApi<typeof subtensor>, inner: { decodedCall: unknown }) {
  return api.tx.Sudo.sudo({ call: inner.decodedCall as any });
}

async function sudoSetNetworkRegistrationAllowed(
  api: TypedApi<typeof subtensor>,
  netuid: number,
  registrationAllowed: boolean,
) {
  const alice = getAlice();
  const inner = api.tx.AdminUtils.sudo_set_network_registration_allowed({
    netuid,
    registration_allowed: registrationAllowed,
  });
  await waitForTransactionWithRetry(
    api,
    asSudoTx(api, inner) as any,
    alice as any,
    `sudo_set_network_registration_allowed_${netuid}`,
  );
}

async function sudoSetCommitRevealWeightsEnabled(
  api: TypedApi<typeof subtensor>,
  netuid: number,
  enabled: boolean,
) {
  const alice = getAlice();
  const inner = api.tx.AdminUtils.sudo_set_commit_reveal_weights_enabled({
    netuid,
    enabled,
  });
  await waitForTransactionWithRetry(
    api,
    asSudoTx(api, inner) as any,
    alice as any,
    `sudo_set_commit_reveal_weights_enabled_${netuid}`,
  );
}

async function sudoSetStakeThreshold(api: TypedApi<typeof subtensor>, minStake: number | bigint) {
  const alice = getAlice();
  const inner = api.tx.AdminUtils.sudo_set_stake_threshold({
    min_stake: typeof minStake === "bigint" ? minStake : BigInt(minStake),
  });
  await waitForTransactionWithRetry(
    api,
    asSudoTx(api, inner) as any,
    alice as any,
    "sudo_set_stake_threshold",
  );
}

async function sudoSetTargetRegistrationsPerInterval(
  api: TypedApi<typeof subtensor>,
  netuid: number,
  targetRegistrations: number,
) {
  const alice = getAlice();
  const inner = api.tx.AdminUtils.sudo_set_target_registrations_per_interval({
    netuid,
    target_registrations_per_interval: targetRegistrations,
  });
  await waitForTransactionWithRetry(
    api,
    asSudoTx(api, inner) as any,
    alice as any,
    `sudo_set_target_registrations_per_interval_${netuid}`,
  );
}

async function waitForSudoOk(
  api: TypedApi<typeof subtensor>,
  tx: any,
  signerPair: ReturnType<Keyring["addFromUri"]>,
  label: string,
) {
  const signer = getPolkadotSigner(signerPair.publicKey, "Sr25519", signerPair.sign);

  await new Promise<void>((resolve, reject) => {
    let settled = false;
    let timeoutId: ReturnType<typeof setTimeout>;

    const finish = (cb: () => void) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutId);
      subscription.unsubscribe();
      cb();
    };

    const subscription = tx.signSubmitAndWatch(signer, { at: "best" }).subscribe({
      next: async (event: any) => {
        if (event.type !== "txBestBlocksState" || !event.found) return;

        if (!event.ok || event.dispatchError) {
          finish(() => reject(new Error(`[${label}] dispatch error: ${JSON.stringify(event.dispatchError)}`)));
          return;
        }

        try {
          const events = await api.query.System.Events.getValue({ at: event.block.hash });
          const sudoEvent = events.find(
            (record: any) =>
              record.phase?.type === "ApplyExtrinsic" &&
              record.phase.value === event.block.index &&
              record.event?.type === "Sudo" &&
              record.event?.value?.type === "Sudid",
          ) as any;

          const sudoResult = sudoEvent?.event?.value?.value?.sudo_result;
          if (sudoResult?.success === false) {
            finish(() => reject(new Error(`[${label}] sudo error: ${JSON.stringify(sudoResult.value)}`)));
            return;
          }

          finish(resolve);
        } catch (error) {
          finish(() => reject(error instanceof Error ? error : new Error(String(error))));
        }
      },
      error: (error: unknown) => {
        finish(() => reject(error instanceof Error ? error : new Error(String(error))));
      },
    });

    timeoutId = setTimeout(() => {
      finish(() => reject(new Error(`[${label}] timed out waiting for sudo inclusion`)));
    }, 30_000);
  });
}

async function sudoSetScopedGroupRateLimit(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  scope: number,
  limit: number,
) {
  const alice = getAlice();
  const inner = api.tx.RateLimiting.set_rate_limit({
    target: rateLimitTargetGroup(groupId) as never,
    scope,
    limit: rateLimitKindExact(limit) as never,
  });
  await waitForSudoOk(api, asSudoTx(api, inner), alice, `set_scoped_group_rate_limit_${groupId}`);
  await waitForFinalizedBlocks(api, 1);
}

async function sudoSetGlobalGroupRateLimit(
  api: TypedApi<typeof subtensor>,
  groupId: number,
  limit: number,
) {
  const alice = getAlice();
  const inner = api.tx.RateLimiting.set_rate_limit({
    target: rateLimitTargetGroup(groupId) as never,
    scope: undefined,
    limit: rateLimitKindExact(limit) as never,
  });
  await waitForSudoOk(api, asSudoTx(api, inner), alice, `set_group_rate_limit_${groupId}`);
  await waitForFinalizedBlocks(api, 1);
}

async function getExistingSubnetNetuids(
  api: TypedApi<typeof subtensor>,
  count: number,
  requireFreeSlot = false,
): Promise<number[]> {
  const totalNetworks = Number(await api.query.SubtensorModule.TotalNetworks.getValue());
  const netuids: number[] = [];

  for (let netuid = 1; netuid < totalNetworks && netuids.length < count; netuid += 1) {
    const added = await api.query.SubtensorModule.NetworksAdded.getValue(netuid);
    if (!added) continue;
    if (requireFreeSlot) {
      const current = Number(await api.query.SubtensorModule.SubnetworkN.getValue(netuid));
      const max = Number(await api.query.SubtensorModule.MaxAllowedUids.getValue(netuid));
      if (current >= max) continue;
    }
    netuids.push(netuid);
  }

  if (netuids.length < count) {
    throw new Error(
      `Expected at least ${count} existing non-root subnets${requireFreeSlot ? " with free UID slots" : ""}, found ${netuids.length}`,
    );
  }

  return netuids;
}

export async function verifyCloneBehavior(filter?: BehaviorPhase) {
  const client = await connectLocalClient();
  try {
    const api = client.getTypedApi(subtensor);
    log("Validating clone behavior on fresh state");
    const [netuidServing, netuidStaking, netuidDelegate, netuidWeights, netuidSwap] =
      await getExistingSubnetNetuids(api, 5, true);
    const netuidOwnerA = netuidServing;
    const netuidOwnerB = netuidStaking;

    await sudoSetAdminFreezeWindow(api, 0);
    await sudoSetGlobalGroupRateLimit(api, GROUP_OWNER_HPARAMS, 0);
    await sudoSetStakeThreshold(api, 0);
    for (const netuid of [netuidServing, netuidStaking, netuidDelegate, netuidWeights, netuidSwap]) {
      await sudoSetNetworkRegistrationAllowed(api, netuid, true);
      await sudoSetTargetRegistrationsPerInterval(api, netuid, 256);
    }

    if (shouldRunPhase(filter, "serving")) {
      log("Behavior: serving");
      const coldkey = generateKeyringPair("sr25519");
      const hotkey = generateKeyringPair("sr25519");

      await forceSetBalancesForRateLimit(api, [coldkey.address, hotkey.address]);
      await burnedRegister(api, netuidServing, hotkey.address, coldkey);

      await sudoSetScopedGroupRateLimit(api, GROUP_SERVE, netuidServing, 2);

      const serveAxon = api.tx.SubtensorModule.serve_axon({
        netuid: netuidServing,
        version: 1,
        ip: 0n,
        port: 3030,
        ip_type: 4,
        protocol: 0,
        placeholder1: 0,
        placeholder2: 0,
      });
      const serveAxonTls = api.tx.SubtensorModule.serve_axon_tls({
        netuid: netuidServing,
        version: 1,
        ip: 0n,
        port: 3030,
        ip_type: 4,
        protocol: 0,
        placeholder1: 0,
        placeholder2: 0,
        certificate: Binary.fromBytes(new Uint8Array([7, 7, 7])),
      });
      const servePrometheus = api.tx.SubtensorModule.serve_prometheus({
        netuid: netuidServing,
        version: 1,
        ip: 1_676_056_785n,
        port: 3031,
        ip_type: 4,
      });

      await waitForRateLimitTransactionWithRetry(api, serveAxon, hotkey, "clone_serve_axon_initial");
      await expectTransactionFailure(api, serveAxonTls, hotkey, "clone_serve_axon_tls_rate_limited");
      await waitForRateLimitTransactionWithRetry(api, servePrometheus, hotkey, "clone_serve_prometheus_initial");
      await expectTransactionFailure(api, servePrometheus, hotkey, "clone_serve_prometheus_rate_limited");
      await waitForFinalizedBlocks(api, 2);
      await waitForRateLimitTransactionWithRetry(api, serveAxonTls, hotkey, "clone_serve_axon_tls_after_window");
      await waitForRateLimitTransactionWithRetry(
        api,
        servePrometheus,
        hotkey,
        "clone_serve_prometheus_after_window",
      );
    }

    if (shouldRunPhase(filter, "staking")) {
      log("Behavior: staking");
      const coldkey = generateKeyringPair("sr25519");
      const hotkey = generateKeyringPair("sr25519");

      await forceSetBalancesForRateLimit(api, [coldkey.address, hotkey.address]);
      await burnedRegister(api, netuidStaking, hotkey.address, coldkey);

      const addStake = api.tx.SubtensorModule.add_stake({
        hotkey: hotkey.address,
        netuid: netuidStaking,
        amount_staked: tao(100),
      });
      await waitForRateLimitTransactionWithRetry(api, addStake, coldkey, "clone_add_stake_initial");

      const alpha = await getStakeValueForRateLimit(api, hotkey.address, coldkey.address, netuidStaking);
      const removeStake = api.tx.SubtensorModule.remove_stake({
        hotkey: hotkey.address,
        netuid: netuidStaking,
        amount_unstaked: alpha,
      });
      await expectTransactionFailure(api, removeStake, coldkey, "clone_remove_stake_rate_limited");
    }

    if (shouldRunPhase(filter, "delegate-take")) {
      log("Behavior: delegate-take");
      const coldkey = generateKeyringPair("sr25519");
      const hotkey = generateKeyringPair("sr25519");

      await forceSetBalancesForRateLimit(api, [coldkey.address, hotkey.address]);
      await burnedRegister(api, netuidDelegate, hotkey.address, coldkey);
      await sudoSetGlobalGroupRateLimit(api, GROUP_DELEGATE_TAKE, 2);

      const currentTake = await api.query.SubtensorModule.Delegates.getValue(hotkey.address);
      assert.ok(currentTake > 0, "expected fresh delegate take to be above zero");
      const loweredTake = currentTake - 1;

      const decreaseTake = api.tx.SubtensorModule.decrease_take({
        hotkey: hotkey.address,
        take: loweredTake,
      });
      const increaseTake = api.tx.SubtensorModule.increase_take({
        hotkey: hotkey.address,
        take: currentTake,
      });

      await waitForRateLimitTransactionWithRetry(api, decreaseTake, coldkey, "clone_decrease_take_initial");
      await expectTransactionFailure(api, increaseTake, coldkey, "clone_increase_take_rate_limited");
      await waitForFinalizedBlocks(api, 2);
      await waitForRateLimitTransactionWithRetry(
        api,
        increaseTake,
        coldkey,
        "clone_increase_take_after_window",
      );
    }

    if (shouldRunPhase(filter, "weights")) {
      log("Behavior: weights");
      const coldkey = generateKeyringPair("sr25519");
      const hotkey = generateKeyringPair("sr25519");

      await forceSetBalancesForRateLimit(api, [coldkey.address, hotkey.address]);
      await burnedRegister(api, netuidWeights, hotkey.address, coldkey);
      await addStake(api, coldkey, hotkey.address, netuidWeights, tao(100));
      await sudoSetCommitRevealWeightsEnabled(api, netuidWeights, false);
      await sudoSetScopedGroupRateLimit(api, GROUP_WEIGHTS_SET, netuidWeights, 2);

      const uid = await api.query.SubtensorModule.Uids.getValue(netuidWeights, hotkey.address);
      assert.notEqual(uid, undefined, "expected registered uid for weights probe");
      const versionKey = await api.query.SubtensorModule.WeightsVersionKey.getValue(netuidWeights);

      const setWeights = api.tx.SubtensorModule.set_weights({
        netuid: netuidWeights,
        dests: [uid!],
        weights: [65_535],
        version_key: versionKey,
      });
      const setMechanismWeights = api.tx.SubtensorModule.set_mechanism_weights({
        netuid: netuidWeights,
        mecid: 0,
        dests: [uid!],
        weights: [65_535],
        version_key: versionKey,
      });

      await waitForRateLimitTransactionWithRetry(api, setWeights, hotkey, "clone_set_weights_initial");
      await expectTransactionFailure(
        api,
        setMechanismWeights,
        hotkey,
        "clone_set_mechanism_weights_rate_limited",
      );
      await waitForBlocks(api, 2);
      await waitForRateLimitTransactionWithRetry(
        api,
        setMechanismWeights,
        hotkey,
        "clone_set_mechanism_weights_after_window",
      );
    }

    if (shouldRunPhase(filter, "swap-keys")) {
      log("Behavior: swap-keys");
      const coldkey = generateKeyringPair("sr25519");
      const oldHotkey = generateKeyringPair("sr25519");
      const newHotkeyA = generateKeyringPair("sr25519");
      const newHotkeyB = generateKeyringPair("sr25519");

      await forceSetBalancesForRateLimit(api, [
        coldkey.address,
        oldHotkey.address,
        newHotkeyA.address,
        newHotkeyB.address,
      ]);
      await burnedRegister(api, netuidSwap, oldHotkey.address, coldkey);
      await sudoSetGlobalGroupRateLimit(api, GROUP_SWAP_KEYS, 2);

      const swapHotkeyFirst = api.tx.SubtensorModule.swap_hotkey({
        hotkey: oldHotkey.address,
        new_hotkey: newHotkeyA.address,
        netuid: undefined,
      });
      const swapHotkeySecond = api.tx.SubtensorModule.swap_hotkey({
        hotkey: newHotkeyA.address,
        new_hotkey: newHotkeyB.address,
        netuid: undefined,
      });

      await waitForRateLimitTransactionWithRetry(api, swapHotkeyFirst, coldkey, "clone_swap_hotkey_initial");
      await expectTransactionFailure(api, swapHotkeySecond, coldkey, "clone_swap_hotkey_rate_limited");
      await waitForFinalizedBlocks(api, 2);
      await waitForRateLimitTransactionWithRetry(
        api,
        swapHotkeySecond,
        coldkey,
        "clone_swap_hotkey_after_window",
      );
    }

    if (shouldRunPhase(filter, "owner-hparams")) {
      log("Behavior: owner-hparams");
      await sudoSetTempo(api, netuidOwnerA, 1);
      await sudoSetTempo(api, netuidOwnerB, 1);
      await sudoSetGlobalGroupRateLimit(api, GROUP_OWNER_HPARAMS, 2);
      const alice = getAlice();

      const activityCutoffA = await api.query.SubtensorModule.ActivityCutoff.getValue(netuidOwnerA);
      const activityCutoffB = await api.query.SubtensorModule.ActivityCutoff.getValue(netuidOwnerB);
      const rhoA = await api.query.SubtensorModule.Rho.getValue(netuidOwnerA);

      const cutoffAFirst = asSudoTx(api, api.tx.AdminUtils.sudo_set_activity_cutoff({
        netuid: netuidOwnerA,
        activity_cutoff: activityCutoffA + 1,
      }));
      const cutoffASecond = asSudoTx(api, api.tx.AdminUtils.sudo_set_activity_cutoff({
        netuid: netuidOwnerA,
        activity_cutoff: activityCutoffA + 2,
      }));
      const rhoACall = asSudoTx(api, api.tx.AdminUtils.sudo_set_rho({
        netuid: netuidOwnerA,
        rho: rhoA + 1,
      }));
      const cutoffB = asSudoTx(api, api.tx.AdminUtils.sudo_set_activity_cutoff({
        netuid: netuidOwnerB,
        activity_cutoff: activityCutoffB + 1,
      }));
      const burnHalfLifeFirst = asSudoTx(api, api.tx.AdminUtils.sudo_set_burn_half_life({
        netuid: netuidOwnerA,
        burn_half_life: 361,
      }));
      const burnHalfLifeSecond = asSudoTx(api, api.tx.AdminUtils.sudo_set_burn_half_life({
        netuid: netuidOwnerA,
        burn_half_life: 362,
      }));
      const burnIncreaseMult = asSudoTx(api, api.tx.AdminUtils.sudo_set_burn_increase_mult({
        netuid: netuidOwnerA,
        burn_increase_mult: 2n,
      }));

      const expectedCutoffAAfterFirst = activityCutoffA + 1;

      await waitForRateLimitTransactionWithRetry(api, cutoffAFirst, alice as any, "clone_owner_cutoff_a");
      await waitForFinalizedBlocks(api, 1);
      await waitForRateLimitTransactionWithRetry(api, rhoACall, alice as any, "clone_owner_rho_a");
      await waitForFinalizedBlocks(api, 1);
      await waitForRateLimitTransactionWithRetry(api, cutoffB, alice as any, "clone_owner_cutoff_b");
      await submitTransactionBestEffort(api, cutoffASecond, alice as any);
      await waitForFinalizedBlocks(api, 2);
      assert.equal(await api.query.SubtensorModule.ActivityCutoff.getValue(netuidOwnerA), expectedCutoffAAfterFirst);
      await waitForFinalizedBlocks(api, 1);
      await waitForRateLimitTransactionWithRetry(
        api,
        cutoffASecond,
        alice as any,
        "clone_owner_cutoff_a_after_window",
      );

      await waitForRateLimitTransactionWithRetry(
        api,
        burnHalfLifeFirst,
        alice as any,
        "clone_owner_burn_half_life_a",
      );
      await expectTransactionFailure(
        api,
        burnHalfLifeSecond,
        alice as any,
        "clone_owner_burn_half_life_rate_limited",
      );
      await waitForFinalizedBlocks(api, 1);
      await waitForRateLimitTransactionWithRetry(
        api,
        burnIncreaseMult,
        alice as any,
        "clone_owner_burn_increase_mult_a",
      );
      await waitForFinalizedBlocks(api, 2);
      await waitForRateLimitTransactionWithRetry(
        api,
        burnHalfLifeSecond,
        alice as any,
        "clone_owner_burn_half_life_a_after_window",
      );
    }
  } finally {
    try {
      client.destroy();
    } catch {}
  }
}
