import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { Keyring } from "@polkadot/keyring";

/**
 * Track 0 (PassOrFail) + Track 1 auto-spawn:
 *   - `referenda.submit(0, call)` takes a plain call; the pallet picks the proposal form
 *     from `DecisionStrategy`. Track 0 is configured with `on_approval =
 *     ScheduleAndReview { review_track: 1 }`, so approval auto-spawns a track-1 poll.
 *   - This suite covers the happy path submission plus two negative origin cases.
 *
 * T01 exercises triumvirate approval and verifies the auto-spawned Review poll appears on
 * track 1 with `submitter = None`. End-to-end balance change is exercised in test-full-flow.
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

            // Alice = Proposer; Bob/Charlie/Dave = Triumvirate.
            for (const inner of [
                api.tx.multiCollective.addMember("Proposers", alice.address),
                api.tx.multiCollective.addMember("Triumvirate", bob.address),
                api.tx.multiCollective.addMember("Triumvirate", charlie.address),
                api.tx.multiCollective.addMember("Triumvirate", dave.address),
            ]) {
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
            title: "submit on track 0; 2-of-3 ayes → Approved + auto-spawned track 1 poll",
            test: async () => {
                const innerCall = api.tx.balances.forceSetBalance(eve.address, 1_000_000_000n);
                const countBefore = (
                    await api.query.referenda.referendumCount()
                ).toNumber();

                await context.createBlock([
                    await api.tx.referenda.submit(0, innerCall).signAsync(alice),
                ]);

                const submittedOuter = (await api.query.system.events()).find(
                    (e) =>
                        e.event.section === "referenda" && e.event.method === "Submitted",
                );
                expect(submittedOuter, "outer Submitted").to.exist;

                const outerPoll = countBefore;

                // Bob aye → 1/3.
                await context.createBlock([
                    await api.tx.signedVoting.vote(outerPoll, true).signAsync(bob),
                ]);

                // Charlie aye → 2/3 = `Perbill::from_rational(2, 3)` — exact threshold match.
                await context.createBlock([
                    await api.tx.signedVoting.vote(outerPoll, true).signAsync(charlie),
                ]);

                const eventsAfterApprove = await api.query.system.events();
                const approvedOuter = eventsAfterApprove.find(
                    (e) => e.event.section === "referenda" && e.event.method === "Approved",
                );
                expect(approvedOuter, "outer Approved").to.exist;

                // ScheduleAndReview fires inline with on_tally_updated → a new Review poll
                // on track 1 with submitter=None should appear in the SAME block.
                const innerSubmitted = eventsAfterApprove.find((e) => {
                    if (e.event.section !== "referenda" || e.event.method !== "Submitted") {
                        return false;
                    }
                    const data = e.event.data as unknown as { track: any; submitter: any };
                    return data.track.toString() === "1" && data.submitter.isNone;
                });
                expect(innerSubmitted, "inner Submitted (track 1, submitter=None)").to.exist;

                const countAfter = (
                    await api.query.referenda.referendumCount()
                ).toNumber();
                expect(countAfter).to.equal(countBefore + 2);
            },
        });

        it({
            id: "T02",
            title: "non-proposer submit → BadOrigin via SubmitOrigin",
            test: async () => {
                // Dave is in Triumvirate but NOT in Proposers → SubmitOrigin returns Err.
                const innerCall = api.tx.balances.forceSetBalance(eve.address, 42n);

                await context.createBlock([
                    await api.tx.referenda.submit(0, innerCall).signAsync(dave),
                ]);

                const failed = (await api.query.system.events()).find(
                    (e) =>
                        e.event.section === "system" && e.event.method === "ExtrinsicFailed",
                );
                expect(failed, "ExtrinsicFailed on non-proposer submit").to.exist;
            },
        });

        it({
            id: "T03",
            title: "non-triumvirate cannot vote on track 0 — NotInVoterSet",
            test: async () => {
                const innerCall = api.tx.balances.forceSetBalance(eve.address, 7n);
                await context.createBlock([
                    await api.tx.referenda.submit(0, innerCall).signAsync(alice),
                ]);

                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                // Eve not in Triumvirate → vote rejected.
                await context.createBlock([
                    await api.tx.signedVoting.vote(poll, true).signAsync(eve),
                ]);

                const failed = (await api.query.system.events()).find(
                    (e) =>
                        e.event.section === "system" && e.event.method === "ExtrinsicFailed",
                );
                expect(failed, "ExtrinsicFailed on non-voter").to.exist;
            },
        });
    },
});
