"""ML-DSA execution and rejection paths on an isolated activated localnet.

These tests require usable development weights. They do not measure weights or
establish production capacity. Only disposable keys and dev sponsors are used.
"""

from __future__ import annotations

import json
import os
from copy import deepcopy
from itertools import permutations
from pathlib import Path

import pytest

import bittensor as bt
from bittensor._transport.contract import SignedExtrinsic
from bittensor._transport.errors import SubstrateRequestException
from bittensor.mldsa import install_message_capability, message_capability
from bittensor.mldsa_signing import PreparedMldsaExtrinsic, sign_prepared_mldsa
from bittensor.receiving import receiving_address
from bittensor.sp_core import Keypair
from tests.harness.samples import dev_wallet

ENDPOINT = os.getenv("E2E_WALLET_ENDPOINT")
pytestmark = [
    pytest.mark.asyncio,
    pytest.mark.skipif(not ENDPOINT, reason="requires an isolated activated Alice-root chain"),
]


def evidence(name, value):
    if directory := os.getenv("E2E_EVIDENCE_DIR"):
        path = Path(directory)
        path.mkdir(parents=True, exist_ok=True)
        (path / f"{name}.json").write_text(json.dumps(value, indent=2, default=str) + "\n")


async def fund(client, key, amount=100):
    result = await client.execute(
        bt.intents.Transfer(receiving_address(key), amount), dev_wallet("//Bob")
    )
    assert result.success, result.to_dict()


async def record(client, key):
    return await client.query(("HashedAccounts", "MlDsaAccounts"), [key.ss58_address])


async def account(client, key):
    return await client.query(bt.storage.System.Account, [key.ss58_address])


def event(result, module, name):
    for item in result.events:
        value = item.get("event", item)
        if (value.get("module_id"), value.get("event_id")) == (module, name):
            return value["attributes"]
    raise AssertionError((module, name, result.events))


