import { spawn, execFileSync } from "node:child_process";
import { writeFile, readFile } from "node:fs/promises";
import { Keyring } from "@polkadot/keyring";
import { log } from "./node.js";

// ---------------------------------------------------------------------------
// Chain spec generation
// ---------------------------------------------------------------------------

/**
 * Generate a raw chain spec. If `patchSpec` is provided, first generates a
 * non-raw spec, applies the patch, then converts to raw. This allows adding
 * extra authorities, balances, etc. without modifying the Rust chain spec.
 */
export const generateChainSpec = async (
  binaryPath: string,
  outputPath: string,
  patchSpec?: (spec: any) => void,
) => {
  if (!patchSpec) {
    return generateRawChainSpec(binaryPath, outputPath, "local");
  }

  // 2-step: generate non-raw → patch → generate raw.
  const nonRawPath = outputPath + ".nonraw.json";

  await new Promise<void>((resolve, reject) => {
    const proc = spawn(binaryPath, [
      "build-spec",
      "--disable-default-bootnode",
      "--chain",
      "local",
    ]);
    const chunks: Buffer[] = [];
    proc.stdout.on("data", (chunk: Buffer) => chunks.push(chunk));
    let stderr = "";
    proc.stderr?.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });
    proc.on("close", async (code) => {
      if (code !== 0) {
        reject(new Error(`Failed to generate non-raw chain spec (exit ${code}): ${stderr}`));
        return;
      }
      await writeFile(nonRawPath, Buffer.concat(chunks));
      resolve();
    });
    proc.on("error", reject);
  });

  const specJson = JSON.parse(await readFile(nonRawPath, "utf-8"));
  patchSpec(specJson);
  await writeFile(nonRawPath, JSON.stringify(specJson, null, 2));

  await generateRawChainSpec(binaryPath, outputPath, nonRawPath);
};

async function generateRawChainSpec(binaryPath: string, outputPath: string, chain: string) {
  return new Promise<void>((resolve, reject) => {
    const proc = spawn(binaryPath, [
      "build-spec",
      "--disable-default-bootnode",
      "--raw",
      "--chain",
      chain,
    ]);

    const chunks: Buffer[] = [];
    proc.stdout.on("data", (chunk: Buffer) => chunks.push(chunk));

    let stderr = "";
    proc.stderr?.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });

    proc.on("close", async (code) => {
      if (code !== 0) {
        reject(new Error(`Failed to generate chain spec (exit ${code}): ${stderr}`));
        return;
      }
      const data = Buffer.concat(chunks);
      await writeFile(outputPath, data);
      log(`Chain spec written to ${outputPath} (${data.length} bytes)`);
      resolve();
    });

    proc.on("error", reject);
  });
}

// ---------------------------------------------------------------------------
// Chain spec patching helpers (composable)
// ---------------------------------------------------------------------------

/**
 * Extract the genesis runtime patch object from a non-raw chain spec.
 * Works with both the `runtimeGenesis.patch` and legacy `runtime` formats.
 */
export function getGenesisPatch(spec: any): any {
  const patch = spec.genesis?.runtimeGenesis?.patch ?? spec.genesis?.runtime;
  if (!patch) throw new Error("Cannot find genesis patch in chain spec");
  return patch;
}

/** Add an Aura authority (sr25519 address) to the chain spec. */
export function addAuraAuthority(patch: any, address: string) {
  if (patch.aura?.authorities) {
    patch.aura.authorities.push(address);
  }
}

/** Add a GRANDPA authority (ed25519 address, weight) to the chain spec. */
export function addGrandpaAuthority(patch: any, address: string, weight = 1) {
  if (patch.grandpa?.authorities) {
    patch.grandpa.authorities.push([address, weight]);
  }
}

/** Add a balance entry to the chain spec. */
export function addBalance(patch: any, address: string, amount: number | bigint) {
  if (patch.balances?.balances) {
    patch.balances.balances.push([address, Number(amount)]);
  }
}

// ---------------------------------------------------------------------------
// Authority key helpers
// ---------------------------------------------------------------------------

export type AuthorityKeys = {
  aura: string;
  grandpa: string;
  account: string;
};

/** Derive authority keys (aura sr25519, grandpa ed25519, account) from a seed. */
export function generateAuthorityKeys(seed: string): AuthorityKeys {
  const sr = new Keyring({ type: "sr25519" });
  const ed = new Keyring({ type: "ed25519" });
  return {
    aura: sr.addFromUri(`//${seed}`).address,
    grandpa: ed.addFromUri(`//${seed}`).address,
    account: sr.addFromUri(`//${seed}`).address,
  };
}

/**
 * Convenience: add a full authority (aura + grandpa + funded account) to a
 * chain spec genesis patch. Derives keys from the given seed.
 */
export function addAuthority(patch: any, seed: string, balance = 2_000_000_000_000) {
  const keys = generateAuthorityKeys(seed);
  addAuraAuthority(patch, keys.aura);
  addGrandpaAuthority(patch, keys.grandpa);
  addBalance(patch, keys.account, balance);
}

// ---------------------------------------------------------------------------
// Key insertion
// ---------------------------------------------------------------------------

/**
 * Insert Aura (sr25519) and GRANDPA (ed25519) keys into a node's keystore.
 * Required for authority nodes that don't have a built-in substrate CLI shortcut.
 */
export const insertKeys = (
  binaryPath: string,
  basePath: string,
  chainSpec: string,
  seed: string,
) => {
  const run = (scheme: string, keyType: string) => {
    execFileSync(binaryPath, [
      "key",
      "insert",
      "--base-path",
      basePath,
      "--chain",
      chainSpec,
      "--suri",
      seed,
      "--scheme",
      scheme,
      "--key-type",
      keyType,
    ]);
  };
  run("sr25519", "aura");
  run("ed25519", "gran");
  log(`Inserted aura+grandpa keys for ${seed} into ${basePath}`);
};
