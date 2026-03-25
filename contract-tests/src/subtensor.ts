import * as assert from "assert";
import { devnet, MultiAddress } from '@polkadot-api/descriptors';
import { TypedApi, TxCallData, Binary, Enum, getTypedCodecs } from 'polkadot-api';
import { KeyPair } from "@polkadot-labs/hdkd-helpers"
import { getAliceSigner, waitForTransactionCompletion, getSignerFromKeypair, waitForTransactionWithRetry } from './substrate'
import { convertH160ToSS58, convertPublicKeyToSs58, ethAddressToH160 } from './address-utils'
import { tao } from './balance-math'
import internal from "stream";
import { createCodec } from "scale-ts";

// create a new subnet and return netuid
export async function addNewSubnetwork(api: TypedApi<typeof devnet>, hotkey: KeyPair, coldkey: KeyPair) {
    const alice = getAliceSigner()
    const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()

    const defaultNetworkLastLockCost = await api.query.SubtensorModule.NetworkLastLockCost.getValue()

    const rateLimit = await api.query.SubtensorModule.NetworkRateLimit.getValue()
    if (rateLimit !== BigInt(0)) {
        const internalCall = api.tx.AdminUtils.sudo_set_network_rate_limit({ rate_limit: BigInt(0) })
        const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })
        await waitForTransactionWithRetry(api, tx, alice)
    }

    const signer = getSignerFromKeypair(coldkey)
    const registerNetworkTx = api.tx.SubtensorModule.register_network({ hotkey: convertPublicKeyToSs58(hotkey.publicKey) })
    await waitForTransactionWithRetry(api, registerNetworkTx, signer)

    const newTotalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()
    // could create multiple subnetworks during retry, just return the first created one
    assert.ok(newTotalNetworks > totalNetworks)

    // reset network last lock cost to 0, to avoid the lock cost calculation error
    await setNetworkLastLockCost(api, defaultNetworkLastLockCost)
    return totalNetworks
}

// force set balance for a ss58 address
export async function forceSetBalanceToSs58Address(api: TypedApi<typeof devnet>, ss58Address: string) {
    const alice = getAliceSigner()
    const balance = tao(1e10)
    const internalCall = api.tx.Balances.force_set_balance({ who: MultiAddress.Id(ss58Address), new_free: balance })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)

    const balanceOnChain = (await api.query.System.Account.getValue(ss58Address)).data.free
    // check the balance except for sudo account becasue of tx fee
    if (ss58Address !== convertPublicKeyToSs58(alice.publicKey)) {
        assert.equal(balance, balanceOnChain)
    }
}

// set balance for an eth address
export async function forceSetBalanceToEthAddress(api: TypedApi<typeof devnet>, ethAddress: string) {
    const ss58Address = convertH160ToSS58(ethAddress)
    await forceSetBalanceToSs58Address(api, ss58Address)
}

export async function setCommitRevealWeightsEnabled(api: TypedApi<typeof devnet>, netuid: number, enabled: boolean) {
    const value = await api.query.SubtensorModule.CommitRevealWeightsEnabled.getValue(netuid)
    if (value === enabled) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.AdminUtils.sudo_set_commit_reveal_weights_enabled({ netuid: netuid, enabled: enabled })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(enabled, await api.query.SubtensorModule.CommitRevealWeightsEnabled.getValue(netuid))
}

export async function setWeightsSetRateLimit(api: TypedApi<typeof devnet>, netuid: number, rateLimit: bigint) {
    const value = await api.query.SubtensorModule.WeightsSetRateLimit.getValue(netuid)
    if (value === rateLimit) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.AdminUtils.sudo_set_weights_set_rate_limit({ netuid: netuid, weights_set_rate_limit: rateLimit })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(rateLimit, await api.query.SubtensorModule.WeightsSetRateLimit.getValue(netuid))
}

// tempo is u16 in rust, but we just number in js. so value should be less than u16::Max
export async function setTempo(api: TypedApi<typeof devnet>, netuid: number, tempo: number) {
    const value = await api.query.SubtensorModule.Tempo.getValue(netuid)
    console.log("init avlue is ", value)
    if (value === tempo) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.AdminUtils.sudo_set_tempo({ netuid: netuid, tempo: tempo })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(tempo, await api.query.SubtensorModule.Tempo.getValue(netuid))
}

