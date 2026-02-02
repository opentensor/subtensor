import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, getBalance, getSignerFromKeypair } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { TypedApi, Binary } from "polkadot-api";
import { convertH160ToSS58, convertPublicKeyToSs58, convertH160ToPublicKey } from "../src/address-utils";
import { tao, raoToEth } from "../src/balance-math";
import {
    forceSetBalanceToSs58Address,
    addNewSubnetwork,
    startCall,
    disableWhiteListCheck,
    forceSetBalanceToEthAddress,

} from "../src/subtensor";
import { ethers } from "ethers";
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { PRECOMPILE_WRAPPER_ABI, PRECOMPILE_WRAPPER_BYTECODE } from "../src/contracts/precompileWrapper";
import { ETH_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { IProxyABI, IPROXY_ADDRESS } from "../src/contracts/proxy"

describe("PrecompileWrapper - Direct Call Tests", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();

    let api: TypedApi<typeof devnet>;
    let publicClient: PublicClient;
    let wrapperContract: ethers.Contract;
    let wrapperAddress: string;
    let netuid: number;

    before(async () => {
        api = await getDevnetApi();
        publicClient = await getPublicClient(ETH_LOCAL_URL);
        await disableWhiteListCheck(api, true);
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey));
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey));
        await forceSetBalanceToEthAddress(api, wallet1.address);
        await forceSetBalanceToEthAddress(api, wallet2.address);
        await addNewSubnetwork(api, hotkey, coldkey);
        netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
        await startCall(api, netuid, coldkey);

        const factory = new ethers.ContractFactory(
            PRECOMPILE_WRAPPER_ABI,
            PRECOMPILE_WRAPPER_BYTECODE,
            wallet1
        );
        const deployContract = await factory.deploy();
        await deployContract.waitForDeployment();
        wrapperAddress = await deployContract.getAddress();
        await forceSetBalanceToEthAddress(api, wrapperAddress);

        console.log("Wrapper contract deployed at:", wrapperAddress);
        console.log("Testing in subnet:", netuid);

        wrapperContract = new ethers.Contract(wrapperAddress, PRECOMPILE_WRAPPER_ABI, wallet1);
    });

    describe("Balance Transfer Precompile Direct Calls", () => {
        it("Should transfer balance via wrapper", async () => {
            const keypair = getRandomSubstrateKeypair();
            const transferAmount = raoToEth(tao(1));

            // Transfer via wrapper
            const transferTx = await wrapperContract.transfer(keypair.publicKey, { value: transferAmount.toString() });
            await transferTx.wait();

            const balance = await getBalance(api, convertPublicKeyToSs58(keypair.publicKey));
            assert.ok(balance >= tao(1), "Balance should be transferred");
        });
    });

    describe("Metagraph Precompile Direct Calls", () => {
        it("Should get UID count via wrapper", async () => {
            const uidCountViaWrapper = await wrapperContract.getUidCount(netuid);
            assert.ok(uidCountViaWrapper !== undefined, "UID count should be not undefined");
        });
    });

    describe("Subnet Precompile Direct Calls", () => {
        it("Should get serving rate limit via wrapper", async () => {
            const rateLimitViaWrapper = await wrapperContract.getServingRateLimit(netuid);

            assert.ok(rateLimitViaWrapper !== undefined, "Rate limit should be not undefined");
        });

        it("Should register network with details via wrapper", async () => {
            const newHotkey = getRandomSubstrateKeypair();
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(newHotkey.publicKey));

            const totalNetworksBefore = await api.query.SubtensorModule.TotalNetworks.getValue();

            const registerTx = await wrapperContract.registerNetworkWithDetails(
                newHotkey.publicKey,
                "Test Subnet",
                "https://github.com/test/repo",
                "test@example.com",
                "https://test.example.com",
                "test#1234",
                "Test description",
                "Additional info",
                { value: raoToEth(tao(100)).toString() }
            );
            await registerTx.wait();

            const totalNetworksAfter = await api.query.SubtensorModule.TotalNetworks.getValue();
            const beforeValue = typeof totalNetworksBefore === 'bigint' ? totalNetworksBefore : BigInt(totalNetworksBefore);
            assert.equal(totalNetworksAfter, beforeValue + BigInt(1), "Network should be registered");
        });
    });

    describe("Neuron Precompile Direct Calls", () => {
        it("Should register neuron via wrapper", async () => {
            const newHotkey = getRandomSubstrateKeypair();
            const newColdkey = getRandomSubstrateKeypair();
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(newHotkey.publicKey));
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(newColdkey.publicKey));

            // Use a reasonable burn amount (100 TAO)
            const burnAmount = tao(100);

            const registerTx = await wrapperContract.burnedRegister(
                netuid,
                newHotkey.publicKey,
                { value: raoToEth(burnAmount).toString() }
            );
            await registerTx.wait();

            const uid = await api.query.SubtensorModule.Uids.getValue(netuid, convertPublicKeyToSs58(newHotkey.publicKey));
            assert.ok(uid !== undefined, "Neuron should be registered");
        });
    });

    describe("Staking Precompile Direct Calls", () => {
        it("Should get total coldkey stake via wrapper", async () => {
            const stakeViaWrapper = await wrapperContract.getTotalColdkeyStake(coldkey.publicKey);
            assert.ok(stakeViaWrapper !== undefined, "Total coldkey stake should be not undefined");
        });

        it("Should get total hotkey stake via wrapper", async () => {
            const stakeViaWrapper = await wrapperContract.getTotalHotkeyStake(hotkey.publicKey);
            assert.ok(stakeViaWrapper !== undefined, "Total hotkey stake should be not undefined");
        });

        it("Should add stake via wrapper", async () => {
            const stakeAmount = tao(2);
            const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
                convertPublicKeyToSs58(hotkey.publicKey),
                convertH160ToSS58(wrapperAddress),
                netuid
            );

            const addStakeTx = await wrapperContract.addStake(
                hotkey.publicKey,
                stakeAmount.toString(),
                netuid,
                { value: raoToEth(stakeAmount).toString() }
            );
            await addStakeTx.wait();

            const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
                convertPublicKeyToSs58(hotkey.publicKey),
                convertH160ToSS58(wrapperAddress),
                netuid
            );
            assert.ok(stakeAfter > stakeBefore, "Stake should be increased");
        });

        it("Should remove stake via wrapper", async () => {
            const removeAmount = tao(1);
            const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
                convertPublicKeyToSs58(hotkey.publicKey),
                convertH160ToSS58(wrapperAddress),
                netuid
            );

            const removeStakeTx = await wrapperContract.removeStake(
                hotkey.publicKey,
                removeAmount.toString(),
                netuid
            );
            await removeStakeTx.wait();

            const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
                convertPublicKeyToSs58(hotkey.publicKey),
                convertH160ToSS58(wrapperAddress),
                netuid
            );
            assert.ok(stakeAfter < stakeBefore, "Stake should be decreased");
        });
    });

    describe("UID Lookup Precompile Direct Calls", () => {
        it("Should lookup UID via wrapper", async () => {
            const evmAddress = wallet1.address;
            const limit = 10;
            const lookupViaWrapper = await wrapperContract.uidLookup(netuid, evmAddress, limit);

            assert.ok(Array.isArray(lookupViaWrapper), "Lookup should return an array");
        });
    });

    describe("Alpha Precompile Direct Calls", () => {
        it("Should get alpha price via wrapper", async () => {
            const priceViaWrapper = await wrapperContract.getAlphaPrice(netuid);
            assert.ok(priceViaWrapper !== undefined, "Alpha price should be not undefined");
        });
    });

    describe("Crowdloan Precompile Direct Calls", () => {
        it("Should get crowdloan via wrapper", async () => {
            // First create a crowdloan via substrate
            const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
            const end = await api.query.System.Number.getValue() + 100;
            const deposit = BigInt(15_000_000_000); // 15 TAO
            const minContribution = BigInt(1_000_000_000); // 1 TAO
            const cap = BigInt(100_000_000_000); // 100 TAO

            const signer = getSignerFromKeypair(coldkey);
            await api.tx.Crowdloan.create({
                deposit,
                min_contribution: minContribution,
                cap,
                end,
                target_address: undefined,
                call: api.tx.System.remark({ remark: Binary.fromText("test") }).decodedCall
            }).signAndSubmit(signer);

            // Wait a bit for the transaction to be included
            await new Promise(resolve => setTimeout(resolve, 2000));

            const crowdloanViaWrapper = await wrapperContract.getCrowdloan(nextId);

            assert.ok(crowdloanViaWrapper !== undefined, "Crowdloan should be not undefined");
        });

        it("Should get contribution via wrapper", async () => {
            const nextId = await api.query.Crowdloan.NextCrowdloanId.getValue();
            const contributionViaWrapper = await wrapperContract.getContribution(nextId - 1, coldkey.publicKey);

            assert.ok(contributionViaWrapper !== undefined, "Contribution should be not undefined");
        });

        it("Should create crowdloan via wrapper", async () => {
            const deposit = BigInt(20_000_000_000); // 20 TAO
            const minContribution = BigInt(2_000_000_000); // 2 TAO
            const cap = BigInt(200_000_000_000); // 200 TAO
            const end = Number(await api.query.System.Number.getValue()) + 100;
            const targetAddress = wallet2.address;

            const nextIdBefore = await api.query.Crowdloan.NextCrowdloanId.getValue();

            const createTx = await wrapperContract.createCrowdloan(
                deposit.toString(),
                minContribution.toString(),
                cap.toString(),
                end,
                targetAddress,
                { value: raoToEth(deposit).toString() }
            );
            await createTx.wait();

            const nextIdAfter = await api.query.Crowdloan.NextCrowdloanId.getValue();
            const beforeId = typeof nextIdBefore === 'bigint' ? nextIdBefore : BigInt(nextIdBefore);
            assert.equal(nextIdAfter, beforeId + BigInt(1), "Crowdloan should be created");
        });
    });


    describe("Leasing Precompile Direct Calls", () => {
        it("Should get contributor share via wrapper", async () => {
            // First create a lease crowdloan
            const nextCrowdloanId = await api.query.Crowdloan.NextCrowdloanId.getValue();
            const crowdloanDeposit = BigInt(100_000_000_000); // 100 TAO
            const networkLastLockCost = await api.query.SubtensorModule.NetworkLastLockCost.getValue();
            const lockCostValue = typeof networkLastLockCost === 'bigint' ? networkLastLockCost : BigInt(networkLastLockCost);
            const crowdloanCap = lockCostValue * BigInt(2);
            const currentBlock = await api.query.System.Number.getValue();
            const crowdloanEnd = currentBlock + 100;
            const leasingEmissionsShare = 15;
            const leasingEndBlock = currentBlock + 300;

            const signer = getSignerFromKeypair(coldkey);
            await api.tx.Crowdloan.create({
                deposit: crowdloanDeposit,
                min_contribution: BigInt(1_000_000_000),
                cap: crowdloanCap,
                end: crowdloanEnd,
                target_address: undefined,
                call: api.tx.SubtensorModule.register_leased_network({
                    emissions_share: leasingEmissionsShare,
                    end_block: leasingEndBlock,
                }).decodedCall
            }).signAndSubmit(signer);

            await new Promise(resolve => setTimeout(resolve, 2000));

            const nextLeaseId = await api.query.SubtensorModule.NextSubnetLeaseId.getValue();

            // Get contributor share
            const shareViaWrapper = await wrapperContract.getContributorShare(nextLeaseId, coldkey.publicKey);

            assert.ok(shareViaWrapper !== undefined, "Share should be not undefined");

        });

        it("Should create lease crowdloan via wrapper", async () => {
            const crowdloanDeposit = BigInt(100_000_000_000); // 100 TAO
            const crowdloanMinContribution = BigInt(1_000_000_000); // 1 TAO
            const networkLastLockCost = await api.query.SubtensorModule.NetworkLastLockCost.getValue();
            const lockCostValue = typeof networkLastLockCost === 'bigint' ? networkLastLockCost : BigInt(networkLastLockCost);
            const crowdloanCap = lockCostValue * BigInt(2);
            const currentBlock = await api.query.System.Number.getValue();
            const currentBlockValue = typeof currentBlock === 'bigint' ? Number(currentBlock) : currentBlock;
            const crowdloanEnd = currentBlockValue + 100;
            const leasingEmissionsShare = 15;
            const hasLeasingEndBlock = true;
            const leasingEndBlock = currentBlockValue + 300;

            const nextCrowdloanIdBefore = await api.query.Crowdloan.NextCrowdloanId.getValue();

            const createTx = await wrapperContract.createLeaseCrowdloan(
                crowdloanDeposit.toString(),
                crowdloanMinContribution.toString(),
                crowdloanCap.toString(),
                crowdloanEnd,
                leasingEmissionsShare,
                hasLeasingEndBlock,
                leasingEndBlock,
                { value: raoToEth(crowdloanDeposit).toString() }
            );
            await createTx.wait();

            const nextCrowdloanIdAfter = await api.query.Crowdloan.NextCrowdloanId.getValue();
            const beforeId = typeof nextCrowdloanIdBefore === 'bigint' ? nextCrowdloanIdBefore : BigInt(nextCrowdloanIdBefore);
            assert.equal(nextCrowdloanIdAfter, beforeId + BigInt(1), "Lease crowdloan should be created");
        });
    });


    describe("Proxy Precompile Direct Calls", () => {
        it("Should get proxies via wrapper", async () => {
            const accountKey = convertH160ToPublicKey(wallet1.address);
            const proxiesViaWrapper = await wrapperContract.getProxies(accountKey);

            assert.ok(proxiesViaWrapper !== undefined, "Proxies should be not undefined");
            assert.ok(Array.isArray(proxiesViaWrapper), "Proxies should be an array");
        });
        it("Should add proxy via wrapper", async () => {
            const delegate = getRandomSubstrateKeypair();
            await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(delegate.publicKey));
            const delegateKey = delegate.publicKey;
            const proxyType = 0;
            const delay = 0;

            const proxiesBefore = await api.query.Proxy.Proxies.getValue(convertH160ToSS58(wrapperAddress));

            const addProxyTx = await wrapperContract.addProxy(delegateKey, proxyType, delay);
            await addProxyTx.wait();

            const proxiesAfter = await api.query.Proxy.Proxies.getValue(convertH160ToSS58(wrapperAddress));
            assert.ok(proxiesAfter[0].length > proxiesBefore[0].length, "Proxy should be added");
        });

        it("Should proxy call via wrapper", async () => {
            const proxyType = 0;
            const delay = 0;

            const proxyContract = new ethers.Contract(IPROXY_ADDRESS, IProxyABI, wallet1);
            const addProxyTx = await proxyContract.addProxy(convertH160ToPublicKey(wrapperAddress), proxyType, delay);
            await addProxyTx.wait();

            // Create a simple call (remark)
            const remarkCall = api.tx.System.remark({ remark: Binary.fromText("") });

            const callData = await remarkCall.getEncodedData();
            const data = callData.asBytes();

            const proxyCallTx = await wrapperContract.proxyCall(
                convertH160ToPublicKey(wallet1.address),
                [proxyType],
                [...data]
            );
            await proxyCallTx.wait();

            // Verify the call was executed (no error means success)
            assert.ok(proxyCallTx, "Proxy call should succeed");
        });
    });

    describe("Address Mapping Precompile Direct Calls", () => {
        it("Should map address via wrapper", async () => {
            const testAddress = wallet1.address;
            const mappedViaWrapper = await wrapperContract.addressMapping(testAddress);

            assert.ok(mappedViaWrapper !== undefined, "Mapped address should be not undefined");
            assert.ok(mappedViaWrapper !== "0x0000000000000000000000000000000000000000000000000000000000000000", "Mapped address should not be zero");
        });
    });




});
