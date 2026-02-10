import * as assert from "assert";
import { getAliceSigner, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { IDISPATCH_ADDRESS, ISTORAGE_QUERY_ADDRESS, ETH_LOCAL_URL } from "../src/config";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi, getTypedCodecs } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { forceSetBalanceToEthAddress, setMaxChildkeyTake, burnedRegister, forceSetBalanceToSs58Address, addStake, setTxRateLimit, addNewSubnetwork, startCall, setTempo } from "../src/subtensor";
import { xxhashAsHex } from "@polkadot/util-crypto";

describe("Test the dispatch precompile", () => {
    let publicClient: PublicClient;
    const wallet1 = generateRandomEthersWallet();
    let api: TypedApi<typeof devnet>
    let alice: PolkadotSigner;
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    let netuid: number;

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner()
        await forceSetBalanceToEthAddress(api, wallet1.address)

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))


        netuid = await addNewSubnetwork(api, hotkey, coldkey)
        // set tempo big enough to avoid stake value updated with fast block feature
        await setTempo(api, netuid, 10000)
        await startCall(api, netuid, coldkey)
        await setTxRateLimit(api, BigInt(0))

        await burnedRegister(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), coldkey)
        await addStake(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), BigInt(1_000_000_000), coldkey)
    })

    it("Dispatch transfer call via precompile contract works correctly", async () => {
        // call for transfer 1 token to alice
        const transferAmount = BigInt(1000000000);

        const unsignedTx = api.tx.Balances.transfer_keep_alive({
            dest: MultiAddress.Id(convertPublicKeyToSs58(alice.publicKey)),
            value: transferAmount,
        });
        const encodedCallDataBytes = await unsignedTx.getEncodedData();

        // encoded call should be 0x050300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d02286bee
        const transferCall = encodedCallDataBytes.asHex()

        const aliceBalance = (await api.query.System.Account.getValue(convertPublicKeyToSs58(alice.publicKey))).data.free
        const txResponse = await wallet1.sendTransaction({
            to: IDISPATCH_ADDRESS,
            data: transferCall,
        })
        await txResponse.wait()

        const aliceBalanceAfterTransfer = (await api.query.System.Account.getValue(convertPublicKeyToSs58(alice.publicKey))).data.free

        assert.equal(aliceBalance + transferAmount, aliceBalanceAfterTransfer)
    })

    it("Storage query only allow some pallets prefixed storage", async () => {
        const authorizedKeys = [
            await api.query.SubtensorModule.TotalNetworks.getKey(),
            await api.query.Swap.AlphaSqrtPrice.getKey(),
            await api.query.Balances.TotalIssuance.getKey(),
            await api.query.Proxy.Announcements.getKey(),
            await api.query.Scheduler.Agenda.getKey(),
            await api.query.Drand.Pulses.getKey(),
            await api.query.Crowdloan.Crowdloans.getKey(),
        ];
        
        for (const key of authorizedKeys) {
            await assert.doesNotReject(
                publicClient.call({
                    to: ISTORAGE_QUERY_ADDRESS,
                    data: key.toString() as `0x${string}`,
                })
            );
        }

        const unauthorizedKeys = [
            await api.query.System.Events.getKey(),
            await api.query.Grandpa.CurrentSetId.getKey(),
            xxhashAsHex(":code" , 128),
        ];

        for (const key of unauthorizedKeys) {
            await assert.rejects(
                publicClient.call({
                    to: ISTORAGE_QUERY_ADDRESS,
                    data: key.toString() as `0x${string}`,
                })
            );
        }
    })


    it("Value type storage query call via precompile contract works correctly", async () => {
        const key = await api.query.SubtensorModule.MaxChildkeyTake.getKey();

        let maxChildkeyTake = 257;
        await setMaxChildkeyTake(api, maxChildkeyTake)

        api.query.SubtensorModule.MaxChildkeyTake.getValue();
        const rawCallResponse = await publicClient.call({
            to: ISTORAGE_QUERY_ADDRESS,
            data: key.toString() as `0x${string}`,
        })
        const rawResultData = rawCallResponse.data ?? "";

        const codec = await getTypedCodecs(devnet);
        const maxChildkeyTakeCodec = codec.query.SubtensorModule.MaxChildkeyTake.value;
        const maxChildkeyTakeFromContract = maxChildkeyTakeCodec.dec(rawResultData);
        assert.equal(maxChildkeyTakeFromContract, maxChildkeyTake, "value should be 257")
    })

    it("Map type storage query call via precompile contract works correctly", async () => {

        const key = await api.query.SubtensorModule.Tempo.getKey(netuid);

        const tempoOnChain = await api.query.SubtensorModule.Tempo.getValue(netuid);
        const rawCallResponse = await publicClient.call({
            to: ISTORAGE_QUERY_ADDRESS,
            data: key.toString() as `0x${string}`,
        })
        const rawResultData = rawCallResponse.data ?? "";

        const codec = await getTypedCodecs(devnet);
        const maxChildkeyTakeValueCodec = codec.query.SubtensorModule.Tempo.value;
        const decodedValue = maxChildkeyTakeValueCodec.dec(rawResultData);
        assert.equal(tempoOnChain, decodedValue, "value should be the same as on chain")
    })

    it("Double map type storage query call via precompile contract works correctly", async () => {
        const key = await api.query.SubtensorModule.TotalHotkeyAlpha.getKey(convertPublicKeyToSs58(hotkey.publicKey), netuid);
        const totalHotkeyAlphaOnChain = await api.query.SubtensorModule.TotalHotkeyAlpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), netuid);

        const rawCallResponse = await publicClient.call({
            to: ISTORAGE_QUERY_ADDRESS,
            data: key.toString() as `0x${string}`,
        })
        const rawResultData = rawCallResponse.data ?? "";
        const codec = await getTypedCodecs(devnet);
        const totalHotkeyAlphaValueCodec = codec.query.SubtensorModule.TotalHotkeyAlpha.value;
        const decodedValue = totalHotkeyAlphaValueCodec.dec(rawResultData);
        assert.equal(totalHotkeyAlphaOnChain, decodedValue, "value should be the same as on chain")
    })

    // Polkadot api can't decode the boolean type for now.
    // it("Double map type storage query call via precompile contract works correctly", async () => {
    //     const key = await api.query.SubtensorModule.IsNetworkMember.getKey(convertPublicKeyToSs58(alice.publicKey), netuid);

    //     const isNetworkMemberOnChain = await api.query.SubtensorModule.IsNetworkMember.getValue(convertPublicKeyToSs58(alice.publicKey), netuid);
    //     const rawCallResponse = await publicClient.call({
    //         to: ISTORAGE_QUERY_ADDRESS,
    //         data: key.toString() as `0x${string}`,
    //     })

    //     const rawResultData = rawCallResponse.data ?? "";
    //     const codec = await getTypedCodecs(devnet);
    //     const isNetworkMemberValueCodec = codec.query.SubtensorModule.IsNetworkMember.value;
    //     const decodedValue = isNetworkMemberValueCodec.dec(rawResultData);
    //     assert.equal(isNetworkMemberOnChain, decodedValue, "value should be the same as on chain")
    // })

});