@pytest.mark.parametrize("code", [7, 8])
async def test_offline_replay_capabilities_native_evm_and_derived_accounts(code):
    key = Keypair.create_from_seed(os.urandom(32), code)
    receipts = []
    async with bt.Client(ENDPOINT, fallback_endpoints=[], archive_endpoints=[]) as client:
        await fund(client, key)
        raw = client._substrate.raw

        async def submit(call, *, success=True):
            result = await client.submit_call(call, key, wait_for_finalization=True)
            assert result.success == success, result.to_dict()
            assert result.block_hash, result.to_dict()
            receipts.append(result.to_dict())
            return result

        # Device/offline signing retains the full >256-byte payload and uses a
        # finalized authority snapshot. Corruption is rejected without a write.
        call = await client.compose(bt.calls.System.remark(remark="0x" + "ab" * 400))
        prepared = await raw.prepare_mldsa_unsigned(call, descriptor=bytes(key.hashed_descriptor))
        prepared = PreparedMldsaExtrinsic.from_dict(prepared.to_dict())
        assert len(prepared.payload) > 400
        codec = await raw._runtimes.codec_at(prepared.authority.block_hash)
        proof = sign_prepared_mldsa(
            prepared, key, codec=codec, expected_genesis_hash=await raw.genesis_hash()
        )
        before = await record(client, key)
        with pytest.raises(ValueError, match="invalid ML-DSA transaction proof"):
            await raw.attach_mldsa_proof(prepared, proof[:-1] + bytes([proof[-1] ^ 1]))
        signed = await raw.attach_mldsa_proof(prepared, proof)
        corrupted = SignedExtrinsic(
            signed.data[:-1] + bytes([signed.data[-1] ^ 1]), "0x" + "00" * 32
        )
        with pytest.raises(SubstrateRequestException):
            await raw.submit_extrinsic(corrupted)
        assert await record(client, key) == before
        included = await raw.submit_extrinsic(signed, wait_for_finalization=True)
        assert included.is_success and included.finalized
        receipts.append({"offline_block": included.block_hash, "extrinsic": signed.data_hex})
        after = await record(client, key)
        assert after["generation"] == before["generation"] + 1
        assert (after["commitment"] == before["commitment"]) == (code == 8)
        with pytest.raises(SubstrateRequestException):
            await raw.submit_extrinsic(signed)
        with pytest.raises(SubstrateRequestException, match="changed since preparation"):
            await raw.attach_mldsa_proof(prepared, proof)
        assert await record(client, key) == after

        # Hex-looking text and non-UTF8 audiences must never collapse to the
        # same bytes. Revocation and replacement consume monotonically new epochs.
        for epoch, audience in enumerate((b"0xff", b"\xff"), 1):
            await submit(
                install_message_capability(
                    key, epoch=epoch, expires_at=await client.block() + 10_000, audience=audience
                )
            )
            cap = await message_capability(client._substrate, key.ss58_address, audience)
            other = b"\xff" if audience == b"0xff" else b"0xff"
            with pytest.raises(ValueError, match="audience"):
                await message_capability(client._substrate, key.ss58_address, other)
            message = bt.wallets.sign_message_key("live capability", key, message_capability=cap)
            assert bt.wallets.verify_message(
                "live capability",
                message["signature"],
                key.ss58_address,
                code,
                message_capability=cap,
            )
        await submit(bt.calls.HashedAccounts.revoke_capability(role="Message"))
        with pytest.raises(ValueError, match="no active"):
            await message_capability(client._substrate, key.ss58_address, b"\xff")

        # Native EVM operations are authorized by the ML-DSA transaction, with
        # the permanent H160 alias and an EVM nonce separate from native nonce.
        source = "0x" + bytes(key.public_key)[:20].hex()
        gas_price = int(await raw._session.request("eth_gasPrice", []), 16)
        common = dict(
            source=source,
            value=[0, 0, 0, 0],
            gas_limit=200_000,
            max_fee_per_gas=[(gas_price >> (64 * i)) & (2**64 - 1) for i in range(4)],
            max_priority_fee_per_gas=None,
            nonce=None,
            access_list=[],
            authorization_list=[],
        )
        initial_nonce = int(
            await raw._session.request("eth_getTransactionCount", [source, "latest"]), 16
        )
        result = await submit(bt.calls.EVM.call(target="0x" + "42" * 20, input="0x", **common))
        event(result, "EVM", "Executed")
        # Localnet retains the ordinary contract deployment allowlist.
        allowed = await client.query(("EVM", "WhitelistedCreators"))
        grant = await client.submit_call(
            bt.calls.Sudo.sudo(
                call=await client.compose(bt.calls.EVM.set_whitelist(new=[*allowed, source]))
            ),
            dev_wallet(),
        )
        assert grant.success, grant.to_dict()
        # Constructor returns one STOP byte; deployed code is checked via RPC.
        init = "0x6001600c60003960016000f300"
        for call in (
            bt.calls.EVM.create(init=init, **common),
            bt.calls.EVM.create2(init=init, salt="0x" + "23" * 32, **common),
        ):
            result = await submit(call)
            created = event(result, "EVM", "Created")
            address = (
                created[0]
                if isinstance(created, list)
                else created.get("address", created.get("contract"))
            )
            assert await raw._session.request("eth_getCode", [address, "latest"]) == "0x00"
        assert (
            int(await raw._session.request("eth_getTransactionCount", [source, "latest"]), 16)
            == initial_nonce + 3
        )

        result = await submit(
            bt.calls.Utility.as_protected_derivative(
                index=7, call=await client.compose(bt.calls.System.remark(remark="0x07"))
            )
        )
        derivative = event(result, "HashedAccounts", "DerivedProtected")["account"]
        assert await client.query(("HashedAccounts", "DerivedAuthorities"), [derivative])
        before = await record(client, key)
        result = await submit(
            bt.calls.Utility.as_protected_derivative(
                index=8,
                call=await client.compose(
                    bt.calls.Balances.transfer_keep_alive(
                        dest=dev_wallet().coldkey.ss58_address, value=10**18
                    )
                ),
            ),
            success=False,
        )
        assert not any(
            e.get("event", e).get("event_id") == "DerivedProtected" for e in result.events
        )
        assert (await record(client, key))["generation"] == before["generation"] + 1
        result = await submit(
            bt.calls.Proxy.create_protected_pure(proxy_type="Any", delay=0, index=9)
        )
        pure = event(result, "Proxy", "PureCreated")["pure"]
        height, index = map(int, result.extrinsic_id.split("-"))
        await submit(
            bt.calls.Proxy.proxy(
                real=pure,
                force_proxy_type=None,
                call=await client.compose(
                    bt.calls.Proxy.kill_pure(
                        spawner=key.ss58_address,
                        proxy_type="Any",
                        index=9,
                        height=height,
                        ext_index=index,
                    )
                ),
            )
        )
        assert await client.query(("HashedAccounts", "DerivedAuthorities"), [pure])
        assert not (await client.query(bt.storage.Proxy.Proxies, [pure]))[0]
        evidence(f"mldsa-authority-{code}", receipts)


