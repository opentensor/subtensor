import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import { ethers } from "ethers";
import type { TypedApi } from "polkadot-api";
import {
    convertH160ToSS58,
    createEthersWallet,
    disableWhiteListCheck,
    forceSetBalance,
    ISTAKING_V2_ADDRESS,
    IStakingV2ABI,
    waitForFinalizedBlocks,
} from "../../utils";

describeSuite({
    id: "claim-root-precompile",
    title: "Staking V2 precompile: claimRoot",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;
        let ethWallet: ethers.Wallet;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
            const provider = context.ethers("EVM").provider as ethers.JsonRpcProvider;
            ethWallet = createEthersWallet(provider);

            await forceSetBalance(api, convertH160ToSS58(ethWallet.address));
            await disableWhiteListCheck(api, true);
            await waitForFinalizedBlocks(api, 1);
        }, 300000);

        it({
            id: "T01",
            title: "claimRoot self-claim dispatches successfully (no-op for an unstaked caller)",
            test: async () => {
                const staking = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, ethWallet);

                // The precompile dispatches the self `claim_root` under the caller's derived
                // coldkey.
                const tx = await staking.claimRoot([1, 2, 3]);
                const receipt = await tx.wait();
                expect(receipt?.status).toBe(1);
            },
        });
    },
});
