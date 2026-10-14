"""Emission weight vector: pure rules, the intent's plan, the read, and the
``btcli subnets emission`` commands against the in-memory chain."""

from __future__ import annotations

import asyncio
import json

import pytest

from bittensor import emission_weights as ew
from bittensor.client import Client
from bittensor.intents import SetSubnetEmissionWeights
from tests.harness.fake_substrate import FakeSubstrate
from tests.harness.samples import EMISSION_WEIGHTS_INFO, TAO_WEIGHT, dev_wallet
from tests.unit.test_cli_commands import invoke, isolated_cli, wallet_dir  # noqa: F401

ONE = ew.ONE
DEFAULT = ew.default_weights()


# --- pure helpers -----------------------------------------------------------


def test_constants_match_chain():
    assert ew.MAX_OWNER == (ONE * 33) // 100
    assert ew.MAX_VALIDATOR == (ONE * 50) // 100
    assert ew.MIN_MINER == (ONE * 33) // 100
    assert ew.DEFAULT_OWNER == (11796 * ONE) // 65535
    assert sum(DEFAULT) == ONE
    assert DEFAULT[ew.BURN] == DEFAULT[ew.RECYCLE] == 0
    assert DEFAULT[ew.VALIDATOR] == (ONE - ew.DEFAULT_OWNER) // 2


@pytest.mark.parametrize(
    "raw, expected",
    [
        ("owner", 2),
        ("Validator", 3),
        (" miner ", 4),
        ("3", 3),
        (0, 0),
        (4, 4),
    ],
)
def test_slot_index_accepts_names_and_uids(raw, expected):
    assert ew.slot_index(raw) == expected


@pytest.mark.parametrize("raw", ["gamma", 5, -1, "5"])
def test_slot_index_rejects_unknown(raw):
    with pytest.raises(ValueError, match="slots are"):
        ew.slot_index(raw)


def test_parse_share_forms():
    assert ew.parse_share(0.5) == ONE // 2 + 1 or ew.parse_share(0.5) == round(ONE / 2)
    assert ew.parse_share("41%") == round(0.41 * ONE)
    assert ew.parse_share("0.41") == round(0.41 * ONE)
    assert ew.parse_share(12345) == 12345
    assert ew.parse_share("12345") == 12345
    assert ew.parse_share("1.0") == ONE
    assert ew.parse_share("0") == 0


@pytest.mark.parametrize("raw", ["1.5", "150%", "-0.1", -5, ONE + 1, "abc", "", True])
def test_parse_share_rejects_out_of_range(raw):
    with pytest.raises(ValueError):
        ew.parse_share(raw)


