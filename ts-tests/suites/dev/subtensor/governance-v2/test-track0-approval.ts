import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair } from "../../../../utils/account";

describeSuite({
    id: "DEV_SUB_GOVV2_TRACK0_01",
    title: "Governance V2 — Track 0 PassOrFail approval",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;
        let sudoer: KeyringPair;

        const proposer = generateKeyringPair("sr25519");
        const triumvirate1 = generateKeyringPair("sr25519");
        const triumvirate2 = generateKeyringPair("sr25519");
        const triumvirate3 = generateKeyringPair("sr25519");
        const outsider = generateKeyringPair("sr25519");

        beforeAll(async () => {
            api = context.polkadotJs();
            sudoer = context.keyring.alice;

            const fund = 1_000_000_000_000n;
            for (const inner of [
                api.tx.balances.forceSetBalance(proposer.address, fund),
                api.tx.balances.forceSetBalance(triumvirate1.address, fund),
                api.tx.balances.forceSetBalance(triumvirate2.address, fund),
                api.tx.balances.forceSetBalance(triumvirate3.address, fund),
                api.tx.balances.forceSetBalance(outsider.address, fund),
                api.tx.multiCollective.addMember("Proposers", proposer.address),
                api.tx.multiCollective.addMember("Triumvirate", triumvirate1.address),
                api.tx.multiCollective.addMember("Triumvirate", triumvirate2.address),
                api.tx.multiCollective.addMember("Triumvirate", triumvirate3.address),
            ]) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(sudoer)]);
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
            title: "submit on track 0; 2-of-3 ayes → Delegated + auto-created track 1 poll",
            test: async () => {
                const innerCall = api.tx.balances.forceSetBalance(outsider.address, 1_000_000_000n);
                const countBefore = (await api.query.referenda.referendumCount()).toNumber();

                await context.createBlock([await api.tx.referenda.submit(0, innerCall).signAsync(proposer)]);

                const submittedOuter = (await api.query.system.events()).find(
                    (e) => e.event.section === "referenda" && e.event.method === "Submitted"
                );
                expect(submittedOuter, "outer Submitted").to.exist;

                const outerPoll = countBefore;

                // 1st aye → 1/3.
                await context.createBlock([await api.tx.signedVoting.vote(outerPoll, true).signAsync(triumvirate1)]);

                // 2nd aye → 2/3 = `Perbill::from_rational(2, 3)` — exact threshold match.
                await context.createBlock([await api.tx.signedVoting.vote(outerPoll, true).signAsync(triumvirate2)]);

                await context.createBlock([]);

                const eventsAfterApprove = await api.query.system.events();
                const delegatedOuter = eventsAfterApprove.find(
                    (e) => e.event.section === "referenda" && e.event.method === "Delegated"
                );
                expect(delegatedOuter, "outer Delegated event").to.exist;

                const delegatedData = delegatedOuter?.event.data as unknown as {
                    index: any;
                    review: any;
                    track: any;
                };
                expect(delegatedData.index.toString()).to.equal(outerPoll.toString());
                expect(delegatedData.track.toString()).to.equal("1");

                const outerStatus = await api.query.referenda.referendumStatusFor(outerPoll);
                expect(outerStatus.toJSON()).to.have.property("delegated");

                const innerPoll = outerPoll + 1;
                const innerStatus = await api.query.referenda.referendumStatusFor(innerPoll);
                expect(innerStatus.isSome, "inner poll stored").to.be.true;
                expect(innerStatus.toJSON()).to.have.property("ongoing");

                const countAfter = (await api.query.referenda.referendumCount()).toNumber();
                expect(countAfter).to.equal(countBefore + 2);
            },
        });

        it({
            id: "T02",
            title: "non-proposer submit → NotProposer module error",
            test: async () => {
                const innerCall = api.tx.balances.forceSetBalance(outsider.address, 42n);

                await context.createBlock([await api.tx.referenda.submit(0, innerCall).signAsync(triumvirate3)]);

                const events = await api.query.system.events();
                const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
                expect(failed, "ExtrinsicFailed on non-proposer submit").to.exist;

                const dispatchError = failed?.event.data[0] as any;
                expect(dispatchError.isModule, "expect module error").to.be.true;
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                expect(decoded.section).to.equal("referenda");
                expect(decoded.name).to.equal("NotProposer");
            },
        });

        it({
            id: "T03",
            title: "non-triumvirate cannot vote on track 0 — NotInVoterSet",
            test: async () => {
                const innerCall = api.tx.balances.forceSetBalance(outsider.address, 7n);
                await context.createBlock([await api.tx.referenda.submit(0, innerCall).signAsync(proposer)]);

                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                // outsider not in Triumvirate → vote rejected.
                await context.createBlock([await api.tx.signedVoting.vote(poll, true).signAsync(outsider)]);

                const events = await api.query.system.events();
                const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
                expect(failed, "ExtrinsicFailed on non-voter").to.exist;

                const dispatchError = failed?.event.data[0] as any;
                expect(dispatchError.isModule).to.be.true;
                const decoded = api.registry.findMetaError(dispatchError.asModule);
                expect(decoded.section).to.equal("signedVoting");
                expect(decoded.name).to.equal("NotInVoterSet");
            },
        });
    },
});
