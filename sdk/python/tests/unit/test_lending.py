"""Loan units, full runtime quotes and the three-command CLI contract."""

from __future__ import annotations

import json

import pytest
from typer.main import get_command
from typer.testing import CliRunner

from bittensor._generated.errors import ERRORS
from bittensor.balance import Balance, UnitMismatchError
from bittensor.cli.commands.lending import _bound, _provided_collateral
from bittensor.cli.main import app
from bittensor.client import Client
from bittensor.intents import CloseLoan, OpenLoan, Policy
from bittensor.result import BittensorError, ChainError, PolicyError
from tests.harness.fake_substrate import DEFAULT_STORAGE, FakeSubstrate
from tests.harness.samples import ALICE, ALICE_HOT, BOB, BOB_HOT, dev_wallet
from tests.unit.test_cli_commands import fake as _cli_fake
from tests.unit.test_cli_commands import isolated_cli, wallet_dir  # noqa: F401

runner = CliRunner()


@pytest.fixture()
def cli_fake(request, monkeypatch):
    from bittensor.cli import prompt

    request.getfixturevalue("isolated_cli")
    monkeypatch.setattr(prompt, "_entered_tokens", [])
    fake = _cli_fake.__wrapped__(None, monkeypatch)
    _seed_alpha(fake, 10**18, 10**18)
    return fake


def _seed_alpha(fake, stake, available, owner=ALICE, hotkey=ALICE_HOT, bond=0):
    fake.seed_runtime(
        "StakeInfoRuntimeApi", "get_stake_info_for_hotkey_coldkey_netuid", {"stake": stake}
    )
    fake.seed_runtime(
        "StakeInfoRuntimeApi",
        "get_stake_availability_for_coldkeys",
        lambda params: {params[0][0]: {1: {"available": available}}},
    )
    fake.seed("SubtensorModule", "MinerCollateral", [(1, hotkey, owner)], {"locked": bond})


def _seed_wallet(fake, free, *, owner=ALICE, reserved=0, frozen=0):
    fake.seed(
        "System",
        "Account",
        [owner],
        {"data": {"free": free, "reserved": reserved, "frozen": frozen}},
    )


def _seed_position(substrate: FakeSubstrate, side: str = "Short", **updates):
    position = {**DEFAULT_STORAGE[("Lending", "Positions")], "side": side, **updates}
    substrate.seed("Lending", "Positions", [ALICE, 1], position)
    return position


@pytest.mark.parametrize("side,collateral_unit,debt_unit", [("Short", 0, 1), ("Long", 1, 0)])
@pytest.mark.asyncio
async def test_open_preserves_exact_units_and_atoms(side, collateral_unit, debt_unit):
    fake = FakeSubstrate()
    intent = OpenLoan(
        netuid=1,
        side=side.lower(),
        collateral=Balance.from_rao(1_000_000_001, collateral_unit),
        min_borrow=Balance.from_rao(250_000_001, debt_unit),
        min_proceeds=Balance.from_rao(250_000_003),
    )
    call = await intent.build(fake, dev_wallet())
    assert call.module == "Lending"
    assert call.function == "open"
    assert call.params == {
        "netuid": 1,
        "side": side,
        "collateral": 1_000_000_001,
        "hotkey": ALICE_HOT,
        "min_borrow": 250_000_001,
        "min_proceeds": 250_000_003,
    }


@pytest.mark.parametrize(
    "side,field,wrong_unit",
    [
        ("Short", "collateral", 1),
        ("Long", "collateral", 0),
        ("Short", "min_borrow", 0),
        ("Long", "min_borrow", 1),
        ("Short", "min_proceeds", 1),
        ("Long", "min_proceeds", 1),
        ("Short", "amount", 0),
        ("Long", "amount", 1),
    ],
)
def test_open_rejects_wrong_currency(side, field, wrong_unit):
    args = {"netuid": 1, "side": side, "collateral": "100", field: Balance.from_rao(1, wrong_unit)}
    with pytest.raises(UnitMismatchError):
        OpenLoan(**args)


@pytest.mark.parametrize("collateral", ["0", "-1", "NaN", "Infinity"])
def test_open_rejects_invalid_collateral(collateral):
    with pytest.raises(BittensorError):
        OpenLoan(netuid=1, side="Short", collateral=collateral)


@pytest.mark.parametrize("netuid", [0, -1, 65536])
def test_root_or_invalid_subnet_cannot_borrow(netuid):
    with pytest.raises(BittensorError):
        OpenLoan(netuid=netuid, side="Short", collateral="100")


@pytest.mark.parametrize(
    "side,wallet,payment_unit,refund_unit",
    [("Short", False, 0, 0), ("Short", True, 1, 0), ("Long", False, 0, 1), ("Long", True, 0, 1)],
)
@pytest.mark.asyncio
async def test_close_derives_units_from_owned_position(side, wallet, payment_unit, refund_unit):
    fake = FakeSubstrate()
    _seed_position(fake, side)
    intent = CloseLoan(
        netuid=1,
        repay_from_wallet=wallet,
        max_payment=Balance.from_rao(700_000_001, payment_unit),
        min_refund=Balance.from_rao(900_000_003, refund_unit),
    )
    call = await intent.build(fake, dev_wallet())
    assert call.params == {
        "netuid": 1,
        "repay_from_wallet": wallet,
        "max_payment": 700_000_001,
        "min_refund": 900_000_003,
    }


@pytest.mark.asyncio
async def test_short_close_defaults_to_wallet_alpha_repayment_without_sale_proceeds():
    fake = FakeSubstrate()
    _seed_position(fake, principal=700_000_001)
    intent = CloseLoan(
        netuid=1,
        max_payment=Balance.from_rao(700_000_001, 1),
        min_refund=Balance.from_rao(900_000_003),
    )
    call = await intent.build(fake, dev_wallet())
    assert call.params["repay_from_wallet"] is True
    assert call.params["max_payment"] == 700_000_001
    assert intent.spend() is None