@pytest.mark.parametrize("codes", [(7, 8), (7, 1), (8, 0)])
async def test_mixed_protected_multisig_cancellation_failure_and_quorum(codes):
    keys = [Keypair.create_from_seed(os.urandom(32), code) for code in codes]
    recipient = Keypair.create_from_seed(os.urandom(32), 1)
    receipts = []
    async with bt.Client(ENDPOINT, fallback_endpoints=[], archive_endpoints=[]) as client:
        for key in keys:
            await fund(client, key)
        group = await client.multisig([receiving_address(key) for key in keys], 2)
        protection = await group.protection_call(deposit_payer=keys[1].ss58_address)
        for key in keys:
            result = await group.approve(protection, key, wait_for_finalization=True)
            assert result.success, result.to_dict()
        assert (await group.protection_status())["registered"]
        result = await client.execute(bt.intents.Transfer(group.address, 10), dev_wallet("//Bob"))
        assert result.success, result.to_dict()
        for value, success in ((10**9, True), (10**18, False), (10**9, True)):
            call = bt.calls.Balances.transfer_keep_alive(dest=recipient.ss58_address, value=value)
            for index, key in enumerate(keys):
                result = await group.approve(call, key, wait_for_finalization=True)
                assert result.success == (True if index == 0 else success), result.to_dict()
                assert result.block_hash, result.to_dict()
                receipts.append(result.to_dict())
        assert (await account(client, recipient))["data"]["free"] == 2 * 10**9
        call = bt.calls.System.remark(remark="0x1234")
        composed = await client.compose(call)
        call_hash = "0x" + bytes(composed.call_hash).hex()
        before = (await account(client, keys[0]))["data"]["reserved"]
        assert (await group.approve(call, keys[0], wait_for_finalization=True)).success
        pending = await client.query(bt.storage.Multisig.Multisigs, [group.address, call_hash])
        assert pending
        result = await client.submit_call(
            bt.calls.Multisig.cancel_as_multi(
                threshold=2,
                other_signatories=[keys[1].ss58_address],
                timepoint=pending["when"],
                call_hash=call_hash,
            ),
            keys[0],
            wait_for_finalization=True,
        )
        assert result.success, result.to_dict()
        assert await client.query(bt.storage.Multisig.Multisigs, [group.address, call_hash]) is None
        assert (await account(client, keys[0]))["data"]["reserved"] == before
        weak = [Keypair.create_from_seed(os.urandom(32), code) for code in (0, 1)]
        unsafe = await client.multisig([keys[0].ss58_address, *(k.ss58_address for k in weak)], 2)
        assert not (await unsafe.protection_status())["eligible"]
        with pytest.raises(ValueError, match="quorum"):
            await unsafe.protection_call(deposit_payer=keys[0].ss58_address)
        evidence(f"mldsa-multisig-{codes[0]}-{codes[1]}", receipts)


