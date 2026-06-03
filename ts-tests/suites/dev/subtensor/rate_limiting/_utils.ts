import type { DevModeContext } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

export const GROUP_SERVE = 0;
export const GROUP_DELEGATE_TAKE = 1;
export const GROUP_WEIGHTS_SET = 2;
export const GROUP_REGISTER_NETWORK = 3;
export const GROUP_OWNER_HPARAMS = 4;
export const GROUP_STAKING_OPS = 5;
export const GROUP_SWAP_KEYS = 6;

export type SubmittableLike = {
    method: { callIndex: Uint8Array | number[] };
    decodedCall?: unknown;
    signAsync: (signer: KeyringPair) => Promise<unknown>;
};

/** Returns the current finalized block number as a plain number. */
export async function currentBlock(api: ApiPromise): Promise<number> {
    const n = await api.query.system.number();
    return (n as unknown as { toNumber: () => number }).toNumber();
}

export async function disableAdminFreezeWindow(
    api: ApiPromise,
    context: DevModeContext,
    sudoSigner: KeyringPair
): Promise<void> {
    const inner = api.tx.adminUtils.sudoSetAdminFreezeWindow(0);
    await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(sudoSigner)]);
}

/** Reads `(pallet_idx, call_idx)` from a SubmittableExtrinsic. */
export function callIndex(tx: SubmittableLike): { pallet: number; call: number } {
    const idx = tx.method.callIndex as Uint8Array;
    return { pallet: Number(idx[0]), call: Number(idx[1]) };
}

/** Returns the GroupId assigned to a call (None ⇒ not in any group). */
export async function callGroupOf(api: ApiPromise, tx: SubmittableLike): Promise<number | null> {
    const { pallet, call } = callIndex(tx);
    const value = await api.query.rateLimiting.callGroups({
        palletIndex: pallet,
        extrinsicIndex: call,
    });
    if (value.isEmpty) return null;
    return Number((value as any).toString());
}

export async function ensureCallInGroup(
    api: ApiPromise,
    context: DevModeContext,
    sudoSigner: KeyringPair,
    tx: SubmittableLike,
    groupId: number,
    readOnly: boolean = false
): Promise<void> {
    const existing = await callGroupOf(api, tx);
    if (existing === groupId) return;
    if (existing !== null && existing !== groupId) {
        throw new Error(
            `call already registered in group ${existing}, expected ${groupId} (cannot reassign without remove_call_from_group)`
        );
    }
    if (!readOnly) {
        // Fast path: register_call also assigns the group and forces read_only=false.
        const register = api.tx.rateLimiting.registerCall(tx, groupId);
        await context.createBlock([await api.tx.sudo.sudo(register).signAsync(sudoSigner)]);
    } else {
        // Two-step: register without group (so we control the read_only flag below), then assign.
        const register = api.tx.rateLimiting.registerCall(tx, null);
        await context.createBlock([await api.tx.sudo.sudo(register).signAsync(sudoSigner)]);
        const { pallet, call } = callIndex(tx);
        const assign = api.tx.rateLimiting.assignCallToGroup(
            { palletIndex: pallet, extrinsicIndex: call },
            groupId,
            true
        );
        await context.createBlock([await api.tx.sudo.sudo(assign).signAsync(sudoSigner)]);
    }
    const after = await callGroupOf(api, tx);
    if (after !== groupId) {
        throw new Error(`failed to register call into group ${groupId} (still ${after})`);
    }
}

export async function setGroupSpan(
    api: ApiPromise,
    context: DevModeContext,
    sudoSigner: KeyringPair,
    groupId: number,
    scope: number | null,
    span: number
): Promise<void> {
    const inner = api.tx.rateLimiting.setRateLimit({ Group: groupId }, scope === null ? null : scope, { Exact: span });
    const wrapped = api.tx.sudo.sudo(inner);
    await context.createBlock([await wrapped.signAsync(sudoSigner)]);
}

export async function expectExtrinsicOk(
    api: ApiPromise,
    context: DevModeContext,
    signedTx: any,
    label: string
): Promise<void> {
    const before = await currentBlock(api);
    await context.createBlock([signedTx]);
    const events = (await api.query.system.events()) as any;
    const failed = events.find((e: any) => e.event.method === "ExtrinsicFailed");
    if (failed) {
        const err = JSON.stringify(failed.event.data?.toHuman?.() ?? failed.event.toHuman());
        throw new Error(`[${label}] expected success but ExtrinsicFailed: ${err}`);
    }
    const after = await currentBlock(api);
    if (after === before) {
        throw new Error(`[${label}] expected success but no block was produced`);
    }
}

export async function expectRateLimited(
    api: ApiPromise,
    context: DevModeContext,
    signedTx: any,
    label: string
): Promise<void> {
    const before = await currentBlock(api);
    try {
        await context.createBlock([signedTx]);
    } catch (e: any) {
        const msg = String(e?.message ?? e);
        if (/Custom.*1|RATE_LIMIT|rate.*limit/i.test(msg)) return;
        throw new Error(`[${label}] expected rate-limit rejection but got: ${msg}`);
    }
    const after = await currentBlock(api);
    if (after > before) {
        // Block was produced; check if the tx made it in (success) or was simply excluded.
        const events = (await api.query.system.events()) as any;
        const success = events.find((e: any) => e.event.method === "ExtrinsicSuccess");
        const failed = events.find((e: any) => e.event.method === "ExtrinsicFailed");
        if (success) {
            throw new Error(`[${label}] expected rate-limit rejection but tx succeeded`);
        }
        if (failed) {
            const err = JSON.stringify(failed.event.data?.toHuman?.() ?? failed.event.toHuman());
            // Dispatch-level failure isn't a rate-limit rejection (which lives at validate).
            throw new Error(`[${label}] expected rate-limit rejection but got dispatch failure: ${err}`);
        }
    }
}
