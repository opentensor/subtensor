import * as assert from "assert";

import { getAliceSigner, getDevnetApi, waitForTransactionCompletion, getRandomSubstrateKeypair, getSignerFromKeypair } from "../src/substrate"
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
import { addNewSubnetwork, forceSetBalanceToSs58Address, startCall } from "../src/subtensor";

describe("Test the UID Lookup precompile", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const evmWallet = generateRandomEthersWallet();
    let publicClient: PublicClient;

    let api: TypedApi<typeof devnet>

    let alice: PolkadotSigner;

    let uid: number;
    let blockNumber: number;
    let netuid: number;
    let blockNumberAssociated: bigint;

    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        alice = await getAliceSigner();

        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))

        netuid = await addNewSubnetwork(api, hotkey, coldkey)
        await startCall(api, netuid, coldkey)

        const maybeUid = await api.query.SubtensorModule.Uids.getValue(netuid, convertPublicKeyToSs58(hotkey.publicKey))

        if (maybeUid === undefined) {
            throw new Error("UID should be defined")
        }
        uid = maybeUid

        // Associate EVM key
        blockNumber = await api.query.System.Number.getValue();
        const blockNumberBytes = u64.enc(BigInt(blockNumber));
        const blockNumberHash = hexToU8a(keccak256(blockNumberBytes));
        const concatenatedArray = new Uint8Array([...hotkey.publicKey, ...blockNumberHash]);
        const signature = await evmWallet.signMessage(concatenatedArray);
        const associateEvmKeyTx = api.tx.SubtensorModule.associate_evm_key({
            netuid: netuid,
            hotkey: convertPublicKeyToSs58(hotkey.publicKey),
            evm_key: convertToFixedSizeBinary(evmWallet.address, 20),
            block_number: BigInt(blockNumber),
            signature: convertToFixedSizeBinary(signature, 65)
        });
        const signer = getSignerFromKeypair(coldkey);
        await waitForTransactionCompletion(api, associateEvmKeyTx, signer)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });

        const storedEvmKey = await api.query.SubtensorModule.AssociatedEvmAddress.getValue(netuid, uid)
        assert.notEqual(storedEvmKey, undefined, "storedEvmKey should be defined")
        if (storedEvmKey !== undefined) {
            assert.equal(storedEvmKey[0].asHex(), convertToFixedSizeBinary(evmWallet.address, 20).asHex())
            blockNumberAssociated = storedEvmKey[1]
        }
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
        assert.deepStrictEqual(uidArray[0], { uid: uid, block_associated: blockNumberAssociated })
    })
});
