import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * Negative tests covering the validation guards in pallet-referenda and pallet-signed-voting.
 * Each case submits a transaction that should fail, and asserts that a matching
 * `system.ExtrinsicFailed` event is present with the expected module error.
 *
 * Note: moonwall `createBlock` includes the tx even when it reverts; errors surface via events.
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

        /** Look up the last extrinsic's dispatch error and decode it. */
        const extrinsicFailed = async () => {
            const events = await api.query.system.events();
            const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
            if (!failed) return null;
            const dispatchError = failed.event.data[0] as any;
            if (dispatchError.isModule) {
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                return { section: decoded.section, name: decoded.name };
            }
            return { section: "?", name: dispatchError.toString() };
        };

        it({
            id: "T01",
            title: "direct Review with non-existent task → TaskNotScheduled",
            test: async () => {
                const phantomTask = "0x" + "FF".repeat(32);
                const submitTx = api.tx.referenda.submit(1, { Review: phantomTask });

                await context.createBlock([await submitTx.signAsync(alice)]);

                const err = await extrinsicFailed();
                log(`error: ${JSON.stringify(err)}`);
                expect(err).not.to.be.null;
                expect(err?.section).to.equal("referenda");
                expect(err?.name).to.equal("TaskNotScheduled");
            },
        });

        it({
            id: "T02",
            title: "Action on track 1 (Adjustable) → IncompatibleProposalKind",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(eve.address, 1n);
                const submitTx = api.tx.referenda.submit(1, { Action: inner });

                await context.createBlock([await submitTx.signAsync(alice)]);

                const err = await extrinsicFailed();
                expect(err?.section).to.equal("referenda");
                expect(err?.name).to.equal("IncompatibleProposalKind");
            },
        });

        it({
            id: "T03",
            title: "Review on track 0 (PassOrFail) → IncompatibleProposalKind",
            test: async () => {
                const submitTx = api.tx.referenda.submit(0, {
                    Review: "0x" + "AA".repeat(32),
                });
                await context.createBlock([await submitTx.signAsync(alice)]);

                const err = await extrinsicFailed();
                expect(err?.section).to.equal("referenda");
                expect(err?.name).to.equal("IncompatibleProposalKind");
            },
        });

        it({
            id: "T04",
            title: "duplicate vote → DuplicateVote; vote switch → ok",
            test: async () => {
                // Seed a fresh poll.
                const inner = api.tx.balances.forceSetBalance(eve.address, 3n);
                await context.createBlock([await api.tx.referenda.submit(0, { Action: inner }).signAsync(alice)]);
                const count = (await api.query.referenda.referendumCount()).toNumber();
                const poll = count - 1;

                // Bob votes aye.
                await context.createBlock([await api.tx.signedVoting.vote(poll, true).signAsync(bob)]);

                // Same aye again → DuplicateVote.
                await context.createBlock([await api.tx.signedVoting.vote(poll, true).signAsync(bob)]);
                const dup = await extrinsicFailed();
                expect(dup?.section).to.equal("signedVoting");
                expect(dup?.name).to.equal("DuplicateVote");

                // Switch to nay — must succeed, no DuplicateVote.
                await context.createBlock([await api.tx.signedVoting.vote(poll, false).signAsync(bob)]);
                const afterSwitch = await extrinsicFailed();
                expect(afterSwitch, "switch aye→nay should succeed").to.be.null;

                const tally = await api.query.signedVoting.tallyOf(poll);
                expect(tally.toJSON()).to.deep.contain({ ayes: 0, nays: 1 });
            },
        });

        it({
            id: "T05",
            title: "remove_vote without prior vote → VoteNotFound",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(eve.address, 4n);
                await context.createBlock([await api.tx.referenda.submit(0, { Action: inner }).signAsync(alice)]);
                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                // Charlie hasn't voted.
                await context.createBlock([await api.tx.signedVoting.removeVote(poll).signAsync(charlie)]);

                const err = await extrinsicFailed();
                expect(err?.section).to.equal("signedVoting");
                expect(err?.name).to.equal("VoteNotFound");
            },
        });
    },
});
