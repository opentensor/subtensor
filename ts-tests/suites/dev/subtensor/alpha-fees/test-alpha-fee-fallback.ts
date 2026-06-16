import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair, KeyringPair } from "@moonwall/util";
import { tao } from "../../../../utils";
import {
    devAddStake,
    devAssociateHotKey,
    devEnableSubtoken,
    devForceSetBalance,
    devGetAlphaStake,
    devRegisterSubnet,
    devSudoSetLockReductionInterval,
} from "../../../../utils/dev-helpers.ts";

describeSuite({
    id: "DEV_SUB_ALPHA_FEES",
    title: "Alpha transaction-fee fallback on remove_stake (proxy/batch unwrap)",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let alice: KeyringPair;
        let aliceHotKey: KeyringPair;
        let netuid: number;
        let ed: bigint;

        const STAKE = tao(100);

        const filterByMethod = (events: any, method: string): any[] =>
            (events as any[]).filter((e: any) => e.event.method === method);

        // Fund a fresh coldkey, stake to a fresh hotkey, then drain it down to the
        // existential deposit so it holds Alpha stake but cannot pay the fee in TAO.
        async function stakedColdkeyWithNoTao(): Promise<{
            coldkey: KeyringPair;
            hotkey: KeyringPair;
            alpha: bigint;
        }> {
            const coldkey = generateKeyringPair("sr25519");
            const hotkey = generateKeyringPair("sr25519");
            await devForceSetBalance(polkadotJs, context, coldkey.address, STAKE + tao(10));
            await devAssociateHotKey(polkadotJs, context, coldkey, hotkey.address);
            await devAddStake(polkadotJs, context, coldkey, hotkey.address, netuid, STAKE);
            const alpha = await devGetAlphaStake(polkadotJs, hotkey.address, coldkey.address, netuid);
            // Drain free TAO to ED: account stays alive but the fee cannot be paid in TAO.
            await devForceSetBalance(polkadotJs, context, coldkey.address, ed);
            return { coldkey, hotkey, alpha };
        }

        beforeAll(async () => {
            polkadotJs = context.polkadotJs();
            alice = context.keyring.alice;
            aliceHotKey = generateKeyringPair("sr25519");
            ed = BigInt(polkadotJs.consts.balances.existentialDeposit.toString());

            await devForceSetBalance(polkadotJs, context, alice.address, tao(1_000_000));
            await devSudoSetLockReductionInterval(polkadotJs, context, alice, 1);
            netuid = await devRegisterSubnet(polkadotJs, context, alice, aliceHotKey);
            await devEnableSubtoken(polkadotJs, context, alice, netuid);
        });

        it({
            id: "T01",
            title: "direct remove_stake with ~0 free TAO pays the fee in Alpha",
            test: async () => {
                const { coldkey, hotkey, alpha } = await stakedColdkeyWithNoTao();
                const unstake = alpha / 2n;

                const before = await devGetAlphaStake(polkadotJs, hotkey.address, coldkey.address, netuid);
                await context.createBlock([
                    await polkadotJs.tx.subtensorModule.removeStake(hotkey.address, netuid, unstake).signAsync(coldkey),
                ]);
                const after = await devGetAlphaStake(polkadotJs, hotkey.address, coldkey.address, netuid);

                const events = await polkadotJs.query.system.events();
                const alphaFee = filterByMethod(events, "TransactionFeePaidWithAlpha");

                expect(alphaFee.length).to.be.equal(1);
                expect(alphaFee[0].event.data[0].toString()).to.be.equal(coldkey.address);
                // Alpha spent exceeds the unstaked amount: the surplus is the fee paid in Alpha.
                expect(before - after > unstake).to.be.true;
            },
        });

        it({
            id: "T02",
            title: "batch([remove_stake]) with ~0 free TAO pays the fee in Alpha (batch unwrap)",
            test: async () => {
                const { coldkey, hotkey, alpha } = await stakedColdkeyWithNoTao();
                const unstake = alpha / 2n;

                const before = await devGetAlphaStake(polkadotJs, hotkey.address, coldkey.address, netuid);
                const inner = polkadotJs.tx.subtensorModule.removeStake(hotkey.address, netuid, unstake);
                await context.createBlock([await polkadotJs.tx.utility.batch([inner]).signAsync(coldkey)]);
                const after = await devGetAlphaStake(polkadotJs, hotkey.address, coldkey.address, netuid);

                const events = await polkadotJs.query.system.events();
                const alphaFee = filterByMethod(events, "TransactionFeePaidWithAlpha");

                expect(alphaFee.length).to.be.equal(1);
                expect(alphaFee[0].event.data[0].toString()).to.be.equal(coldkey.address);
                expect(before - after > unstake).to.be.true;
            },
        });

        it({
            id: "T03",
            title: "proxy(real, remove_stake) + RealPaysFee charges the fee to real's Alpha",
            test: async () => {
                // real holds the stake and opts in to pay; delegate signs with ~0 TAO.
                const real = generateKeyringPair("sr25519");
                const hotkey = generateKeyringPair("sr25519");
                const delegate = generateKeyringPair("sr25519");

                await devForceSetBalance(polkadotJs, context, real.address, STAKE + tao(10));
                await devAssociateHotKey(polkadotJs, context, real, hotkey.address);
                await devAddStake(polkadotJs, context, real, hotkey.address, netuid, STAKE);
                const alpha = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);

                // Grant proxy and opt in to RealPaysFee (both signed by real, while funded).
                await context.createBlock([
                    await polkadotJs.tx.proxy.addProxy(delegate.address, "Any", 0).signAsync(real),
                ]);
                await context.createBlock([
                    await polkadotJs.tx.proxy.setRealPaysFee(delegate.address, true).signAsync(real),
                ]);

                // The delegate needs a bare account (ED) so the custom CheckNonce does not
                // reject it before the fee-payer wrapper runs; it still cannot pay in TAO.
                await devForceSetBalance(polkadotJs, context, delegate.address, ed);
                // Drain real so the fee must come from Alpha.
                await devForceSetBalance(polkadotJs, context, real.address, ed);

                const unstake = alpha / 2n;
                const before = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);
                const inner = polkadotJs.tx.subtensorModule.removeStake(hotkey.address, netuid, unstake);
                await context.createBlock([
                    await polkadotJs.tx.proxy.proxy(real.address, null, inner).signAsync(delegate),
                ]);
                const after = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);

                const events = await polkadotJs.query.system.events();
                const alphaFee = filterByMethod(events, "TransactionFeePaidWithAlpha");

                expect(alphaFee.length).to.be.equal(1);
                // The fee is charged to real, not the signing delegate.
                expect(alphaFee[0].event.data[0].toString()).to.be.equal(real.address);
                expect(before - after > unstake).to.be.true;
            },
        });

        it({
            id: "T04",
            title: "proxy(real, remove_stake) WITHOUT RealPaysFee is rejected (delegate pays, no funds)",
            test: async () => {
                const real = generateKeyringPair("sr25519");
                const hotkey = generateKeyringPair("sr25519");
                const delegate = generateKeyringPair("sr25519");

                await devForceSetBalance(polkadotJs, context, real.address, STAKE + tao(10));
                await devAssociateHotKey(polkadotJs, context, real, hotkey.address);
                await devAddStake(polkadotJs, context, real, hotkey.address, netuid, STAKE);
                const alpha = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);

                // Grant proxy but do NOT opt in to RealPaysFee → the delegate is the fee payer.
                await context.createBlock([
                    await polkadotJs.tx.proxy.addProxy(delegate.address, "Any", 0).signAsync(real),
                ]);
                // Give the delegate a bare account (ED) so it is the CheckNonce that passes and
                // the rejection is attributable to the fee payer having no funds/Alpha.
                await devForceSetBalance(polkadotJs, context, delegate.address, ed);
                await devForceSetBalance(polkadotJs, context, real.address, ed);

                const unstake = alpha / 2n;
                const before = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);
                const inner = polkadotJs.tx.subtensorModule.removeStake(hotkey.address, netuid, unstake);

                // Validation rejects the tx (delegate has neither TAO nor Alpha on this hotkey).
                let rejected = false;
                try {
                    await context.createBlock([
                        await polkadotJs.tx.proxy.proxy(real.address, null, inner).signAsync(delegate),
                    ]);
                } catch (_e) {
                    rejected = true;
                }

                const after = await devGetAlphaStake(polkadotJs, hotkey.address, real.address, netuid);
                const events = await polkadotJs.query.system.events();
                const alphaFee = filterByMethod(events, "TransactionFeePaidWithAlpha");

                // Either the submission was rejected, or no fee/unstake happened — real's
                // stake is unchanged and no Alpha fee was charged.
                expect(rejected || (after === before && alphaFee.length === 0)).to.be.true;
            },
        });
    },
});
