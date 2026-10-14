"""`btcli subnets emission`: view and change how a subnet splits its alpha emission.

The vector is ``[burn, recycle, owner, validator, miner]``. ``show`` prints it
with the rules that bound it; ``set`` writes a full vector; ``set-value``,
``increase`` and ``decrease`` change one slot and renormalise the rest so the
vector still sums to one. Every write validates against the chain's rules
locally first and explains each broken rule in the same words the chain
error carries, so a rejected change teaches the constraint.
"""

from __future__ import annotations

from typing import Any, Optional

import typer

from ... import emission_weights as ew
from ...intents import SetSubnetEmissionWeights
from ...reads.subnets import SubnetEmissionWeights
from ...settings import guide_docs_url
from ..context import AppContext, ctx_of
from ..globals import with_globals, with_tx_globals

app = typer.Typer(
    no_args_is_help=True,
    help=(
        "Subnet emission weights: the owner / validator / miner split of alpha emission.\n\n"
        f"Guide: {guide_docs_url('subnets')}"
    ),
)

_NETUID_HELP = "Numeric identifier of the subnet."
_SLOT_HELP = "Slot to change, by name (owner, validator, miner) or uid (2, 3, 4)."
_SHARE_HELP = "Share as a 0..1 fraction (0.41), a percentage (41%), or raw u64 parts."


def _fetch(app_ctx: AppContext, netuid: int) -> SubnetEmissionWeights:
    info = app_ctx.run(lambda c: c.read("subnet_emission_weights", netuid=netuid))
    if info is None:
        app_ctx.output.error(
            f"subnet {netuid} does not exist or has no emission vector",
            help="the root subnet (netuid 0) has no alpha emission to divide",
        )
        raise typer.Exit(1)
    return info


def _json_record(info: SubnetEmissionWeights, extra: Optional[dict[str, Any]] = None) -> dict:
    record: dict[str, Any] = {
        "netuid": info.netuid,
        "weights": info.weights,
        "shares": info.shares,
        "base_validator": info.base_validator,
        "min_validator": info.min_validator,
        "tao_weight_multiplier": info.tao_weight_multiplier,
        "effective_tao_weight": info.effective_tao_weight,
        "created_via_lease": info.created_via_lease,
    }
    if extra:
        record.update(extra)
    return record


def _rule_rows(rules: ew.Rules) -> list[tuple[str, str, Optional[str]]]:
    return [(slot, rule, why) for slot, rule, why in rules.table()]


def _parse_share(app_ctx: AppContext, raw: str, flag: str) -> int:
    try:
        return ew.parse_share(raw)
    except ValueError as error:
        app_ctx.output.error(f"invalid value for `{flag}`: {error}", help=_SHARE_HELP)
        raise typer.Exit(2) from error


def _parse_slot(app_ctx: AppContext, raw: str) -> int:
    try:
        slot = ew.slot_index(raw)
    except ValueError as error:
        app_ctx.output.error(str(error))
        raise typer.Exit(2) from error
    if slot not in ew.ADJUSTABLE_SLOTS:
        app_ctx.output.error(
            f"{ew.SLOT_NAMES[slot]} (slot {slot}) is pinned to 0 in this runtime",
            help="only owner, validator and miner can be changed",
        )
        raise typer.Exit(2)
    return slot


def _explain_and_exit(
    app_ctx: AppContext, info: SubnetEmissionWeights, proposed: list[int]
) -> None:
    """Print every rule the proposed vector breaks (with the rule table) and exit 2."""
    violations = ew.validate(proposed, info.rules())
    if not violations:
        return
    app_ctx.output.error(
        "the chain would reject this emission vector:\n"
        + "\n".join(f"  - {violation}" for violation in violations),
        help=(
            f"`btcli subnets emission show --netuid {info.netuid}` prints the current vector "
            "and the rule table; set-value/increase/decrease renormalise the other slots for you"
        ),
    )
    raise typer.Exit(2)


def _submit(
    app_ctx: AppContext,
    info: SubnetEmissionWeights,
    proposed: list[int],
    *,
    slots: list[int],
    renormalize: bool,
) -> None:
    _explain_and_exit(app_ctx, info, proposed)
    intent = SetSubnetEmissionWeights(
        netuid=info.netuid,
        slots=list(slots),
        values=[proposed[slot] for slot in slots],
        renormalize=renormalize,
    )
    multiplier = ew.tao_weight_multiplier(info.base_validator, proposed[ew.VALIDATOR])
    rows: list[tuple] = []
    for slot in ew.ADJUSTABLE_SLOTS:
        value = ew.format_share(proposed[slot])
        if proposed[slot] != info.weights[slot]:
            value = f"{ew.format_share(info.weights[slot])} -> {value}"
        rows.append((ew.SLOT_NAMES[slot], value))
    rows.append(("tao weight multiplier", f"{multiplier:.4f}"))
    rows.append(
        (
            "note",
            f"multiplier = base validator share {ew.format_share(info.base_validator)} / "
            "validator share; it scales the global TaoWeight on this subnet",
            "dim",
        )
    )
    app_ctx.submit(intent, card_sections=[("Emission weights", rows)])


