"""Later approvals validate recipient guards against the original encoded call."""

import json
from dataclasses import replace
from unittest.mock import AsyncMock

import pytest

from bittensor._generated.runtime_apis import StakeInfoRuntimeApi, SwapRuntimeApi
from bittensor._transport.contract import CallBytes
from bittensor.executor import _compose_intent_call
from bittensor.hashed import _call_variants, descriptor_value
from bittensor.intents import Batch
from bittensor.intents.multisig import MultisigExecute, MultisigIntentAdapter
from bittensor.intents.proxy import ExecuteProxyAnnounced
from bittensor.intents.staking import (
    AddStake,
    AddStakeLimit,
    MoveStake,
    MoveSwapStake,
    RemoveStake,
    SwapStake,
)
from bittensor.receiving import receiving_address
from bittensor.sp_core import Keypair
from tests.harness.fake_substrate import FakeSubstrate
from tests.harness.samples import ALICE, BOB, dev_wallet


class CallCodec:
    """Opaque call bytes with the production decoder's nested-call shape."""

    def compose_call(self, module, function, params):
        def normalize(value):
            if isinstance(value, CallBytes):
                return json.loads(value.data)
            if isinstance(value, dict):
                if len(value) == 1:
                    name, inner = next(iter(value.items()))
                    if name in ("Utility", "Proxy", "HashedAccounts", "SubtensorModule"):
                        method, args = next(iter(inner.items()))
                        return json.loads(self.compose_call(name, method, args).data)
                return {key: normalize(child) for key, child in value.items()}
            if isinstance(value, (list, tuple)):
                return [normalize(child) for child in value]
            return value

        decoded = {
            "call_module": module,
            "call_function": function,
            "call_args": [
                {"name": key, "value": normalize(value)} for key, value in sorted(params.items())
            ],
        }
        return CallBytes(json.dumps(decoded, sort_keys=True).encode(), 478)

    def decode(self, type_name, data):
        assert type_name == "Call"
        return json.loads(data)


@pytest.fixture
def setup():
    chain = FakeSubstrate()
    codec = CallCodec()
    chain.compose = AsyncMock(side_effect=lambda call: codec.compose_call(*call))
    chain.decode_scale = AsyncMock(side_effect=codec.decode)
    chain.seed_constant("HashedAccounts", "Enabled", True)
    chain.seed_constant("HashedAccounts", "RegistrationDeposit", 200_000_000)
    chain.seed_runtime(*SwapRuntimeApi.current_alpha_price, 1_000_000_000)
    chain.seed_runtime(
        *StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid, {"stake": 2_000_000_000}
    )
    for address in (ALICE, BOB):
        chain.seed("System", "Account", [address], {"data": {"free": 10**12, "frozen": 0}})
    return chain, codec


def register(chain, scheme, seed):
    key = Keypair.create_from_seed(bytes([seed]) * 32, scheme)
    chain.seed(
        "HashedAccounts",
        "Accounts",
        [key.ss58_address],
        {"descriptor": descriptor_value(bytes(key.hashed_descriptor))},
    )
    return receiving_address(key)


def staking(op, first, second, amount):
    return {
        "add": lambda: AddStake(first, 1, amount),
        "remove": lambda: RemoveStake(first, 1, amount),
        "move": lambda: MoveStake(first, 1, second, 2, amount),
        "swap": lambda: SwapStake(first, 1, 2, amount),
        "move_swap": lambda: MoveSwapStake(first, 1, second, 2, amount),
    }[op]()


async def opening(chain, semantic, **kwargs):
    adapter = MultisigIntentAdapter(
        dispatch=MultisigExecute(2, [BOB], semantic.to_dict()), semantic=semantic
    )
    origin = adapter.origin_view(chain, dev_wallet())
    chain.seed("System", "Account", [origin.ss58_address], {"data": {"free": 10**12}})
    _, extras = await _compose_intent_call(chain, adapter, dev_wallet(), **kwargs)
    adapter.inner_call_data = extras["multisig_call_data"]
    adapter.dispatch = replace(
        adapter.dispatch, other_signatories=[ALICE], timepoint={"height": 100, "index": 1}
    )
    return adapter, extras


@pytest.mark.parametrize("scheme", [4, 6])
@pytest.mark.parametrize("op", ["add", "remove", "move", "swap", "move_swap"])
@pytest.mark.parametrize("amount", [1, "all"])
async def test_subsequent_approval_preserves_price_amount_and_guards(setup, scheme, op, amount):
    chain, codec = setup
    semantic = staking(op, register(chain, scheme, 92), register(chain, scheme, 93), amount)
    adapter, original = await opening(chain, semantic)
    chain.seed_runtime(
        *SwapRuntimeApi.current_alpha_price,
        lambda params: 1_010_000_000 if params[0] == 1 else 1_000_000_000,
    )
    chain.seed_runtime(
        *StakeInfoRuntimeApi.get_stake_info_for_hotkey_coldkey_netuid, {"stake": 3_000_000_000}
    )
    origin = adapter.origin_view(chain, dev_wallet("//Bob"))
    chain.seed("System", "Account", [origin.ss58_address], {"data": {"free": 20**12}})
    _, later = await _compose_intent_call(chain, adapter, dev_wallet("//Bob"))
    assert later["multisig_call_data"] == original["multisig_call_data"]
    assert later["multisig_call_hash"] == original["multisig_call_hash"]
    decoded = codec.decode("Call", bytes.fromhex(later["multisig_call_data"][2:]))
    assert decoded["call_module"] == "Utility"
    assert decoded["call_function"] == "batch_all"

    # Replaying approved economic inputs must not depend on a new price quote.
    chain.runtime_call = AsyncMock(side_effect=AssertionError("unexpected quote"))
    _, again = await _compose_intent_call(chain, adapter, dev_wallet("//Bob"))
    assert again["multisig_call_data"] == original["multisig_call_data"]