@pytest.mark.asyncio
async def test_default_short_close_rejects_tao_tagged_as_wallet_alpha_payment():
    fake = FakeSubstrate()
    _seed_position(fake)
    with pytest.raises(UnitMismatchError):
        await CloseLoan(netuid=1, max_payment=Balance.from_rao(700)).build(fake, dev_wallet())


@pytest.mark.parametrize("side,tao_spend", [("Short", None), ("Long", 700_000_000_001)])
@pytest.mark.asyncio
async def test_plain_close_amount_gets_currency_before_tao_spend_policy(side, tao_spend):
    fake = FakeSubstrate()
    _seed_position(fake, side, principal=700_000_000_001)
    async with Client("local", substrate=fake) as client:
        plan = await client.plan(
            CloseLoan(netuid=1, max_payment="700.000000001"),
            dev_wallet(),
            policy=Policy(max_spend_tao=0),
        )
    assert plan.call.params["max_payment"] == 700_000_000_001
    if tao_spend is None:
        assert plan.spend is None
        assert plan.ok
    else:
        assert plan.spend == Balance.from_rao(tao_spend)
        assert any("exceeds max_spend_tao" in item for item in plan.violations)


@pytest.mark.asyncio
async def test_close_rejects_wrong_refund_currency():
    fake = FakeSubstrate()
    _seed_position(fake, "Long")
    with pytest.raises(UnitMismatchError):
        await CloseLoan(netuid=1, max_payment="50", min_refund=Balance.from_rao(1)).build(
            fake, dev_wallet()
        )


@pytest.mark.asyncio
async def test_close_rejects_missing_position():
    fake = FakeSubstrate()
    fake.seed("Lending", "Positions", [ALICE, 1], None)
    with pytest.raises(BittensorError, match="no lending position"):
        await CloseLoan(netuid=1, max_payment="50").build(fake, dev_wallet())


@pytest.mark.asyncio
async def test_position_interest_includes_carried_fraction_and_deregistration_cutoff():
    fake = FakeSubstrate()
    fake.block = 120
    fake.seed_constant("Lending", "BlocksPerYear", 10)
    _seed_position(fake, "Long", annual_interest=7, last_accrued=100, interest_remainder=4)
    fake.seed("Lending", "Dissolutions", [1], {"frozen_at": 103})
    async with Client("local", substrate=fake) as client:
        result = await client.read("lending_position", coldkey_ss58=ALICE, netuid=1)
    assert result["interest_due"].rao == 2  # (7 * 3 + 4) // 10, frozen at block 103
    assert result["interest_due"].netuid == 1
    assert result["principal"].netuid == 0
    assert result["principal"].rao == 250_000_000


@pytest.mark.asyncio
async def test_vault_cap_uses_available_plus_outstanding_and_excludes_pending():
    fake = FakeSubstrate()
    fake.seed(
        "Lending",
        "Vaults",
        [1],
        {"available_tao": 950, "outstanding_tao": 50, "pending_tao": 10_000, "lost_tao": 100},
    )
    async with Client("local", substrate=fake) as client:
        result = await client.read("lending_reserves", netuid=1)
    assert result["borrow_headroom_tao"].rao == 50
    assert result["available_tao"].rao == 950


@pytest.mark.asyncio
async def test_quote_preserves_alpha_delivery_and_reference_value_without_sale_proceeds():
    fake = FakeSubstrate()
    seen = []

    def quote(params):
        seen.append(params)
        return {"Ok": {"principal": 234, "annual_interest": 230, "opening_value": 230}}

    fake.seed_runtime("LendingRuntimeApi", "quote_open", quote)
    async with Client("local", substrate=fake) as client:
        result = await client.read(
            "lending_open_quote", netuid=1, side="short", collateral="1.000000001"
        )
    assert seen == [[1, "Short", 1_000_000_001]]
    assert result["principal"].rao == 234
    assert result["principal"].netuid == 1
    assert result["opening_value"] == Balance.from_rao(230)
    assert result["annual_interest"].rao == 230
    assert result["annual_interest"].netuid == 0
    assert "proceeds" not in result


@pytest.mark.parametrize("wallet,payment_unit", [(True, 1), (False, 0)])
@pytest.mark.asyncio
async def test_short_close_quote_selects_wallet_or_collateral_payment(wallet, payment_unit):
    fake = FakeSubstrate()
    _seed_position(fake, principal=700)
    seen = []

    def quote(params):
        seen.append(params)
        return {"Ok": {"payment": 700 if params[2] else 250, "refund": 800}}

    fake.seed_runtime("LendingRuntimeApi", "quote_close", quote)
    async with Client("local", substrate=fake) as client:
        kwargs = {} if wallet else {"repay_from_wallet": False}
        result = await client.read("lending_close_quote", coldkey_ss58=ALICE, netuid=1, **kwargs)
    assert seen == [[ALICE, 1, wallet]]
    assert result["payment"] == Balance.from_rao(700 if wallet else 250, payment_unit)
    assert result["refund"] == Balance.from_rao(800)


@pytest.mark.asyncio
async def test_collateral_buyback_quote_refusal_never_becomes_wallet_repayment():
    fake = FakeSubstrate()
    _seed_position(fake)
    fake.seed_runtime("LendingRuntimeApi", "quote_close", {"Err": "InsufficientEscrow"})
    async with Client("local", substrate=fake) as client:
        with pytest.raises(BittensorError, match="InsufficientEscrow"):
            await client.read(
                "lending_close_quote", coldkey_ss58=ALICE, netuid=1, repay_from_wallet=False
            )


@pytest.mark.parametrize("side,principal_unit,collateral_unit", [("short", 1, 0), ("long", 0, 1)])
@pytest.mark.asyncio
async def test_owner_quote_uses_exact_addition_and_returns_incremental_amounts(
    side, principal_unit, collateral_unit
):
    fake = FakeSubstrate()
    _seed_position(fake, side, principal=9_000_000_000, annual_interest=8_000_000_000)
    seen = []

    def quote(params):
        seen.append(params)
        return {"Ok": {"principal": 234, "annual_interest": 230, "opening_value": 231}}

    fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)
    fake.seed_runtime("LendingRuntimeApi", "quote_open", {"Err": "must use owner quote"})
    async with Client("local", substrate=fake) as client:
        result = await client.read(
            "lending_open_quote",
            netuid=1,
            side=side,
            collateral="1.000000001",
            coldkey_ss58=ALICE,
            hotkey_ss58=ALICE_HOT,
        )
    assert seen == [[ALICE, 1, side.capitalize(), 1_000_000_001, ALICE_HOT]]
    assert result["principal"] == Balance.from_rao(234, principal_unit)
    assert result["annual_interest"] == Balance.from_rao(230, collateral_unit)
    assert result["opening_value"] == Balance.from_rao(231)