async def test_two_of_three_all_mixed_quorums_and_runtime_rejection_of_weak_quorum():
    keys = [Keypair.create_from_seed(os.urandom(32), code) for code in (7, 8, 1)]
    recipient = Keypair.create_from_seed(os.urandom(32), 0)
    receipts = []
    async with bt.Client(ENDPOINT, fallback_endpoints=[], archive_endpoints=[]) as client:
        for key in keys:
            await fund(client, key)
        group = await client.multisig([key.ss58_address for key in keys], 2)
        protection = await group.protection_call(deposit_payer=keys[1].ss58_address)
        for key in keys[:2]:
            result = await group.approve(protection, key, wait_for_finalization=True)
            assert result.success, result.to_dict()
        assert (await group.protection_status())["registered"]
        result = await client.execute(bt.intents.Transfer(group.address, 20), dev_wallet("//Bob"))
        assert result.success, result.to_dict()
        call = bt.calls.Balances.transfer_keep_alive(dest=recipient.ss58_address, value=10**9)
        for first, second in permutations(keys, 2):
            reserved = (await account(client, first))["data"]["reserved"]
            for key in (first, second):
                result = await group.approve(call, key, wait_for_finalization=True)
                assert result.success and result.block_hash, result.to_dict()
                receipts.append(result.to_dict())
            assert (await account(client, first))["data"]["reserved"] == reserved
        assert (await account(client, recipient))["data"]["free"] == 6 * 10**9

        # Bypass SDK eligibility checks deliberately: a wholly classical quorum
        # cannot register a PQ floor merely because an ML-DSA member is listed.
        weak = [Keypair.create_from_seed(os.urandom(32), code) for code in (0, 1)]
        for key in weak:
            await fund(client, key)
        unsafe = await client.multisig([keys[0].ss58_address, *(k.ss58_address for k in weak)], 2)
        attempt = bt.calls.HashedAccounts.register_derived_sponsored(
            descriptor={"Multisig": {"threshold": 2, "signatories": unsafe.signatories}},
            deposit_payer=weak[1].ss58_address,
        )
        assert (await unsafe.approve(attempt, weak[0], wait_for_finalization=True)).success
        rejected = await unsafe.approve(attempt, weak[1], wait_for_finalization=True)
        assert not rejected.success and rejected.block_hash, rejected.to_dict()
        assert rejected.error.name == "InvalidDerivedAuthority", rejected.to_dict()
        assert (
            await client.query(("HashedAccounts", "DerivedAuthorities"), [unsafe.address]) is None
        )
        receipts.append(rejected.to_dict())
        evidence("mldsa-two-of-three", receipts)