export async function setCommitRevealWeightsInterval(api: TypedApi<typeof devnet>, netuid: number, interval: bigint) {
    const value = await api.query.SubtensorModule.RevealPeriodEpochs.getValue(netuid)
    if (value === interval) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.AdminUtils.sudo_set_commit_reveal_weights_interval({ netuid: netuid, interval: interval })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(interval, await api.query.SubtensorModule.RevealPeriodEpochs.getValue(netuid))
}


export async function forceSetChainID(api: TypedApi<typeof devnet>, chainId: bigint) {
    const value = await api.query.EVMChainId.ChainId.getValue()
    if (value === chainId) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.AdminUtils.sudo_set_evm_chain_id({ chain_id: chainId })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(chainId, await api.query.EVMChainId.ChainId.getValue())
}

export async function disableWhiteListCheck(api: TypedApi<typeof devnet>, disabled: boolean) {
    const value = await api.query.EVM.DisableWhitelistCheck.getValue()
    if (value === disabled) {
        return;
    }

    const alice = getAliceSigner()
    const internalCall = api.tx.EVM.disable_whitelist({ disabled: disabled })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(disabled, await api.query.EVM.DisableWhitelistCheck.getValue())
}

export async function burnedRegister(api: TypedApi<typeof devnet>, netuid: number, ss58Address: string, keypair: KeyPair) {
    const registered = await api.query.SubtensorModule.Uids.getValue(netuid, ss58Address);
    // just return if already registered
    if (registered !== undefined) {
        console.log("hotkey ", ss58Address, " already registered in netuid ", netuid)
        return;
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
    const uids = await api.query.SubtensorModule.SubnetworkN.getValue(netuid)
    const signer = getSignerFromKeypair(keypair)
    const tx = api.tx.SubtensorModule.burned_register({ hotkey: ss58Address, netuid: netuid })
    await waitForTransactionWithRetry(api, tx, signer)
    assert.equal(uids + 1, await api.query.SubtensorModule.SubnetworkN.getValue(netuid))
}


export async function sendProxyCall(api: TypedApi<typeof devnet>, calldata: TxCallData, ss58Address: string, keypair: KeyPair) {
    const signer = getSignerFromKeypair(keypair)
    const tx = api.tx.Proxy.proxy({
        call: calldata,
        real: MultiAddress.Id(ss58Address),
        force_proxy_type: undefined
    });
    await waitForTransactionWithRetry(api, tx, signer)
}


export async function setTxRateLimit(api: TypedApi<typeof devnet>, txRateLimit: bigint) {
    const value = await api.query.SubtensorModule.TxRateLimit.getValue()
    if (value === txRateLimit) {
        return;
    }
    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_tx_rate_limit({ tx_rate_limit: txRateLimit })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })


    await waitForTransactionWithRetry(api, tx, alice)
}

