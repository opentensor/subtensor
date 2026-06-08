#!/usr/bin/env tsx
/**
 * Generates a slim Chopsticks config that forks finney for a SINGLE netuid.
 *
 * Vanilla Chopsticks forking of finney is unusably slow (~15 min) because the
 * `prefetch-storages` list makes Chopsticks lazily live-fetch enormous maps
 * (Keys, StakingHotkeys, ...) the first time block production touches them.
 *
 * Given an input Chopsticks YAML with a `prefetch-storages` list, this script:
 *   1. Connects to the configured WS endpoints (a finney archive node).
 *   2. Finds the first non-root registered netuid (or uses --netuid <n>).
 *   3. For each prefetch storage item, introspects its key structure via metadata
 *      and fetches only data for that netuid from the live chain.
 *   4. Writes a new Chopsticks YAML with those values baked into import-storage and
 *      NO prefetch-storages, plus a self-contained `block:` field — so Chopsticks
 *      starts in ~1 min and only processes one subnet.
 *
 * It also zeroes emission-flow storages so the forked subnet's alpha price does
 * not drift during a test (mirrors the upstream relayer generator).
 *
 * Output: <output.yml> and a sibling <output.meta.json> with {blockNumber, netuid, hotkey}.
 *
 * This is run by moonwall as a pre-test step (the chopsticks_fork env's `runScripts`),
 * so it regenerates the slim config from the live chain on every `moonwall test` run.
 *
 * The subnet selection flags also have env equivalents, because moonwall's static
 * `runScripts` entry can't take CLI args at `moonwall test` time:
 *   --netuid <n>     | FORK_NETUID=<n>
 *   --random-subnet  | FORK_RANDOM_SUBNET=1
 *
 * Usage:
 *   tsx scripts/gen-chopsticks-fork.ts <input.yml> <output.yml> [--netuid <n>] [--random-subnet]
 */

import { ApiPromise, WsProvider } from "@polkadot/api";
import { readFileSync, writeFileSync } from "node:fs";

// ── CLI args (subnet selection also reads env, for the moonwall runScripts path) ─

const args = process.argv.slice(2);
const inputPath = args[0];
const outputPath = args[1];

const truthy = (v: string | undefined): boolean => v === "1" || v === "true";

const netuidFlagIdx = args.indexOf("--netuid");
const forcedNetuid =
    netuidFlagIdx !== -1
        ? Number.parseInt(args[netuidFlagIdx + 1], 10)
        : process.env.FORK_NETUID
          ? Number.parseInt(process.env.FORK_NETUID, 10)
          : null;
const randomSubnet = args.includes("--random-subnet") || truthy(process.env.FORK_RANDOM_SUBNET);

if (!inputPath || !outputPath) {
    console.error(
        "Usage: tsx scripts/gen-chopsticks-fork.ts <input.yml> <output.yml> [--netuid <n>] [--random-subnet]"
    );
    process.exit(1);
}

const metaPath = outputPath.replace(/\.yml$/, ".meta.json");

// ── Minimal YAML parser (only what we need from the chopsticks config) ────────