@pytest.mark.parametrize("wrapper", ["batch", "proxy", "announced", "crowdloan"])
async def test_nested_staking_reuses_each_approved_limit(setup, wrapper):
    from bittensor.intents.crowdloan import CreateCrowdloan

    chain, _ = setup
    address = register(chain, 4, 92)
    semantic = AddStake(address, 1, 1)
    kwargs = {}
    if wrapper == "batch":
        semantic = Batch(
            [
                AddStakeLimit(address, 1, 1, 1_100_000_000),
                semantic,
                AddStake(address, 1, 2, rate_tolerance=0.1),
            ]
        )
    elif wrapper == "proxy":
        kwargs = {"proxy_for": ALICE, "proxy_type": "Any"}
    elif wrapper == "announced":
        semantic = ExecuteProxyAnnounced(BOB, ALICE, semantic.op, semantic.to_dict())
    else:
        semantic = CreateCrowdloan(1, 1, 10, 1000, call=semantic.to_dict())
    adapter, original = await opening(chain, semantic, **kwargs)
    chain.runtime_call = AsyncMock(side_effect=AssertionError("unexpected quote"))
    _, later = await _compose_intent_call(chain, adapter, dev_wallet("//Bob"), **kwargs)
    assert later["multisig_call_data"] == original["multisig_call_data"]


@pytest.mark.parametrize("tamper", ["amount", "hotkey", "netuid", "guard", "origin", "partial"])
async def test_pinned_staking_still_rejects_mismatched_payment_and_guards(setup, tamper):
    chain, codec = setup
    address = register(chain, 4, 92)
    adapter, _ = await opening(chain, AddStake(address, 1, 1))
    if tamper == "amount":
        adapter.semantic = replace(adapter.semantic, amount_tao=2)
    elif tamper == "hotkey":
        adapter.semantic = replace(adapter.semantic, hotkey_ss58=register(chain, 4, 93))
    elif tamper == "netuid":
        adapter.semantic = replace(adapter.semantic, netuid=2)
    elif tamper in ("guard", "partial"):
        decoded = codec.decode("Call", bytes.fromhex(adapter.inner_call_data[2:]))
        children = decoded["call_args"][0]["value"]
        if tamper == "guard":
            decoded = children[-1]
        else:
            for arg in children[-1]["call_args"]:
                if arg["name"] == "allow_partial":
                    arg["value"] = True
        call = codec.compose_call(
            decoded["call_module"],
            decoded["call_function"],
            {arg["name"]: _call_variants(arg["value"]) for arg in decoded["call_args"]},
        )
        adapter.inner_call_data = "0x" + call.data.hex()
    kwargs = {"proxy_for": ALICE} if tamper == "origin" else {}
    adapter.wrap_call = AsyncMock()
    with pytest.raises(ValueError, match="do not match the verified recipient"):
        await _compose_intent_call(chain, adapter, dev_wallet("//Bob"), **kwargs)
    adapter.wrap_call.assert_not_awaited()


async def test_explicit_limit_is_not_replaced_by_pinned_price(setup):
    chain, _ = setup
    address = register(chain, 4, 92)
    semantic = Batch([AddStakeLimit(address, 1, 1, 1_100_000_000), AddStake(address, 1, 1)])
    adapter, _ = await opening(chain, semantic)
    adapter.semantic = Batch(
        [
            AddStakeLimit(address, 1, 1, 1_200_000_000),
            AddStake(address, 1, 1),
        ]
    )
    adapter.wrap_call = AsyncMock()
    with pytest.raises(ValueError, match="do not match the verified recipient"):
        await _compose_intent_call(chain, adapter, dev_wallet("//Bob"))
    adapter.wrap_call.assert_not_awaited()


async def test_pinned_context_does_not_leak_into_next_plan(setup):
    chain, _ = setup
    semantic = AddStake(register(chain, 4, 92), 1, 1)
    adapter, original = await opening(chain, semantic)
    await _compose_intent_call(chain, adapter, dev_wallet("//Bob"))
    chain.seed_runtime(*SwapRuntimeApi.current_alpha_price, 1_010_000_000)
    fresh, _ = await _compose_intent_call(chain, semantic, dev_wallet())
    assert fresh.data.hex() != original["multisig_call_data"][2:]