export async function setMaxAllowedValidators(api: TypedApi<typeof devnet>, netuid: number, maxAllowedValidators: number) {
    const value = await api.query.SubtensorModule.MaxAllowedValidators.getValue(netuid)
    if (value === maxAllowedValidators) {
        return;
    }

    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_max_allowed_validators({
        netuid: netuid,
        max_allowed_validators: maxAllowedValidators
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(maxAllowedValidators, await api.query.SubtensorModule.MaxAllowedValidators.getValue(netuid))
}

export async function setSubnetOwnerCut(api: TypedApi<typeof devnet>, subnetOwnerCut: number) {
    const value = await api.query.SubtensorModule.SubnetOwnerCut.getValue()
    if (value === subnetOwnerCut) {
        return;
    }

    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_subnet_owner_cut({
        subnet_owner_cut: subnetOwnerCut
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(subnetOwnerCut, await api.query.SubtensorModule.SubnetOwnerCut.getValue())
}

export async function setActivityCutoff(api: TypedApi<typeof devnet>, netuid: number, activityCutoff: number) {
    const value = await api.query.SubtensorModule.ActivityCutoff.getValue(netuid)
    if (value === activityCutoff) {
        return;
    }

    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_activity_cutoff({
        netuid: netuid,
        activity_cutoff: activityCutoff
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(activityCutoff, await api.query.SubtensorModule.ActivityCutoff.getValue(netuid))
}

export async function setMinDelegateTake(api: TypedApi<typeof devnet>, minDelegateTake: number) {
    const value = await api.query.SubtensorModule.MinDelegateTake.getValue()
    if (value === minDelegateTake) {
        return;
    }

    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_min_delegate_take({
        take: minDelegateTake
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
    assert.equal(minDelegateTake, await api.query.SubtensorModule.MinDelegateTake.getValue())
}

export async function addStake(api: TypedApi<typeof devnet>, netuid: number, ss58Address: string, amount_staked: bigint, keypair: KeyPair) {
    const signer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.add_stake({
        netuid: netuid,
        hotkey: ss58Address,
        amount_staked: amount_staked
    })

    await waitForTransactionWithRetry(api, tx, signer)
}

export async function setWeight(api: TypedApi<typeof devnet>, netuid: number, dests: number[], weights: number[], version_key: bigint, keypair: KeyPair) {
    const signer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.set_weights({
        netuid: netuid,
        dests: dests,
        weights: weights,
        version_key: version_key
    })

    await waitForTransactionWithRetry(api, tx, signer)
}

export async function rootRegister(api: TypedApi<typeof devnet>, ss58Address: string, keypair: KeyPair) {
    const signer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.root_register({
        hotkey: ss58Address
    })

    await waitForTransactionWithRetry(api, tx, signer)
}

export async function setSubtokenEnable(api: TypedApi<typeof devnet>, netuid: number, subtokenEnable: boolean) {
    const signer = getAliceSigner()
    let internalTx = api.tx.AdminUtils.sudo_set_subtoken_enabled({
        netuid: netuid,
        subtoken_enabled: subtokenEnable
    })
    let tx = api.tx.Sudo.sudo({ call: internalTx.decodedCall })

    await waitForTransactionWithRetry(api, tx, signer)
}

export async function startCall(api: TypedApi<typeof devnet>, netuid: number, keypair: KeyPair) {
    const registerBlock = Number(await api.query.SubtensorModule.NetworkRegisteredAt.getValue(netuid))
    let currentBlock = await api.query.System.Number.getValue()
    const duration = Number(await api.constants.SubtensorModule.InitialStartCallDelay)

    while (currentBlock - registerBlock <= duration) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        currentBlock = await api.query.System.Number.getValue()
    }
    // wait for chain to run coinbase
    await new Promise((resolve) => setTimeout(resolve, 2000));

    const signer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.start_call({
        netuid: netuid,
    })

    await waitForTransactionWithRetry(api, tx, signer)

    await new Promise((resolve) => setTimeout(resolve, 1000));
    const callStarted = await api.query.SubtensorModule.FirstEmissionBlockNumber
        .getValue(netuid);
    assert.notEqual(callStarted, undefined);
}

export async function setMaxChildkeyTake(api: TypedApi<typeof devnet>, take: number) {
    const alice = getAliceSigner()
    const internalCall = api.tx.SubtensorModule.sudo_set_max_childkey_take({ take })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionWithRetry(api, tx, alice)
}

// Swap coldkey to contract address
export async function swapColdkey(
    api: TypedApi<typeof devnet>,
    coldkey: KeyPair,
    contractAddress: string,
) {
    const alice = getAliceSigner();
    const internal_tx = api.tx.SubtensorModule.swap_coldkey({
        old_coldkey: convertPublicKeyToSs58(coldkey.publicKey),
        new_coldkey: convertH160ToSS58(contractAddress),
        swap_cost: tao(10),
    });
    const tx = api.tx.Sudo.sudo({
        call: internal_tx.decodedCall,
    });
    await waitForTransactionWithRetry(api, tx, alice);
}

// Set target registrations per interval to 1000
export async function setTargetRegistrationsPerInterval(
    api: TypedApi<typeof devnet>,
    netuid: number,
) {
    const alice = getAliceSigner();
    const internal_tx = api.tx.AdminUtils
        .sudo_set_target_registrations_per_interval({
            netuid,
            target_registrations_per_interval: 1000,
        });
    const tx = api.tx.Sudo.sudo({
        call: internal_tx.decodedCall,
    });
    await waitForTransactionWithRetry(api, tx, alice);
}

// Disable admin freeze window and owner hyperparam rate limiting for tests
export async function disableAdminFreezeWindowAndOwnerHyperparamRateLimit(api: TypedApi<typeof devnet>) {
    const alice = getAliceSigner()

    const currentAdminFreezeWindow = await api.query.SubtensorModule.AdminFreezeWindow.getValue()
    if (currentAdminFreezeWindow !== 0) {
        // Set AdminFreezeWindow to 0
        const setFreezeWindow = api.tx.AdminUtils.sudo_set_admin_freeze_window({ window: 0 })
        const sudoFreezeTx = api.tx.Sudo.sudo({ call: setFreezeWindow.decodedCall })
        await waitForTransactionWithRetry(api, sudoFreezeTx, alice)
    }

    const currentOwnerHyperparamRateLimit = await api.query.SubtensorModule.OwnerHyperparamRateLimit.getValue()
    if (currentOwnerHyperparamRateLimit !== 0) {
        // Set OwnerHyperparamRateLimit to 0
        const setOwnerRateLimit = api.tx.AdminUtils.sudo_set_owner_hparam_rate_limit({ epochs: 0 })
        const sudoOwnerRateTx = api.tx.Sudo.sudo({ call: setOwnerRateLimit.decodedCall })
        await waitForTransactionWithRetry(api, sudoOwnerRateTx, alice)
    }

    assert.equal(0, await api.query.SubtensorModule.AdminFreezeWindow.getValue())
    assert.equal(BigInt(0), await api.query.SubtensorModule.OwnerHyperparamRateLimit.getValue())
}

export async function sendWasmContractExtrinsic(api: TypedApi<typeof devnet>, coldkey: KeyPair, contractAddress: string, data: Binary) {
    const signer = getSignerFromKeypair(coldkey)
    const tx = await api.tx.Contracts.call({
        value: BigInt(0),
        dest: MultiAddress.Id(contractAddress),
        data: Binary.fromBytes(data.asBytes()),
        gas_limit: {
            ref_time: BigInt(10000000000),
            proof_size: BigInt(10000000),
        },
        storage_deposit_limit: BigInt(1000000000)
    })
    await waitForTransactionWithRetry(api, tx, signer)
}

export async function setNetworkLastLockCost(api: TypedApi<typeof devnet>, defaultNetworkLastLockCost: bigint) {
    const alice = getAliceSigner()
    const key = await api.query.SubtensorModule.NetworkLastLockCost.getKey()
    const codec = await getTypedCodecs(devnet);
    const value = codec.query.SubtensorModule.NetworkLastLockCost.value.enc(defaultNetworkLastLockCost)
    const internalCall = api.tx.System.set_storage({
        items: [[Binary.fromHex(key), Binary.fromBytes(value)]]
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })
    await waitForTransactionWithRetry(api, tx, alice)

    const valueOnChain = await api.query.SubtensorModule.NetworkLastLockCost.getValue()
    assert.equal(defaultNetworkLastLockCost, valueOnChain)
}

export function getSubnetAccountId(netuid: number): string {
    // Hardcode to speed up tests
    const NETUID_TO_ACCOUNT_ID: Record<number, string> = {
        0: "5EYCAe5jLQhn6ofDSvqF6iY53erXNkwhyE1aCEgvi1NNs91F",
        1: "5EYCAe5jLQhn6ofDSvqWqk5fA9XiqK3ahtx5kBNmAqF78mqL",
        2: "5EYCAe5jLQhn6ofDSvqnamdFGeCvHs9TSZtbJ84bdf7qQRc6",
        3: "5EYCAe5jLQhn6ofDSvr4KoAqP8t7kRFLBEq6r4kS6UzZgCb5",
        4: "5EYCAe5jLQhn6ofDSvrL4piRVdZKCyMCuumcQ1SGZJsHwmeE",
        5: "5EYCAe5jLQhn6ofDSvrborG1c8EWfXT5eai7wx8728k2DHK7",
        6: "5EYCAe5jLQhn6ofDSvrsYsobicui85YxPFedVtowUxckUuF8",
        7: "5EYCAe5jLQhn6ofDSvs9HuMBq7auadeq7vb93qVmwnVUkg5A",
        8: "5EYCAe5jLQhn6ofDSvsR2vtmwcG73BkhrbXebnBcQcND2Bdh",
        9: "5EYCAe5jLQhn6ofDSvsgmxSN46wJVjrabGUA9isSsSEwHnFy",
        10: "5EYCAe5jLQhn6ofDSvsxWyyxAbcVxHxTKwQfhfZHLG7fZUJG",
        11: "5EYCAe5jLQhn6ofDSvtEG1XYH6HhQr4L4cMBFcF7o5zPpyA3",
        12: "5EYCAe5jLQhn6ofDSvtW1358PaxtsQACoHHgoYvxFus86kJK",
        13: "5EYCAe5jLQhn6ofDSvtmk4ciW5e6KxG5XxECMVcnijjrN8rz",
        14: "5EYCAe5jLQhn6ofDSvu3V6AJcaKHnWMxGdAhuSJdBZcadwDn",
        15: "5EYCAe5jLQhn6ofDSvuKE7htj4zVF4Tq1J7DTNzTePVJucfX",
        16: "5EYCAe5jLQhn6ofDSvuay9FUqZfghcZhjy3j1KgJ7DN3BDc2",
        17: "5EYCAe5jLQhn6ofDSvuriAo4x4LtAAfaUdzEZGN8a3EmSncG",
        18: "5EYCAe5jLQhn6ofDSvv8TCLf4Z25cimTDJvk7D3y2s7ViZEm",
        19: "5EYCAe5jLQhn6ofDSvvQCDtFB3hH5GsKwysFf9joVgzDytnb",
        20: "5EYCAe5jLQhn6ofDSvvfwFRqHYNUXpyCgeomD6RdxWrxFpQR",
        21: "5EYCAe5jLQhn6ofDSvvwgGyRQ33fzP55RKkGm37URLjgXG7M",
        22: "5EYCAe5jLQhn6ofDSvwDRJX1WXisSwAx9zgnJyoJtAcQo59Y",
        23: "5EYCAe5jLQhn6ofDSvwVAL4bd2Q4uVGptfdHrvV9LzV94VBb",
        24: "5EYCAe5jLQhn6ofDSvwkuMcBjX5GN3NhdLZoQsAyopMsL7A7",
        25: "5EYCAe5jLQhn6ofDSvx2eP9mr1kTpbUaN1WJxorpGeEbbfgG",
        26: "5EYCAe5jLQhn6ofDSvxJPQhMxWRfH9aT6gSpWkYejU7KsbGp",
        27: "5EYCAe5jLQhn6ofDSvxa8SEx516rjhgKqMPL4hEVCHz49DPw",
        28: "5EYCAe5jLQhn6ofDSvxqsTnYBVn4CFnCa2KqcdvKf7rnQo7f",
        29: "5EYCAe5jLQhn6ofDSvy7cVL8HzTFeot5JhGMAacA7wjWgPix",
        30: "5EYCAe5jLQhn6ofDSvyPMWsiQV8T7Myx3NCriXHzamcEwyqa",
        31: "5EYCAe5jLQhn6ofDSvyf6YRJWyoeZv5pn39NGTyq3bUyDc8k",
        32: "5EYCAe5jLQhn6ofDSvyvqZxtdUUr2UBhWi5spQffWRMhV5hU",
        33: "5EYCAe5jLQhn6ofDSvzCabWUjyA3V2HaFP2PNMMVyFERkxPm",
        34: "5EYCAe5jLQhn6ofDSvzUKd44rTqEwaPSz3xtvJ3LS57A2Td3",
        35: "5EYCAe5jLQhn6ofDSvzk4ebexxWSQ8VKiiuQUEjAttytJ8Nx",
        36: "5EYCAe5jLQhn6ofDSw11og9F5TBdrgbCTPqv2BR1MircZp68",
        37: "5EYCAe5jLQhn6ofDSw1HYhgqBwrqKEh5C4nRa86qpYjLqCQd",
        38: "5EYCAe5jLQhn6ofDSw1ZHjERJSY2mnnwvjiw84ngHNc56t9n",
        39: "5EYCAe5jLQhn6ofDSw1q2kn1QwDEELtpfQfSg1UWkCUoNVFh",
        40: "5EYCAe5jLQhn6ofDSw26mnKbXRtRgtzhQ5bxDxAMD2MXeL6A",
        41: "5EYCAe5jLQhn6ofDSw2NWosBdvZd9T6a8kYTmtrBfrEFusmX",
        42: "5EYCAe5jLQhn6ofDSw2eFqQmkREpc1CSsRUyKqY28g6zBbxD",
        43: "5EYCAe5jLQhn6ofDSw2uzrxMruv24ZJKc6RUsnDrbVyiT4uZ",
        44: "5EYCAe5jLQhn6ofDSw3BjtVwyQbDX7QCLmMzRiuh4KrSienC",
        45: "5EYCAe5jLQhn6ofDSw3TUv3Y5uGQyfW55SJVyfbXX9jAzHBc",
        46: "5EYCAe5jLQhn6ofDSw3jDwb8CPwcSDbwp7F1XcHMyybuFsV6",
        47: "5EYCAe5jLQhn6ofDSw3zxy8iJtcotmhpYnBX5YyCSoUdXS9C",
        48: "5EYCAe5jLQhn6ofDSw4GhzgJRPJ1MKohHT82dVf2udMMo854",
        49: "5EYCAe5jLQhn6ofDSw4YT2DtXsyCosua284YBSLsNTE64sjn",
        50: "5EYCAe5jLQhn6ofDSw4pC3mUeNeQGS1Sko13jP2hqH6pLJc1",
        51: "5EYCAe5jLQhn6ofDSw55w5K4ksKbiz7KVTwZHKiYJ6yYc62p",
        52: "5EYCAe5jLQhn6ofDSw5Mg6resMzoBYDCE8t4qGQNkvrGsYLi",
        53: "5EYCAe5jLQhn6ofDSw5dR8QEyrfze6K4xopaPD6DDkj19BcH",
        54: "5EYCAe5jLQhn6ofDSw5uA9wq6MMC6eQwhUm5w9n3gabjR243",
        55: "5EYCAe5jLQhn6ofDSw6AuBVRCr2PZCWpS9hbV6Tt9QUTghER",
        56: "5EYCAe5jLQhn6ofDSw6SeD31KLhb1kchApe7339icEMBxKr7",
        57: "5EYCAe5jLQhn6ofDSw6iPEabRqNnUJiZuVacayqZ54DvDhGB",
        58: "5EYCAe5jLQhn6ofDSw6z8G8BYL3yvrpSeAX88vXPXt6eVRoY",
        59: "5EYCAe5jLQhn6ofDSw7FsHfmepjBPQvKNqTdgsDDzhyNky3e",
        60: "5EYCAe5jLQhn6ofDSw7XcKDMmKQNqy2C7WQ9Eou4TXr72f1B",
        61: "5EYCAe5jLQhn6ofDSw7oMLkwsp5aJX84rBLenkatvMiqJC2W",
        62: "5EYCAe5jLQhn6ofDSw856NJXzJkmm5DwarHALhGjPBbZa5wW",
        63: "5EYCAe5jLQhn6ofDSw8LqPr86oRyDdKpKXDftdxZr1UHqWzq",
        64: "5EYCAe5jLQhn6ofDSw8caRPiDJ7AgBRh4CABSaeQJqM278gq",
        65: "5EYCAe5jLQhn6ofDSw8tKSwJKnnN8jXZns6gzXLEmfDkNtXu",
        66: "5EYCAe5jLQhn6ofDSw9A4UUtSHTZbHdSXY3CYU25EV6UeLH2",
        67: "5EYCAe5jLQhn6ofDSw9RoW2UYn8m3qjKGCyi6QhuhJyCv9nu",
        68: "5EYCAe5jLQhn6ofDSw9hYXa4fGoxWPqBzsvDeMPkA8qwBecQ",
        69: "5EYCAe5jLQhn6ofDSw9yHZ7emmV9xww4jYrjCJ5acxifTH7b",
        70: "5EYCAe5jLQhn6ofDSwAF2afEtGAMRW2wUDoEkEmR5nbPiuFf",
        71: "5EYCAe5jLQhn6ofDSwAWmcCpzkqYt48pCtjkJBTFYcU7ziWG",
        72: "5EYCAe5jLQhn6ofDSwAnWdkR7FWkLcEgwZgFr8961SLrGPJp",
        73: "5EYCAe5jLQhn6ofDSwB4FfJ1DkBwoALZgEcmQ4pvUGDaXxGw",
        74: "5EYCAe5jLQhn6ofDSwBKzgqbLEs9FiSSQuZGx1Wkw66JoNQY",
        75: "5EYCAe5jLQhn6ofDSwBbjiPBSjYLiGYK9aVnVxCbPuy357eQ",
        76: "5EYCAe5jLQhn6ofDSwBsUjvmZEDYApeBtFSJ3ttRrjqmLmRP",
        77: "5EYCAe5jLQhn6ofDSwC9DmUMfitjdNk4cvNobqaGKZiVcSd4",
        78: "5EYCAe5jLQhn6ofDSwCQxo1wnDZw5vqwMbKK9nG6nPbDsr3v",
        79: "5EYCAe5jLQhn6ofDSwCghpZXtiF8YUwp6GFphiwwFDTx9ZXw",
        80: "5EYCAe5jLQhn6ofDSwCxSr781CvL133gpwCLFfdmi3LgRGUs",
        81: "5EYCAe5jLQhn6ofDSwDEBsei7hbXTb9ZZc8qocKcAsDQgmDH",
        82: "5EYCAe5jLQhn6ofDSwDVvuCJECGiv9FSJH5MMZ1Sdh68xe6G",
        83: "5EYCAe5jLQhn6ofDSwDmfvjtLgwvNhMK2x1ruVhH6WxsE2Rh",
        84: "5EYCAe5jLQhn6ofDSwE3QxHUTBd7qFTBmcxNTSP7ZLqbVqHX",
        85: "5EYCAe5jLQhn6ofDSwEK9yq4ZgJKHoZ4WHtt1P4x2AiKmP2V",
        86: "5EYCAe5jLQhn6ofDSwEau1NegAyWkMewExqPZKknUzb42r36",
        87: "5EYCAe5jLQhn6ofDSwEre2vEnfeiCukoydmu7GScwpTnJa5d",
        88: "5EYCAe5jLQhn6ofDSwF8P4TpuAKufTrgiJiQfD8TQeLWaGop",
        89: "5EYCAe5jLQhn6ofDSwFQ861R1f1781xZSyevD9pHsUDEqiBR",
        90: "5EYCAe5jLQhn6ofDSwFfs7Z189gJaa4SBebRm6W8LJ5y7dfH",
        91: "5EYCAe5jLQhn6ofDSwFwc96bEeMW38AJvKXwK3Bxo7xhP3yn",
        92: "5EYCAe5jLQhn6ofDSwGDMAeBM92hVgGBezUSrysoFwqReqrS",
        93: "5EYCAe5jLQhn6ofDSwGV6CBmTdhtxEN4PfQxQvZdimi9vW9r",
        94: "5EYCAe5jLQhn6ofDSwGkqDjMa8P6QnTw8LMTxsFUBbatC8C5",
        95: "5EYCAe5jLQhn6ofDSwH2aFGwgd4HsLZos1HyWowJeRTcTVsg",
        96: "5EYCAe5jLQhn6ofDSwHJKGpXo7jVKtfgbgEV4kd97FLLjBeJ",
        97: "5EYCAe5jLQhn6ofDSwHa4JN7ucQgnSmZLMAzchJya5D4zq8v",
        98: "5EYCAe5jLQhn6ofDSwHqoKui275tEzsS527WAdzp2u5oGNSd",
        99: "5EYCAe5jLQhn6ofDSwJ7YMTJ8bm5hYyJoh41iageVixXYH59",
        100: "5EYCAe5jLQhn6ofDSwJPHNztF6SHA75BYMzXGXNUxYqFoj9g",
        101: "5EYCAe5jLQhn6ofDSwJf2QYUMb7UcfB4H2w2pU4KRNhz5GP5",
        102: "5EYCAe5jLQhn6ofDSwJvmS64U5ng5DGw1hsYNQk9tCaiLvoS",
        103: "5EYCAe5jLQhn6ofDSwKCWTdeaaTsXmNokNp3vMRzM2TScknA",
        104: "5EYCAe5jLQhn6ofDSwKUFVBEh594zKUgV3kZUJ7porLAtE76",
        105: "5EYCAe5jLQhn6ofDSwKjzWipoZpGSsaZDih52EofGgCu9mbP",
        106: "5EYCAe5jLQhn6ofDSwL1jYGQv4VTuRgRxPdaaBVVjW5dRU9u",
        107: "5EYCAe5jLQhn6ofDSwLHUZp12ZAfMynJh4a688BLCKxMhEMq",
        108: "5EYCAe5jLQhn6ofDSwLZDbMb93qrpXtBRjWbg4sAf9q5xtB8",
        109: "5EYCAe5jLQhn6ofDSwLpxcuBFYX4H5z4AQT7E1Z17yhpELLK",
        110: "5EYCAe5jLQhn6ofDSwM6heSmN3CFje5vu5PcmxEqaoaYW1KP",
        111: "5EYCAe5jLQhn6ofDSwMNSfzMUXsTCCBodkL8Ktvg3dTGmYbX",
        112: "5EYCAe5jLQhn6ofDSwMeBhXwb2YeekHgNRGdsqcWWTL13NLP",
        113: "5EYCAe5jLQhn6ofDSwMuvj5XhXDr7JPZ76D9RnJLyHCjK2Zy",
        114: "5EYCAe5jLQhn6ofDSwNBfkd7p1u3ZrVRqm9eyizBS75TaPgK",
        115: "5EYCAe5jLQhn6ofDSwNTQnAhvWaF2QbJaS6AXfg1tvxBrDUN",
        116: "5EYCAe5jLQhn6ofDSwNj9oiJ31FSUxhBK72g5cMrMkpv7iJx",
        117: "5EYCAe5jLQhn6ofDSwNztqFt9VvdwWo43myBdZ3gpahePQpf",
        118: "5EYCAe5jLQhn6ofDSwPGdroUFzbqQ4tvnSuhBVjXHQaNet2o",
        119: "5EYCAe5jLQhn6ofDSwPYNtM4NVH2rczoX7rCjSRMkET6vioH",
        120: "5EYCAe5jLQhn6ofDSwPp7uteUyxEKB6gFnniHP7CD4KqCQDN",
        121: "5EYCAe5jLQhn6ofDSwQ5rwSEbUdRmjCYzTjDqKo2ftCZTubr",
        122: "5EYCAe5jLQhn6ofDSwQMbxyphyJdEHJRj8fjPGUs8i5HjcA3",
        123: "5EYCAe5jLQhn6ofDSwQdLzXQpTypgqQJTocEwDAhbXx21Awy",
        124: "5EYCAe5jLQhn6ofDSwQu624zvxf29PWBCUYkV9rY4MpkGu1f",
        125: "5EYCAe5jLQhn6ofDSwRAq3cb3TLDbwc3w9VG36YNXBhUYKDi",
        126: "5EYCAe5jLQhn6ofDSwRSa5AB9x1R4VhvfpRmb3ECz1aCp2ze",
        127: "5EYCAe5jLQhn6ofDSwRiK6hmGSgcX3ooQVNH8yv3SqSw5mpH",
        128: "5EYCAe5jLQhn6ofDSwRz48FMNwMoybug9AJngvbsufKfME2t",
    }

    return NETUID_TO_ACCOUNT_ID[netuid];
}