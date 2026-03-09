import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair, type KeyringPair } from "@moonwall/util";
import { BN } from "@polkadot/util";

describeSuite({
    id: "DEV_SUB_STAKING_ADD_STAKING_01",
    title: "Add staking test suite",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let netuid1: number;

        let alice: KeyringPair;
        let bob: KeyringPair;
        let appAccount: KeyringPair;
        const appFees = new BN(100_000);

        beforeAll(() => {
            polkadotJs = context.polkadotJs();

            alice = context.keyring.alice;
            bob = context.keyring.bob;
            appAccount = generateKeyringPair("sr25519"); // some random app account
            console.log("appAccount", appAccount.address);
        });

        it({
            id: "T01",
            title: "Add stake payable",
            test: async () => {
                const alice = context.keyring.alice;
                const bob = context.keyring.bob;
                const appFees = new BN(100_000);

                // Register network
                let tx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await tx.signAsync(alice)]);

                let events = await polkadotJs.query.system.events();
                const event = events.filter((a) => {
                    return a.event.method === "NetworkAdded";
                });
                expect(event.length).to.be.equal(1);
                netuid1 = event[0].event.data[0];

                // Enabling subtokens
                const tx1 = polkadotJs.tx.adminUtils.sudoSetSubtokenEnabled(netuid1, true);
                await context.createBlock([await polkadotJs.tx.sudo.sudo(tx1).signAsync(alice)]);

                // Adding stake
                tx = polkadotJs.tx.subtensorModule.addStakePayable(
                    bob.address,
                    netuid1,
                    1000_000_000,
                    appAccount.address,
                    appFees
                );
                await context.createBlock([await tx.signAsync(alice)]);

                events = await polkadotJs.query.system.events();
                const stakeAddedEvent = events.filter((a) => {
                    return a.event.method === "StakeAdded";
                });

                const feeTransferredEvent = events.filter((a) => {
                    return a.event.method === "FeesTransferred";
                });

                expect(stakeAddedEvent.length).to.be.equal(1);
                expect(feeTransferredEvent.length).to.be.equal(1);

                const appAccountBalance = (
                    await polkadotJs.query.system.account(appAccount.address)
                ).data.free.toString();
                const appAccountBalanceBN = new BN(appAccountBalance);
                expect(appAccountBalanceBN.eq(appFees)).to.be.true;
            },
        });

        it({
            id: "T02",
            title: "Remove stake payable",
            test: async () => {
                // Removing stake
                const tx = polkadotJs.tx.subtensorModule.removeStakePayable(
                    bob.address,
                    netuid1,
                    500_000_000,
                    appAccount.address,
                    appFees
                );
                await context.createBlock([await tx.signAsync(alice)]);

                const events = await polkadotJs.query.system.events();
                const stakeAddedEvent = events.filter((a) => {
                    return a.event.method === "StakeRemoved";
                });

                const feeTransferredEvent = events.filter((a) => {
                    return a.event.method === "FeesTransferred";
                });

                expect(stakeAddedEvent.length).to.be.equal(1);
                expect(feeTransferredEvent.length).to.be.equal(1);

                const appAccountBalance = (
                    await polkadotJs.query.system.account(appAccount.address)
                ).data.free.toString();
                const appAccountBalanceBN = new BN(appAccountBalance);
                console.log(appAccountBalanceBN.toNumber());
                expect(appAccountBalanceBN.eq(appFees.add(appFees))).to.be.true; // We expect fees has been paid twice
            },
        });
    },
});
