import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// ── Read meta written by scripts/gen-chopsticks-fork.ts ───────────────────────
//
// The slim config and this meta file are produced by the generator before the
// chopsticks fork is launched (see `pnpm moonwall test chopsticks_fork`). The meta tells
// us which netuid was forked and an existing registered hotkey on that subnet.

const meta = JSON.parse(readFileSync(resolve(process.cwd(), "tmp/chopsticks-fork-slim.meta.json"), "utf-8")) as {
    blockNumber: number;
    netuid: number;
    hotkey: string;
};

const STAKE_AMOUNT = 200n * 1_000_000_000n; // 200 TAO (9 decimals)

// AlphaV2 stake is stored as a U64F64 fixed-point ({ bits } or { mantissa, exponent }
// depending on metadata). For a before/after comparison we only need a monotonic
// integer, so collapse whatever representation comes back to a BigInt.
function alphaToBigint(value: any): bigint {
    const json = value?.toJSON?.() ?? value;
    if (json == null) return 0n;
    if (typeof json === "number" || typeof json === "string") return BigInt(json);
    if (typeof json === "object") {
        if ("bits" in json) return BigInt(json.bits);
        const mantissa = BigInt(json.mantissa ?? 0);
        const exponent = BigInt(json.exponent ?? 0);
        return exponent >= 0n ? mantissa * 10n ** exponent : mantissa / 10n ** -exponent;
    }
    return BigInt(json);
}

async function getAlphaStake(api: ApiPromise, hotkey: string, coldkey: string, netuid: number): Promise<bigint> {
    const value = await (api.query.subtensorModule as any).alphaV2(hotkey, coldkey, netuid);
    return alphaToBigint(value);
}

describeSuite({
    id: "CHOP_FORK_ADD_STAKE",
    title: "Chopsticks finney fork — runtime upgrade (wasm-override) + addStake",
    foundationMethods: "chopsticks",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;
        let alice: KeyringPair;

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;

            log(
                `Forked finney at block ${meta.blockNumber}, netuid=${meta.netuid}, ` +
                    `hotkey=${meta.hotkey}. Runtime version: ${api.runtimeVersion.specVersion.toString()}`
            );

            // The fork boots on the locally-built runtime via --wasm-override. Produce
            // the first block so Executive::on_runtime_upgrade runs any migrations
            // bundled in that runtime against the real mainnet state before we test.
            await context.createBlock();
        });

        it({
            id: "T01",
            title: "addStake increases the coldkey's alpha stake on the forked subnet",
            test: async () => {
                const coldkey = alice.address;

                const stakeBefore = await getAlphaStake(api, meta.hotkey, coldkey, meta.netuid);
                log(`alpha stake before: ${stakeBefore}`);

                await api.tx.subtensorModule.addStake(meta.hotkey, meta.netuid, STAKE_AMOUNT).signAndSend(alice);

                // Seal the block; createChopsticksBlock throws on any ExtrinsicFailed event.
                await context.createBlock();

                const stakeAfter = await getAlphaStake(api, meta.hotkey, coldkey, meta.netuid);
                log(`alpha stake after:  ${stakeAfter}`);

                expect(stakeAfter > stakeBefore, "addStake should increase alpha stake").toBe(true);
            },
        });
    },
});
