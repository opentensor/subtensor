import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { disableAdminFreezeWindow } from "../rate_limiting/_utils.ts";
import { generateKeyringPair } from "../../../../utils";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_STAKING_ADD_STAKING_01",
    title: "Add staking test suite",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let netuid1: number;

        let alice: KeyringPair;
        let snOwner: KeyringPair;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();

            alice = context.keyring.alice;
            snOwner = generateKeyringPair("sr25519");
        });

        it({
            id: "T01",
            title: "Add stake payable",
            test: async () => {
                await context.createBlock([
                    await polkadotJs.tx.sudo
                        .sudo(polkadotJs.tx.balances.forceSetBalance(snOwner.address, 10_000_000_000))
                        .signAsync(alice),
                ]);

                await disableAdminFreezeWindow(polkadotJs, context, alice);

                // Register network
                let tx = polkadotJs.tx.subtensorModule.registerNetwork(snOwner.address);
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
                tx = polkadotJs.tx.subtensorModule.addStake(snOwner.address, netuid1, 1000_000_000);
                await context.createBlock([await tx.signAsync(alice)]);

                events = await polkadotJs.query.system.events();
                const stakeAddedEvent = events.filter((a) => {
                    return a.event.method === "StakeAdded";
                });

                expect(stakeAddedEvent.length).to.be.equal(1);
            },
        });

        it({
            id: "T02",
            title: "Remove stake payable",
            test: async () => {
                // We need enough blocks to bypass rate limit
                await seal(context, 10);

                // Removing stake
                const tx = polkadotJs.tx.subtensorModule.removeStake(snOwner.address, netuid1, 500_000_000);
                await context.createBlock([await tx.signAsync(alice)]);

                const events = await polkadotJs.query.system.events();
                const removeAddedEvent = events.filter((a) => {
                    return a.event.method === "StakeRemoved";
                });

                expect(removeAddedEvent.length).to.be.equal(1);
            },
        });
    },
});