@pytest.mark.parametrize("addresses", [{"coldkey_ss58": ALICE}, {"hotkey_ss58": ALICE_HOT}])
@pytest.mark.asyncio
async def test_owner_quote_requires_both_addresses(addresses):
    fake = FakeSubstrate()
    async with Client("local", substrate=fake) as client:
        with pytest.raises(BittensorError, match="requires both coldkey and hotkey"):
            await client.read(
                "lending_open_quote", netuid=1, side="short", collateral="100", **addresses
            )


@pytest.mark.asyncio
async def test_refused_owner_quote_does_not_fall_back_to_fresh_quote():
    fake = FakeSubstrate()
    fake.seed_runtime("LendingRuntimeApi", "quote_open_for", {"Err": "PositionExists"})
    async with Client("local", substrate=fake) as client:
        with pytest.raises(BittensorError, match="PositionExists"):
            await client.read(
                "lending_open_quote",
                netuid=1,
                side="short",
                collateral="100",
                coldkey_ss58=ALICE,
                hotkey_ss58=ALICE_HOT,
            )


@pytest.mark.parametrize("raw", [None, {"Err": {"Module": "BorrowingLimit"}}])
@pytest.mark.asyncio
async def test_unavailable_quote_never_becomes_unprotected_submission(raw):
    fake = FakeSubstrate()
    fake.seed_runtime("LendingRuntimeApi", "quote_open", raw)
    async with Client("local", substrate=fake) as client:
        with pytest.raises(BittensorError):
            await client.read("lending_open_quote", netuid=1, side="short", collateral="1000")


def test_quote_bounds_round_in_protective_direction():
    amount = Balance.from_rao(101, 1)
    assert _bound(amount, 1).rao == 99
    assert _bound(amount, 1, ceiling=True).rao == 103
    assert _bound(amount, 1).netuid == 1


@pytest.mark.parametrize("rao", [1, (1 << 64) - 1])
def test_quote_bounds_keep_tiny_positive_margins_at_atomic_precision(rao):
    amount = Balance.from_rao(rao, 1)
    assert _bound(amount, 1e-26).rao == rao - 1
    assert _bound(amount, 1e-26, ceiling=True).rao == rao + 1


def test_lending_cli_has_exactly_three_commands():
    command = get_command(app).commands["lending"]
    assert set(command.commands) == {"open", "close", "list"}


@pytest.mark.parametrize("side", ["short", "long"])
def test_cli_open_quotes_full_loan_and_sets_floor(cli_fake, side):
    seen = []

    def quote(params):
        seen.append(params)
        return {
            "Ok": {
                "principal": 250_000_000,
                "annual_interest": 250_000_000,
                "opening_value": 250_000_000,
            }
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)
    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open", {"Err": "must use owner quote"})
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--collateral",
            "1000",
        ],
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["op"] == "open_loan"
    assert plan["args"]["min_borrow"] == "0.2475"
    assert plan["args"]["min_proceeds"] == ("0.2475" if side == "short" else "0")
    assert len(seen) == 2  # CLI quote, then the mandatory SDK pre-sign quote.
    assert seen[0] == seen[1]
    owner, netuid, runtime_side, collateral, hotkey = seen[0]
    assert owner
    assert hotkey
    assert [netuid, runtime_side, collateral] == [1, side.capitalize(), 1_000_000_000_000]
    assert cli_fake.submissions == []


def test_cli_close_sets_payment_and_refund_bounds(cli_fake):
    result = runner.invoke(
        app,
        ["--json", "--dry-run", "--no-mev-shield", "lending", "close", "--netuid", "1"],
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["args"]["repay_from_wallet"] is True
    assert plan["args"]["max_payment"] == "0.2525"
    assert plan["args"]["min_refund"] == "0.99"
    assert cli_fake.submissions == []


@pytest.mark.parametrize(
    "flags,wallet,payment,protected_payment",
    [([], True, 700_000_000, "0.707"), (["--no-repay-from-wallet"], False, 250_000_000, "0.2525")],
)
def test_cli_short_close_keeps_repayment_choice_in_quote_and_call(
    cli_fake, flags, wallet, payment, protected_payment
):
    _seed_position(cli_fake, principal=700_000_000)
    seen = []

    def quote(params):
        seen.append(params)
        return {"Ok": {"payment": payment, "refund": 900_000_000}}

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_close", quote)
    result = runner.invoke(
        app,
        ["--json", "--dry-run", "--no-mev-shield", "lending", "close", "--netuid", "1", *flags],
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert len(seen) == 1
    assert seen[0][1:] == [1, wallet]
    assert plan["args"]["repay_from_wallet"] is wallet
    assert plan["args"]["max_payment"] == protected_payment
    assert plan["args"]["min_refund"] == "0.891"
    assert cli_fake.submissions == []


def test_cli_collateral_only_buyback_refusal_does_not_submit(cli_fake):
    cli_fake.seed_runtime("LendingRuntimeApi", "quote_close", {"Err": "InsufficientEscrow"})
    result = runner.invoke(
        app,
        ["--json", "--dry-run", "lending", "close", "--netuid", "1", "--no-repay-from-wallet"],
    )
    assert result.exit_code != 0
    assert cli_fake.submissions == []


@pytest.mark.parametrize(
    "reason",
    [
        "BorrowingLimit",
        "PositionExists",
        "InsufficientEscrow",
        "InsufficientRedemptionBacking",
        "RedemptionUnavailable",
    ],
)
def test_cli_refuses_open_when_quote_fails(cli_fake, reason):
    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", {"Err": reason})
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1000",
        ],
    )
    assert result.exit_code != 0
    assert cli_fake.submissions == []


