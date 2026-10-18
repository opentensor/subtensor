"""Policy and previews account for reserves and the complete submission route."""

import json
from unittest.mock import AsyncMock, Mock

import pytest
from typer.testing import CliRunner

from bittensor import Balance, Client
from bittensor.cli.context import AppContext
from bittensor.cli.main import app
from bittensor.executor import Executor
from bittensor.hashed import descriptor_value
from bittensor.intents import Batch, CreatePureProxy, Policy, Transfer
from bittensor.receiving import receiving_address
from bittensor.result import PolicyError
from bittensor.sp_core import Keypair
from tests.harness.fake_substrate import FakeSubstrate, success_result
from tests.harness.samples import BOB, dev_wallet
from tests.unit import test_cli

fake = test_cli.fake
wallet_dir = test_cli.wallet_dir

DEPOSIT = 200_000_000
INNER = 100
CARRIER = 70


def chain():
    substrate = FakeSubstrate()
    substrate.seed("AdminUtils", "HashedAccountsEnabled", [], True)
    substrate.seed_constant("HashedAccounts", "MldsaAccountProfiles", "0x0405")
    for name in ("RegistrationDeposit", "MlDsaRegistrationDeposit", "DerivedRegistrationDeposit"):
        substrate.seed_constant("HashedAccounts", name, DEPOSIT)
    substrate.seed_constant("Balances", "ExistentialDeposit", 0)
    substrate.seed_constant("Proxy", "ProxyDepositBase", 30)
    substrate.seed_constant("Proxy", "ProxyDepositFactor", 20)
    substrate.seed_default("System", "Account", {"data": {"free": 10**13, "frozen": 0}})
    substrate.fee = Balance.from_rao(INNER)
    substrate.estimate_shielded_carrier_fee = AsyncMock(return_value=Balance.from_rao(CARRIER))
    substrate.mev_key = bytes(1184)
    return substrate


def protected_wallet(substrate, crypto):
    key = Keypair.create_from_seed(bytes([82]) * 32, crypto)
    substrate.seed(
        "HashedAccounts",
        "MlDsaAccounts",
        [key.ss58_address],
        {"descriptor": descriptor_value(bytes(key.hashed_descriptor))},
    )
    return key


@pytest.mark.parametrize("crypto", [7, 8])
@pytest.mark.parametrize("count", [1, 2])
@pytest.mark.parametrize("shielded", [False, True])
async def test_pure_creation_reserve_is_quoted_aggregated_and_enforced(
    crypto, count, shielded, monkeypatch
):
    substrate = chain()
    wallet = protected_wallet(substrate, crypto)
    intent = (
        CreatePureProxy(proxy_type="Any")
        if count == 1
        else Batch([CreatePureProxy(proxy_type="Any", index=i) for i in range(count)])
    )
    executor = Executor(substrate)
    plan = await executor.plan(intent, wallet, shielded=shielded, policy=Policy(max_spend_tao=0))
    assert not plan.ok
    assert plan.spend == Balance.from_rao(count * DEPOSIT)
    assert any("max_spend_tao" in v for v in plan.violations)
    assert any("killing the proxy does not refund" in e for e in plan.effects)
    assert plan.extras["pure_proxy_deposit_rao"] == count * 50
    preview = await executor.preflight(intent, wallet, shielded=shielded)
    assert any("permanent" in label for label, _ in preview.facts)
    assert preview.registration_sponsorship is None
    assert preview.required_free == Balance.from_rao(
        count * (DEPOSIT + 50) + INNER + (CARRIER if shielded else 0)
    )
    unlock = Mock(side_effect=AssertionError("must enforce before unlock"))
    monkeypatch.setattr("bittensor.executor.resolve_signer", unlock)
    with pytest.raises(PolicyError, match="max_spend_tao"):
        await executor.execute(intent, wallet, shielded=shielded, policy=Policy(max_spend_tao=0))
    unlock.assert_not_called()
    assert not substrate.submissions


