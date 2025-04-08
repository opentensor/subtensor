import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58 } from "../src/address-utils"
import { raoToEth, tao } from "../src/balance-math"
import { ethers } from "ethers"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils"
import { convertH160ToPublicKey } from "../src/address-utils"
import {
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister,
    sendProxyCall,
} from "../src/subtensor"
import { ETH_LOCAL_URL } from "../src/config";
import { ISTAKING_ADDRESS, ISTAKING_V2_ADDRESS, IStakingABI, IStakingV2ABI } from "../src/contracts/staking"
import { PublicClient } from "viem";

describe("Test neuron precompile reveal weights", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    let publicClient: PublicClient;
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const proxy = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        api = await getDevnetApi()

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(proxy.publicKey))
        await forceSetBalanceToEthAddress(api, wallet1.address)
        await forceSetBalanceToEthAddress(api, wallet2.address)
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)

        console.log("test the case on subnet ", netuid)

        await burnedRegister(api, netuid, convertH160ToSS58(wallet1.address), coldkey)
        await burnedRegister(api, netuid, convertH160ToSS58(wallet2.address), coldkey)
    })

    it("Can add stake", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        // ETH unit
        let stakeBalance = raoToEth(tao(20))
        const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), netuid)
        const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);
        const tx = await contract.addStake(hotkey.publicKey, netuid, { value: stakeBalance.toString() })
        await tx.wait()

        const stakeFromContract = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );

        assert.ok(stakeFromContract > stakeBefore)
        const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), netuid)
        assert.ok(stakeAfter > stakeBefore)
    })

    it("Can add stake V2", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        // the unit in V2 is RAO, not ETH
        let stakeBalance = tao(20)
        const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet2.address), netuid)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);
        const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), netuid)
        await tx.wait()

        const stakeFromContract = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), netuid)
        );

        assert.ok(stakeFromContract > stakeBefore)
        const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet2.address), netuid)
        assert.ok(stakeAfter > stakeBefore)
    })

    it("Can not add stake if subnet doesn't exist", async () => {
        // wrong netuid
        let netuid = 12345;
        let stakeBalance = raoToEth(tao(20))
        const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), netuid)
        const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);
        try {
            const tx = await contract.addStake(hotkey.publicKey, netuid, { value: stakeBalance.toString() })
            await tx.wait()
            assert.fail("Transaction should have failed");
        } catch (error) {
            // Transaction failed as expected
        }

        const stakeFromContract = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );
        assert.equal(stakeFromContract, stakeBefore)
        const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet1.address), netuid)
        assert.equal(stakeAfter, stakeBefore)
    });

    it("Can not add stake V2 if subnet doesn't exist", async () => {
        // wrong netuid
        let netuid = 12345;
        // the unit in V2 is RAO, not ETH
        let stakeBalance = tao(20)
        const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet2.address), netuid)
        const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

        try {
            const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), netuid);
            await tx.wait();
            assert.fail("Transaction should have failed");
        } catch (error) {
            // Transaction failed as expected
        }

        const stakeFromContract = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), netuid)
        );
        assert.equal(stakeFromContract, stakeBefore)
        const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet2.address), netuid)
        assert.equal(stakeAfter, stakeBefore)
    })

    it("Can get stake via contract read method", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        // TODO need check how to pass bytes32 as parameter of readContract
        // const value = await publicClient.readContract({
        //     address: ISTAKING_ADDRESS,
        //     abi: IStakingABI,
        //     functionName: "getStake",
        //     args: [hotkey.publicKey, // Convert to bytes32 format
        //     convertH160ToPublicKey(wallet1.address),
        //         netuid]
        // })
        // if (value === undefined || value === null) {
        //     throw new Error("value of getStake from contract is undefined")
        // }
        // const intValue = BigInt(value.toString())

        const contractV1 = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);
        const stakeFromContractV1 = BigInt(
            await contractV1.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );

        const contractV2 = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);
        // unit from contract V2 is RAO, not ETH
        const stakeFromContractV2 = Number(
            await contractV2.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );

        assert.equal(stakeFromContractV1, tao(stakeFromContractV2))

    })

    it("Can remove stake", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const contract = new ethers.Contract(
            ISTAKING_ADDRESS,
            IStakingABI,
            wallet1
        );

        const stakeBeforeRemove = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );

        let stakeBalance = raoToEth(tao(10))
        const tx = await contract.removeStake(hotkey.publicKey, stakeBalance, netuid)
        await tx.wait()

        const stakeAfterRemove = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), netuid)
        );
        assert.ok(stakeAfterRemove < stakeBeforeRemove)

    })

    it("Can remove stake V2", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        const contract = new ethers.Contract(
            ISTAKING_V2_ADDRESS,
            IStakingV2ABI,
            wallet2
        );

        const stakeBeforeRemove = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), netuid)
        );

        let stakeBalance = tao(10)
        const tx = await contract.removeStake(hotkey.publicKey, stakeBalance, netuid)
        await tx.wait()

        const stakeAfterRemove = BigInt(
            await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), netuid)
        );

        assert.ok(stakeAfterRemove < stakeBeforeRemove)
    })

    it("Can add/remove proxy", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        // add/remove are done in a single test case, because we can't use the same private/public key
        // between substrate and EVM, but to test the remove part, we must predefine the proxy first.
        // it makes `remove` being dependent on `add`, because we should use `addProxy` from contract
        // to prepare the proxy for `removeProxy` testing - the proxy is specified for the
        // caller/origin.

        // first, check we don't have proxies
        const ss58Address = convertH160ToSS58(wallet1.address);
        // the result include two items array, first one is delegate info, second one is balance
        const initProxies = await api.query.Proxy.Proxies.getValue(ss58Address);
        assert.equal(initProxies[0].length, 0);

        // intialize the contract
        const contract = new ethers.Contract(
            ISTAKING_ADDRESS,
            IStakingABI,
            wallet1
        );

        // test "add"
        let tx = await contract.addProxy(proxy.publicKey);
        await tx.wait();

        const proxiesAfterAdd = await api.query.Proxy.Proxies.getValue(ss58Address);

        assert.equal(proxiesAfterAdd[0][0].delegate, convertPublicKeyToSs58(proxy.publicKey))

        let stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(hotkey.publicKey),
            ss58Address,
            netuid
        )

        const call = api.tx.SubtensorModule.add_stake({
            hotkey: convertPublicKeyToSs58(hotkey.publicKey),
            netuid: netuid,
            amount_staked: tao(1)
        })
        await sendProxyCall(api, call.decodedCall, ss58Address, proxy)

        let stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(hotkey.publicKey),
            ss58Address,
            netuid
        )

        assert.ok(stakeAfter > stakeBefore)
        // test "remove"
        tx = await contract.removeProxy(proxy.publicKey);
        await tx.wait();

        const proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(ss58Address);
        assert.equal(proxiesAfterRemove[0].length, 0)
    });

    it("Can add/remove proxy V2", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        // add/remove are done in a single test case, because we can't use the same private/public key
        // between substrate and EVM, but to test the remove part, we must predefine the proxy first.
        // it makes `remove` being dependent on `add`, because we should use `addProxy` from contract
        // to prepare the proxy for `removeProxy` testing - the proxy is specified for the
        // caller/origin.

        // first, check we don't have proxies
        const ss58Address = convertH160ToSS58(wallet1.address);
        // the result include two items array, first one is delegate info, second one is balance
        const initProxies = await api.query.Proxy.Proxies.getValue(ss58Address);
        assert.equal(initProxies[0].length, 0);

        // intialize the contract
        // const signer = new ethers.Wallet(fundedEthWallet.privateKey, provider);
        const contract = new ethers.Contract(
            ISTAKING_V2_ADDRESS,
            IStakingV2ABI,
            wallet1
        );

        // test "add"
        let tx = await contract.addProxy(proxy.publicKey);
        await tx.wait();

        const proxiesAfterAdd = await api.query.Proxy.Proxies.getValue(ss58Address);

        assert.equal(proxiesAfterAdd[0][0].delegate, convertPublicKeyToSs58(proxy.publicKey))

        let stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(hotkey.publicKey),
            ss58Address,
            netuid
        )

        const call = api.tx.SubtensorModule.add_stake({
            hotkey: convertPublicKeyToSs58(hotkey.publicKey),
            netuid: netuid,
            amount_staked: tao(1)
        })

        await sendProxyCall(api, call.decodedCall, ss58Address, proxy)

        let stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(hotkey.publicKey),
            ss58Address,
            netuid
        )

        assert.ok(stakeAfter > stakeBefore)
        // test "remove"
        tx = await contract.removeProxy(proxy.publicKey);
        await tx.wait();

        const proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(ss58Address);
        assert.equal(proxiesAfterRemove[0].length, 0)
    });
});
