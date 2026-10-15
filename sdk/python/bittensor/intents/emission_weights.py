"""Subnet emission weights: how a subnet splits its alpha emission.

Each subnet carries a five-slot vector ``[burn, recycle, owner, validator,
miner]`` of ``u64 / u64::MAX`` shares. The subnet owner (or root) changes it with
``AdminUtils.sudo_set_subnet_emission_weights``, a sparse update: name the slots
you are changing and their new shares; untouched owner/validator/miner slots
keep their value. The chain enforces the rules spelled out in
:mod:`bittensor.emission_weights` and rate-limits owners to one change per week
per subnet (root is exempt; the first change on a subnet is always allowed).

Read the current vector and its bounds with the ``subnet_emission_weights``
read.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Optional

from .. import emission_weights as ew
from .._generated import calls
from .._generated import runtime_apis as api
from ..reads.subnets import root_preservation_inputs
from .base import Intent
from .registry import register

SLOTS_HELP = (
    "Slots to set, by name or uid: 0 burn, 1 recycle, 2 owner, 3 validator, 4 miner. "
    "Omitted owner/validator/miner slots keep their current share; omitted burn/recycle "
    "are zero."
)
VALUES_HELP = (
    "New share for each slot, pairwise with `slots`: a 0..1 fraction (0.41), a percentage "
    '("41%"), or raw u64 parts of 2^64-1. Rules: burn = recycle = 0; owner <= 33% '
    "(>= 18% on lease-created subnets); validator <= 50% and high enough that root's "
    "share of emission can be preserved (see `btcli subnets emission show`); miner >= 33%; "
    "all five sum to exactly 1."
)
RENORMALIZE_HELP = (
    "Keep the named slots as given and rescale the other owner/validator/miner slots "
    "proportionally so the vector sums to 1 (the miner slot absorbs rounding). Without "
    "it, the update is applied verbatim and must already sum to 1."
)


def _slot_label(slot: int) -> str:
    return f"{ew.SLOT_NAMES[slot]} (slot {slot})"


@register
@dataclass
class SetSubnetEmissionWeights(Intent):
    """Set slots of a subnet's emission weight vector (btcli ``subnets emission``).

    Dispatches ``AdminUtils.sudo_set_subnet_emission_weights`` for the named
    slots. The signer must be the subnet's owner coldkey (root may also call
    it). With ``renormalize`` the untouched owner/validator/miner slots are
    rescaled so the vector still sums to one; without it the merged vector is
    submitted as given. Changing the validator share makes the chain re-solve
    the subnet's effective TAO weight so root stakers keep exactly their
    current share of emission — the pre-sign plan shows that share and the
    resulting weight, and lists every rule the chain would reject (including
    a validator share too low to keep root whole), so a bad vector is caught
    before signing. Owners may change the vector once per week per subnet.
    """

    op = "set_subnet_emission_weights"
    signer = "coldkey"
    origin = "subnet_owner"
    verify = "subnet_emission_weights"
    wraps = (("AdminUtils", "sudo_set_subnet_emission_weights"),)

    netuid: int = field(metadata={"help": "Subnet to configure; the signer must be its owner."})
    slots: list[int | str] = field(metadata={"help": SLOTS_HELP})
    values: list[int | float | str] = field(metadata={"help": VALUES_HELP})
    renormalize: bool = field(default=False, metadata={"help": RENORMALIZE_HELP})

    def __post_init__(self):
        if len(self.slots) != len(self.values):
            raise ValueError(
                f"slots and values must pair up: got {len(self.slots)} slots and "
                f"{len(self.values)} values"
            )
        if not self.slots:
            raise ValueError("name at least one slot to set")
        self.slots = [ew.slot_index(slot) for slot in self.slots]
        if len(set(self.slots)) != len(self.slots):
            raise ValueError("each slot may appear at most once")
        self.values = [ew.parse_share(value) for value in self.values]
        for slot, value in self.updates().items():
            if slot in (ew.BURN, ew.RECYCLE) and value != 0:
                raise ValueError(
                    f"{_slot_label(slot)} must be 0 in this runtime; it is reserved for a "
                    "later upgrade"
                )

    def updates(self) -> dict[int, int]:
        return {int(slot): int(value) for slot, value in zip(self.slots, self.values)}

    async def _current(self, substrate) -> Optional[dict]:
        return await substrate.runtime_call(
            *api.SubnetInfoRuntimeApi.get_subnet_emission_weights, [self.netuid]
        )

    def _merged(self, current: Optional[dict]) -> list[int]:
        base = ew.normalize_weights(current["weights"]) if current else ew.default_weights()
        if self.renormalize:
            return ew.renormalize(base, self.updates())
        return ew.merge(base, self.updates())

    async def build(self, substrate, wallet: Any):
        if self.renormalize:
            merged = self._merged(await self._current(substrate))
            uids, values = list(range(len(merged))), merged
        else:
            uids = sorted(self.updates())
            values = [self.updates()[uid] for uid in uids]
        return await substrate.compose(
            calls.AdminUtils.sudo_set_subnet_emission_weights(self.netuid, uids, values)
        )

    async def effects(self, substrate, signer_address: str) -> list[str]:
        current = await self._current(substrate)
        if current is None:
            return [self.summary()]
        before = ew.normalize_weights(current["weights"])
        after = self._merged(current)
        lines = [self.summary()]
        for slot in ew.ADJUSTABLE_SLOTS:
            if before[slot] != after[slot]:
                lines.append(
                    f"{ew.SLOT_NAMES[slot]}: {ew.format_share(before[slot])} -> "
                    f"{ew.format_share(after[slot])}"
                )
        if before[ew.VALIDATOR] != after[ew.VALIDATOR]:
            tao_weight = ew.share_to_float(int(current["effective_tao_weight"]))
            root_tao, alpha_issuance = await root_preservation_inputs(
                lambda item, keys: substrate.query(*item, keys), self.netuid
            )
            root_share = ew.root_emission_share(
                ew.share_to_float(before[ew.VALIDATOR]), tao_weight, root_tao, alpha_issuance
            )
            solved = ew.solve_tao_weight(
                root_share, ew.share_to_float(after[ew.VALIDATOR]), root_tao, alpha_issuance
            )
            if root_tao > 0 and alpha_issuance > 0 and before[ew.VALIDATOR] > 0:
                lines.append(
                    f"root keeps {root_share * 100:.4f}% of this subnet's emission; its TAO "
                    f"weight here moves {tao_weight * 100:.2f}% -> "
                    + (f"{solved * 100:.2f}%" if solved is not None else "no weight <= 100% fits")
                )
            else:
                lines.append(
                    "no root stake or alpha issuance on this subnet yet: its TAO weight stays "
                    f"{tao_weight * 100:.2f}%"
                )
        return lines

    async def blocks(self, substrate, signer_address: str) -> list[str]:
        current = await self._current(substrate)
        if current is None:
            return [f"subnet {self.netuid} does not exist (the root subnet has no emission vector)"]
        root_tao, alpha_issuance = await root_preservation_inputs(
            lambda item, keys: substrate.query(*item, keys), self.netuid
        )
        floor = ew.validator_floor(
            ew.share_to_float(ew.normalize_weights(current["weights"])[ew.VALIDATOR]),
            ew.share_to_float(int(current["effective_tao_weight"])),
            root_tao,
            alpha_issuance,
        )
        rules = ew.Rules(
            min_validator=None if floor is None else int(floor * ew.ONE),
            min_owner=ew.DEFAULT_OWNER,
            created_via_lease=bool(current["created_via_lease"]),
        )
        try:
            merged = self._merged(current)
        except ValueError as error:
            return [str(error)]
        return [str(violation) for violation in ew.validate(merged, rules)]

    def summary(self) -> str:
        parts = ", ".join(
            f"{ew.SLOT_NAMES[slot]}={ew.format_share(value)}"
            for slot, value in self.updates().items()
        )
        mode = " (renormalizing the rest)" if self.renormalize else ""
        return f"set emission weights on netuid {self.netuid}: {parts}{mode}"