async def test_pure_reserve_uses_runtime_value_and_aggregates_with_payment():
    substrate = chain()
    wallet = protected_wallet(substrate, 7)
    substrate.seed_constant("HashedAccounts", "DerivedRegistrationDeposit", DEPOSIT + 1)
    intent = Batch([CreatePureProxy(proxy_type="Any"), Transfer(BOB, 1)])
    executor = Executor(substrate)
    for headroom in (-1, 0):
        cap = Balance.from_rao(DEPOSIT + 1 + 10**9 + headroom)
        plan = await executor.plan(intent, wallet, shielded=False, policy=Policy(max_spend_tao=cap))
        assert plan.ok is (headroom == 0)
        assert plan.spend == Balance.from_rao(DEPOSIT + 1 + 10**9)


async def test_classical_pure_creation_keeps_ordinary_route_and_no_registration_cost():
    plan = await Executor(chain()).plan(
        CreatePureProxy(), dev_wallet(), policy=Policy(max_spend_tao=0)
    )
    assert plan.ok
    assert plan.call.function == "create_pure"
    assert plan.spend is None
    assert not plan.shielded


@pytest.mark.parametrize("shielded", [False, True])
async def test_pure_insufficient_funds_includes_both_deposits_and_selected_fees(shielded):
    substrate = chain()
    wallet = protected_wallet(substrate, 8)
    substrate.seed(
        "System",
        "Account",
        [wallet.ss58_address],
        {"data": {"free": DEPOSIT + 50 + INNER + (CARRIER if shielded else 0) - 1}},
    )
    plan = await Executor(substrate).plan(
        CreatePureProxy(proxy_type="Any"), wallet, shielded=shielded
    )
    assert not plan.ok
    assert any("reserve" in v or "deposit" in v for v in plan.violations)


@pytest.mark.parametrize("automatic", [False, True])
@pytest.mark.parametrize("headroom", [-1, 0, 1])
async def test_shield_fee_cap_uses_total_before_unlock(automatic, headroom, monkeypatch):
    substrate = chain()
    key = Keypair.create_from_seed(bytes([83]) * 32, 7)
    intent = Transfer(receiving_address(key), 1) if automatic else Transfer(BOB, 1)
    kwargs = {} if automatic else {"shielded": True}
    executor = Executor(substrate)
    executor._submit_encrypted_call = AsyncMock(return_value=success_result())
    cap = Policy(max_fee_tao=Balance.from_rao(INNER + CARRIER + headroom))
    plan = await executor.plan(intent, dev_wallet(), policy=cap, period=16, **kwargs)
    assert plan.shielded
    assert plan.fee.rao == INNER + CARRIER
    assert plan.ok is (headroom >= 0)
    assert substrate.estimate_shielded_carrier_fee.await_args.kwargs["period"] == 16
    if headroom < 0:
        unlock = Mock(side_effect=AssertionError("must enforce before unlock"))
        monkeypatch.setattr("bittensor.executor.resolve_signer", unlock)
        with pytest.raises(PolicyError, match="max_fee_tao"):
            await executor.execute(intent, dev_wallet(), policy=cap, **kwargs)
        unlock.assert_not_called()
        executor._submit_encrypted_call.assert_not_awaited()
    else:
        result = await executor.execute(
            intent,
            dev_wallet(),
            policy=cap,
            max_registration_deposit=Balance.from_rao(DEPOSIT),
            **kwargs,
        )
        assert result.success
        executor._submit_encrypted_call.assert_awaited_once()


@pytest.mark.parametrize("failed_component", ["inner", "carrier"])
async def test_unavailable_total_never_passes_fee_cap(failed_component, monkeypatch):
    substrate = chain()
    target = "estimate_fee" if failed_component == "inner" else "estimate_shielded_carrier_fee"
    setattr(substrate, target, AsyncMock(side_effect=RuntimeError("quote unavailable")))
    executor = Executor(substrate)
    policy = Policy(max_fee_tao=1)
    plan = await executor.plan(Transfer(BOB, 1), dev_wallet(), shielded=True, policy=policy)
    assert plan.fee is None
    assert not plan.ok
    unlock = Mock(side_effect=AssertionError("must enforce before unlock"))
    monkeypatch.setattr("bittensor.executor.resolve_signer", unlock)
    with pytest.raises(PolicyError, match="could not be estimated"):
        await executor.submit_shielded(Transfer(BOB, 1), dev_wallet(), policy=policy)
    unlock.assert_not_called()