@pytest.mark.parametrize(
    "index,info",
    [(index, info) for index, info in ERRORS.items() if info.pallet == "Lending"],
    ids=[info.name for info in ERRORS.values() if info.pallet == "Lending"],
)
@pytest.mark.asyncio
async def test_lending_module_quote_errors_have_names_and_descriptions(index, info):
    fake = FakeSubstrate()
    module, error = index
    fake.seed_runtime(
        "LendingRuntimeApi",
        "quote_open",
        {"Err": {"Module": {"index": module, "error": bytes([error, 0, 0, 0]).hex()}}},
    )
    async with Client("local", substrate=fake) as client:
        with pytest.raises(ChainError) as failure:
            await client.read("lending_open_quote", netuid=1, side="short", collateral="10")
    assert failure.value.name == info.name
    assert failure.value.description
    assert "Module" not in str(failure.value)


@pytest.mark.parametrize("method", ["quote_open_for", "quote_close"])
@pytest.mark.asyncio
async def test_owner_and_close_quotes_decode_module_errors(method):
    fake = FakeSubstrate()
    _seed_position(fake)
    fake.seed_runtime(
        "LendingRuntimeApi", method, {"Err": {"Module": {"index": 33, "error": "0x0e000000"}}}
    )
    async with Client("local", substrate=fake) as client:
        with pytest.raises(ChainError) as failure:
            if method == "quote_close":
                await client.read("lending_close_quote", coldkey_ss58=ALICE, netuid=1)
            else:
                await client.read(
                    "lending_open_quote",
                    netuid=1,
                    side="short",
                    collateral="10",
                    coldkey_ss58=ALICE,
                    hotkey_ss58=ALICE_HOT,
                )
    assert failure.value.name == "InsufficientEscrow"


@pytest.mark.parametrize("error_index,name", [(3, "ReferenceWarmingUp"), (6, "AmountTooSmall")])
def test_cli_renders_decoded_lending_quote_error_without_submitting(cli_fake, error_index, name):
    cli_fake.seed_runtime(
        "LendingRuntimeApi",
        "quote_open_for",
        {"Err": {"Module": {"index": 33, "error": f"0x{error_index:02x}000000"}}},
    )
    result = runner.invoke(
        app,
        ["lending", "open", "--netuid", "1", "--side", "short", "--collateral", "10"],
    )
    assert result.exit_code != 0
    assert name in result.output
    assert "Module" not in result.output
    assert "0x" not in result.output
    assert cli_fake.submissions == []


def test_cli_lists_a_subnet_without_wallet_unlock(cli_fake):
    cli_fake.seed_map("Lending", "OpenByNetuid", [(ALICE, None)])
    result = runner.invoke(app, ["--json", "lending", "list", "--netuid", "1"])
    assert result.exit_code == 0, result.output
    positions = json.loads(result.output)
    assert len(positions) == 1
    assert positions[0]["coldkey"] == ALICE
    assert "proceeds" not in positions[0]
    assert cli_fake.submissions == []


def test_cli_position_table_does_not_present_a_locked_sale_balance(cli_fake):
    cli_fake.seed_map("Lending", "OpenByNetuid", [(ALICE, None)])
    result = runner.invoke(app, ["lending", "list", "--netuid", "1"])
    assert result.exit_code == 0, result.output
    assert "proceeds" not in result.output.lower()
    assert "principal" in result.output.lower()


@pytest.mark.parametrize("side,collateral_unit,debt_unit", [("Short", 0, 1), ("Long", 1, 0)])
@pytest.mark.asyncio
async def test_exact_open_builds_the_exact_principal_call(side, collateral_unit, debt_unit):
    fake = FakeSubstrate()
    intent = OpenLoan(
        netuid=1,
        side=side,
        collateral=Balance.from_rao(1_000_000_003, collateral_unit),
        amount=Balance.from_rao(250_000_001, debt_unit),
        min_borrow=Balance.from_rao(250_000_001, debt_unit),
        min_proceeds="0.1",
    )
    call = await intent.build(fake, dev_wallet())
    assert call.function == "open_exact"
    assert call.params == {
        "netuid": 1,
        "side": side,
        "collateral": 1_000_000_003,
        "hotkey": ALICE_HOT,
        "principal": 250_000_001,
        "min_proceeds": 100_000_000,
    }
    assert intent.to_dict()["amount"] == "0.250000001"
    assert "exactly" in intent.summary()


@pytest.mark.parametrize("amount", ["0", "-1", "NaN", "Infinity"])
def test_exact_open_rejects_invalid_amount(amount):
    with pytest.raises(BittensorError):
        OpenLoan(netuid=1, side="short", collateral="1", amount=amount)


def test_exact_open_rejects_minimum_above_requested_principal():
    with pytest.raises(BittensorError, match="min_borrow cannot exceed"):
        OpenLoan(netuid=1, side="long", collateral="1", amount="0.1", min_borrow="0.100000001")


@pytest.mark.parametrize(
    "free,reserved,frozen,allowed,reason",
    [
        (1_000_000_000 + 124_414 + 500, 0, 0, True, None),
        (1_000_000_000 + 124_414 + 499, 0, 0, False, "collateral plus transaction fee"),
        (1_000_000_000 + 500, 0, 0, False, "collateral plus transaction fee"),
        (2_000_000_000, 0, 1_100_000_000, False, "insufficient spendable collateral"),
        (2_000_000_000, 200_000_000, 1_100_000_000, True, None),
    ],
)
@pytest.mark.asyncio
async def test_short_preflight_uses_spendable_collateral_and_combined_fees(
    free, reserved, frozen, allowed, reason
):
    fake = FakeSubstrate()
    _seed_wallet(fake, free, reserved=reserved, frozen=frozen)
    async with Client("local", substrate=fake) as client:
        plan = await client.plan(OpenLoan(netuid=1, side="short", collateral="1"), dev_wallet())
    assert plan.ok is allowed
    if reason is not None:
        assert any(reason in block for block in plan.violations)
    assert fake.submissions == []


