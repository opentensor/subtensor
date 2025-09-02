import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { devnet } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58 } from "../src/address-utils"
import { tao } from "../src/balance-math"
import {
    forceSetBalanceToSs58Address, addNewSubnetwork, addStake,
    startCall
} from "../src/subtensor"
import { ethers } from "ethers";
import { generateRandomEthersWallet } from "../src/utils"
import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/contracts/staking"
import { log } from "console";

describe("Test staking precompile get methods", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const wallet1 = generateRandomEthersWallet();

    let api: TypedApi<typeof devnet>

    before(async () => {
        api = await getDevnetApi()
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        await addNewSubnetwork(api, hotkey, coldkey)
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
        await startCall(api, netuid, coldkey)
        console.log("will test in subnet: ", netuid)
    })

    it("Staker receives rewards", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

        await addStake(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), tao(1), coldkey)

        const contract = new ethers.Contract(
            ISTAKING_V2_ADDRESS,
            IStakingV2ABI,
            wallet1
        );

        const stake = BigInt(
            await contract.getStake(hotkey.publicKey, coldkey.publicKey, netuid)
        );

        // validator returned as bigint now. 
        const validators =
            await contract.getAlphaStakedValidators(hotkey.publicKey, netuid)

        const alpha = BigInt(
            await contract.getTotalAlphaStaked(hotkey.publicKey, netuid)
        );
        assert.ok(stake > 0)
        assert.equal(validators.length, 1)
        assert.ok(alpha > 0)

    })

    it("Get nominator min required stake", async () => {
        const contract = new ethers.Contract(
            ISTAKING_V2_ADDRESS,
            IStakingV2ABI,
            wallet1
        );

        const stake = await contract.getNominatorMinRequiredStake()
        const stakeOnChain = await api.query.SubtensorModule.NominatorMinRequiredStake.getValue()

        assert.ok(stake !== undefined)
        assert.equal(stake.toString(), stakeOnChain.toString())

    })
})
