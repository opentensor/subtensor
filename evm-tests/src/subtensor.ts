import * as assert from "assert";
import { devnet, MultiAddress } from '@polkadot-api/descriptors';
import { TypedApi, TxCallData } from 'polkadot-api';
import { KeyPair } from "@polkadot-labs/hdkd-helpers"
import { getAliceSigner, waitForTransactionCompletion, getSignerFromKeypair } from './substrate'
import { convertH160ToSS58, convertPublicKeyToSs58 } from './address-utils'
import { tao } from './balance-math'

// create a new subnet and return netuid 
export async function addNewSubnetwork(api: TypedApi<typeof devnet>, hotkey: KeyPair, coldkey: KeyPair) {
    const alice = getAliceSigner()
    const totalNetworks = await api.query.SubtensorModule.TotalNetworks.getValue()

    const rateLimit = await api.query.SubtensorModule.NetworkRateLimit.getValue()
    if (rateLimit !== BigInt(0)) {
        const internalCall = api.tx.AdminUtils.sudo_set_network_rate_limit({ rate_limit: BigInt(0) })
        const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })
        await waitForTransactionCompletion(api, tx, alice)
            .then(() => { })
            .catch((error) => { console.log(`transaction error ${error}`) });
    }

    const signer = getSignerFromKeypair(coldkey)
    const registerNetworkTx = api.tx.SubtensorModule.register_network({ hotkey: convertPublicKeyToSs58(hotkey.publicKey) })
    await waitForTransactionCompletion(api, registerNetworkTx, signer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

    assert.equal(totalNetworks + 1, await api.query.SubtensorModule.TotalNetworks.getValue())
    return totalNetworks
}

// force set balance for a ss58 address
export async function forceSetBalanceToSs58Address(api: TypedApi<typeof devnet>, ss58Address: string) {
    const alice = getAliceSigner()
    const balance = tao(1e8)
    const internalCall = api.tx.Balances.force_set_balance({ who: MultiAddress.Id(ss58Address), new_free: balance })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(disabled, await api.query.EVM.DisableWhitelistCheck.getValue())
}

export async function burnedRegister(api: TypedApi<typeof devnet>, netuid: number, ss58Address: string, keypair: KeyPair) {
    const uids = await api.query.SubtensorModule.SubnetworkN.getValue(netuid)
    const signer = getSignerFromKeypair(keypair)
    const tx = api.tx.SubtensorModule.burned_register({ hotkey: ss58Address, netuid: netuid })
    await waitForTransactionCompletion(api, tx, signer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(uids + 1, await api.query.SubtensorModule.SubnetworkN.getValue(netuid))
}


export async function sendProxyCall(api: TypedApi<typeof devnet>, calldata: TxCallData, ss58Address: string, keypair: KeyPair) {
    const signer = getSignerFromKeypair(keypair)
    const tx = api.tx.Proxy.proxy({
        call: calldata,
        real: MultiAddress.Id(ss58Address),
        force_proxy_type: undefined
    });
    await waitForTransactionCompletion(api, tx, signer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
}


export async function setTxRateLimit(api: TypedApi<typeof devnet>, txRateLimit: bigint) {
    const value = await api.query.SubtensorModule.TxRateLimit.getValue()
    if (value === txRateLimit) {
        return;
    }
    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_tx_rate_limit({ tx_rate_limit: txRateLimit })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })


    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(txRateLimit, await api.query.SubtensorModule.TxRateLimit.getValue())
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(activityCutoff, await api.query.SubtensorModule.ActivityCutoff.getValue(netuid))
}

export async function setMaxAllowedUids(api: TypedApi<typeof devnet>, netuid: number, maxAllowedUids: number) {
    const value = await api.query.SubtensorModule.MaxAllowedUids.getValue(netuid)
    if (value === maxAllowedUids) {
        return;
    }

    const alice = getAliceSigner()

    const internalCall = api.tx.AdminUtils.sudo_set_max_allowed_uids({
        netuid: netuid,
        max_allowed_uids: maxAllowedUids
    })
    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(maxAllowedUids, await api.query.SubtensorModule.MaxAllowedUids.getValue(netuid))
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

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
    assert.equal(minDelegateTake, await api.query.SubtensorModule.MinDelegateTake.getValue())
}

export async function becomeDelegate(api: TypedApi<typeof devnet>, ss58Address: string, keypair: KeyPair) {
    const singer = getSignerFromKeypair(keypair)

    const tx = api.tx.SubtensorModule.become_delegate({
        hotkey: ss58Address
    })
    await waitForTransactionCompletion(api, tx, singer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });
}

export async function addStake(api: TypedApi<typeof devnet>, netuid: number, ss58Address: string, amount_staked: bigint, keypair: KeyPair) {
    const singer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.add_stake({
        netuid: netuid,
        hotkey: ss58Address,
        amount_staked: amount_staked
    })

    await waitForTransactionCompletion(api, tx, singer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

}

export async function setWeight(api: TypedApi<typeof devnet>, netuid: number, dests: number[], weights: number[], version_key: bigint, keypair: KeyPair) {
    const singer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.set_weights({
        netuid: netuid,
        dests: dests,
        weights: weights,
        version_key: version_key
    })

    await waitForTransactionCompletion(api, tx, singer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

}

export async function rootRegister(api: TypedApi<typeof devnet>, ss58Address: string, keypair: KeyPair) {
    const singer = getSignerFromKeypair(keypair)
    let tx = api.tx.SubtensorModule.root_register({
        hotkey: ss58Address
    })

    await waitForTransactionCompletion(api, tx, singer)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

}