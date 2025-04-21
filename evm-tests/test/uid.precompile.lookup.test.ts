import * as assert from "assert";

import { getAliceSigner, getDevnetApi, waitForTransactionCompletion, convertPublicKeyToMultiAddress, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
import { convertToFixedSizeBinary, generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { hexToU8a } from "@polkadot/util";
import { u64 } from "scale-ts";
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IUIDLookupABI, IUID_LOOKUP_ADDRESS } from "../src/contracts/uidLookup"
import { keccak256 } from 'ethers';
import { addNewSubnetwork, burnedRegister, forceSetBalanceToSs58Address } from "../src/subtensor";

describe("Test the UID Lookup precompile", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const evmWallet = generateRandomEthersWallet();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>

    // sudo account alice as signer
    let alice: PolkadotSigner;

    let uid: number;
    let blockNumber: number;

    // init other variable
    let netuid: number;

    before(async () => {
        // init variables got from await and async
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        // Fund the hotkey account
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))

        // Fund the coldkey account
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))

        // Add new subnet
        netuid = await addNewSubnetwork(api, hotkey, coldkey)

        // Register neuron
        const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey)
        await burnedRegister(api, netuid, hotkeyAddress, coldkey)

        uid = (await api.query.SubtensorModule.Uids.getValue(netuid, hotkeyAddress))!

        assert.notEqual(uid, undefined, "UID should be defined")

        // Associate EVM key
        blockNumber = await api.query.System.Number.getValue();
        const blockNumberBytes = u64.enc(BigInt(blockNumber));
        const blockNumberHash = hexToU8a(keccak256(blockNumberBytes));
        const concatenatedArray = new Uint8Array([...hotkey.publicKey, ...blockNumberHash]);
        const concatenatedHash = keccak256(concatenatedArray);
        const signature = await evmWallet.signMessage(concatenatedHash);
        const associateEvmKeyTx = api.tx.SubtensorModule.associate_evm_key({
            netuid: netuid,
            hotkey: convertPublicKeyToSs58(hotkey.publicKey),
            evm_key: convertToFixedSizeBinary(evmWallet.address, 20),
            block_number: BigInt(blockNumber),
            signature: convertToFixedSizeBinary(signature, 65)
        });
        await waitForTransactionCompletion(api, associateEvmKeyTx, alice)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });

        const storedEvmKey = await api.query.SubtensorModule.AssociatedEvmAddress.getValue(netuid, uid)
        assert.equal(storedEvmKey, [convertToFixedSizeBinary(evmWallet.address, 20), BigInt(blockNumber)])
    })

    it("UID lookup via precompile contract works correctly", async () => {
        // Get UID for the EVM address
        const uidArray = await publicClient.readContract({
            abi: IUIDLookupABI,
            address: toViemAddress(IUID_LOOKUP_ADDRESS),
            functionName: "uidLookup",
            args: [netuid, evmWallet.address, 1024]
        })

        assert.notEqual(uidArray, undefined, "UID should be defined")
        assert.ok(Array.isArray(uidArray), `UID should be an array, got ${typeof uidArray}`)
        assert.ok(uidArray.length > 0, "UID array should not be empty")
        assert.equal(uidArray[0], [uid, BigInt(blockNumber)])
    })
});
