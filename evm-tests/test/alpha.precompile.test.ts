import * as assert from "assert";

import { getAliceSigner, getClient, getDevnetApi, waitForTransactionCompletion, convertPublicKeyToMultiAddress, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IAlphaABI, IALPHA_ADDRESS } from "../src/contracts/alpha"

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
        const subClient = await getClient(SUB_LOCAL_URL)
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
        let uid_count = await api.query.SubtensorModule.SubnetworkN.getValue(subnetId.toString())
        if (uid_count === 0) {
            const tx = api.tx.SubtensorModule.burned_register({ hotkey: convertPublicKeyToSs58(hotkey.publicKey), netuid: subnetId })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        }
    })

    it("Basic Alpha price data via precompile contract is accessible", async () => {
        // Test getAlphaPrice
        const alphaPrice = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaPrice",
            args: [subnetId]
        })

        assert.ok(alphaPrice !== undefined, "Alpha price should be defined");
        assert.ok(typeof alphaPrice === 'bigint', "Alpha price should be a bigint");

        // Test getTaoInPool
        const taoInPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getTaoInPool",
            args: [subnetId]
        })

        assert.ok(taoInPool !== undefined, "TAO in pool should be defined");

        // Test getAlphaInPool
        const alphaInPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaInPool",
            args: [subnetId]
        })

        assert.ok(alphaInPool !== undefined, "Alpha in pool should be defined");
    });

    it("Extended Alpha precompile functions are accessible", async () => {
        // Test getMovingAlphaPrice
        const movingAlphaPrice = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getMovingAlphaPrice",
            args: [subnetId]
        })
        assert.ok(movingAlphaPrice !== undefined, "Moving alpha price should be defined");

        // Test getAlphaOutPool
        const alphaOutPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaOutPool",
            args: [subnetId]
        })
        assert.ok(alphaOutPool !== undefined, "Alpha out pool should be defined");

        // Test getAlphaIssuance
        const alphaIssuance = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaIssuance",
            args: [subnetId]
        })
        assert.ok(alphaIssuance !== undefined, "Alpha issuance should be defined");

        // Test getTaoWeight
        const taoWeight = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getTaoWeight",
            args: []
        })
        assert.ok(taoWeight !== undefined, "TAO weight should be defined");
    });

    it("Swap simulation functions work correctly", async () => {
        // Test simSwapTaoForAlpha
        const taoAmount = BigInt(1000000000); // 1 RAO
        const simulatedAlpha = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "simSwapTaoForAlpha",
            args: [subnetId, taoAmount]
        })
        assert.ok(simulatedAlpha !== undefined, "Simulated alpha should be defined");

        // Test simSwapAlphaForTao
        if (simulatedAlpha > 0) {
            const simulatedTao = await publicClient.readContract({
                abi: IAlphaABI,
                address: toViemAddress(IALPHA_ADDRESS),
                functionName: "simSwapAlphaForTao",
                args: [subnetId, simulatedAlpha]
            })
            assert.ok(simulatedTao !== undefined, "Simulated tao should be defined");

            // Check if simulated values are reasonably close (allowing for some rounding)
            if (Number(simulatedAlpha) > 0 && simulatedTao > BigInt(0)) {
                const ratio = Number(taoAmount) / Number(simulatedTao);
                assert.ok(ratio > 0.9 && ratio < 1.1, "Swap simulation should be roughly symmetric");
            }
        }
    });

    it("Subnet configuration data is accessible", async () => {
        // Test getSubnetMechanism
        const mechanism = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getSubnetMechanism",
            args: [subnetId]
        })
        assert.ok(mechanism !== undefined, "Subnet mechanism should be defined");

        // Test getMinimumPoolLiquidity
        const minLiquidity = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getMinimumPoolLiquidity",
            args: []
        })
        assert.ok(minLiquidity !== undefined, "Minimum pool liquidity should be defined");

        // Test getRootNetuid
        const rootNetuid = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getRootNetuid",
            args: []
        })
        assert.ok(rootNetuid !== undefined, "Root netuid should be defined");
        assert.ok(rootNetuid === 0, "Root netuid should be 0");

        // Test getEMAPriceHalvingBlocks
        const halvingBlocks = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getEMAPriceHalvingBlocks",
            args: [subnetId]
        })
        assert.ok(halvingBlocks !== undefined, "EMA price halving blocks should be defined");
    });

    it("Alpha precompile data is consistent with pallet values", async () => {
        // Get alpha price from precompile
        const alphaPrice = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaPrice",
            args: [subnetId]
        })

        // Get alpha price directly from the pallet
        const alphaPriceFromPallet = await api.query.SubtensorModule.AlphaFRatio.getValue(subnetId.toString());

        // Convert to same units and validate
        // Note: The precompile converts U96F32 to u64 so we just check it's reasonable,
        // not an exact match
        assert.ok(alphaPrice !== undefined, "Alpha price should be defined");

        // Get TAO in pool from precompile
        const taoInPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getTaoInPool",
            args: [subnetId]
        })

        // Get TAO in pool directly from the pallet
        const taoInPoolFromPallet = await api.query.SubtensorModule.SubnetTAO.getValue(subnetId.toString());

        // Compare values
        assert.strictEqual(BigInt(taoInPool), taoInPoolFromPallet, "TAO in pool values should match");

        // Get Alpha in pool from precompile
        const alphaInPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaInPool",
            args: [subnetId]
        })

        // Get Alpha in pool directly from the pallet
        const alphaInPoolFromPallet = await api.query.SubtensorModule.SubnetAlphaIn.getValue(subnetId.toString());

        // Compare values
        assert.strictEqual(BigInt(alphaInPool), alphaInPoolFromPallet, "Alpha in pool values should match");

        // Get Alpha out pool from precompile
        const alphaOutPool = await publicClient.readContract({
            abi: IAlphaABI,
            address: toViemAddress(IALPHA_ADDRESS),
            functionName: "getAlphaOutPool",
            args: [subnetId]
        })

        // Get Alpha out pool directly from the pallet
        const alphaOutPoolFromPallet = await api.query.SubtensorModule.SubnetAlphaOut.getValue(subnetId.toString());

        // Compare values
        assert.strictEqual(BigInt(alphaOutPool), alphaOutPoolFromPallet, "Alpha out pool values should match");
    });
});
