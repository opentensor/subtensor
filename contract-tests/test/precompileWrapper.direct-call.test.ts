import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, getBalance } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils";
import { tao, raoToEth } from "../src/balance-math";
import {
    forceSetBalanceToSs58Address,
    addNewSubnetwork,
    addStake,
    startCall,
    disableWhiteListCheck,
} from "../src/subtensor";
import { ethers } from "ethers";
import { generateRandomEthersWallet } from "../src/utils";
import { PRECOMPILE_WRAPPER_ABI, PRECOMPILE_WRAPPER_BYTECODE } from "../src/contracts/precompileWrapper";

describe("PrecompileWrapper - Direct Call Tests", () => {
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const wallet1 = generateRandomEthersWallet();

    let api: TypedApi<typeof devnet>;
    let wrapperContract: ethers.Contract;
    let wrapperAddress: string;
    let netuid: number;

    before(async () => {
        api = await getDevnetApi();
        await disableWhiteListCheck(api, true)
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey));
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey));
        await forceSetBalanceToSs58Address(api, convertH160ToSS58(wallet1.address));
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
        await forceSetBalanceToSs58Address(api, convertH160ToSS58(wrapperAddress));

        console.log("Wrapper contract deployed at:", wrapperAddress);
        console.log("Testing in subnet:", netuid);

        wrapperContract = new ethers.Contract(wrapperAddress, PRECOMPILE_WRAPPER_ABI, wallet1);
    });

    describe("Balance Transfer Precompile Direct Calls", () => {
        it("Should transfer balance via wrapper", async () => {
            const keypair = getRandomSubstrateKeypair();
            const transfer = await wrapperContract.transfer(keypair.publicKey, { value: raoToEth(tao(1)).toString() });
            await transfer.wait();

            const balance = await getBalance(api, convertPublicKeyToSs58(keypair.publicKey));

            assert.equal(balance, tao(1), "Wrapper and direct calls should match");
        })
    });

});