@app.command("show")
@with_globals
def show(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", help=_NETUID_HELP),
):
    """Show the emission weight vector, its TAO-weight state, and the rules.

    The rules table is what any change must satisfy; `set-value`, `increase`
    and `decrease` check it locally before submitting.
    """
    app_ctx: AppContext = ctx_of(ctx)
    info = _fetch(app_ctx, netuid)
    vector_rows: list[tuple[str, str, Optional[str]]] = [
        (f"{i} {name}", ew.format_share(info.weights[i]), f"{info.weights[i]} parts")
        for i, name in enumerate(ew.SLOT_NAMES)
    ]
    tao_rows: list[tuple[str, str, Optional[str]]] = [
        (
            "base validator share",
            ew.format_share(info.base_validator),
            "V0: the share the multiplier is measured against",
        ),
        (
            "tao weight multiplier",
            f"{info.tao_weight_multiplier:.4f}",
            "V0 / validator, applied to the global TaoWeight on this subnet",
        ),
        (
            "effective tao weight",
            f"{info.effective_tao_weight * 100:.2f}%",
            "root TAO vs alpha in root dividends and consensus stake",
        ),
        ("created via lease", "yes" if info.created_via_lease else "no", None),
    ]
    app_ctx.output.kv_sections(
        f"subnet {netuid} emission weights",
        [
            ("Vector", vector_rows),
            ("TAO weight", tao_rows),
            ("Rules", _rule_rows(info.rules())),
        ],
        _json_record(info, {"rules": [list(row) for row in info.rules().table()]}),
        hint=(
            "change one slot with `btcli subnets emission set-value --netuid "
            f"{netuid} --slot owner --value 0.10` (the rest renormalises)"
        ),
    )


@app.command("set")
@with_tx_globals
def set_weights(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", help=_NETUID_HELP),
    owner: str = typer.Option(..., "--owner", help=f"Owner share. {_SHARE_HELP}"),
    validator: str = typer.Option(..., "--validator", help=f"Validator share. {_SHARE_HELP}"),
    miner: str = typer.Option(..., "--miner", help=f"Miner share. {_SHARE_HELP}"),
):
    """Set the full owner / validator / miner split (burn and recycle stay 0).

    The three shares must sum to exactly 100%; a rounding gap of a few parts
    is absorbed by the miner slot. Only the subnet owner coldkey (or root)
    can sign this, once per week per subnet.
    """
    app_ctx: AppContext = ctx_of(ctx)
    info = _fetch(app_ctx, netuid)
    owner_parts = _parse_share(app_ctx, owner, "--owner")
    validator_parts = _parse_share(app_ctx, validator, "--validator")
    miner_parts = _parse_share(app_ctx, miner, "--miner")
    total = owner_parts + validator_parts + miner_parts
    if 0 < abs(ew.ONE - total) <= 1_000:
        # Decimal shares rarely hit u64::MAX exactly; let the miner slot soak up the dust.
        miner_parts += ew.ONE - total
    proposed = [0, 0, owner_parts, validator_parts, miner_parts]
    _submit(
        app_ctx,
        info,
        proposed,
        slots=list(range(len(ew.SLOT_NAMES))),
        renormalize=False,
    )


@app.command("set-value")
@with_tx_globals
def set_value(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", help=_NETUID_HELP),
    slot: str = typer.Option(..., "--slot", help=_SLOT_HELP),
    value: str = typer.Option(..., "--value", help=f"New share for the slot. {_SHARE_HELP}"),
):
    """Set one slot and renormalise the other two so the vector sums to 100%.

    The untouched slots keep their relative proportions (e.g. lowering the
    owner share from 18% to 10% hands the 8% to validators and miners in
    their current 41:41 ratio).
    """
    app_ctx: AppContext = ctx_of(ctx)
    info = _fetch(app_ctx, netuid)
    index = _parse_slot(app_ctx, slot)
    parts = _parse_share(app_ctx, value, "--value")
    _renormalized_submit(app_ctx, info, index, parts)


@app.command("increase")
@with_tx_globals
def increase(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", help=_NETUID_HELP),
    slot: str = typer.Option(..., "--slot", help=_SLOT_HELP),
    by: str = typer.Option(..., "--by", help=f"Amount to add to the slot. {_SHARE_HELP}"),
):
    """Raise one slot by an amount and renormalise the other two."""
    app_ctx: AppContext = ctx_of(ctx)
    info = _fetch(app_ctx, netuid)
    index = _parse_slot(app_ctx, slot)
    delta = _parse_share(app_ctx, by, "--by")
    _renormalized_submit(app_ctx, info, index, min(ew.ONE, info.weights[index] + delta))


@app.command("decrease")
@with_tx_globals
def decrease(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", help=_NETUID_HELP),
    slot: str = typer.Option(..., "--slot", help=_SLOT_HELP),
    by: str = typer.Option(..., "--by", help=f"Amount to take from the slot. {_SHARE_HELP}"),
):
    """Lower one slot by an amount and renormalise the other two."""
    app_ctx: AppContext = ctx_of(ctx)
    info = _fetch(app_ctx, netuid)
    index = _parse_slot(app_ctx, slot)
    delta = _parse_share(app_ctx, by, "--by")
    if delta > info.weights[index]:
        app_ctx.output.error(
            f"{ew.SLOT_NAMES[index]} is only {ew.format_share(info.weights[index])}; cannot "
            f"take {ew.format_share(delta)} from it"
        )
        raise typer.Exit(2)
    _renormalized_submit(app_ctx, info, index, info.weights[index] - delta)


def _renormalized_submit(
    app_ctx: AppContext, info: SubnetEmissionWeights, index: int, parts: int
) -> None:
    try:
        proposed = ew.renormalize(info.weights, {index: parts})
    except ValueError as error:
        app_ctx.output.error(str(error))
        raise typer.Exit(2) from error
    _submit(app_ctx, info, proposed, slots=[index], renormalize=True)
