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
    assert ew.parse_share(0.5) == ew.fraction(1, 2)
    assert ew.parse_share("41%") == ew.fraction(41, 100)
    assert ew.parse_share("0.41") == ew.fraction(41, 100)
    assert ew.parse_share(" 12.5% ") == ew.fraction(125, 1000)
    assert ew.parse_share("1e-1") == ew.fraction(1, 10)
    assert ew.parse_share(12345) == 12345
    assert ew.parse_share("12345") == 12345
    assert ew.parse_share("1.0") == ONE
    assert ew.parse_share("100%") == ONE
    assert ew.parse_share("0") == 0
    assert ew.parse_share("0%") == 0


@pytest.mark.parametrize(
    ("raw", "bound"),
    [
        ("33%", ew.MAX_OWNER),
        ("0.33", ew.MAX_OWNER),
        (0.33, ew.MAX_OWNER),
        ("50%", ew.MAX_VALIDATOR),
        ("0.5", ew.MAX_VALIDATOR),
        (0.5, ew.MAX_VALIDATOR),
        ("33%", ew.MIN_MINER),
    ],
)
def test_parse_share_lands_exactly_on_chain_bounds(raw, bound):
    # The chain's bounds are floor(ONE * n / d); every spelling of the documented
    # inclusive maxima must parse to exactly that value, never one part above.
    assert ew.parse_share(raw) == bound


@pytest.mark.parametrize(
    "pinned",
    [
        # owner at its inclusive maximum, miner absorbs the rest
        {"owner": "33%", "validator": "34%"},
        {"owner": "0.33", "validator": "0.34"},
        {"owner": 0.33, "validator": 0.34},
        # validator at its inclusive maximum
        {"owner": "17%", "validator": "50%"},
        {"owner": "0.17", "validator": "0.5"},
        # miner exactly at its inclusive minimum (validator absorbs the rounding ulps)
        {"owner": "18%", "miner": "33%"},
        {"owner": 0.18, "miner": 0.33},
    ],
)
def test_documented_bounds_pass_validation_in_every_spelling(pinned):
    rules = ew.Rules()
    weights = ew.renormalize(
        ew.default_weights(),
        {slot: ew.parse_share(value) for slot, value in pinned.items()},
    )
    assert ew.validate(weights, rules) == [], pinned


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
        "ValidatorEmissionTooLowToPreserveRoot"
    ]
    assert errors([0, 0, 0, 0, ONE]) == ["ValidatorEmissionTooLowToPreserveRoot"]
    # With no root stake there is nothing to preserve: even V = 0 passes.
    assert errors([0, 0, 0, 0, ONE], ew.Rules()) == []
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


def test_root_preservation_helpers_match_sn4_numbers():
    # Mainnet SN4-like: 5.443M TAO on root, 6.457M alpha issued, TaoWeight 18%, V = 41%.
    root_tao, issuance, tao_weight = 5_443_000e9, 6_457_000e9, 0.18
    validator = ew.share_to_float(DEFAULT[ew.VALIDATOR])
    root_share = ew.root_emission_share(validator, tao_weight, root_tao, issuance)
    assert root_share == pytest.approx(0.0540, abs=1e-4)
    floor = ew.validator_floor(validator, tao_weight, root_tao, issuance)
    assert floor == pytest.approx(0.1181, abs=1e-4)
    # Lowering V to 11.9% fits with a weight just under 1; 11.7% does not.
    solved = ew.solve_tao_weight(root_share, 0.119, root_tao, issuance)
    assert solved is not None and 0.98 < solved <= 1.0
    assert ew.root_emission_share(0.119, solved, root_tao, issuance) == pytest.approx(root_share)
    assert ew.solve_tao_weight(root_share, 0.117, root_tao, issuance) is None
    # V at or below root's share can never work; raising V lowers the weight.
    assert ew.solve_tao_weight(root_share, root_share, root_tao, issuance) is None
    assert ew.solve_tao_weight(root_share, 0.5, root_tao, issuance) < tao_weight
    # Nothing to preserve without root stake or issuance.
    assert ew.validator_floor(validator, tao_weight, 0, issuance) is None
    assert ew.validator_floor(validator, tao_weight, root_tao, 0) is None
    assert ew.root_emission_share(validator, tao_weight, 0, 0) == 0.0


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


ROOT_TAO = 5_443_000 * 10**9
ALPHA_ISSUANCE = 6_457_000 * 10**9


def _client() -> tuple[Client, FakeSubstrate]:
    fake = FakeSubstrate()
    fake.seed_runtime("SubnetInfoRuntimeApi", "get_subnet_emission_weights", EMISSION_WEIGHTS_INFO)
    fake.seed("SubtensorModule", "TaoWeight", [], TAO_WEIGHT)
    fake.seed("SubtensorModule", "SubnetTAO", [0], ROOT_TAO)
    fake.seed("SubtensorModule", "SubnetAlphaIn", [1], 0)
    fake.seed("SubtensorModule", "SubnetAlphaOut", [1], ALPHA_ISSUANCE)
    fake.seed("Swap", "BalancerAlphaReservoir", [1], 0)
    return Client("local", substrate=fake), fake


def _expected_floor() -> float:
    return ew.validator_floor(
        ew.share_to_float(DEFAULT[ew.VALIDATOR]),
        ew.share_to_float(TAO_WEIGHT),
        ROOT_TAO,
        ALPHA_ISSUANCE,
    )


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


def test_intent_plan_renormalizes_and_reports_root_preservation():
    client, _ = _client()
    intent = SetSubnetEmissionWeights(
        netuid=1, slots=["validator"], values=["30%"], renormalize=True
    )
    plan = asyncio.run(client.plan(intent, dev_wallet()))
    assert plan.ok, plan.violations
    assert any("validator: 41.00% -> 30.00%" in line for line in plan.effects)
    tao_weight = ew.share_to_float(TAO_WEIGHT)
    root_share = ew.root_emission_share(
        ew.share_to_float(DEFAULT[ew.VALIDATOR]), tao_weight, ROOT_TAO, ALPHA_ISSUANCE
    )
    solved = ew.solve_tao_weight(root_share, 0.30, ROOT_TAO, ALPHA_ISSUANCE)
    assert any(
        f"root keeps {root_share * 100:.4f}%" in line
        and f"{tao_weight * 100:.2f}% -> {solved * 100:.2f}%" in line
        for line in plan.effects
    ), plan.effects

    # Below the live floor the plan is blocked before signing.
    too_low = SetSubnetEmissionWeights(
        netuid=1, slots=["validator"], values=["3%"], renormalize=True
    )
    plan = asyncio.run(client.plan(too_low, dev_wallet()))
    assert not plan.ok
    assert plan.violations[0].startswith("ValidatorEmissionTooLowToPreserveRoot")


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
    assert info.effective_tao_weight == pytest.approx(0.0527, abs=1e-4)
    assert (info.root_tao, info.alpha_issuance) == (ROOT_TAO, ALPHA_ISSUANCE)
    assert info.root_share == pytest.approx(0.0174, abs=1e-4)
    assert info.rules().min_validator == int(_expected_floor() * ONE)
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
    assert "validator floor now" in human.output
    assert "root share of emission" in human.output
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
    assert "ValidatorEmissionTooLowToPreserveRoot" in too_low.output

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
