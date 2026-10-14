"""Per-subnet emission weight vector: slot names, rules, parsing and renormalisation.

A subnet's alpha emission is divided by a five-slot vector
``[burn, recycle, owner, validator, miner]`` whose entries are ``u64`` parts of
``u64::MAX`` (so the slots sum to exactly one). The owner or root sets it with
``AdminUtils.sudo_set_subnet_emission_weights(netuid, uids, values)`` — a
*sparse* update: every ``uids[i]`` names a slot and ``values[i]`` its new share.
Omitted owner/validator/miner slots keep their current value; omitted burn and
recycle are treated as zero. The chain then checks the v1 rules:

* burn (0) and recycle (1) must be zero — reserved for a later runtime;
* ``0 <= owner <= 0.33``; lease-created subnets may not drop below the launch
  default (18% on mainnet);
* ``V0 * TaoWeight <= validator <= 0.50`` where ``V0`` is the subnet's base
  validator share — lowering the validator share scales the subnet's TAO
  weight by ``V0 / V`` so root payouts hold, and that scaled weight may not
  exceed one (``V = 0`` is never accepted);
* ``miner >= 0.33``;
* the five slots sum to exactly one.

Everything in this module is pure (no chain access) so the CLI can preview and
explain a change before submitting it; :func:`validate` mirrors the on-chain
checks and names the same error codes.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable, Mapping, Optional, Union

U64_MAX = 2**64 - 1

#: Denominator of every emission weight (``u64::MAX``).
ONE = U64_MAX

#: Slot order on chain; the index of a name is its ``uid``.
SLOT_NAMES: tuple[str, ...] = ("burn", "recycle", "owner", "validator", "miner")
BURN, RECYCLE, OWNER, VALIDATOR, MINER = range(len(SLOT_NAMES))

#: Slots the v1 rules let an owner move (burn/recycle are pinned to zero).
ADJUSTABLE_SLOTS: tuple[int, ...] = (OWNER, VALIDATOR, MINER)

#: Fixed-point scale of ``tao_weight_multiplier`` in the runtime API and EVM
#: views (``1.0`` is reported as ``1_000_000_000``; the multiplier can exceed one).
TAO_WEIGHT_MULTIPLIER_SCALE = 1_000_000_000


def fraction(numerator: int, denominator: int) -> int:
    """``numerator / denominator`` in parts of :data:`ONE` (floor, as on chain)."""
    return (ONE * numerator) // denominator


MAX_OWNER = fraction(33, 100)
MAX_VALIDATOR = fraction(50, 100)
MIN_MINER = fraction(33, 100)

#: Mainnet launch owner cut ``11796 / 65535`` scaled to u64 parts; also the
#: floor for lease-created subnets and the base from which the default vector
#: ``[0, 0, 0.18, 0.41, 0.41]`` is derived.
DEFAULT_OWNER = (11796 * ONE) // 65535

ShareLike = Union[int, float, str]


def slot_index(slot: Union[int, str]) -> int:
    """Resolve a slot given by name (``"owner"``) or uid (``2``)."""
    if isinstance(slot, str):
        key = slot.strip().lower()
        if key.isdigit():
            return slot_index(int(key))
        if key not in SLOT_NAMES:
            raise ValueError(
                f"unknown emission slot {slot!r}; slots are "
                + ", ".join(f"{i} {name}" for i, name in enumerate(SLOT_NAMES))
            )
        return SLOT_NAMES.index(key)
    if not 0 <= slot < len(SLOT_NAMES):
        raise ValueError(
            f"emission slot {slot} is out of range; slots are "
            + ", ".join(f"{i} {name}" for i, name in enumerate(SLOT_NAMES))
        )
    return int(slot)


def parse_share(value: ShareLike) -> int:
    """Turn a human share into u64 parts.

    Accepts a 0..1 fraction (``0.41`` / ``"0.41"``), a percentage (``"41%"``),
    or a raw integer part count (``7560000000000000000``). Integers are taken
    as raw parts; anything with a decimal point or percent sign is a fraction.
    """
    if isinstance(value, bool):
        raise ValueError("emission share must be a number, not a boolean")
    if isinstance(value, int):
        parts = value
    elif isinstance(value, float):
        parts = _fraction_to_parts(value, repr(value))
    else:
        text = value.strip().replace("_", "")
        if not text:
            raise ValueError("emission share is empty")
        if text.endswith("%"):
            parts = _fraction_to_parts(float(text[:-1]) / 100, value)
        elif "." in text or "e" in text.lower():
            parts = _fraction_to_parts(float(text), value)
        else:
            try:
                parts = int(text)
            except ValueError as error:
                raise ValueError(
                    f"cannot read emission share {value!r}; give a 0..1 fraction "
                    "(0.41), a percentage (41%), or raw u64 parts"
                ) from error
    if not 0 <= parts <= ONE:
        raise ValueError(f"emission share {value!r} is outside 0..1 (0..{ONE} parts)")
    return parts


def _fraction_to_parts(fraction_value: float, shown: object) -> int:
    if not 0.0 <= fraction_value <= 1.0:
        raise ValueError(f"emission share {shown!r} must be between 0 and 1 (0% and 100%)")
    return min(ONE, round(fraction_value * ONE))


def share_to_float(parts: int) -> float:
    """u64 parts -> 0..1 float."""
    return parts / ONE


def format_share(parts: int, places: int = 2) -> str:
    """u64 parts -> ``"41.00%"``."""
    return f"{share_to_float(parts) * 100:.{places}f}%"


@dataclass(frozen=True)
class Rules:
    """The per-subnet inputs the chain rules depend on.

    ``min_validator`` is the TAO-weight floor ``V0 * TaoWeight`` (never below one
    part); ``min_owner`` is the lease floor, zero for subnets not created
    through a lease.
    """

    min_validator: int = 1
    min_owner: int = 0
    created_via_lease: bool = False
    base_validator: int = 0

    def table(self) -> list[tuple[str, str, str]]:
        """``(slot, rule, why)`` rows for help output."""
        owner_low = format_share(self.min_owner) if self.created_via_lease else "0%"
        owner_why = (
            "lease-created subnet: contributors are paid from the owner cut"
            if self.created_via_lease
            else "0 disables the owner cut"
        )
        return [
            ("0 burn", "= 0", "reserved for a future runtime"),
            ("1 recycle", "= 0", "reserved for a future runtime"),
            ("2 owner", f"{owner_low} <= owner <= {format_share(MAX_OWNER)}", owner_why),
            (
                "3 validator",
                f"{format_share(self.min_validator, 4)} <= validator <= "
                f"{format_share(MAX_VALIDATOR)}",
                "floor = base share x TaoWeight; lowering V scales TAO weight by V0/V",
            ),
            ("4 miner", f"miner >= {format_share(MIN_MINER)}", "miner incentive floor"),
            ("sum", "= 100%", f"exactly {ONE} parts"),
        ]


@dataclass(frozen=True)
class Violation:
    """One broken rule: the chain error name and an explanation."""

    error: str
    message: str

    def __str__(self) -> str:
        return f"{self.error}: {self.message}"


def default_weights(owner: int = DEFAULT_OWNER) -> list[int]:
    """``[0, 0, owner, rest/2, rest - rest/2]`` — the chain's seed vector."""
    owner = min(owner, ONE)
    validator = (ONE - owner) // 2
    return [0, 0, owner, validator, ONE - owner - validator]


