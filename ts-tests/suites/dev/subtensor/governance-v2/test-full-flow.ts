import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * End-to-end two-phase governance flow per DESIGN.md:
 *   1. Alice submits an `Action` on Track 0 whose call is `utility.batchAll([
 *        scheduler.scheduleNamed(task, when, ..., forceSetBalance(...)),
 *        referenda.submit(1, Review(task)),
 *      ])`.
 *   2. Triumvirate (Bob + Charlie) votes aye → batch executes with Root.
 *   3. The scheduleNamed puts a task in scheduler; the inner submit creates a Review poll
 *      on Track 1 with `submitter = None` (root-initiated).
 *   4. Economic collective (Eve + Ferdie) votes aye — 2/2 = 100% ≥ 75% fast_track threshold.
 *   5. Scheduler dispatches the pushed-forward task; balance changes.
 */
describeSuite({
    id: "DEV_SUB_GOVV2_FULLFLOW_01",
    title: "Governance V2 — full two-phase flow (track 0 + track 1)",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        let alice: KeyringPair;
        let bob: KeyringPair;
        let charlie: KeyringPair;
        let dave: KeyringPair;
        let eve: KeyringPair;
        let ferdie: KeyringPair;

        // 32-byte identifier used by scheduler.scheduleNamed and referenced by the Review poll.
        const TASK_ID = "0x" + "01".repeat(32);

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
            charlie = context.keyring.charlie;
            dave = context.keyring.dave;
            // Moonwall's keyring only has alice/bob/charlie/dave — construct eve/ferdie.
            const sr = new Keyring({ type: "sr25519" });
            eve = sr.addFromUri("//Eve");
            ferdie = sr.addFromUri("//Ferdie");

            // Collectives setup.
            const adds = [
                api.tx.multiCollective.addMember("Proposers", alice.address),
                api.tx.multiCollective.addMember("Triumvirate", bob.address),
                api.tx.multiCollective.addMember("Triumvirate", charlie.address),
                api.tx.multiCollective.addMember("Triumvirate", dave.address),
                api.tx.multiCollective.addMember("Economic", eve.address),
                api.tx.multiCollective.addMember("Economic", ferdie.address),
            ];
            for (const inner of adds) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(alice)]);
            }

            const economic = await api.query.multiCollective.members("Economic");
            log(`Economic: ${economic.toJSON()}`);
            expect(economic.toJSON()).to.have.length(2);
        });

        it({
            id: "T01",
            title: "Alice submits batch on track 0; triumvirate approves; track 1 poll is auto-created",
            test: async () => {
                const targetAmount = 2_000_000_000n;
                const header = await api.rpc.chain.getHeader();
                const currentBlock = header.number.toNumber();
                const whenBlock = currentBlock + 200;

                // Inner payload — a Root-only call that will be scheduled as a named task.
                const payload = api.tx.balances.forceSetBalance(ferdie.address, targetAmount);
                const scheduleCall = api.tx.scheduler.scheduleNamed(TASK_ID, whenBlock, null, 0, payload);
                const submitReview = api.tx.referenda.submit(1, { Review: TASK_ID });
                const batchCall = api.tx.utility.batchAll([scheduleCall, submitReview]);

                const submitAction = api.tx.referenda.submit(0, { Action: batchCall });

                const countBefore = (await api.query.referenda.referendumCount()).toNumber();

                await context.createBlock([await submitAction.signAsync(alice)]);

                const submitted = (await api.query.system.events()).find(
                    (e) => e.event.section === "referenda" && e.event.method === "Submitted"
                );
                expect(submitted, "outer Submitted").to.exist;

                const outerPoll = countBefore; // index of the just-created poll

                // Triumvirate votes aye (2/3 → approved).
                await context.createBlock([await api.tx.signedVoting.vote(outerPoll, true).signAsync(bob)]);
                await context.createBlock([await api.tx.signedVoting.vote(outerPoll, true).signAsync(charlie)]);

                const approvedOuter = (await api.query.system.events()).find(
                    (e) => e.event.section === "referenda" && e.event.method === "Approved"
                );
                expect(approvedOuter, "outer Approved").to.exist;

                // Next block: scheduler dispatches the batch → schedule_named + inner submit run.
                await context.createBlock([]);

                const eventsAfterBatch = await api.query.system.events();
                const batchDispatched = eventsAfterBatch.find(
                    (e) => e.event.section === "scheduler" && e.event.method === "Dispatched"
                );
                expect(batchDispatched, "batch dispatched").to.exist;

                // A new poll must have been submitted on track 1 with submitter = None (Root).
                const innerSubmitted = eventsAfterBatch.find((e) => {
                    if (e.event.section !== "referenda" || e.event.method !== "Submitted") {
                        return false;
                    }
                    const data = e.event.data as unknown as { track: any; submitter: any };
                    // Track is the 2nd field; submitter is Option<AccountId>.
                    return data.track.toString() === "1" && data.submitter.isNone;
                });
                expect(innerSubmitted, "inner Submitted (track 1, submitter=None)").to.exist;

                const countAfter = (await api.query.referenda.referendumCount()).toNumber();
                expect(countAfter).to.equal(countBefore + 2);

                const innerPoll = outerPoll + 1;
                const innerStatus = await api.query.referenda.referendumStatusFor(innerPoll);
                expect(innerStatus.isSome, "inner poll storage").to.be.true;

                // Collective votes fast_track — both members aye → 100% ≥ 75%.
                await context.createBlock([await api.tx.signedVoting.vote(innerPoll, true).signAsync(eve)]);
                await context.createBlock([await api.tx.signedVoting.vote(innerPoll, true).signAsync(ferdie)]);

                const afterFastTrack = await api.query.system.events();
                const reschedEvent = afterFastTrack.find(
                    (e) => e.event.section === "referenda" && e.event.method === "TaskRescheduled"
                );
                expect(reschedEvent, "TaskRescheduled on fast_track").to.exist;

                const approvedInner = afterFastTrack.find(
                    (e) => e.event.section === "referenda" && e.event.method === "Approved"
                );
                expect(approvedInner, "inner Approved (track 1)").to.exist;

                // Next block: the task executes with Root.
                await context.createBlock([]);

                const ferdieFinal = (await api.query.system.account(ferdie.address)).data.free.toBigInt();
                expect(ferdieFinal).to.equal(targetAmount);
            },
        });
    },
});
