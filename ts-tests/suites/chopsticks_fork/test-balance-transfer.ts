import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

// Second suite in the chopsticks_fork env. It shares the SAME pre-generated fork
// config (tmp/chopsticks-fork-slim.yml) that moonwall builds once via the env's
// `runScripts` step — so this file existing and passing alongside test-add-stake.ts
// confirms the pre-generation works and is reused across every test in the suite.
//
// Alice and Bob are both pre-funded in configs/chopsticks-fork.yml (import-storage).

const TRANSFER_AMOUNT = 100n * 1_000_000_000n; // 100 TAO (9 decimals)

async function getFreeBalance(api: ApiPromise, address: string): Promise<bigint> {
    const account = (await api.query.system.account(address)) as any;
    return BigInt(account.data.free.toString());
}

describeSuite({
    id: "CHOP_FORK_BALANCE_TRANSFER",
    title: "Chopsticks finney fork — balance transfer",
    foundationMethods: "chopsticks",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;
        let alice: KeyringPair;
        let bob: KeyringPair;

        beforeAll(async () => {
            api = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
        });

        it({
            id: "T01",
            title: "transferKeepAlive moves balance from Alice to Bob",
            test: async () => {
                const bobBefore = await getFreeBalance(api, bob.address);
                log(`Bob free balance before: ${bobBefore}`);

                await api.tx.balances.transferKeepAlive(bob.address, TRANSFER_AMOUNT).signAndSend(alice);

                // Seal the block; createChopsticksBlock throws on any ExtrinsicFailed event.
                await context.createBlock();

                const bobAfter = await getFreeBalance(api, bob.address);
                log(`Bob free balance after:  ${bobAfter}`);

                expect(bobAfter - bobBefore, "Bob should receive exactly the transferred amount").toBe(TRANSFER_AMOUNT);
            },
        });
    },
});
