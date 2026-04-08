import { beforeAll, expect } from "vitest";
import { describeSuite } from "@moonwall/cli";
import { generateKeyringPair, tao } from "../../../../utils";
import type { ApiPromise } from "@polkadot/api";
import { devForceSetBalance, devSetWeightsTx, devTryAssociateHotkey } from "../../../../utils/dev-helpers.ts";

describeSuite({
    id: "00_transaction_payment_wrapper_dev",
    title: "Transaction payment wrapper",
    foundationMethods: "dev",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(() => {
            api = context.polkadotJs();
        });

        it({
            id: "T01",
            title: "Check set_weights",
            test: async () => {
                const coldkey = generateKeyringPair("sr25519");
                const hotkey = generateKeyringPair("sr25519");

                log(`coldkey: ${coldkey.address}`);
                log(`hotkey: ${hotkey.address}`);

                const initialBalance = tao(1e10);

                log("Set Up");
                await devForceSetBalance(api, context, coldkey.address, initialBalance);
                await devForceSetBalance(api, context, hotkey.address, initialBalance);
                await devTryAssociateHotkey(api, context, coldkey, hotkey.address);

                const coldkeyBalanceBefore = (await api.query.system.account(coldkey.address)).data.free.toBigInt();

                log("Execute the tx from hotkey, but coldkey will pay");
                await devSetWeightsTx(api, context, hotkey, 0, [], [], 0n);

                const events = await api.query.system.events();
                const feeEvent = events.filter((a) => {
                    return a.event.method.toString() === "TransactionFeePaid";
                });

                const hotkeyBalance = (await api.query.system.account(hotkey.address)).data.free.toBigInt();
                const coldkeyBalanceAfter = (await api.query.system.account(coldkey.address)).data.free.toBigInt();
                // Fees paid by the hotkey
                const txFee = feeEvent[0].event.data.actualFee.toBigInt();
                expect(txFee).toBeGreaterThan(0n);
                expect(coldkeyBalanceAfter).toEqual(coldkeyBalanceBefore - txFee);
                expect(hotkeyBalance).toEqual(initialBalance);
            },
        });
    },
});
