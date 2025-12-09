import * as assert from "assert";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { getBalance, getDevnetApi } from "../src/substrate";
import { forceSetBalanceToEthAddress } from "../src/subtensor";
import { PrecompileGas_CONTRACT_ABI, PrecompileGas_CONTRACT_BYTECODE } from "../src/contracts/precompileGas";
import { ethers } from "ethers";
import { TypedApi } from "polkadot-api";
import { devnet } from "@polkadot-api/descriptors";
import { disableWhiteListCheck } from "../src/subtensor";
import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils";

describe("SR25519 ED25519 Precompile Gas Test", () => {
    const wallet = generateRandomEthersWallet();
    let api: TypedApi<typeof devnet>;

    // scope of precompile gas usage for sr25519 and ed25519
    const minPrecompileGas = BigInt(6000);
    const maxPrecompileGas = BigInt(10000);

    before(async () => {
        api = await getDevnetApi();
        await forceSetBalanceToEthAddress(api, wallet.address);
        await disableWhiteListCheck(api, true);
    });

    it("Can deploy and call attackHardcoded", async () => {
        const fee = await api.query.BaseFee.BaseFeePerGas.getValue()
        assert.ok(fee[0] > 1000000000);
        const baseFee = BigInt(fee[0]) / BigInt(1000000000);
        console.log("Base fee per gas:", baseFee);

        const contractFactory = new ethers.ContractFactory(PrecompileGas_CONTRACT_ABI, PrecompileGas_CONTRACT_BYTECODE, wallet);
        const contractDeploy = await contractFactory.deploy();

        const result = await contractDeploy.waitForDeployment();
        console.log("Contract deployed to:", result.target);


        let oneIterationGas = BigInt(0);

        for (const iter of [1, 11, 101]) {
            const balanceBefore = await getBalance(api, convertH160ToSS58(wallet.address));
            const contract = new ethers.Contract(result.target, PrecompileGas_CONTRACT_ABI, wallet);
            const iterations = iter;
            const tx = await contract.callED25519(iterations)
            await tx.wait()

            const balanceAfter = await getBalance(api, convertH160ToSS58(wallet.address));
            assert.ok(balanceAfter < balanceBefore);

            const usedGas = balanceBefore - balanceAfter;
            if (iterations === 1) {
                oneIterationGas = usedGas;
                continue;
            }

            assert.ok(usedGas >= oneIterationGas);

            const precompileUsedGas = BigInt(usedGas - oneIterationGas);
            assert.ok(precompileUsedGas >= minPrecompileGas * BigInt(iterations - 1) * baseFee);
            assert.ok(precompileUsedGas <= maxPrecompileGas * BigInt(iterations - 1) * baseFee);
        }

        for (const iter of [1, 11, 101]) {
            const balanceBefore = await getBalance(api, convertH160ToSS58(wallet.address));
            const contract = new ethers.Contract(result.target, PrecompileGas_CONTRACT_ABI, wallet);
            const iterations = iter;
            const tx = await contract.callSR25519(iterations)
            await tx.wait()

            const balanceAfter = await getBalance(api, convertH160ToSS58(wallet.address));
            assert.ok(balanceAfter < balanceBefore);

            const usedGas = balanceBefore - balanceAfter;
            if (iterations === 1) {
                oneIterationGas = usedGas;
                continue;
            }

            assert.ok(usedGas >= oneIterationGas);

            const precompileUsedGas = BigInt(usedGas - oneIterationGas);
            assert.ok(precompileUsedGas >= minPrecompileGas * BigInt(iterations - 1) * baseFee);
            assert.ok(precompileUsedGas <= maxPrecompileGas * BigInt(iterations - 1) * baseFee);
        }
    });
});
