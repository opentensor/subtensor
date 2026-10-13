"""Exact debt sizing, atomic collateral boundaries and fail-closed quote search."""

from __future__ import annotations

import pytest

from bittensor.balance import Balance, UnitMismatchError
from bittensor.reads.lending import lending_open_quote, lending_required_collateral
from bittensor.result import BittensorError, ChainError
from tests.harness.samples import ALICE, ALICE_HOT


class PinnedQuotes:
    """A runtime quote oracle with an arbitrary admissible collateral threshold."""

    block = 101

    def __init__(self, threshold, *, failure=None, existing=None, wrong_principal=False):
        self.threshold = threshold
        self.failure = failure
        self.existing = existing
        self.wrong_principal = wrong_principal
        self.calls = []

    async def at(self):
        return self

    async def query(self, item, params):
        return self.existing if item[1] == "Positions" else None

    async def constant(self, item):
        return 2_628_000

    def balance(self, rao, netuid):
        return Balance.from_rao(rao, netuid)

    async def runtime(self, method, params):
        assert tuple(method) == ("LendingRuntimeApi", "quote_open_exact_for")
        self.calls.append((self.block, list(params)))
        if self.failure:
            raise ChainError(self.failure, name=self.failure)
        opening = params[-1] if params[2] == "Long" else params[-1] // 10
        if opening < 100_000_000:
            raise ChainError("below minimum loan value", name="AmountTooSmall")
        if params[3] < self.threshold:
            raise ChainError("more collateral needed", name="InsufficientCollateral")
        return {
            "Ok": {
                "principal": params[-1] + int(self.wrong_principal),
                "opening_value": opening,
                "annual_interest": opening,
            }
        }


@pytest.mark.asyncio
@pytest.mark.parametrize("side,debt_unit,collateral_unit", [("short", 7, 0), ("long", 0, 7)])
@pytest.mark.parametrize("threshold", [1, 95_500_000_001, (1 << 64) - 1])
async def test_minimum_is_exact_to_one_atom_and_keeps_requested_debt(
    side, debt_unit, collateral_unit, threshold
):
    view = PinnedQuotes(threshold)
    result = await lending_required_collateral(view, 7, side, "1", ALICE, ALICE_HOT)
    assert result["collateral"] == Balance.from_rao(threshold, collateral_unit)
    assert result["principal"] == Balance.from_rao(1_000_000_000, debt_unit)
    assert len(view.calls) <= 128
    assert {block for block, _ in view.calls} == {view.block}
    assert all(
        params[0] == ALICE
        and params[1] == 7
        and params[4] == ALICE_HOT
        and params[-1] == 1_000_000_000
        for _, params in view.calls
    )
    if threshold > 1:
        assert any(params[3] == threshold - 1 for _, params in view.calls)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "name",
    [
        "ReferenceWarmingUp",
        "InsufficientReserves",
        "BorrowingLimit",
        "InsufficientRedemptionBacking",
        "Arithmetic",
    ],
)
async def test_global_refusal_is_not_treated_as_a_request_for_more_collateral(name):
    view = PinnedQuotes(1, failure=name)
    with pytest.raises(ChainError) as error:
        await lending_required_collateral(view, 7, "long", "1", ALICE, ALICE_HOT)
    assert error.value.name == name
    assert len(view.calls) == 1


@pytest.mark.asyncio
@pytest.mark.parametrize("amount", ["0", "18446744073.709551616"])
async def test_invalid_requested_amount_does_not_probe_quotes(amount):
    view = PinnedQuotes(1)
    with pytest.raises(BittensorError):
        await lending_required_collateral(view, 7, "long", amount, ALICE, ALICE_HOT)
    assert not view.calls


@pytest.mark.asyncio
async def test_below_minimum_fixed_debt_cannot_be_repaired_by_more_collateral():
    view = PinnedQuotes(1)
    with pytest.raises(ChainError) as error:
        await lending_required_collateral(view, 7, "long", "0.099999999", ALICE, ALICE_HOT)
    assert error.value.name == "AmountTooSmall"
    assert len(view.calls) == 1


@pytest.mark.asyncio
async def test_minimum_value_boundary_is_admitted():
    result = await lending_required_collateral(
        PinnedQuotes(400_000_000), 7, "long", "0.1", ALICE, ALICE_HOT
    )
    assert result["principal"] == Balance.from_tao("0.1")
    assert result["collateral"] == Balance.from_rao(400_000_000, 7)


@pytest.mark.asyncio
async def test_requested_amount_keeps_its_borrowed_currency():
    view = PinnedQuotes(1)
    with pytest.raises(UnitMismatchError):
        await lending_required_collateral(view, 7, "short", Balance.from_tao(1), ALICE, ALICE_HOT)
    assert not view.calls


@pytest.mark.asyncio
async def test_exact_quote_requires_owner_and_rejects_a_different_principal():
    view = PinnedQuotes(1)
    with pytest.raises(BittensorError, match="both coldkey and hotkey"):
        await lending_open_quote(view, 7, "long", "4", amount="1")
    assert not view.calls
    with pytest.raises(BittensorError, match="different borrowed amount"):
        await lending_required_collateral(
            PinnedQuotes(1, wrong_principal=True), 7, "long", "1", ALICE, ALICE_HOT
        )


def position(collateral):
    return {
        "side": "Long",
        "hotkey": ALICE_HOT,
        "principal": 1_000_000_000,
        "collateral": collateral,
        "proceeds": 0,
        "annual_interest": 1,
        "last_accrued": 101,
        "interest_remainder": 0,
        "due": 50_501,
    }


@pytest.mark.asyncio
async def test_existing_collateral_bounds_addition_without_overflow():
    maximum = (1 << 64) - 1
    view = PinnedQuotes(16, existing=position(maximum - 16))
    result = await lending_required_collateral(view, 7, "long", "1", ALICE, ALICE_HOT)
    assert result["collateral"].rao == 16
    assert all(params[3] <= 16 for _, params in view.calls)
    view = PinnedQuotes(17, existing=position(maximum - 16))
    with pytest.raises(BittensorError, match="position limit"):
        await lending_required_collateral(view, 7, "long", "1", ALICE, ALICE_HOT)
    assert all(params[3] <= 16 for _, params in view.calls)


@pytest.mark.asyncio
async def test_exhausted_position_stops_before_search():
    view = PinnedQuotes(1, existing=position(0))
    with pytest.raises(BittensorError, match="exhausted"):
        await lending_required_collateral(view, 7, "long", "1", ALICE, ALICE_HOT)
    assert not view.calls