def normalize_weights(weights: Iterable[int]) -> list[int]:
    """Coerce a chain-shaped vector (any length <= 5, ints or Compact dicts) to five ints."""
    out = [int(w) for w in weights]
    if len(out) > len(SLOT_NAMES):
        raise ValueError(f"emission weight vector has {len(out)} slots; at most 5 are known")
    out.extend([0] * (len(SLOT_NAMES) - len(out)))
    return out


def merge(current: Iterable[int], updates: Mapping[Union[int, str], int]) -> list[int]:
    """Apply a sparse update exactly as the chain does: named slots are
    replaced, omitted owner/validator/miner keep their value, omitted
    burn/recycle become zero."""
    merged = normalize_weights(current)
    merged[BURN] = 0
    merged[RECYCLE] = 0
    for slot, value in updates.items():
        merged[slot_index(slot)] = int(value)
    return merged


def renormalize(
    current: Iterable[int],
    fixed: Mapping[Union[int, str], int],
) -> list[int]:
    """Pin the ``fixed`` slots and rescale the other adjustable slots so the
    vector sums to one again.

    The untouched owner/validator/miner slots keep their relative proportions
    (split evenly when they are all zero); the last of them in slot order —
    normally the miner — absorbs the rounding remainder. Raises if the pinned
    shares alone exceed one, or if every adjustable slot is pinned but the
    total is not one.
    """
    pinned = {slot_index(slot): int(value) for slot, value in fixed.items()}
    for slot in (BURN, RECYCLE):
        if pinned.get(slot, 0) != 0:
            raise ValueError(f"{SLOT_NAMES[slot]} (slot {slot}) must stay 0 in this runtime")
        pinned.setdefault(slot, 0)
    base = normalize_weights(current)
    free = [slot for slot in ADJUSTABLE_SLOTS if slot not in pinned]
    remaining = ONE - sum(pinned.values())
    if remaining < 0:
        raise ValueError(
            "the pinned shares alone exceed 100%: "
            + ", ".join(f"{SLOT_NAMES[s]}={format_share(v)}" for s, v in pinned.items() if v)
        )
    out = base[:]
    for slot, value in pinned.items():
        out[slot] = value
    if not free:
        if remaining != 0:
            raise ValueError(
                f"all adjustable slots are pinned but they sum to {format_share(ONE - remaining)}, "
                "not 100%"
            )
        return out
    free_total = sum(base[slot] for slot in free)
    if free_total == 0:
        shares = [remaining // len(free)] * len(free)
    else:
        shares = [(remaining * base[slot]) // free_total for slot in free]
    shares[-1] += remaining - sum(shares)
    for slot, value in zip(free, shares):
        out[slot] = value
    return out


def validate(weights: Iterable[int], rules: Optional[Rules] = None) -> list[Violation]:
    """Every rule the chain would reject, in the order it checks them (empty
    when the vector is accepted)."""
    rules = rules or Rules()
    w = normalize_weights(weights)
    found: list[Violation] = []
    if w[BURN] != 0:
        found.append(
            Violation(
                "BurnEmissionMustBeZero",
                f"burn (slot 0) is {format_share(w[BURN])}; burning emission is reserved for a "
                "future runtime, so it must be 0 (or omitted)",
            )
        )
    if w[RECYCLE] != 0:
        found.append(
            Violation(
                "RecycleEmissionMustBeZero",
                f"recycle (slot 1) is {format_share(w[RECYCLE])}; recycling emission is reserved "
                "for a future runtime, so it must be 0 (or omitted)",
            )
        )
    if w[OWNER] > MAX_OWNER:
        found.append(
            Violation(
                "OwnerEmissionAboveMaximum",
                f"owner (slot 2) is {format_share(w[OWNER])}; the owner cut may be at most "
                f"{format_share(MAX_OWNER)}",
            )
        )
    if rules.created_via_lease and w[OWNER] < rules.min_owner:
        found.append(
            Violation(
                "LeasedSubnetOwnerCutBelowMinimum",
                f"owner (slot 2) is {format_share(w[OWNER])}; this subnet was created through a "
                f"lease, so its owner cut may not drop below {format_share(rules.min_owner)} "
                "(lease contributors are paid from it)",
            )
        )
    if w[VALIDATOR] > MAX_VALIDATOR:
        found.append(
            Violation(
                "ValidatorEmissionAboveMaximum",
                f"validator (slot 3) is {format_share(w[VALIDATOR])}; validator dividends may "
                f"draw at most {format_share(MAX_VALIDATOR)}",
            )
        )
    if w[VALIDATOR] < max(rules.min_validator, 1):
        found.append(
            Violation(
                "ValidatorEmissionBelowTaoWeightFloor",
                f"validator (slot 3) is {format_share(w[VALIDATOR], 4)}; lowering it scales this "
                "subnet's TAO weight by base/new so root payouts hold, and that may not exceed 1, "
                f"so the validator share must be at least {format_share(rules.min_validator, 4)} "
                "(and never 0)",
            )
        )
    if w[MINER] < MIN_MINER:
        found.append(
            Violation(
                "MinerEmissionBelowMinimum",
                f"miner (slot 4) is {format_share(w[MINER])}; miner incentive must receive at "
                f"least {format_share(MIN_MINER)}",
            )
        )
    total = sum(w)
    if total != ONE:
        found.append(
            Violation(
                "EmissionWeightsDoNotSumToOne",
                f"the five slots sum to {format_share(total, 6)} ({total} parts), not exactly "
                f"100% ({ONE} parts); renormalise another slot in the same call "
                "(`btcli subnets emission set-value` does this for you)",
            )
        )
    return found


def tao_weight_multiplier(base_validator: int, validator: int) -> float:
    """``V0 / V`` as the chain stores it (``inf`` when ``V`` is zero)."""
    if validator == 0:
        return float("inf")
    return base_validator / validator


def effective_tao_weight(global_tao_weight: int, base_validator: int, validator: int) -> float:
    """``min(1, TaoWeight * V0 / V)`` as a 0..1 float."""
    return min(
        1.0, share_to_float(global_tao_weight) * tao_weight_multiplier(base_validator, validator)
    )


def describe(weights: Iterable[int]) -> dict[str, str]:
    """``{slot name: "41.00%"}`` for display."""
    return {
        name: format_share(value) for name, value in zip(SLOT_NAMES, normalize_weights(weights))
    }