async def test_opt_out_prices_only_inner_and_public_client_previews_total():
    substrate = chain()
    key = Keypair.create_from_seed(bytes([84]) * 32, 7)
    intent = Transfer(receiving_address(key), 1)
    client = Client(substrate=substrate)
    plan = await client.plan(
        intent, dev_wallet(), shielded=False, policy=Policy(max_fee_tao=Balance.from_rao(INNER))
    )
    assert plan.ok and not plan.shielded
    assert plan.fee.rao == INNER
    substrate.estimate_shielded_carrier_fee.assert_not_awaited()
    preview = await client.preflight(intent, dev_wallet())
    assert preview.estimated_fee.rao == INNER + CARRIER
    assert any("Total transaction fee" in label for label, _ in preview.facts)
    substrate.mev_key = None
    with pytest.warns(UserWarning, match="submitting account registration unshielded"):
        fallback = await client.plan(intent, dev_wallet())
    assert not fallback.shielded
    assert fallback.fee.rao == INNER


async def test_proxy_origin_pays_registration_while_delegate_only_pays_fees():
    substrate = chain()
    owner = protected_wallet(substrate, 7)
    substrate.seed("System", "Account", [BOB], {"data": {"free": INNER + CARRIER}})
    from bittensor.sp_core import Keypair

    delegate = Keypair.create_from_uri("//Bob")
    executor = Executor(substrate)
    plan = await executor.plan(
        CreatePureProxy(proxy_type="Any"), delegate, proxy_for=owner.ss58_address
    )
    assert plan.ok
    assert plan.spend.rao == DEPOSIT
    assert plan.extras["pure_proxy_reserves"] == {owner.ss58_address: DEPOSIT + 50}
    rejected = await executor.plan(
        CreatePureProxy(proxy_type="Any"),
        delegate,
        proxy_for=owner.ss58_address,
        policy=Policy(max_spend_tao=0),
    )
    assert not rejected.ok


@pytest.mark.parametrize("batched", [False, True])
async def test_pinned_multisig_approval_keeps_registration_cost(batched):
    from bittensor.intents.multisig import MultisigExecute, MultisigIntentAdapter
    from tests.harness.samples import ALICE

    substrate = chain()
    group = substrate.multisig_account([ALICE, BOB], 2)
    substrate.seed(
        "HashedAccounts",
        "DerivedAuthorities",
        [group.ss58_address],
        {"descriptor": {"Multisig": {}}},
    )
    semantic = CreatePureProxy(proxy_type="Any")
    if batched:
        semantic = Batch([semantic, CreatePureProxy(proxy_type="Any", index=1)])
    dispatch = MultisigExecute(threshold=2, other_signatories=[BOB], call=semantic.to_dict())
    adapter = MultisigIntentAdapter(dispatch=dispatch, semantic=semantic)
    executor = Executor(substrate)
    original = await executor.plan(adapter, dev_wallet(), shielded=False)
    adapter.inner_call_data = original.extras["multisig_call_data"]
    from bittensor.fee_filters import _call_dict

    substrate.decode_scale = AsyncMock(return_value=_call_dict(original.call.params["call"]))
    plan = await executor.plan(
        adapter, dev_wallet(), shielded=False, policy=Policy(max_spend_tao=0)
    )
    assert not plan.ok
    assert plan.spend.rao == DEPOSIT * (2 if batched else 1)
    assert any("max_spend_tao" in v for v in plan.violations)
    adapter.inner_call_data = "0x00"
    with pytest.raises(ValueError, match="imported multisig call bytes do not match"):
        await executor.plan(adapter, dev_wallet(), shielded=False)