@pytest.mark.parametrize("code", [7, 8])
async def test_original_limit_order_execution_cancellation_and_capability_revocation(code):
    owner = Keypair.create_from_seed(os.urandom(32), code)
    hot = Keypair.create_from_seed(os.urandom(32), code)
    sponsor = dev_wallet("//Bob")
    receipts = []
    async with bt.Client(ENDPOINT, fallback_endpoints=[], archive_endpoints=[]) as client:
        lock_cost = await client.read("subnet_registration_cost")
        await fund(client, owner, bt.Balance.from_rao(lock_cost.rao + 1000 * 10**9))
        await fund(client, hot)
        result = await client.submit_call(
            bt.calls.SubtensorModule.register_network(hotkey=hot.ss58_address), owner
        )
        assert result.success, result.to_dict()
        netuid = max(subnet.netuid for subnet in await client.subnets.all())
        result = await client.submit_call(
            bt.calls.Sudo.sudo(
                call=await client.compose(
                    bt.calls.AdminUtils.sudo_set_subtoken_enabled(
                        netuid=netuid, subtoken_enabled=True
                    )
                )
            ),
            dev_wallet(),
        )
        assert result.success, result.to_dict()
        chain_id = await client.query(("EVMChainId", "ChainId"))
        order = {
            "V1": dict(
                signer=owner.ss58_address,
                hotkey=hot.ss58_address,
                netuid=netuid,
                order_type="LimitBuy",
                amount=10**9,
                limit_price=10**15,
                expiry=2_000_000_000_000,
                fee_rate=0,
                fee_recipient=sponsor.coldkey.ss58_address,
                relayer=[sponsor.coldkey.ss58_address],
                max_slippage=10**9,
                chain_id=chain_id,
                partial_fills_enabled=False,
            )
        }
        orders = client.limit_orders

        async def rejected(signed_orders, name):
            result = await orders.execute(signed_orders, sponsor)
            assert not result.success and result.block_hash, result.to_dict()
            assert result.error.name == name, result.to_dict()
            receipts.append(result.to_dict())

        first = await orders.prepare(order, nonce=1)
        result = await orders.authorize(first, owner)
        assert result.success, result.to_dict()
        result = await orders.execute([first.account_approved()], sponsor)
        assert result.success, result.to_dict()
        receipts.append(result.to_dict())
        event(result, "LimitOrders", "OrderExecuted")
        await rejected([first.account_approved()], "OrderAlreadyProcessed")
        second = await orders.prepare(order, nonce=2)
        assert (await orders.authorize(second, owner)).success
        assert (await orders.cancel(second, owner)).success
        await rejected([second.account_approved()], "OrderCancelled")
        result = await orders.install_capability(
            owner, owner, expires_at=await client.block() + 10_000, scope="UnrestrictedTrading"
        )
        assert result.success, result.to_dict()
        third = await orders.prepare(order, nonce=3)
        signed = await orders.sign(third, owner)
        result = await orders.execute([signed], sponsor)
        assert result.success, result.to_dict()
        receipts.append(result.to_dict())
        event(result, "LimitOrders", "OrderExecuted")
        await rejected([signed], "OrderAlreadyProcessed")
        fourth = await orders.prepare(order, nonce=4)
        revoked = await orders.sign(fourth, owner)
        assert (await orders.revoke_capability(owner)).success
        await rejected([revoked], "CapabilityMissing")
        # Atomic execution cannot persist a good order when a second order has
        # no authorization; the good order remains usable afterward.
        fifth = await orders.prepare(order, nonce=5)
        changed = deepcopy(order)
        changed["V1"]["amount"] += 1
        sixth = await orders.prepare(changed, nonce=6)
        assert (await orders.authorize(fifth, owner)).success
        await rejected([fifth.account_approved(), sixth.account_approved()], "OrderNotApproved")
        result = await orders.execute_batched(netuid, [fifth.account_approved()], sponsor)
        assert result.success, result.to_dict()
        receipts.append(result.to_dict())
        scope = {
            "Trading": {
                "netuid": netuid,
                "hotkey": hot.ss58_address,
                "fee_recipient": sponsor.coldkey.ss58_address,
                "max_fee_perbill": 0,
                "max_tao_per_order": 10**9,
                "max_alpha_per_order": 0,
                "total_tao": 2 * 10**9,
                "total_alpha": 0,
            }
        }
        result = await orders.install_capability(
            owner, owner, expires_at=await client.block() + 10_000, scope=scope
        )
        assert result.success, result.to_dict()
        # All three proofs are valid when signed. The runtime must enforce the
        # shared remaining budget when a relayer later submits them separately.
        bounded = [
            await orders.sign(await orders.prepare(order, nonce=nonce), owner)
            for nonce in (7, 8, 9)
        ]
        for signed_order in bounded[:2]:
            result = await orders.execute([signed_order], sponsor)
            assert result.success, result.to_dict()
            receipts.append(result.to_dict())
        await rejected([bounded[2]], "CapabilityScopeViolation")
        capability = await client.query(
            ("HashedAccounts", "ActiveCapabilities"), [owner.ss58_address, "Order"]
        )
        assert capability["epoch"] == 3 and capability["spent_tao"] == 2 * 10**9
        with pytest.raises(ValueError, match="budget"):
            await orders.sign(await orders.prepare(order, nonce=10), owner)
        evidence(f"mldsa-orders-{code}", receipts)
