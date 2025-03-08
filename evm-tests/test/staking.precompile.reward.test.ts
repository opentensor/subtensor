import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { tao } from "../src/balance-math"
import {
    forceSetBalanceToSs58Address, addNewSubnetwork, burnedRegister,
    setTxRateLimit, setTempo, setWeightsSetRateLimit, setSubnetOwnerCut, setMaxAllowedUids,
    setMinDelegateTake, becomeDelegate, setActivityCutoff, addStake, setWeight, rootRegister
} from "../src/subtensor"

describe("Test neuron precompile reveal weights", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    const validator = getRandomSubstrateKeypair();
    const miner = getRandomSubstrateKeypair();
    const nominator = getRandomSubstrateKeypair();

    let api: TypedApi<typeof devnet>

    before(async () => {
        const root_netuid = 0;
        const root_tempo = 1; // neet root epoch to happen before subnet tempo
        const subnet_tempo = 1;
        api = await getDevnetApi()

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(alice.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(validator.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(miner.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(nominator.publicKey))
        // await forceSetBalanceToEthAddress(api, wallet1.address)
        // await forceSetBalanceToEthAddress(api, wallet2.address)
        let netuid = await addNewSubnetwork(api, hotkey, coldkey)

        console.log("test the case on subnet ", netuid)

        await setTxRateLimit(api, BigInt(0))
        await setTempo(api, root_netuid, root_tempo)
        await setTempo(api, netuid, subnet_tempo)
        await setWeightsSetRateLimit(api, netuid, BigInt(0))

        await burnedRegister(api, netuid, convertPublicKeyToSs58(validator.publicKey), coldkey)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(miner.publicKey), coldkey)
        await burnedRegister(api, netuid, convertPublicKeyToSs58(nominator.publicKey), coldkey)
        await setSubnetOwnerCut(api, 0)
        await setActivityCutoff(api, netuid, 65535)
        await setMaxAllowedUids(api, netuid, 65535)
        await setMinDelegateTake(api, 0)
        await becomeDelegate(api, convertPublicKeyToSs58(validator.publicKey), coldkey)
        await becomeDelegate(api, convertPublicKeyToSs58(miner.publicKey), coldkey)
    })

    it("Staker receives rewards", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        await addStake(api, netuid, convertPublicKeyToSs58(miner.publicKey), tao(1), coldkey)
        await addStake(api, netuid, convertPublicKeyToSs58(nominator.publicKey), tao(1), coldkey)

        await addStake(api, netuid, convertPublicKeyToSs58(validator.publicKey), tao(100), coldkey)

        const miner_alpha_before_emission = await api.query.SubtensorModule.Alpha.getValue(
            convertPublicKeyToSs58(miner.publicKey),
            convertPublicKeyToSs58(coldkey.publicKey),
            netuid
        )

        await setWeight(api, netuid, [0, 1], [0xffff, 0xffff], BigInt(0), validator)
        await rootRegister(api, convertPublicKeyToSs58(validator.publicKey), coldkey)

        let index = 0;
        while (index < 60) {
            const pending = await api.query.SubtensorModule.PendingEmission.getValue(netuid);
            if (pending > 0) {
                console.log("pending amount is ", pending);
                break;
            }

            await new Promise((resolve) => setTimeout(resolve, 1000));
            console.log("wait for the pendingEmission update");
            index += 1;
        }

        index = 0;
        while (index < 60) {
            let miner_current_alpha = await api.query.SubtensorModule.Alpha.getValue(
                convertPublicKeyToSs58(miner.publicKey),
                convertPublicKeyToSs58(coldkey.publicKey),
                netuid
            )

            if (miner_current_alpha > miner_alpha_before_emission) {
                console.log("miner got reward");
                break;
            }

            await new Promise((resolve) => setTimeout(resolve, 1000));
            console.log(" waiting for emission");
            index += 1;
        }
    })
})