@pytest.mark.parametrize(
    "constant", ["DerivedRegistrationDeposit", "ProxyDepositBase", "ProxyDepositFactor"]
)
async def test_unknown_pure_creation_cost_fails_closed(constant):
    substrate = chain()
    wallet = protected_wallet(substrate, 7)
    substrate.seed_constant(
        "HashedAccounts" if constant.startswith("Derived") else "Proxy", constant, None
    )
    with pytest.raises(ValueError, match="could not quote"):
        await Executor(substrate).plan(CreatePureProxy(proxy_type="Any"), wallet)


@pytest.mark.parametrize("dry_run", [False, True])
@pytest.mark.parametrize("automatic", [False, True])
@pytest.mark.parametrize("opt_out", [False, True])
def test_cli_review_quotes_selected_route_total(fake, monkeypatch, dry_run, automatic, opt_out):
    fake.mev_key = bytes(1184)
    fake.seed("MevShield", "NextKey", [], fake.mev_key)
    fake.seed("AdminUtils", "HashedAccountsEnabled", [], True)
    fake.seed_constant("HashedAccounts", "MldsaAccountProfiles", "0x0405")
    fake.seed_constant("HashedAccounts", "RegistrationDeposit", DEPOSIT)
    fake.seed_constant("HashedAccounts", "MlDsaRegistrationDeposit", DEPOSIT)
    fake.seed_default("System", "Account", {"data": {"free": 10**13, "frozen": 0}})
    fake.estimate_shielded_carrier_fee = AsyncMock(return_value=Balance.from_rao(CARRIER))
    encrypt = AsyncMock(return_value=success_result())
    monkeypatch.setattr(Executor, "_submit_encrypted_call", encrypt)
    cards = []
    monkeypatch.setattr(
        AppContext, "_transaction_card", lambda *args, **kwargs: cards.append(kwargs)
    )
    recipient = (
        receiving_address(Keypair.create_from_seed(bytes([85]) * 32, 7)) if automatic else BOB
    )
    args = [
        "--yes",
        "--json",
        "--network",
        "local",
        "wallet",
        "transfer",
        "--dest",
        recipient,
        "--amount",
        "1",
        "--max-registration-deposit",
        "0.2",
    ]
    if opt_out:
        args.append("--no-mev-shield")
    elif not automatic:
        args.append("--mev-shield")
    if dry_run:
        args.append("--dry-run")
    result = CliRunner().invoke(app, args)
    assert result.exit_code == 0, result.output
    total = fake.fee.rao + (0 if opt_out else CARRIER)
    if dry_run:
        plan = json.loads(result.output)
        assert Balance.from_tao(plan["fee_tao"]).rao == total
        assert plan["shielded"] is (not opt_out)
    else:
        assert cards[-1]["estimated_fee"].rao == total
        assert cards[-1]["shield"] is (not opt_out)
        assert bool(encrypt.await_count) is (not opt_out)


@pytest.mark.parametrize("batched", [False, True])
async def test_announced_proxy_preserves_reserve_cost_and_payer(batched):
    from bittensor.intents.proxy import ExecuteProxyAnnounced

    substrate = chain()
    owner = protected_wallet(substrate, 7)
    intent = ExecuteProxyAnnounced(
        delegate_ss58=BOB,
        real_ss58=owner.ss58_address,
        inner_op="create_pure_proxy",
        inner_args={"proxy_type": "Any"},
    )
    if batched:
        intent = Batch([intent])
    assert intent.requires_build_for_policy()
    plan = await Executor(substrate).plan(intent, dev_wallet(), policy=Policy(max_spend_tao=0))
    assert not plan.ok
    assert plan.spend.rao == DEPOSIT
    assert plan.extras["pure_proxy_reserves"] == {owner.ss58_address: DEPOSIT + 50}
    assert any("max_spend_tao" in v for v in plan.violations)


@pytest.mark.parametrize("period", [8, 16, 64])
async def test_explicit_shield_quote_uses_actual_submission_period(period):
    substrate = chain()
    executor = Executor(substrate)
    executor._submit_encrypted_call = AsyncMock(return_value=success_result())
    await executor.submit_shielded(Transfer(BOB, 1), dev_wallet(), period=period)
    assert substrate.estimate_shielded_carrier_fee.await_args.kwargs["period"] == period
    assert executor._submit_encrypted_call.await_args.kwargs["period"] == period