function parseSimpleYaml(text: string): { endpoints: string[]; prefetchStorages: string[] } {
    // We only need `endpoint` and `prefetch-storages` from the input.
    const endpoints: string[] = [];
    const prefetchStorages: string[] = [];

    let inEndpoint = false;
    let inPrefetch = false;

    for (const raw of text.split("\n")) {
        const line = raw.replace(/#.*$/, "").trimEnd();
        if (!line.trim()) {
            inEndpoint = false;
            inPrefetch = false;
            continue;
        }

        if (/^endpoint:/.test(line)) {
            inEndpoint = true;
            inPrefetch = false;
            const inline = line.replace(/^endpoint:\s*/, "").trim();
            if (inline && !inline.startsWith("-")) endpoints.push(inline.replace(/^['"]|['"]$/g, ""));
            continue;
        }
        if (/^prefetch-storages:/.test(line)) {
            inPrefetch = true;
            inEndpoint = false;
            continue;
        }
        if (/^[a-zA-Z]/.test(line)) {
            inEndpoint = false;
            inPrefetch = false;
        }

        if (inEndpoint && /^\s*-\s/.test(line)) {
            endpoints.push(
                line
                    .replace(/^\s*-\s*/, "")
                    .replace(/^['"]|['"]$/g, "")
                    .trim()
            );
        }
        if (inPrefetch && /^\s*-\s/.test(line)) {
            prefetchStorages.push(line.replace(/^\s*-\s*/, "").trim());
        }
    }

    return { endpoints, prefetchStorages };
}

// ── Storage introspection ─────────────────────────────────────────────────────

type StorageKind = { kind: "plain" } | { kind: "map"; keyCount: number };

function getStorageKind(api: ApiPromise, pallet: string, name: string): StorageKind {
    const query = (api.query as any)[pallet]?.[name];
    if (!query) throw new Error(`Storage not found: ${pallet}.${name}`);

    const meta = query.creator.meta;
    if (meta.type.isPlain) return { kind: "plain" };
    if (meta.type.isMap) return { kind: "map", keyCount: meta.type.asMap.hashers.length };
    return { kind: "plain" };
}

// ── Value serialisation ───────────────────────────────────────────────────────

// Convert a polkadot-js Codec to a YAML-safe primitive.
function toYamlValue(codec: any): unknown {
    // toJSON gives human-readable output; large integers come back as hex strings.
    return codec.toJSON();
}

// ── YAML emission helpers ─────────────────────────────────────────────────────

function indent(n: number): string {
    return "  ".repeat(n);
}

// Serialise a JS value to inline YAML (single line, no block scalars).
function yamlInline(val: unknown): string {
    if (val === null) return "null";
    if (val === true) return "true";
    if (val === false) return "false";
    if (typeof val === "number") return String(val);
    if (typeof val === "bigint") return String(val);
    if (typeof val === "string") {
        if (/^(true|false|null|~|\d)/.test(val) || /[:#\[\]{},&*!|>'"%@`]/.test(val) || val.includes("\n")) {
            return JSON.stringify(val);
        }
        return val;
    }
    if (Array.isArray(val)) {
        return `[${val.map(yamlInline).join(", ")}]`;
    }
    if (typeof val === "object" && val !== null) {
        const pairs = Object.entries(val as Record<string, unknown>)
            .map(([k, v]) => `${k}: ${yamlInline(v)}`)
            .join(", ");
        return `{${pairs}}`;
    }
    return String(val);
}

// Emit a map entry with any number of keys: `- [[key1, key2, ...], value]`
function emitEntry(depth: number, keys: unknown[], value: unknown): string {
    const lines = keys.map((k, i) =>
        i === 0 ? `${indent(depth)}- - - ${yamlInline(k)}` : `${indent(depth)}    - ${yamlInline(k)}`
    );
    lines.push(`${indent(depth)}  - ${yamlInline(value)}`);
    return lines.join("\n");
}

// ── Fetch helpers ─────────────────────────────────────────────────────────────

async function fetchStorageEntries(
    api: ApiPromise,
    pallet: string,
    name: string,
    netuid: number,
    kind: StorageKind
): Promise<string[]> {
    const query = (api.query as any)[pallet][name];
    const lines: string[] = [];

    if (kind.kind === "plain") {
        // Plain storage is a global scalar — not expressible as an iterable map entry.
        // Skip it; Chopsticks falls back to the remote fork value for these items.
        return [];
    }

    if (kind.keyCount === 1) {
        // Zero out emission flow storage so tests don't inherit mainnet emission rates.
        if (name === "subnetTaoFlow") {
            // Zero raw flow so the subnet gets zero share of block emission.
            lines.push(`${indent(2)}${name}:`);
            lines.push(emitEntry(3, [netuid], 0));
            return lines;
        }
        if (name === "subnetEmaTaoFlow") {
            // Empty array + $removePrefix removes all stored EMA entries so historical
            // EMA can't drive emissions during the test.
            lines.push(`${indent(2)}${name}: []`);
            return lines;
        }
        if (name === "firstEmissionBlockNumber") {
            // Clearing this (None) makes the subnet fail get_subnets_to_emit_to, so no
            // fresh TAO block emission is assigned and the per-block swap path never fires.
            lines.push(`${indent(2)}${name}: []`);
            return lines;
        }
        if (
            name === "pendingServerEmission" ||
            name === "pendingValidatorEmission" ||
            name === "pendingOwnerCut" ||
            name === "pendingRootAlphaDivs"
        ) {
            // Zero queued alpha emissions snapshotted from mainnet so on_initialize
            // doesn't distribute them on the first block and push the price up.
            lines.push(`${indent(2)}${name}:`);
            lines.push(emitEntry(3, [netuid], 0));
            return lines;
        }
        lines.push(`${indent(2)}${name}:`);
        lines.push(emitEntry(3, [netuid], toYamlValue(await query(netuid))));
        return lines;
    }

    if (name === "lastRateLimitedBlock") {
        // Keyed by RateLimitKey enum, not by netuid. Migrations iterate iter_keys()
        // over every entry (one per staker) causing chopsticks to live-fetch them all.
        // Seeding empty makes the migration's iter_keys() return nothing instantly.
        lines.push(`${indent(2)}${name}: []`);
        return lines;
    }

    // keyCount >= 2: prefix scan on first key = netuid; use all decoded args as keys
    const entries: [any, any][] = await query.entries(netuid);
    if (entries.length === 0) {
        // An empty iterable array is required — a bare key with no value becomes YAML
        // null and Chopsticks crashes with "storage is not iterable".
        lines.push(`${indent(2)}${name}: []`);
        return lines;
    }
    lines.push(`${indent(2)}${name}:`);
    for (const [storageKey, val] of entries) {
        const keys = (storageKey.args as any[]).map((a) => toYamlValue(a));
        lines.push(emitEntry(3, keys, toYamlValue(val)));
    }
    return lines;
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
    const inputText = readFileSync(inputPath, "utf-8");
    const { endpoints, prefetchStorages } = parseSimpleYaml(inputText);

    if (endpoints.length === 0) throw new Error("No endpoints found in input YAML");
    if (prefetchStorages.length === 0) throw new Error("No prefetch-storages found in input YAML");

    console.log(`Endpoints: ${endpoints.join(", ")}`);
    console.log(`Prefetch items: ${prefetchStorages.length}`);

    // Connect — try each endpoint in order until one works
    let api: ApiPromise | null = null;
    let blockNumber = 0;
    for (const ep of endpoints) {
        process.stdout.write(`Connecting to ${ep}...`);
        const provider = new WsProvider(ep, 0);
        try {
            await new Promise<void>((res, rej) => {
                const timer = setTimeout(() => rej(new Error("timeout")), 15_000);
                provider.on("connected", () => {
                    clearTimeout(timer);
                    res();
                });
                provider.on("error", () => {
                    clearTimeout(timer);
                    rej(new Error("ws error"));
                });
                provider.connect().catch(rej);
            });
            api = await ApiPromise.create({ provider, noInitWarn: true });
            const header = await api.rpc.chain.getHeader();
            blockNumber = header.number.toNumber();
            console.log(" connected");
            break;
        } catch {
            console.log(" failed, trying next");
            await provider.disconnect().catch(() => {});
        }
    }
    if (!api) throw new Error("Could not connect to any endpoint");

    // Find target netuid
    let netuid: number;
    if (forcedNetuid !== null) {
        netuid = forcedNetuid;
        console.log(`Using forced netuid=${netuid}`);
    } else {
        const entries = (await (api.query.subtensorModule as any).networksAdded.entries()) as any[];
        const netuids = entries
            .filter(([, v]: [any, any]) => v.toPrimitive() === true)
            .map(([k]: [any, any]) => k.args[0].toNumber() as number)
            .filter((n: number) => n > 0)
            .sort((a: number, b: number) => a - b);
        if (netuids.length === 0) throw new Error("No non-root subnets found");
        if (randomSubnet) {
            netuid = netuids[Math.floor(Math.random() * netuids.length)];
            console.log(`Picked netuid=${netuid} (random, from ${netuids.length} non-root subnets)`);
        } else {
            netuid = netuids[0];
            console.log(`Picked netuid=${netuid} (first non-root)`);
        }
    }

    // Group prefetch items by pallet
    const byPallet = new Map<string, string[]>();
    for (const item of prefetchStorages) {
        const dot = item.indexOf(".");
        if (dot === -1) {
            console.warn(`Skipping malformed prefetch item: ${item}`);
            continue;
        }
        const pallet = item.slice(0, dot);
        const name = item.slice(dot + 1);
        if (!byPallet.has(pallet)) byPallet.set(pallet, []);
        byPallet.get(pallet)?.push(name);
    }

    // Extract plain scalar overrides from the static import-storage for a pallet.
    // These are lines like `    Key: value` that are NOT list entries or directives.
    // We carry them into the generated section so they survive the prefetch rewrite.
    function parsePlainOverrides(palletName: string): string[] {
        const lines: string[] = [];
        let inImportStorage = false;
        let inTargetPallet = false;
        for (const raw of inputText.split("\n")) {
            const stripped = raw.replace(/#.*$/, "").trimEnd();
            if (/^import-storage:/.test(stripped)) {
                inImportStorage = true;
                continue;
            }
            if (!inImportStorage) continue;
            if (/^[a-zA-Z]/.test(raw) && !/^\s/.test(raw)) {
                inImportStorage = false;
                continue;
            }
            if (!stripped.trim()) continue;
            const palletMatch = raw.match(/^ {2}([A-Za-z][A-Za-z0-9]*):/);
            if (palletMatch) {
                inTargetPallet = palletMatch[1] === palletName;
                continue;
            }
            if (!inTargetPallet) continue;
            const m = stripped.match(/^ {4}([A-Za-z][A-Za-z0-9]*):\s*(\S.*)$/);
            if (m && !/^[-\[{]/.test(m[2])) {
                lines.push(`${indent(2)}${m[1]}: ${m[2]}`);
            }
        }
        return lines;
    }

    // Build import-storage sections per pallet
    const palletSections: string[] = [];
    for (const [pallet, names] of Array.from(byPallet)) {
        const palletCamel = pallet.charAt(0).toLowerCase() + pallet.slice(1);
        const mapNames: string[] = []; // names that are maps (need $removePrefix)

        const entryLines: string[] = [];

        for (const name of names) {
            const nameCamel = name.charAt(0).toLowerCase() + name.slice(1);
            process.stdout.write(`  Fetching ${pallet}.${name}...`);
            try {
                const kind = getStorageKind(api, palletCamel, nameCamel);
                if (kind.kind === "map") mapNames.push(name);
                const lines = await fetchStorageEntries(api, palletCamel, nameCamel, netuid, kind);
                entryLines.push(...lines);
                console.log(` ok (${kind.kind}${kind.kind === "map" ? `, keys=${kind.keyCount}` : ""})`);
            } catch (e) {
                console.log(` SKIPPED (${(e as Error).message})`);
            }
        }

        const plainOverrides = parsePlainOverrides(pallet);

        const removePrefix =
            mapNames.length > 0
                ? `${indent(2)}$removePrefix:\n${mapNames.map((n) => `${indent(3)}- ${n}`).join("\n")}\n`
                : "";

        palletSections.push(
            `${indent(1)}${pallet}:\n${removePrefix}${plainOverrides.join("\n")}${plainOverrides.length > 0 ? "\n" : ""}${entryLines.join("\n")}`
        );
    }

    // Preserve everything already in import-storage that isn't from prefetch pallets.
    const prefetchPallets = new Set(Array.from(byPallet.keys()));
    const staticImportLines: string[] = [];
    let inImportStorage = false;
    let currentPalletIsFromPrefetch = false;

    for (const raw of inputText.split("\n")) {
        if (/^import-storage:/.test(raw)) {
            inImportStorage = true;
            continue;
        }
        if (inImportStorage) {
            if (/^[a-zA-Z]/.test(raw) && !/^\s/.test(raw)) {
                inImportStorage = false;
                continue;
            }
            const palletMatch = raw.match(/^ {2}([A-Za-z][A-Za-z0-9]*):/);
            if (palletMatch) {
                currentPalletIsFromPrefetch = prefetchPallets.has(palletMatch[1]);
            }
            if (!currentPalletIsFromPrefetch) staticImportLines.push(raw);
        }
    }

    const outputLines = [
        "# Auto-generated by gen-chopsticks-fork.ts",
        `# Source: ${inputPath}`,
        `# Netuid: ${netuid}`,
        "",
        "endpoint:",
        ...endpoints.map((e) => `  - ${e}`),
        "",
        // Pin the fork block so this config is self-contained (moonwall does not pass --block).
        `block: ${blockNumber}`,
        "",
        "mock-signature-host: true",
        "allow-unresolved-imports: true",
        "",
        "import-storage:",
        ...staticImportLines.filter((l) => l.trim()),
        ...palletSections,
        "",
        "# prefetch-storages intentionally omitted — all data is baked into import-storage above",
    ];

    const output = `${outputLines.join("\n")}\n`;
    writeFileSync(outputPath, output);
    console.log(`\nWritten to ${outputPath}`);

    // Pick an existing registered hotkey from UID 0 on the target subnet.
    const hotkeyCodec = await (api.query.subtensorModule as any).keys(netuid, 0);
    const hotkey: string = hotkeyCodec.toString();
    console.log(`Picked existing hotkey at uid=0: ${hotkey}`);

    writeFileSync(metaPath, JSON.stringify({ blockNumber, netuid, hotkey }, null, 2));
    console.log(`Written meta to ${metaPath}`);

    await api.disconnect();
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