@pytest.mark.parametrize(
    "stake,available,bond,expected_available",
    [(200, 90, 0, 90), (90, 200, 0, 90), (200, 300, 120, 80)],
)
@pytest.mark.asyncio
async def test_long_preflight_limits_collateral_by_hotkey_locks_and_bonds(
    stake, available, bond, expected_available
):
    fake = FakeSubstrate()
    _seed_wallet(fake, 1_000_000)
    _seed_alpha(fake, stake, available, bond=bond)
    async with Client("local", substrate=fake) as client:
        intent = OpenLoan(netuid=1, side="long", collateral=Balance.from_rao(100, 1))
        preview = await client.preflight(intent, dev_wallet())
    assert any("insufficient spendable collateral" in block for block in preview.blocks)
    assert ("available collateral", str(Balance.from_rao(expected_available, 1))) in preview.facts


@pytest.mark.asyncio
async def test_other_hotkey_bond_does_not_freeze_selected_hotkey_alpha():
    fake = FakeSubstrate()
    _seed_wallet(fake, 1_000_000)
    _seed_alpha(fake, 200, 150, hotkey=BOB_HOT, bond=200)
    async with Client("local", substrate=fake) as client:
        preview = await client.preflight(
            OpenLoan(netuid=1, side="long", collateral=Balance.from_rao(100, 1)), dev_wallet()
        )
    assert preview.blocks == []


@pytest.mark.asyncio
async def test_long_borrowed_tao_cannot_pay_its_own_inclusion_fee():
    fake = FakeSubstrate()
    _seed_wallet(fake, 500)
    _seed_alpha(fake, 10**9, 10**9)
    async with Client("local", substrate=fake) as client:
        plan = await client.plan(OpenLoan(netuid=1, side="long", collateral="1"), dev_wallet())
    assert not plan.ok
    assert any("transaction fee" in block for block in plan.violations)


@pytest.mark.parametrize("payer_funded", [False, True])
@pytest.mark.asyncio
async def test_proxy_collateral_owner_and_fee_payer_are_checked_separately(payer_funded):
    fake = FakeSubstrate()
    _seed_wallet(fake, 1_000_000_500)
    _seed_wallet(fake, 124_914 if payer_funded else 500, owner=BOB)
    seen = []

    def quote(params):
        seen.append(params)
        return {
            "Ok": {"principal": 250_000_000, "opening_value": 250_000_000, "annual_interest": 1}
        }

    fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)
    async with Client("local", substrate=fake) as client:
        plan = await client.plan(
            OpenLoan(netuid=1, side="short", collateral="1", hotkey_ss58=ALICE_HOT),
            dev_wallet("//Bob"),
            proxy_for=ALICE,
        )
    assert plan.ok is payer_funded
    assert seen[0][0] == ALICE
    assert plan.signer_address == BOB
    if not payer_funded:
        assert any("transaction fee" in block for block in plan.violations)


@pytest.mark.parametrize("failure", ["fee", "balance", "alpha", "quote"])
@pytest.mark.asyncio
async def test_mandatory_lending_preflight_failures_stop_before_signing(monkeypatch, failure):
    from bittensor import executor

    fake = FakeSubstrate()
    _seed_wallet(fake, 10**12)
    _seed_alpha(fake, 10**12, 10**12)
    side = "long" if failure == "alpha" else "short"

    async def unavailable(*_args, **_kwargs):
        raise RuntimeError("unavailable")

    if failure == "fee":
        monkeypatch.setattr(fake, "estimate_fee", unavailable)
    elif failure == "balance":
        fake.seed("System", "Account", [ALICE], {"data": {"free": "invalid"}})
    elif failure == "alpha":
        fake.seed_runtime("StakeInfoRuntimeApi", "get_stake_availability_for_coldkeys", None)
    else:
        fake.seed_runtime("LendingRuntimeApi", "quote_open_for", None)

    def unexpected_signer(*_args, **_kwargs):
        pytest.fail("failed lending preflight must not request a signer")

    monkeypatch.setattr(executor, "resolve_signer", unexpected_signer)
    async with Client("local", substrate=fake) as client:
        with pytest.raises(PolicyError, match="could not verify lending"):
            await client.execute(OpenLoan(netuid=1, side=side, collateral="1"), dev_wallet())
    assert fake.submissions == []


@pytest.mark.parametrize(
    "amount,minimum,opening,quoted,reason",
    [
        ("0.25", "0", "0", 250_000_001, "does not match the exact"),
        (None, "0.250000001", "0", 250_000_000, "minimum additional principal"),
        (None, "0", "0.250000001", 250_000_000, "minimum opening value"),
    ],
)
@pytest.mark.asyncio
async def test_fresh_preflight_quote_preserves_principal_and_opening_bounds(
    amount, minimum, opening, quoted, reason
):
    fake = FakeSubstrate()
    _seed_wallet(fake, 10**12)
    fake.seed_runtime(
        "LendingRuntimeApi",
        "quote_open_exact_for" if amount else "quote_open_for",
        {"Ok": {"principal": quoted, "opening_value": quoted, "annual_interest": quoted}},
    )
    async with Client("local", substrate=fake) as client:
        preview = await client.preflight(
            OpenLoan(
                netuid=1,
                side="short",
                collateral="1",
                amount=amount,
                min_borrow=minimum,
                min_proceeds=opening,
            ),
            dev_wallet(),
        )
    assert any(reason in block for block in preview.blocks)


def test_cli_requires_one_input_in_scripts_without_querying(cli_fake):
    def unexpected_quote(_params):
        pytest.fail("missing input must fail before querying")

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", unexpected_quote)
    result = runner.invoke(app, ["--json", "lending", "open", "--netuid", "1", "--side", "short"])
    assert result.exit_code == 2
    assert "--amount or --collateral" in result.output
    assert cli_fake.submissions == []


def test_cli_rejects_two_input_modes(cli_fake):
    result = runner.invoke(
        app,
        [
            "--json",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--amount",
            "0.1",
            "--collateral",
            "1",
        ],
    )
    assert result.exit_code == 2
    assert "mutually exclusive" in result.output
    assert cli_fake.submissions == []


