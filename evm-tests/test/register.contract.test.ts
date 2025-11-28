import * as assert from "assert";
import { ethers } from "ethers";
import { getBalance, getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { convertH160ToSS58, convertPublicKeyToSs58 } from "../src/address-utils";
import { generateRandomEthersWallet } from "../src/utils";
import { addNewSubnetwork, disableWhiteListCheck, forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, startCall } from "../src/subtensor";
import { Register_ABI, Register_BYTECODE } from "../src/contracts/register";
import { raoToEth, tao } from "../src/balance-math";

describe("Test Register Contract", () => {
    let api: TypedApi<typeof devnet>;
    const wallet = generateRandomEthersWallet();
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    const hotkey2 = getRandomSubstrateKeypair();

    before(async () => {
        api = await getDevnetApi();

        // Fund the wallet and keys
        await forceSetBalanceToEthAddress(api, wallet.address);
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey));
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey));

        await addNewSubnetwork(api, hotkey, coldkey);
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;

        await startCall(api, netuid, coldkey);

        await disableWhiteListCheck(api, true);
    });

    it("Deploy Register Contract and Register Neuron", async () => {
        let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;

        // Deploy Register contract
        const factory = new ethers.ContractFactory(Register_ABI, Register_BYTECODE, wallet);
        const contractDeploy = await factory.deploy();
        await contractDeploy.waitForDeployment();
        const contractAddress = await contractDeploy.getAddress();
        console.log("Register contract deployed at:", contractAddress);

        const contract = new ethers.Contract(contractAddress, Register_ABI, wallet);
        const walletBalance = await getBalance(api, convertH160ToSS58(wallet.address));
        console.log("Wallet balance:", walletBalance);

        const registrationCost = raoToEth(tao(100)).toString();

        const tx = await contract.burnedRegisterNeuron(netuid, hotkey2.publicKey, { value: registrationCost });
        await tx.wait();

        const contractBalance = await getBalance(api, convertH160ToSS58(contractAddress));
        console.log("Contract balance:", contractBalance);

        const walletBalanceAfterRegister = await getBalance(api, convertH160ToSS58(wallet.address));
        console.log("Wallet balance:", walletBalanceAfterRegister);

        console.log("balance diff:", walletBalance - walletBalanceAfterRegister);

    });
});