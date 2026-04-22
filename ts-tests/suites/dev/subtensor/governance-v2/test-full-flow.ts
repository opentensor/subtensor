import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * End-to-end two-phase governance:
 *   1. Alice submits a plain call on track 0 — just `referenda.submit(0, call)`.
 *      No batch_all, no scheduler.scheduleNamed, no TaskName. Pallet handles everything.
 *   2. Triumvirate (Bob + Charlie) votes aye → `ApprovalAction::ScheduleAndReview { 1 }`
 *      fires: the call is scheduled as a named enactment task at `now + initial_delay`,
 *      and a Review poll is auto-spawned on track 1 with `submitter = None`.
 *   3. Economic collective (Eve + Ferdie) votes aye — 2/2 = 100% ≥ 75% fast_track.
 *   4. Scheduler executes the task on the next block; Ferdie balance changes.
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

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
            charlie = context.keyring.charlie;
            dave = context.keyring.dave;
            const sr = new Keyring({ type: "sr25519" });
            eve = sr.addFromUri("//Eve");
            ferdie = sr.addFromUri("//Ferdie");

            for (const inner of [
                api.tx.multiCollective.addMember("Proposers", alice.address),
                api.tx.multiCollective.addMember("Triumvirate", bob.address),
                api.tx.multiCollective.addMember("Triumvirate", charlie.address),
                api.tx.multiCollective.addMember("Triumvirate", dave.address),
                api.tx.multiCollective.addMember("Economic", eve.address),
                api.tx.multiCollective.addMember("Economic", ferdie.address),
            ]) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(alice)]);
            }
            const economic = await api.query.multiCollective.members("Economic");
            log(`Economic: ${economic.toJSON()}`);
            expect(economic.toJSON()).to.have.length(2);
        });

        it({
            id: "T01",
            title: "Alice submits call; triumvirate approves; collective fast-tracks; balance changes",
            test: async () => {
                const targetAmount = 2_000_000_000n;
                const countBefore = (
                    await api.query.referenda.referendumCount()
                ).toNumber();

                const payload = api.tx.balances.forceSetBalance(ferdie.address, targetAmount);

                await context.createBlock([
                    await api.tx.referenda.submit(0, payload).signAsync(alice),
                ]);
                const outerPoll = countBefore;

                // Triumvirate reaches 2/3 aye.
                await context.createBlock([
                    await api.tx.signedVoting.vote(outerPoll, true).signAsync(bob),
                ]);
                await context.createBlock([
                    await api.tx.signedVoting.vote(outerPoll, true).signAsync(charlie),
                ]);

                // In the same block as the second vote, `on_tally_updated` fires
                // `ScheduleAndReview`, which synchronously schedules the enactment task and
                // creates the Review poll on track 1.
                const approveEvents = await api.query.system.events();
                const approvedOuter = approveEvents.find(
                    (e) => e.event.section === "referenda" && e.event.method === "Approved",
                );
                expect(approvedOuter, "outer Approved").to.exist;

                const innerSubmitted = approveEvents.find((e) => {
                    if (e.event.section !== "referenda" || e.event.method !== "Submitted") {
                        return false;
                    }
                    const data = e.event.data as unknown as { track: any; submitter: any };
                    return data.track.toString() === "1" && data.submitter.isNone;
                });
                expect(innerSubmitted, "inner Submitted (track 1)").to.exist;

                const innerPoll = outerPoll + 1;
                const innerStatus = await api.query.referenda.referendumStatusFor(innerPoll);
                expect(innerStatus.isSome, "inner poll stored").to.be.true;

                // Economic collective votes aye (2/2 = 100% ≥ 75% fast_track).
                await context.createBlock([
                    await api.tx.signedVoting.vote(innerPoll, true).signAsync(eve),
                ]);
                await context.createBlock([
                    await api.tx.signedVoting.vote(innerPoll, true).signAsync(ferdie),
                ]);

                const fastTrackEvents = await api.query.system.events();
                const rescheduled = fastTrackEvents.find(
                    (e) =>
                        e.event.section === "referenda" &&
                        e.event.method === "TaskRescheduled",
                );
                expect(rescheduled, "TaskRescheduled on fast_track").to.exist;

                const fastTracked = fastTrackEvents.find(
                    (e) =>
                        e.event.section === "referenda" && e.event.method === "FastTracked",
                );
                expect(fastTracked, "inner FastTracked (new status)").to.exist;

                // Next block: named enactment task runs with Root.
                await context.createBlock([]);

                const finalEvents = await api.query.system.events();
                const dispatched = finalEvents.find(
                    (e) =>
                        e.event.section === "scheduler" && e.event.method === "Dispatched",
                );
                expect(dispatched, "scheduler.Dispatched").to.exist;

                const ferdieFinal = (
                    await api.query.system.account(ferdie.address)
                ).data.free.toBigInt();
                expect(ferdieFinal).to.equal(targetAmount);
            },
        });
    },
});