def test_format_share():
    assert ew.format_share(ONE) == "100.00%"
    assert ew.format_share(ONE // 2) == "50.00%"
    assert ew.format_share(0) == "0.00%"


def test_merge_zeroes_reserved_slots_and_keeps_the_rest():
    current = [7, 9, 10, 20, 30]
    merged = ew.merge(current, {"owner": 11})
    assert merged == [0, 0, 11, 20, 30]
    assert ew.merge(current, {3: 1, "miner": 2}) == [0, 0, 10, 1, 2]


def test_renormalize_scales_untouched_slots_proportionally():
    owner = ew.parse_share("10%")
    out = ew.renormalize(DEFAULT, {"owner": owner})
    assert sum(out) == ONE
    assert out[ew.OWNER] == owner
    # Validator and miner were (within a part) equal and stay so; miner absorbs rounding.
    assert abs(out[ew.VALIDATOR] - out[ew.MINER]) <= 4
    assert out[ew.VALIDATOR] > DEFAULT[ew.VALIDATOR]


def test_renormalize_keeps_two_pinned_and_fills_the_third():
    out = ew.renormalize(DEFAULT, {"owner": ONE // 10, "validator": ONE // 2})
    assert out == [0, 0, ONE // 10, ONE // 2, ONE - ONE // 10 - ONE // 2]


def test_renormalize_rejects_overflow_and_nonzero_reserved():
    with pytest.raises(ValueError, match="exceed 100%"):
        ew.renormalize(DEFAULT, {"owner": ONE, "validator": 1})
    with pytest.raises(ValueError, match="must stay 0"):
        ew.renormalize(DEFAULT, {"burn": 1})
    with pytest.raises(ValueError, match="not 100%"):
        ew.renormalize(DEFAULT, {"owner": 1, "validator": 1, "miner": 1})


def test_renormalize_splits_evenly_when_free_slots_are_zero():
    out = ew.renormalize([0, 0, ONE, 0, 0], {"owner": ONE // 2})
    assert sum(out) == ONE
    assert abs(out[ew.VALIDATOR] - out[ew.MINER]) <= 1


def test_validate_accepts_default_and_names_each_rule():
    rules = ew.Rules(min_validator=ew.parse_share("2.2%"))
    assert ew.validate(DEFAULT, rules) == []

    def errors(weights, rules=rules):
        return [violation.error for violation in ew.validate(weights, rules)]

    assert errors([1, 0, *DEFAULT[2:]]) == [
        "BurnEmissionMustBeZero",
        "EmissionWeightsDoNotSumToOne",
    ]
    assert "RecycleEmissionMustBeZero" in errors([0, 1, *DEFAULT[2:]])
    assert "OwnerEmissionAboveMaximum" in errors(
        ew.renormalize(DEFAULT, {"owner": ew.parse_share("34%")})
    )
    assert "ValidatorEmissionAboveMaximum" in errors(
        ew.renormalize(DEFAULT, {"validator": ew.parse_share("51%")})
    )
    assert errors(ew.renormalize(DEFAULT, {"validator": ew.parse_share("1%")})) == [
        "ValidatorEmissionBelowTaoWeightFloor"
    ]
    assert errors([0, 0, 0, 0, ONE], ew.Rules()) == ["ValidatorEmissionBelowTaoWeightFloor"]
    assert "MinerEmissionBelowMinimum" in errors(
        [0, 0, ew.MAX_OWNER, ew.MAX_VALIDATOR, ONE - ew.MAX_OWNER - ew.MAX_VALIDATOR]
    )
    assert errors([0, 0, 0, ONE // 2, ONE // 2]) == ["EmissionWeightsDoNotSumToOne"]


def test_validate_lease_floor_only_on_lease_created_subnets():
    low_owner = ew.renormalize(DEFAULT, {"owner": ew.parse_share("5%")})
    assert ew.validate(low_owner, ew.Rules()) == []
    leased = ew.Rules(min_owner=ew.DEFAULT_OWNER, created_via_lease=True)
    violations = ew.validate(low_owner, leased)
    assert [v.error for v in violations] == ["LeasedSubnetOwnerCutBelowMinimum"]
    assert "lease" in violations[0].message


def test_tao_weight_helpers():
    base = DEFAULT[ew.VALIDATOR]
    assert ew.tao_weight_multiplier(base, base) == 1.0
    assert ew.tao_weight_multiplier(base, base // 2) == pytest.approx(2.0)
    assert ew.tao_weight_multiplier(base, 0) == float("inf")
    tao_weight = ew.parse_share("18%")
    assert ew.effective_tao_weight(tao_weight, base, base) == pytest.approx(0.18)
    assert ew.effective_tao_weight(tao_weight, base, base // 2) == pytest.approx(0.36)
    assert ew.effective_tao_weight(tao_weight, base, base // 100) == 1.0


def test_rules_table_mentions_every_slot():
    rows = ew.Rules(min_validator=ew.parse_share("2.2%"), created_via_lease=True).table()
    assert [row[0] for row in rows] == [
        "0 burn",
        "1 recycle",
        "2 owner",
        "3 validator",
        "4 miner",
        "sum",
    ]
    assert "lease" in rows[2][2]


# --- intent -----------------------------------------------------------------


def _client() -> tuple[Client, FakeSubstrate]:
    fake = FakeSubstrate()
    fake.seed_runtime("SubnetInfoRuntimeApi", "get_subnet_emission_weights", EMISSION_WEIGHTS_INFO)
    fake.seed("SubtensorModule", "TaoWeight", [], TAO_WEIGHT)
    return Client("local", substrate=fake), fake


def test_intent_normalises_slots_and_shares():
    intent = SetSubnetEmissionWeights(netuid=1, slots=["owner", "3"], values=["10%", 0.45])
    assert intent.slots == [2, 3]
    assert intent.values == [ew.parse_share("10%"), ew.parse_share(0.45)]
    assert "owner=10.00%" in intent.summary()


@pytest.mark.parametrize(
    "kwargs, match",
    [
        ({"slots": ["owner"], "values": []}, "pair up"),
        ({"slots": [], "values": []}, "at least one"),
        ({"slots": ["owner", 2], "values": [1, 2]}, "at most once"),
        ({"slots": ["burn"], "values": [1]}, "must be 0"),
        ({"slots": ["gamma"], "values": [1]}, "slots are"),
    ],
)
def test_intent_rejects_malformed_input(kwargs, match):
    with pytest.raises(ValueError, match=match):
        SetSubnetEmissionWeights(netuid=1, **kwargs)


def test_intent_plan_renormalizes_and_reports_multiplier():
    client, _ = _client()
    intent = SetSubnetEmissionWeights(
        netuid=1, slots=["validator"], values=["30%"], renormalize=True
    )
    plan = asyncio.run(client.plan(intent, dev_wallet()))
    assert plan.ok, plan.violations
    assert any("validator: 41.00% -> 30.00%" in line for line in plan.effects)
    assert any("multiplier 1.3667" in line for line in plan.effects)
    effective = ew.effective_tao_weight(TAO_WEIGHT, DEFAULT[ew.VALIDATOR], ew.parse_share("30%"))
    assert any(
        f"effective TAO weight on this subnet {effective * 100:.2f}%" in line
        for line in plan.effects
    )


def test_intent_plan_blocks_on_chain_rules():
    client, _ = _client()
    intent = SetSubnetEmissionWeights(netuid=1, slots=["owner"], values=["40%"], renormalize=True)
    plan = asyncio.run(client.plan(intent, dev_wallet()))
    assert not plan.ok
    assert plan.violations[0].startswith("OwnerEmissionAboveMaximum")

    verbatim = SetSubnetEmissionWeights(netuid=1, slots=["owner"], values=["10%"])
    plan = asyncio.run(client.plan(verbatim, dev_wallet()))
    assert [v.split(":")[0] for v in plan.violations] == ["EmissionWeightsDoNotSumToOne"]


def test_intent_build_sends_full_vector_when_renormalizing():
    _, fake = _client()
    owner = ew.parse_share("10%")
    intent = SetSubnetEmissionWeights(netuid=1, slots=["owner"], values=["10%"], renormalize=True)
    call = asyncio.run(intent.build(fake, dev_wallet()))
    assert (call.module, call.function) == ("AdminUtils", "sudo_set_subnet_emission_weights")
    params = call.params
    assert params["uids"] == [0, 1, 2, 3, 4]
    assert params["values"][2] == owner
    assert sum(params["values"]) == ONE

    sparse = SetSubnetEmissionWeights(netuid=1, slots=["miner", "owner"], values=[ONE // 2, 1])
    params = asyncio.run(sparse.build(fake, dev_wallet())).params
    assert params["uids"] == [2, 4]
    assert params["values"] == [1, ONE // 2]


# --- read -------------------------------------------------------------------


def test_read_decodes_runtime_payload():
    client, _ = _client()
    info = asyncio.run(client.read("subnet_emission_weights", netuid=1))
    assert info.weights == EMISSION_WEIGHTS_INFO["weights"]
    assert info.shares["owner"] == pytest.approx(0.18, abs=1e-4)
    assert info.tao_weight_multiplier == 1.0
    assert info.effective_tao_weight == pytest.approx(0.0527, abs=1e-4)
    assert info.rules().min_validator == EMISSION_WEIGHTS_INFO["min_validator"]
    assert not info.rules().created_via_lease


# --- CLI --------------------------------------------------------------------


@pytest.fixture()
def emission_cli(isolated_cli, monkeypatch) -> FakeSubstrate:  # noqa: F811
    from bittensor.cli import context as cli_context

    client, fake = _client()
    fake.seed_default("System", "Account", {"data": {"free": 10**18, "reserved": 0, "frozen": 0}})
    monkeypatch.setattr(cli_context, "Client", lambda network, **kwargs: client)
    return fake


def test_cli_show_prints_vector_and_rules(emission_cli):
    result = invoke("--json", "subnets", "emission", "show", "--netuid", "1")
    assert result.exit_code == 0, result.output
    record = json.loads(result.output)
    assert record["weights"] == EMISSION_WEIGHTS_INFO["weights"]
    assert record["rules"][2][0] == "2 owner"

    human = invoke("subnets", "emission", "show", "--netuid", "1")
    assert human.exit_code == 0, human.output
    assert "tao weight multiplier" in human.output
    assert "3 validator" in human.output


def test_cli_set_value_dry_run_renormalizes(emission_cli):
    result = invoke(
        "--json",
        "--dry-run",
        "subnets",
        "emission",
        "set-value",
        "--netuid",
        "1",
        "--slot",
        "owner",
        "--value",
        "10%",
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["ok"], plan
    assert plan["args"]["renormalize"] is True
    assert plan["args"]["slots"] == [2]
    assert emission_cli.submissions == []


def test_cli_increase_and_decrease_dry_run(emission_cli):
    for verb in ("increase", "decrease"):
        result = invoke(
            "--json",
            "--dry-run",
            "subnets",
            "emission",
            verb,
            "--netuid",
            "1",
            "--slot",
            "validator",
            "--by",
            "5%",
        )
        assert result.exit_code == 0, (verb, result.output)
        plan = json.loads(result.output)
        assert plan["ok"], plan
        delta = ew.parse_share("5%") if verb == "increase" else -ew.parse_share("5%")
        assert plan["args"]["values"][0] == EMISSION_WEIGHTS_INFO["validator"] + delta


def test_cli_set_full_vector_dry_run(emission_cli):
    result = invoke(
        "--json",
        "--dry-run",
        "subnets",
        "emission",
        "set",
        "--netuid",
        "1",
        "--owner",
        "0.10",
        "--validator",
        "0.45",
        "--miner",
        "0.45",
    )
    assert result.exit_code == 0, result.output
    plan = json.loads(result.output)
    assert plan["ok"], plan
    assert plan["args"]["slots"] == [0, 1, 2, 3, 4]
    assert sum(plan["args"]["values"]) == ONE


def test_cli_explains_rule_violations_before_submitting(emission_cli):
    result = invoke(
        "subnets",
        "emission",
        "set-value",
        "--netuid",
        "1",
        "--slot",
        "owner",
        "--value",
        "40%",
    )
    assert result.exit_code == 2, result.output
    assert "OwnerEmissionAboveMaximum" in result.output
    assert "may be at" in result.output and "33.00%" in result.output
    assert emission_cli.submissions == []

    too_low = invoke(
        "subnets", "emission", "set-value", "--netuid", "1", "--slot", "validator", "--value", "0"
    )
    assert too_low.exit_code == 2, too_low.output
    assert "ValidatorEmissionBelowTaoWeightFloor" in too_low.output

    pinned = invoke(
        "subnets", "emission", "set-value", "--netuid", "1", "--slot", "burn", "--value", "1%"
    )
    assert pinned.exit_code == 2, pinned.output
    assert "pinned to 0" in pinned.output


def test_cli_decrease_rejects_more_than_the_slot_holds(emission_cli):
    result = invoke(
        "subnets", "emission", "decrease", "--netuid", "1", "--slot", "owner", "--by", "50%"
    )
    assert result.exit_code == 2, result.output
    assert "cannot take" in result.output