def _mock_required_collateral(monkeypatch, fake, collateral=1_000_000_003):
    fake.seed_default("Lending", "Positions", None)
    original = Client.read
    requested = []

    async def read(client, name, **kwargs):
        if name != "lending_required_collateral":
            return await original(client, name, **kwargs)
        requested.append(kwargs)
        return {"collateral": Balance.from_rao(collateral, 0 if kwargs["side"] == "short" else 1)}

    monkeypatch.setattr(Client, "read", read)
    seen = []

    def quote(params):
        seen.append(params)
        return {
            "Ok": {
                "principal": params[5],
                "opening_value": 100_000_000,
                "annual_interest": 100_000_000,
            }
        }

    fake.seed_runtime("LendingRuntimeApi", "quote_open_exact_for", quote)
    return requested, seen


@pytest.mark.parametrize("side,unit", [("short", 1), ("long", 0)])
def test_cli_amount_uses_required_collateral_and_exact_principal(cli_fake, monkeypatch, side, unit):
    required, seen = _mock_required_collateral(monkeypatch, cli_fake)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--amount",
            "0.100000001",
        ],
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["args"]["amount"] == "0.100000001"
    assert plan["args"]["min_borrow"] == "0.100000001"
    assert plan["args"]["collateral"] == "1.010000004"
    assert required[0]["amount"] == Balance.from_rao(100_000_001, unit)
    assert len(seen) == 2  # Fresh CLI quote plus the SDK pre-sign quote.
    assert all(params[3] == 1_010_000_004 and params[5] == 100_000_001 for params in seen)
    assert cli_fake.submissions == []


@pytest.mark.parametrize("side,label", [("short", "subnet 1 alpha"), ("long", "TAO")])
def test_cli_prompt_defaults_to_borrow_amount_with_contextual_units(
    cli_fake, monkeypatch, side, label
):
    from bittensor.cli import prompt

    monkeypatch.setattr(prompt, "interactive", lambda _ctx: True)
    _mock_required_collateral(monkeypatch, cli_fake)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--hotkey",
            ALICE_HOT,
        ],
        input="\n0.1\n",
    )
    assert result.exit_code == 0, result.output
    assert "[amount]" in result.output
    assert f"Additional {label} to borrow" in result.output
    assert "borrow exactly" in result.output
    assert "--amount 0.1" in result.output
    assert "--collateral 0.1" not in result.output


@pytest.mark.parametrize("side,label", [("short", "TAO"), ("long", "subnet 1 alpha")])
def test_cli_prompt_can_choose_collateral_with_contextual_units(cli_fake, monkeypatch, side, label):
    from bittensor.cli import prompt

    monkeypatch.setattr(prompt, "interactive", lambda _ctx: True)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--hotkey",
            ALICE_HOT,
        ],
        input="2\n1\n",
    )
    assert result.exit_code == 0, result.output
    assert f"Additional {label} to lock" in result.output
    assert "borrowing limit" in " ".join(result.output.split())
    assert "--collateral 1" in result.output
    assert "--amount 1" not in result.output


def test_cli_failed_wallet_check_prevents_transaction_review(cli_fake, monkeypatch):
    from bittensor.cli.context import AppContext

    cli_fake.seed_default("System", "Account", {"data": {"free": 500, "reserved": 0, "frozen": 0}})

    def unexpected_review(*_args, **_kwargs):
        pytest.fail("failed wallet check must prevent the transaction review")

    monkeypatch.setattr(AppContext, "_transaction_card", unexpected_review)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
        ],
    )
    assert result.exit_code == 1, result.output
    assert "insufficient spendable collateral" in result.output
    assert cli_fake.submissions == []


def test_cli_proxy_quote_uses_the_collateral_owner(cli_fake):
    owners = []

    def quote(params):
        owners.append(params[0])
        return {
            "Ok": {"principal": 250_000_000, "opening_value": 250_000_000, "annual_interest": 1}
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
            "--proxy-for",
            ALICE,
        ],
    )
    assert result.exit_code == 0, result.output
    assert owners == [ALICE, ALICE]


class _PublicSigner:
    crypto_type = 1

    def __init__(self):
        key = dev_wallet("//Bob").coldkey
        self.ss58_address = key.ss58_address
        self.public_key = key.public_key

    def sign(self, _payload):
        pytest.fail("failed preflight must not request an external signature")

    async def close(self):
        pass


@pytest.mark.parametrize("failure", ["collateral", "fee", "fee_rpc"])
def test_external_wallet_checks_stop_before_review_and_signature(cli_fake, monkeypatch, failure):
    from bittensor.cli.context import AppContext

    signer = _PublicSigner()
    monkeypatch.setattr(AppContext, "ledger_signer", lambda _ctx: signer)
    _seed_wallet(cli_fake, 500 if failure != "fee_rpc" else 10**12, owner=BOB)
    side = "long" if failure == "fee" else "short"
    owners = []

    def quote(params):
        owners.append(params[0])
        return {
            "Ok": {
                "principal": 250_000_000,
                "opening_value": 250_000_000,
                "annual_interest": 250_000_000,
            }
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)

    if failure == "fee_rpc":

        async def unavailable(*_args, **_kwargs):
            raise RuntimeError("fee RPC unavailable")

        monkeypatch.setattr(cli_fake, "estimate_fee", unavailable)

    def unexpected_review(*_args, **_kwargs):
        pytest.fail("external loan wallet checks must run before review")

    monkeypatch.setattr(AppContext, "_transaction_card", unexpected_review)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--collateral",
            "1",
            "--hotkey",
            ALICE_HOT,
            "--signer",
            "ledger",
            "--signer-address",
            BOB,
        ],
    )
    assert result.exit_code == 1, result.output
    assert owners == [BOB, BOB]
    expected = {
        "collateral": "insufficient spendable collateral",
        "fee": "transaction fee",
        "fee_rpc": "fee RPC unavailable",
    }[failure]
    assert expected in result.output
    assert cli_fake.submissions == []


