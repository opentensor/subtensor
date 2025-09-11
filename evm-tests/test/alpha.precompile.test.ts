import * as assert from "assert";

import { getAliceSigner, getDevnetApi, waitForTransactionCompletion, convertPublicKeyToMultiAddress, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IAlphaABI, IALPHA_ADDRESS } from "../src/contracts/alpha"
import { u64 } from "@polkadot-api/substrate-bindings";

describe("Test Alpha Precompile", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>;

    // sudo account alice as signer
    let alice: PolkadotSigner;

    // init other variable
    let subnetId = 0;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        // Fund the hotkey account
        {
            const multiAddress = convertPublicKeyToMultiAddress(hotkey.publicKey)
            const internalCall = api.tx.Balances.force_set_balance({ who: multiAddress, new_free: BigInt(1e12) })
            const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

            await waitForTransactionCompletion(api, tx, alice)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }

        // Fund the coldkey account
        {
            const multiAddress = convertPublicKeyToMultiAddress(coldkey.publicKey)
            const internalCall = api.tx.Balances.force_set_balance({ who: multiAddress, new_free: BigInt(1e12) })
            const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

            await waitForTransactionCompletion(api, tx, alice)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }

        // Register a new subnet
        const signer = getSignerFromKeypair(coldkey)
        const registerNetworkTx = api.tx.SubtensorModule.register_network({ hotkey: convertPublicKeyToSs58(hotkey.publicKey) })
        await waitForTransactionCompletion(api, registerNetworkTx, signer)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });

        // Get the newly created subnet ID
        let totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()
        assert.ok(totalNetworks > 1)
        subnetId = totalNetworks - 1

        // Register a neuron on the subnet if needed
        let uid_count = await api.query.SubtensorModule.SubnetworkN.getValue(subnetId)
        if (uid_count === 0) {
            const tx = api.tx.SubtensorModule.burned_register({ hotkey: convertPublicKeyToSs58(hotkey.publicKey), netuid: subnetId })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }
    })

    describe("Alpha Price Functions", () => {
        it("getAlphaPrice returns valid price for subnet", async () => {
            const alphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaPrice",
                args: [subnetId]
            })

            assert.ok(alphaPrice !== undefined, "Alpha price should be defined");
            assert.ok(typeof alphaPrice === 'bigint', "Alpha price should be a bigint");
            assert.ok(alphaPrice >= BigInt(0), "Alpha price should be non-negative");
        });

        it("getMovingAlphaPrice returns valid moving price for subnet", async () => {
            const movingAlphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getMovingAlphaPrice",
                args: [subnetId]
            })

            assert.ok(movingAlphaPrice !== undefined, "Moving alpha price should be defined");
            assert.ok(typeof movingAlphaPrice === 'bigint', "Moving alpha price should be a bigint");
            assert.ok(movingAlphaPrice >= BigInt(0), "Moving alpha price should be non-negative");
        });

        it("alpha prices are consistent for same subnet", async () => {
            const alphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaPrice",
                args: [subnetId]
            })

            const movingAlphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getMovingAlphaPrice",
                args: [subnetId]
            })

            // Both should be defined and valid
            assert.ok(alphaPrice !== undefined && movingAlphaPrice !== undefined);
        });

        it("Tao in / Alpha in / Alpha out are consistent for same subnet", async () => {
            const taoInEmission = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getTaoInEmission",
                args: [subnetId]
            })

            const alphaInEmission = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaInEmission",
                args: [subnetId]
            })

            const alphaOutEmission = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaOutEmission",
                args: [subnetId]
            })

            // all should be defined and valid
            assert.ok(taoInEmission !== undefined && alphaInEmission !== undefined && alphaOutEmission !== undefined);
        });

        it("getSumAlphaPrice returns valid sum of alpha prices", async () => {
            const sumAlphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getSumAlphaPrice",
                args: []
            })

            assert.ok(sumAlphaPrice !== undefined, "Sum alpha price should be defined");
        })
    });

    describe("Pool Data Functions", () => {
        it("getTaoInPool returns valid TAO amount", async () => {
            const taoInPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getTaoInPool",
                args: [subnetId]
            })

            assert.ok(taoInPool !== undefined, "TAO in pool should be defined");
            assert.ok(typeof taoInPool === 'bigint', "TAO in pool should be a bigint");
            assert.ok(taoInPool >= BigInt(0), "TAO in pool should be non-negative");
        });

        it("getAlphaInPool returns valid Alpha amount", async () => {
            const alphaInPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaInPool",
                args: [subnetId]
            })

            assert.ok(alphaInPool !== undefined, "Alpha in pool should be defined");
            assert.ok(typeof alphaInPool === 'bigint', "Alpha in pool should be a bigint");
            assert.ok(alphaInPool >= BigInt(0), "Alpha in pool should be non-negative");
        });

        it("getAlphaOutPool returns valid Alpha out amount", async () => {
            const alphaOutPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaOutPool",
                args: [subnetId]
            })

            assert.ok(alphaOutPool !== undefined, "Alpha out pool should be defined");
            assert.ok(typeof alphaOutPool === 'bigint', "Alpha out pool should be a bigint");
            assert.ok(alphaOutPool >= BigInt(0), "Alpha out pool should be non-negative");
        });

        it("getAlphaIssuance returns valid issuance amount", async () => {
            const alphaIssuance = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaIssuance",
                args: [subnetId]
            })

            assert.ok(alphaIssuance !== undefined, "Alpha issuance should be defined");
            assert.ok(typeof alphaIssuance === 'bigint', "Alpha issuance should be a bigint");
            assert.ok(alphaIssuance >= BigInt(0), "Alpha issuance should be non-negative");
        });

        it("getCKBurn returns valid CK burn rate", async () => {
            const ckBurn = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getCKBurn",
                args: [subnetId]
            })

            const ckBurnOnChain = await api.query.SubtensorModule.CKBurn.getValue(subnetId);

            assert.strictEqual(ckBurn, ckBurnOnChain, "CK burn should match on chain");
            assert.ok(ckBurn !== undefined, "CK burn should be defined");
            const ckBurnPercentage = BigInt(ckBurn) * BigInt(100) / BigInt(2 ** 64 - 1)
            assert.ok(ckBurnPercentage >= BigInt(0), "CK burn percentage should be non-negative");
            assert.ok(ckBurnPercentage <= BigInt(100), "CK burn percentage should be less than or equal to 100");
            assert.ok(typeof ckBurn === 'bigint', "CK burn should be a bigint");
        });
    });

    describe("Global Functions", () => {
        it("getTaoWeight returns valid TAO weight", async () => {
            const taoWeight = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getTaoWeight",
                args: []
            })

            assert.ok(taoWeight !== undefined, "TAO weight should be defined");
            assert.ok(typeof taoWeight === 'bigint', "TAO weight should be a bigint");
            assert.ok(taoWeight >= BigInt(0), "TAO weight should be non-negative");
        });

        it("getRootNetuid returns correct root netuid", async () => {
            const rootNetuid = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getRootNetuid",
                args: []
            })

            assert.ok(rootNetuid !== undefined, "Root netuid should be defined");
            assert.ok(typeof rootNetuid === 'number', "Root netuid should be a number");
            assert.strictEqual(rootNetuid, 0, "Root netuid should be 0");
        });
    });

    describe("Swap Simulation Functions", () => {
        it("simSwapTaoForAlpha returns valid simulation", async () => {
            const taoAmount = BigInt(1000000000); // 1 TAO in RAO
            const simulatedAlpha = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapTaoForAlpha",
                args: [subnetId, taoAmount]
            })

            assert.ok(simulatedAlpha !== undefined, "Simulated alpha should be defined");
            assert.ok(typeof simulatedAlpha === 'bigint', "Simulated alpha should be a bigint");
            assert.ok(simulatedAlpha >= BigInt(0), "Simulated alpha should be non-negative");
        });

        it("simSwapAlphaForTao returns valid simulation", async () => {
            const alphaAmount = BigInt(1000000000); // 1 Alpha
            const simulatedTao = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapAlphaForTao",
                args: [subnetId, alphaAmount]
            })

            assert.ok(simulatedTao !== undefined, "Simulated tao should be defined");
            assert.ok(typeof simulatedTao === 'bigint', "Simulated tao should be a bigint");
            assert.ok(simulatedTao >= BigInt(0), "Simulated tao should be non-negative");
        });

        it("swap simulations handle zero amounts", async () => {
            const zeroTaoForAlpha = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapTaoForAlpha",
                args: [subnetId, BigInt(0)]
            })

            const zeroAlphaForTao = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapAlphaForTao",
                args: [subnetId, BigInt(0)]
            })

            assert.strictEqual(zeroTaoForAlpha, BigInt(0), "Zero TAO should result in zero Alpha");
            assert.strictEqual(zeroAlphaForTao, BigInt(0), "Zero Alpha should result in zero TAO");
        });

        it("swap simulations are internally consistent", async () => {
            const taoAmount = BigInt(1000000000); // 1 TAO

            // Simulate TAO -> Alpha
            const simulatedAlpha = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapTaoForAlpha",
                args: [subnetId, taoAmount]
            })

            // If we got alpha, simulate Alpha -> TAO
            if ((simulatedAlpha as bigint) > BigInt(0)) {
                const simulatedTao = await publicClient.readContract({
                    abi: IAlphaABI,
                    address: toViemAddress(IALPHA_ADDRESS),
                    functionName: "simSwapAlphaForTao",
                    args: [subnetId, simulatedAlpha]
                })

                // Check if simulated values are reasonably close (allowing for rounding/fees)
                if ((simulatedTao as bigint) > BigInt(0)) {
                    const ratio = Number(taoAmount) / Number(simulatedTao);
                    assert.ok(ratio >= 0.5 && ratio <= 2.0, "Swap simulation should be within reasonable bounds");
                }
            }
        });
    });

    describe("Subnet Configuration Functions", () => {
        it("getSubnetMechanism returns valid mechanism", async () => {
            const mechanism = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getSubnetMechanism",
                args: [subnetId]
            })

            assert.ok(mechanism !== undefined, "Subnet mechanism should be defined");
            assert.ok(typeof mechanism === 'number', "Subnet mechanism should be a number");
            assert.ok(mechanism >= 0, "Subnet mechanism should be non-negative");
        });

        it("getEMAPriceHalvingBlocks returns valid halving period", async () => {
            const halvingBlocks = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getEMAPriceHalvingBlocks",
                args: [subnetId]
            })

            assert.ok(halvingBlocks !== undefined, "EMA price halving blocks should be defined");
            assert.ok(typeof halvingBlocks === 'bigint', "EMA halving blocks should be a bigint");
            assert.ok(halvingBlocks >= BigInt(0), "EMA halving blocks should be non-negative");
        });

        it("getSubnetVolume returns valid volume data", async () => {
            const subnetVolume = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getSubnetVolume",
                args: [subnetId]
            })

            assert.ok(subnetVolume !== undefined, "Subnet volume should be defined");
            assert.ok(typeof subnetVolume === 'bigint', "Subnet volume should be a bigint");
            assert.ok(subnetVolume >= BigInt(0), "Subnet volume should be non-negative");
        });
    });

    describe("Data Consistency with Pallet", () => {
        it("precompile data matches pallet values", async () => {
            // Get TAO in pool from precompile
            const taoInPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getTaoInPool",
                args: [subnetId]
            })

            // Get TAO in pool directly from the pallet
            const taoInPoolFromPallet = await api.query.SubtensorModule.SubnetTAO.getValue(subnetId);

            // Compare values
            assert.strictEqual(taoInPool as bigint, taoInPoolFromPallet, "TAO in pool values should match");

            // Get Alpha in pool from precompile
            const alphaInPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaInPool",
                args: [subnetId]
            })

            // Get Alpha in pool directly from the pallet
            const alphaInPoolFromPallet = await api.query.SubtensorModule.SubnetAlphaIn.getValue(subnetId);

            // Compare values
            assert.strictEqual(alphaInPool as bigint, alphaInPoolFromPallet, "Alpha in pool values should match");

            // Get Alpha out pool from precompile
            const alphaOutPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaOutPool",
                args: [subnetId]
            })

            // Get Alpha out pool directly from the pallet
            const alphaOutPoolFromPallet = await api.query.SubtensorModule.SubnetAlphaOut.getValue(subnetId);

            // Compare values
            assert.strictEqual(alphaOutPool as bigint, alphaOutPoolFromPallet, "Alpha out pool values should match");
        });

        it("subnet volume data is consistent", async () => {
            const subnetVolume = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getSubnetVolume",
                args: [subnetId]
            })

            const subnetVolumeFromPallet = await api.query.SubtensorModule.SubnetVolume.getValue(subnetId);

            assert.strictEqual(subnetVolume as bigint, subnetVolumeFromPallet, "Subnet volume values should match");
        });
    });

    describe("Edge Cases and Error Handling", () => {
        it("handles non-existent subnet gracefully", async () => {
            const nonExistentSubnet = 9999;

            // These should not throw but return default values
            const alphaPrice = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getAlphaPrice",
                args: [nonExistentSubnet]
            })

            const taoInPool = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "getTaoInPool",
                args: [nonExistentSubnet]
            })

            // Should return default values, not throw
            assert.ok(alphaPrice !== undefined, "Should handle non-existent subnet gracefully");
            assert.ok(taoInPool !== undefined, "Should handle non-existent subnet gracefully");
        });

        it("simulation functions handle large amounts", async () => {
            const largeAmount = BigInt("1000000000000000000"); // Very large amount

            const simulatedAlpha = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapTaoForAlpha",
                args: [subnetId, largeAmount]
            })

            const simulatedTao = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapAlphaForTao",
                args: [subnetId, largeAmount]
            })

            // Should handle large amounts without throwing
            assert.ok(simulatedAlpha !== undefined, "Should handle large TAO amounts");
            assert.ok(simulatedTao !== undefined, "Should handle large Alpha amounts");
        });
    });
});
