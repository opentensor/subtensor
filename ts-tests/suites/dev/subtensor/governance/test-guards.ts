import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair } from "../../../../utils/account";

describeSuite({
    id: "DEV_SUB_GOVV2_GUARDS_01",
    title: "Governance V2 — validation guards",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;
        let sudoer: KeyringPair;

        const proposer = generateKeyringPair("sr25519");
        const triumvirate1 = generateKeyringPair("sr25519");
        const triumvirate2 = generateKeyringPair("sr25519");
        const outsider = generateKeyringPair("sr25519");

        beforeAll(async () => {
            api = context.polkadotJs();
            sudoer = context.keyring.alice;

            const fund = 1_000_000_000_000n;
            for (const inner of [
                api.tx.balances.forceSetBalance(proposer.address, fund),
                api.tx.balances.forceSetBalance(triumvirate1.address, fund),
                api.tx.balances.forceSetBalance(triumvirate2.address, fund),
                api.tx.balances.forceSetBalance(outsider.address, fund),
                api.tx.multiCollective.addMember("Proposers", proposer.address),
                api.tx.multiCollective.addMember("Triumvirate", triumvirate1.address),
                api.tx.multiCollective.addMember("Triumvirate", triumvirate2.address),
            ]) {
                await context.createBlock([await api.tx.sudo.sudo(inner).signAsync(sudoer)]);
            }
        });

        const extrinsicFailed = async () => {
            const events = await api.query.system.events();
            const failed = events.find((e) => e.event.section === "system" && e.event.method === "ExtrinsicFailed");
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
            title: "submit on track 1 by non-proposer (Triumvirate-only) → NotProposer",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(outsider.address, 1n);
                await context.createBlock([await api.tx.referenda.submit(1, inner).signAsync(triumvirate1)]);

                const err = await extrinsicFailed();
                log(`error: ${JSON.stringify(err)}`);
                expect(err).not.to.be.null;
                expect(err?.section).to.equal("referenda");
                expect(err?.name).to.equal("NotProposer");
            },
        });

        it({
            id: "T02",
            title: "submit on unknown track → BadTrack",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(outsider.address, 2n);
                await context.createBlock([await api.tx.referenda.submit(99, inner).signAsync(proposer)]);

                const err = await extrinsicFailed();
                expect(err).not.to.be.null;
                expect(err?.section).to.equal("referenda");
                expect(err?.name).to.equal("BadTrack");
            },
        });

        it({
            id: "T03",
            title: "duplicate vote → DuplicateVote; vote switch → ok",
            test: async () => {
                const inner = api.tx.balances.forceSetBalance(outsider.address, 3n);
                await context.createBlock([await api.tx.referenda.submit(0, inner).signAsync(proposer)]);
                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                await context.createBlock([await api.tx.signedVoting.vote(poll, true).signAsync(triumvirate1)]);

                await context.createBlock([await api.tx.signedVoting.vote(poll, true).signAsync(triumvirate1)]);
                const dup = await extrinsicFailed();
                expect(dup?.section).to.equal("signedVoting");
                expect(dup?.name).to.equal("DuplicateVote");

                await context.createBlock([await api.tx.signedVoting.vote(poll, false).signAsync(triumvirate1)]);
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
                const inner = api.tx.balances.forceSetBalance(outsider.address, 4n);
                await context.createBlock([await api.tx.referenda.submit(0, inner).signAsync(proposer)]);
                const poll = (await api.query.referenda.referendumCount()).toNumber() - 1;

                await context.createBlock([await api.tx.signedVoting.removeVote(poll).signAsync(triumvirate2)]);

                const err = await extrinsicFailed();
                expect(err?.section).to.equal("signedVoting");
                expect(err?.name).to.equal("VoteNotFound");
            },
        });
    },
});