def test_combined_carrier_and_collateral_shortfall_stops_before_review(cli_fake, monkeypatch):
    from bittensor.cli.context import AppContext

    # Each fee by itself fits. Reserving collateral and the inner fee leaves
    # nothing for the outer carrier, so the combined check must refuse.
    cli_fake.seed_default(
        "System",
        "Account",
        {"data": {"free": 1_000_000_000 + cli_fake.fee.rao + 500, "reserved": 0, "frozen": 0}},
    )
    cli_fake.seed("MevShield", "NextKey", [], bytes(32))

    def unexpected_review(*_args, **_kwargs):
        pytest.fail("combined collateral and shield fee shortfall must prevent review")

    monkeypatch.setattr(AppContext, "_transaction_card", unexpected_review)
    result = runner.invoke(
        app,
        [
            "--yes",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
            "--mev-shield",
        ],
    )
    assert result.exit_code == 1, result.output
    assert "loan transaction and MEV-shield carrier fees" in " ".join(result.output.split())
    assert cli_fake.submissions == []


@pytest.mark.parametrize("failure", ["combined_shortfall", "carrier_rpc", "key_rpc", "inactive"])
def test_dry_run_shield_affordability_is_a_hard_violation(cli_fake, monkeypatch, failure):
    free = 1_000_000_000 + cli_fake.fee.rao + 500 if failure == "combined_shortfall" else 10**12
    cli_fake.seed_default("System", "Account", {"data": {"free": free, "reserved": 0, "frozen": 0}})
    if failure != "inactive":
        cli_fake.seed("MevShield", "NextKey", [], bytes(32))
    if failure == "carrier_rpc":
        original_fee = cli_fake.estimate_fee

        async def estimate(call, keypair):
            if call.function == "submit_encrypted":
                raise RuntimeError("carrier fee unavailable")
            return await original_fee(call, keypair)

        monkeypatch.setattr(cli_fake, "estimate_fee", estimate)
    elif failure == "key_rpc":
        original_query = cli_fake.query

        async def query(module, item, *args, **kwargs):
            if (module, item) == ("MevShield", "NextKey"):
                raise RuntimeError("shield key unavailable")
            return await original_query(module, item, *args, **kwargs)

        monkeypatch.setattr(cli_fake, "query", query)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--json",
            "--dry-run",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
            "--mev-shield",
        ],
    )
    assert result.exit_code == 1, result.output
    plan = json.loads(result.output)
    assert plan["ok"] is False
    expected = {
        "combined_shortfall": "loan transaction and MEV-shield carrier fees",
        "carrier_rpc": "carrier fee unavailable",
        "key_rpc": "shield key unavailable",
        "inactive": "MEV shield is not active",
    }[failure]
    assert any(expected in violation for violation in plan["violations"])
    assert cli_fake.submissions == []


@pytest.mark.asyncio
async def test_sdk_shield_combined_shortfall_stops_before_signing(monkeypatch):
    from bittensor import executor

    fake = FakeSubstrate()
    _seed_wallet(fake, 1_000_000_000 + fake.fee.rao + 500)

    def unexpected_signer(*_args, **_kwargs):
        pytest.fail("combined shield fee shortfall must prevent a signature")

    monkeypatch.setattr(executor, "resolve_signer", unexpected_signer)
    async with Client("local", substrate=fake) as client:
        with pytest.raises(PolicyError, match="carrier fee"):
            await client.submit_shielded(
                OpenLoan(netuid=1, side="short", collateral="1"), dev_wallet()
            )
    assert fake.submissions == []


@pytest.mark.parametrize("change", ["principal", "cap"])
def test_fresh_owner_quote_failure_stops_before_review(cli_fake, monkeypatch, change):
    from bittensor.cli.context import AppContext

    count = 0

    def quote(_params):
        nonlocal count
        count += 1
        if count > 1 and change == "cap":
            return {"Err": {"Module": {"index": 33, "error": "0x08000000"}}}
        principal = 250_000_000 if count == 1 else 240_000_000
        return {
            "Ok": {"principal": principal, "opening_value": principal, "annual_interest": principal}
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)

    def unexpected_review(*_args, **_kwargs):
        pytest.fail("a refused fresh quote must prevent review")

    monkeypatch.setattr(AppContext, "_transaction_card", unexpected_review)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
        ],
    )
    assert result.exit_code == 1, result.output
    assert count == 2
    assert (
        "minimum additional principal" if change == "principal" else "BorrowingLimit"
    ) in result.output
    assert cli_fake.submissions == []


def test_extension_uses_one_selected_account_for_quote_and_dry_run(cli_fake, monkeypatch):
    from bittensor import config
    from bittensor.cli.context import AppContext

    # The saved default is only a picker default. The real account selection
    # must define the quote owner and remain selected through SDK preflight.
    config.set_value("signer_address", ALICE)
    selections = []
    requests = []
    signer = _PublicSigner()

    async def extension(context, *, pick_account=False):
        requests.append(pick_account)
        if context._extension_selection is None or pick_account:
            selections.append(BOB)
            context._extension_selection = signer
        return signer

    monkeypatch.setattr(AppContext, "extension_signer", extension)
    owners = []

    def quote(params):
        owners.append(params[0])
        return {
            "Ok": {
                "principal": 250_000_000,
                "opening_value": 250_000_000,
                "annual_interest": 250_000_000,
            }
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_for", quote)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--collateral",
            "1",
            "--hotkey",
            ALICE_HOT,
            "--signer",
            "extension",
        ],
    )
    assert result.exit_code == 0, result.output
    assert json.loads(result.output)["signer_address"] == BOB
    assert owners == [BOB, BOB]
    assert selections == [BOB]
    assert requests == [False, False]
    assert cli_fake.submissions == []


