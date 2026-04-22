import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * Track 0 PassOrFail flow:
 *   1. Alice (Proposer) submits an `Action` with a Root-only call.
 *   2. Bob and Charlie (Triumvirate) vote aye → 2/3 reaches the approve threshold.
 *   3. Scheduler dispatches the approved call with Root origin in the next block.
 */
describeSuite({
    id: "DEV_SUB_GOVV2_TRACK0_01",
    title: "Governance V2 — Track 0 PassOrFail approval",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        let alice: KeyringPair;
        let bob: KeyringPair;
        let charlie: KeyringPair;
        let dave: KeyringPair;
        let eve: KeyringPair;

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
            charlie = context.keyring.charlie;
            dave = context.keyring.dave;
            const sr = new Keyring({ type: "sr25519" });
            eve = sr.addFromUri("//Eve");

            // Populate collectives via sudo.
            // Alice is the lone allowed Proposer; Bob/Charlie/Dave form the Triumvirate.
            const adds = [
                api.tx.multiCollective.addMember("Proposers", alice.address),
                api.tx.multiCollective.addMember("Triumvirate", bob.address),
                api.tx.multiCollective.addMember("Triumvirate", charlie.address),
                api.tx.multiCollective.addMember("Triumvirate", dave.address),
            ];
            for (const inner of adds) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(alice)]);
            }

            const triumvirate = await api.query.multiCollective.members("Triumvirate");
            const proposers = await api.query.multiCollective.members("Proposers");
            log(`Proposers: ${proposers.toJSON()}`);
            log(`Triumvirate: ${triumvirate.toJSON()}`);

            expect(triumvirate.toJSON()).to.have.length(3);
            expect(proposers.toJSON()).to.have.length(1);
        });

        it({
            id: "T01",
            title: "submit Action and 2-of-3 triumvirate ayes → approved",
            test: async () => {
                const targetAmount = 1_000_000_000n;
                const eveBefore = (await api.query.system.account(eve.address)).data.free.toBigInt();

                const innerCall = api.tx.balances.forceSetBalance(eve.address, targetAmount);
                const submitTx = api.tx.referenda.submit(0, { Action: innerCall });

                await context.createBlock([await submitTx.signAsync(alice)]);

                const submittedEvent = (await api.query.system.events()).find(
                    (e) => e.event.section === "referenda" && e.event.method === "Submitted"
                );
                expect(submittedEvent, "Submitted event").to.exist;

                // Bob votes aye → 1/3 = 33%, below threshold.
                await context.createBlock([await api.tx.signedVoting.vote(0, true).signAsync(bob)]);
                const tallyAfterBob = await api.query.signedVoting.tallyOf(0);
                expect(tallyAfterBob.toJSON()).to.deep.contain({ ayes: 1, nays: 0, total: 3 });

                // Charlie votes aye → 2/3 = matches `Perbill::from_rational(2, 3)`.
                await context.createBlock([await api.tx.signedVoting.vote(0, true).signAsync(charlie)]);

                const approved = (await api.query.system.events()).find(
                    (e) => e.event.section === "referenda" && e.event.method === "Approved"
                );
                expect(approved, "Approved event").to.exist;

                // Next block: scheduler dispatches the action with Root.
                await context.createBlock([]);

                const events = await api.query.system.events();
                const dispatched = events.find(
                    (e) => e.event.section === "scheduler" && e.event.method === "Dispatched"
                );
                expect(dispatched, "scheduler.Dispatched event").to.exist;

                const eveAfter = (await api.query.system.account(eve.address)).data.free.toBigInt();
                expect(eveAfter).to.equal(targetAmount);
                expect(eveAfter).not.to.equal(eveBefore);
            },
        });

        it({
            id: "T02",
            title: "non-proposer rejected with NotAllowedProposer",
            test: async () => {
                // Dave is NOT in Proposers (he's a Triumvirate member).
                const innerCall = api.tx.balances.forceSetBalance(eve.address, 42n);
                const submitTx = api.tx.referenda.submit(0, { Action: innerCall });

                await context.createBlock([await submitTx.signAsync(dave)]);

                const events = await api.query.system.events();
                const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
                expect(failed, "ExtrinsicFailed").to.exist;
            },
        });

        it({
            id: "T03",
            title: "non-triumvirate cannot vote on track 0 — NotInVoterSet",
            test: async () => {
                // Submit a fresh poll first.
                const innerCall = api.tx.balances.forceSetBalance(eve.address, 7n);
                await context.createBlock([await api.tx.referenda.submit(0, { Action: innerCall }).signAsync(alice)]);

                const count = (await api.query.referenda.referendumCount()).toNumber();
                const newPoll = count - 1;

                // Eve is not in Triumvirate (she's not in any collective for track 0).
                await context.createBlock([await api.tx.signedVoting.vote(newPoll, true).signAsync(eve)]);

                const events = await api.query.system.events();
                const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
                expect(failed, "ExtrinsicFailed").to.exist;
            },
        });
    },
});
