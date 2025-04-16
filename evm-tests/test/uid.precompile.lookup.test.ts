import * as assert from "assert";

import { getAliceSigner, getDevnetApi, waitForTransactionCompletion, convertPublicKeyToMultiAddress, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
import { convertToFixedSizeBinary, generateRandomEthersWallet, getPublicClient, hexStringToUint8Array } from "../src/utils";
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { toViemAddress, convertPublicKeyToSs58 } from "../src/address-utils"
import { IUIDLookupABI, IUID_LOOKUP_ADDRESS } from "../src/contracts/uidLookup"
import { keccak256 } from 'ethers';

describe("Test the UID Lookup precompile", () => {
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const evmWallet = generateRandomEthersWallet();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>

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

        // Register neuron
        const signer = getSignerFromKeypair(coldkey)
        const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey)
        const tx = api.tx.SubtensorModule.burned_register({ hotkey: hotkeyAddress, netuid: subnetId })
        await waitForTransactionCompletion(api, tx, signer)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });

        // Associate EVM key
        const blockNumber = await api.query.System.Number.getValue();
        const blockNumberBytes = hexStringToUint8Array("0x" + blockNumber.toString(16));
        const blockNumberHash = hexStringToUint8Array(keccak256(blockNumberBytes));
        const concatenatedArray = new Uint8Array([...hotkey.publicKey, ...blockNumberHash]);
        const concatenatedHash = keccak256(concatenatedArray);
        const signature = await evmWallet.signMessage(concatenatedHash);
        const associateEvmKeyTx = api.tx.SubtensorModule.associate_evm_key({
            netuid: subnetId,
            hotkey: convertPublicKeyToSs58(hotkey.publicKey),
            evm_key: convertToFixedSizeBinary(evmWallet.address, 20),
            block_number: BigInt(blockNumber),
            signature: convertToFixedSizeBinary(signature, 65)
        });
        await waitForTransactionCompletion(api, associateEvmKeyTx, alice)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });
    })

    it("UID lookup via precompile contract works correctly", async () => {
        // Get UID for the EVM address
        const uidArray = await publicClient.readContract({
            abi: IUIDLookupABI,
            address: toViemAddress(IUID_LOOKUP_ADDRESS),
            functionName: "uidLookup",
            args: [subnetId, evmWallet.address, 1024]
        })

        console.info(uidArray)
        assert.ok(uidArray !== undefined, "UID should be defined")
        assert.ok(Array.isArray(uidArray), `UID should be an array, got ${typeof uidArray}`)
        assert.ok(uidArray.length > 0, "UID array should not be empty")
    })
});