@pytest.mark.parametrize("side", ["short", "long"])
def test_cli_amount_runs_real_required_collateral_search_and_requotes(cli_fake, side):
    cli_fake.seed_default("Lending", "Positions", None)
    threshold = 400_000_003
    seen = []

    def quote(params):
        seen.append(params)
        if params[3] < threshold:
            raise ChainError("more collateral needed", name="InsufficientCollateral")
        return {
            "Ok": {
                "principal": params[5],
                "opening_value": params[5],
                "annual_interest": params[5],
            }
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_exact_for", quote)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--amount",
            "0.1",
        ],
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["args"]["collateral"] == "0.404000004"
    assert plan["args"]["amount"] == "0.1"
    assert seen[-2] == seen[-1]
    assert all(params[-1] == 100_000_000 for params in seen)
    assert all(params[3] == 404_000_004 for params in seen[-2:])
    assert cli_fake.submissions == []


@pytest.mark.parametrize("side", ["short", "long"])
@pytest.mark.parametrize("percent", [0, 1])
def test_amount_collateral_buffer_covers_large_existing_coupon_across_block_change(
    cli_fake, monkeypatch, side, percent
):
    _mock_required_collateral(monkeypatch, cli_fake, collateral=400_000_000)
    cli_fake.seed_constant("Lending", "BlocksPerYear", 100)
    cli_fake.seed_default(
        "Lending",
        "Positions",
        {
            **DEFAULT_STORAGE[("Lending", "Positions")],
            "side": side.title(),
            "hotkey": ALICE_HOT,
            "principal": 1_000_000_000,
            "collateral": 4_000_000_000,
            "annual_interest": 1_000_000_000,
        },
    )
    original = Client.read

    async def read(client, name, **kwargs):
        result = await original(client, name, **kwargs)
        if name == "lending_required_collateral":
            cli_fake.block += 1
        return result

    monkeypatch.setattr(Client, "read", read)
    seen = []

    def quote(params):
        seen.append(params)
        # One block's old interest already exceeds 1% of this small increase.
        required = 400_000_000 + (cli_fake.block - 100) * 10_000_000
        if params[3] < required:
            raise ChainError("old coupon accrued", name="InsufficientCollateral")
        return {
            "Ok": {
                "principal": params[5],
                "opening_value": 100_000_000,
                "annual_interest": 100_000_000,
            }
        }

    cli_fake.seed_runtime("LendingRuntimeApi", "quote_open_exact_for", quote)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            side,
            "--amount",
            "0.1",
            "--hotkey",
            ALICE_HOT,
            "--max-slippage",
            str(percent),
        ],
    )
    if percent == 0:
        assert result.exit_code == 1, result.output
        assert "InsufficientCollateral" in result.output
        assert seen[0][3] == 400_000_000
    else:
        assert result.exit_code == 0, result.output
        plan = json.loads(result.output)
        # 404m with 1% headroom, plus 1b * 128 / 100 of old coupon.
        assert plan["args"]["collateral"] == "1.684"
        assert plan["args"]["amount"] == "0.1"
        assert plan["args"]["min_borrow"] == "0.1"
        assert all(params[3] == 1_684_000_000 for params in seen)
    assert all(params[5] == 100_000_000 for params in seen)
    assert cli_fake.submissions == []


def test_amount_review_distinguishes_minimum_and_buffered_collateral(cli_fake, monkeypatch):
    from bittensor.cli.context import AppContext

    _mock_required_collateral(monkeypatch, cli_fake)
    rows = {}

    def submit(_context, intent, **kwargs):
        rows.update(kwargs["card_sections"][0][1])
        assert intent.amount == Balance.from_rao(100_000_000, 1)
        assert intent.collateral == Balance.from_rao(1_010_000_004)

    monkeypatch.setattr(AppContext, "submit", submit)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--amount",
            "0.1",
        ],
    )
    assert result.exit_code == 0, result.output
    assert rows["calculated minimum collateral"] == str(Balance.from_rao(1_000_000_003))
    assert rows["provided collateral"] == str(Balance.from_rao(1_010_000_004))
    assert rows["additional interest per year"] == str(Balance.from_rao(100_000_000))


@pytest.mark.asyncio
async def test_tiny_positive_collateral_headroom_rounds_up_one_atom():
    fake = FakeSubstrate()
    fake.seed_default("Lending", "Positions", None)
    async with Client("local", substrate=fake) as client:
        provided = await _provided_collateral(client, Balance.from_rao(1), 1, ALICE, 1e-26)
    assert provided == Balance.from_rao(2)


@pytest.mark.asyncio
async def test_zero_collateral_headroom_does_not_query_old_coupon():
    class NoReads:
        async def at(self):
            pytest.fail("zero tolerance must keep the calculated minimum without extra reads")

    minimum = Balance.from_rao(1_000_000_001, 1)
    assert await _provided_collateral(NoReads(), minimum, 1, ALICE, 0) == minimum


def test_buffered_collateral_overflow_stops_before_composition(cli_fake, monkeypatch):
    _required, seen = _mock_required_collateral(monkeypatch, cli_fake, collateral=(1 << 64) - 1)

    async def unexpected_compose(*_args, **_kwargs):
        pytest.fail("an overflowing collateral deposit must not reach composition")

    monkeypatch.setattr(cli_fake, "compose", unexpected_compose)
    result = runner.invoke(
        app,
        [
            "--json",
            "--dry-run",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--amount",
            "0.1",
        ],
    )
    assert result.exit_code == 1, result.output
    assert "buffered collateral does not fit" in result.output
    assert seen == []
    assert cli_fake.submissions == []


def test_wallet_must_cover_buffered_collateral_before_review(cli_fake, monkeypatch):
    from bittensor.cli.context import AppContext

    _mock_required_collateral(monkeypatch, cli_fake)
    cli_fake.seed_default(
        "System",
        "Account",
        {"data": {"free": 1_000_000_003 + cli_fake.fee.rao + 500, "reserved": 0, "frozen": 0}},
    )

    def unexpected_review(*_args, **_kwargs):
        pytest.fail("buffer affordability must be checked before review")

    monkeypatch.setattr(AppContext, "_transaction_card", unexpected_review)
    result = runner.invoke(
        app,
        [
            "--yes",
            "--no-mev-shield",
            "lending",
            "open",
            "--netuid",
            "1",
            "--side",
            "short",
            "--amount",
            "0.1",
        ],
    )
    assert result.exit_code == 1, result.output
    assert "insufficient spendable collateral" in result.output
    assert cli_fake.submissions == []
