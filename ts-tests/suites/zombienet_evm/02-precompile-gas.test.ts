import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import { ethers } from "ethers";
import type { TypedApi } from "polkadot-api";
import {
    convertH160ToSS58,
    createEthersWallet,
    disableWhiteListCheck,
    forceSetBalance,
    getBalance,
    PRECOMPILE_GAS_CONTRACT_ABI,
    PRECOMPILE_GAS_CONTRACT_BYTECODE,
    waitForFinalizedBlocks,
} from "../../utils";

const MIN_PRECOMPILE_GAS = BigInt(6000);
const MAX_PRECOMPILE_GAS = BigInt(10000);
const ITERATION_COUNTS = [1, 11, 101] as const;

async function assertPrecompileGasScaling(
    api: TypedApi<typeof subtensor>,
    contract: ethers.Contract,
    wallet: ethers.Wallet,
    call: (iterations: number) => Promise<ethers.ContractTransactionResponse>,
    baseFee: bigint
): Promise<void> {
    let oneIterationGas = BigInt(0);

    for (const iterations of ITERATION_COUNTS) {
        const balanceBefore = await getBalance(api, convertH160ToSS58(wallet.address));
        const tx = await call(iterations);
        await tx.wait();
        await waitForFinalizedBlocks(api, 1);

        const balanceAfter = await getBalance(api, convertH160ToSS58(wallet.address));
        expect(balanceAfter).toBeLessThan(balanceBefore);

        const usedGas = balanceBefore - balanceAfter;
        if (iterations === 1) {
            oneIterationGas = usedGas;
            continue;
        }

        expect(usedGas >= oneIterationGas).toBe(true);

        const precompileUsedGas = usedGas - oneIterationGas;
        const minExpected = MIN_PRECOMPILE_GAS * BigInt(iterations - 1) * baseFee;
        const maxExpected = MAX_PRECOMPILE_GAS * BigInt(iterations - 1) * baseFee;

        expect(precompileUsedGas >= minExpected).toBe(true);
        expect(precompileUsedGas <= maxExpected).toBe(true);
    }
}

describeSuite({
    id: "precompile-gas",
    title: "SR25519 and ED25519 precompile gas tests",
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
            title: "Can deploy and call precompile gas contract",
            test: async () => {
                const fee = await api.query.BaseFee.BaseFeePerGas.getValue();
                expect(fee[0]).toBeGreaterThan(1_000_000_000);
                const baseFee = BigInt(fee[0]) / BigInt(1_000_000_000);

                const contractFactory = new ethers.ContractFactory(
                    PRECOMPILE_GAS_CONTRACT_ABI,
                    PRECOMPILE_GAS_CONTRACT_BYTECODE,
                    ethWallet
                );
                const contractDeploy = await contractFactory.deploy();
                await contractDeploy.waitForDeployment();
                await waitForFinalizedBlocks(api, 1);

                const contractAddress = await contractDeploy.getAddress();
                expect(contractAddress).toBeDefined();

                const contract = new ethers.Contract(contractAddress, PRECOMPILE_GAS_CONTRACT_ABI, ethWallet);

                await assertPrecompileGasScaling(api, contract, ethWallet, (iterations) => contract.callED25519(iterations), baseFee);
                await assertPrecompileGasScaling(api, contract, ethWallet, (iterations) => contract.callSR25519(iterations), baseFee);
            },
        });
    },
});
