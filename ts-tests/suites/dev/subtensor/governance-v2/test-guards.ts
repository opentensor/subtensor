import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * Negative tests covering validation guards after the governance-v2 refactor.
 *
 * With the new API there is no `Proposal` variant at the extrinsic layer — the pallet
 * derives the proposal shape from `DecisionStrategy`. That removes the need for
 * `IncompatibleProposalKind` / `TaskNotScheduled` errors; origin gating (`SubmitOrigin`)
 * now handles track access, and misuse surfaces as `BadOrigin`.
 */
describeSuite({
    id: "DEV_SUB_GOVV2_GUARDS_01",
    title: "Governance V2 — validation guards",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        let alice: KeyringPair;
        let bob: KeyringPair;
        let charlie: KeyringPair;
        let eve: KeyringPair;

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
            charlie = context.keyring.charlie;
            const sr = new Keyring({ type: "sr25519" });
            eve = sr.addFromUri("//Eve");

            for (const inner of [
                api.tx.multiCollective.addMember("Proposers", alice.address),
                api.tx.multiCollective.addMember("Triumvirate", bob.address),
                api.tx.multiCollective.addMember("Triumvirate", charlie.address),
            ]) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(alice)]);
            }
        });

        /** Decode the last block's dispatch error if any. */
        const extrinsicFailed = async () => {
            const events = await api.query.system.events();
            const failed = events.find(
                (e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed",
            );
            if (!failed) return null;
            const dispatchError = failed.event.data[0] as any;
            if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                return { kind: "module", section: decoded.section, name: decoded.name };
            }
            return { kind: dispatchError.type ?? "other", name: dispatchError.toString() };
        };

        it({
            id: "T01",
            title: "direct submit to track 1 by signed user → BadOrigin",
            test: async () => {
                // Track 1 SubmitOrigin only accepts Root. Alice (Proposer) is rejected.
                const inner = api.tx.balances.forceSetBalance(eve.address, 1n);
                await context.createBlock([
                    await api.tx.referenda.submit(1, inner).signAsync(alice),
                ]);

                const err = await extrinsicFailed();
                log(`error: ${JSON.stringify(err)}`);
                expect(err).not.to.be.null;
                // BadOrigin is a top-level dispatch error, not a Module error — its
                // representation in polkadot-js is `.type === "BadOrigin"`.
                expect(err?.name).to.match(/BadOrigin/);
            },
        });

        it({
            id: "T02",
            title: "submit on unknown track → BadOrigin via SubmitOrigin",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(eve.address, 2n);
                await context.createBlock([
                    await api.tx.referenda.submit(99, inner).signAsync(alice),
                ]);

                const err = await extrinsicFailed();
                expect(err).not.to.be.null;
                expect(err?.name).to.match(/BadOrigin/);
            },
        });

        it({
            id: "T03",
            title: "duplicate vote → DuplicateVote; vote switch → ok",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(eve.address, 3n);
                await context.createBlock([
                    await api.tx.referenda.submit(0, inner).signAsync(alice),
                ]);
                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                await context.createBlock([
                    await api.tx.signedVoting.vote(poll, true).signAsync(bob),
                ]);

                await context.createBlock([
                    await api.tx.signedVoting.vote(poll, true).signAsync(bob),
                ]);
                const dup = await extrinsicFailed();
                expect(dup?.section).to.equal("signedVoting");
                expect(dup?.name).to.equal("DuplicateVote");

                await context.createBlock([
                    await api.tx.signedVoting.vote(poll, false).signAsync(bob),
                ]);
                const afterSwitch = await extrinsicFailed();
                expect(afterSwitch, "vote switch should succeed").to.be.null;

                const tally = await api.query.signedVoting.tallyOf(poll);
                expect(tally.toJSON()).to.deep.contain({ ayes: 0, nays: 1 });
            },
        });

        it({
            id: "T04",
            title: "remove_vote without prior vote → VoteNotFound",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(eve.address, 4n);
                await context.createBlock([
                    await api.tx.referenda.submit(0, inner).signAsync(alice),
                ]);
                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                await context.createBlock([
                    await api.tx.signedVoting.removeVote(poll).signAsync(charlie),
                ]);

                const err = await extrinsicFailed();
                expect(err?.section).to.equal("signedVoting");
                expect(err?.name).to.equal("VoteNotFound");
            },
        });
    },
});
