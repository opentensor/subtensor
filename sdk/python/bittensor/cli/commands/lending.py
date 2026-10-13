"""Three commands for fixed-principal subnet loans: open, close and list."""

from __future__ import annotations

from decimal import ROUND_CEILING, Decimal
from enum import Enum
from typing import Optional, cast

import typer

from ...balance import Balance
from ...intents import CloseLoan, OpenLoan
from ...intents._money import alpha_amount, tao_amount
from ...result import BittensorError
from ...settings import guide_docs_url
from ..context import address_cli_name, ctx_of, ss58_param_help
from ..globals import with_globals, with_tx_globals
from ..prompt import PromptSpec, ask, confirm_wallet, fill_missing

Side = Enum("Side", [("short", "short"), ("long", "long")], type=str)

app = typer.Typer(
    no_args_is_help=True,
    help=f"Subnet reserve loans: open, close, list.\n\nGuide: {guide_docs_url('pool-lending')}",
)


def _bound(amount: Balance, percent: float, *, ceiling: bool = False) -> Balance:
    """Integer quote protection with ceilings rounded up and floors down."""
    factor = Decimal(1) + Decimal(str(percent)) / 100 * (1 if ceiling else -1)
    raw = Decimal(amount.rao) * factor
    rao = int(raw.to_integral_value(rounding=ROUND_CEILING)) if ceiling else int(raw)
    return Balance.from_rao(max(rao, 0), amount.netuid)


def _loan_input(context, netuid: int, side: str) -> tuple[Optional[str], Optional[str]]:
    """Choose one input mode and record only its actual CLI flag for replay."""

    def choose_mode(_ctx, value):
        choice = value.lower()
        if choice in ("1", "amount", "borrow"):
            return "amount"
        if choice in ("2", "collateral"):
            return "collateral"
        raise ValueError("choose amount (1) or collateral (2)")

    def choose(console, app_ctx, answers):
        mode, _raw = ask(
            console,
            app_ctx,
            PromptSpec(
                "mode",
                "loan input",
                "1: desired amount to borrow; 2: collateral to provide.",
                choose_mode,
                default="amount",
            ),
        )
        debt = "TAO" if side == "long" else f"subnet {netuid} alpha"
        collateral_unit = "TAO" if side == "short" else f"subnet {netuid} alpha"
        unit = debt if mode == "amount" else collateral_unit

        def parse(_ctx, value):
            try:
                balance = cast(
                    Balance, tao_amount(value) if unit == "TAO" else alpha_amount(value, netuid)
                )
                if balance.rao <= 0:
                    raise BittensorError(f"{mode} must be greater than zero")
                return balance
            except BittensorError as error:
                raise ValueError(str(error)) from error

        value, raw = ask(
            console,
            app_ctx,
            PromptSpec(
                mode,
                f"--{mode}",
                (
                    f"Additional {unit} to borrow; btcli finds the minimum required collateral."
                    if mode == "amount"
                    else f"Additional {unit} to lock; btcli quotes your borrowing limit."
                ),
                parse,
            ),
        )
        answers[mode] = format(value.decimal, "f")
        return [f"--{mode}", raw]

    answers = {"amount": None, "collateral": None}
    fill_missing(
        context,
        [PromptSpec("loan_input", "--amount or --collateral", None, choose_mode, custom=choose)],
        answers,
    )
    return answers["amount"], answers["collateral"]


def _loan_owner(context, proxy: Optional[str]) -> Optional[str]:
    if proxy is not None:
        return proxy
    multisig = context._saved_multisig_address(context.multisig_wallet_name or context.wallet_name)
    if multisig is not None:
        return multisig
    if not context.uses_external_signer():
        return context.review_account()
    address = context.external_signer_address()
    if address is not None and not context.uses_extension_signer():
        return address

    async def select(_client):
        signer = await context.resolve_signing_wallet("coldkey")
        try:
            return signer.ss58_address
        finally:
            if hasattr(signer, "close"):
                await signer.close()

    return context.run(select)


@app.command("open")
@with_tx_globals
def open_loan(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", min=1, max=65535, help="Subnet to borrow on."),
    side: Side = typer.Option(..., "--side", help="short: TAO collateral; long: alpha collateral."),
    collateral: Optional[str] = typer.Option(
        None, "--collateral", help=OpenLoan.field_help("collateral")
    ),
    amount: Optional[str] = typer.Option(None, "--amount", help=OpenLoan.field_help("amount")),
    hotkey_ss58: Optional[str] = typer.Option(
        None, address_cli_name("hotkey_ss58"), help=OpenLoan.field_help("hotkey_ss58")
    ),
    max_slippage: float = typer.Option(
        1.0,
        "--max-slippage",
        min=0.0,
        max=99.0,
        help="Maximum fall in collateral-mode principal or short opening value (percent).",
    ),
):
    """Add collateral and open or increase a short or long on a subnet.

    Pass --amount for the additional quantity to borrow (alpha for a short,
    TAO for a long) and btcli finds the minimum required collateral. Pass
    --collateral instead to borrow its quoted limit. The minimum additional
    loan has a 0.1 TAO opening value.

    Shorts receive freely usable alpha on the selected hotkey; opening does
    not sell it. Longs borrow transferable TAO against existing alpha stake.
    An existing position must
    use the same side and hotkey. btcli quotes additional debt after checking
    the combined position and refuses to submit when a quote is unavailable.
    """
    context = ctx_of(ctx)
    if amount is not None and collateral is not None:
        context.output.error("--amount and --collateral are mutually exclusive; provide one")
        raise typer.Exit(2)
    if amount is None and collateral is None:
        amount, collateral = _loan_input(context, netuid, side.value)
    confirm_wallet(context, help_text="Wallet whose coldkey owns this loan.")
    proxy = context.resolve_dispatch_proxy()
    owner = _loan_owner(context, proxy)
    if owner is None:
        context.output.error("select the account that owns this loan before borrowing")
        raise typer.Exit(2)
    hotkey = context.resolve_address("hotkey_ss58", hotkey_ss58)
    requested = None
    if amount is not None:
        requested = cast(
            Balance, tao_amount(amount) if side == Side.long else alpha_amount(amount, netuid)
        )
        if requested.rao <= 0:
            context.output.error("amount to borrow must be greater than zero")
            raise typer.Exit(2)
        with context.output.activity("finding the minimum required collateral…"):
            required = context.run(
                lambda client: client.read(
                    "lending_required_collateral",
                    netuid=netuid,
                    side=side.value,
                    amount=requested,
                    coldkey_ss58=owner,
                    hotkey_ss58=hotkey,
                )
            )
        collateral = required["collateral"]
    if collateral is None:
        context.output.error("provide --amount or --collateral")
        raise typer.Exit(2)
    intent = OpenLoan(
        netuid=netuid,
        side=side.value,
        collateral=collateral,
        hotkey_ss58=hotkey,
        amount=requested,
    )
    with context.output.activity("quoting the loan…"):
        quote = context.run(
            lambda client: client.read(
                "lending_open_quote",
                netuid=netuid,
                side=side.value,
                collateral=intent.collateral,
                coldkey_ss58=owner,
                hotkey_ss58=hotkey,
                **({"amount": requested} if requested is not None else {}),
            )
        )
    # Amount mode never turns the desired principal into a floor that permits
    # borrowing more. The exact-principal dispatch transfers that quantity.
    intent.min_borrow = (
        requested if requested is not None else _bound(quote["principal"], max_slippage)
    )
    if side == Side.short:
        intent.min_proceeds = _bound(quote["opening_value"], max_slippage)
    context.submit(
        intent,
        proxy_for=proxy or "self",
        card_sections=[
            (
                "Loan",
                [
                    ("loan owner", owner),
                    ("hotkey", hotkey),
                    (
                        "minimum required collateral"
                        if requested is not None
                        else "provided collateral",
                        str(intent.collateral),
                    ),
                    (
                        "exact amount to borrow" if requested is not None else "borrowing limit",
                        str(quote["principal"]),
                    ),
                    ("minimum additional principal", str(intent.min_borrow)),
                    ("minimum additional opening value", str(intent.min_proceeds)),
                    ("additional interest per year", str(quote["annual_interest"])),
                ],
            )
        ],
    )


@app.command("close")
@with_tx_globals
def close_loan(
    ctx: typer.Context,
    netuid: int = typer.Option(..., "--netuid", min=1, max=65535, help="Subnet of your position."),
    repay_from_wallet: bool = typer.Option(
        True,
        "--repay-from-wallet/--no-repay-from-wallet",
        help="Repay short alpha from its saved hotkey; disable to buy with remaining collateral.",
    ),
    max_slippage: float = typer.Option(
        1.0,
        "--max-slippage",
        min=0.0,
        max=99.0,
        help="Maximum percentage above quoted payment or below quoted refund (default 1%).",
    ),
):
    """Repay fixed principal and return remaining collateral.

    A short normally supplies alpha from its saved hotkey. Opt out to buy
    that alpha using only remaining TAO collateral. A long repays
    TAO from your wallet. The full runtime quote sets a payment ceiling and
    refund floor; a failed quote stops submission.
    """
    context = ctx_of(ctx)
    owner = context.review_account()
    if owner is None:
        context.output.error("select the account that owns this position before closing")
        raise typer.Exit(2)
    with context.output.activity("quoting the repayment…"):
        quote = context.run(
            lambda client: client.read(
                "lending_close_quote",
                coldkey_ss58=owner,
                netuid=netuid,
                repay_from_wallet=repay_from_wallet,
            )
        )
    intent = CloseLoan(
        netuid=netuid,
        repay_from_wallet=repay_from_wallet,
        max_payment=_bound(quote["payment"], max_slippage, ceiling=True),
        min_refund=_bound(quote["refund"], max_slippage),
    )
    context.submit(
        intent,
        card_sections=[
            (
                "Repayment",
                [
                    ("quoted payment", str(quote["payment"])),
                    ("maximum payment", str(intent.max_payment)),
                    ("quoted refund", str(quote["refund"])),
                    ("minimum refund", str(intent.min_refund)),
                ],
            )
        ],
    )


@app.command("list")
@with_globals
def list_positions(
    ctx: typer.Context,
    netuid: int = typer.Option(
        ..., "--netuid", min=1, max=65535, help="Subnet whose loans to list."
    ),
    coldkey_ss58: Optional[str] = typer.Option(
        None, address_cli_name("coldkey_ss58"), help=ss58_param_help("coldkey_ss58")
    ),
):
    """List open loans on a subnet; optionally select one coldkey."""
    context = ctx_of(ctx)
    owner = (
        context.resolve_address("coldkey_ss58", coldkey_ss58) if coldkey_ss58 is not None else None
    )

    async def _positions(client):
        if owner is None:
            return await client.read("lending_positions", netuid=netuid)
        position = await client.read("lending_position", coldkey_ss58=owner, netuid=netuid)
        return [position] if position is not None else []

    positions = context.run(_positions)
    context.output.table(
        f"loans on netuid {netuid}",
        ["owner", "side", "principal", "collateral", "interest due", "runway"],
        [
            [
                position["coldkey"],
                position["side"],
                str(position["principal"]),
                str(position["collateral"]),
                str(position["interest_due"]),
                f"{position['runway_days']:.0f}d" if position["runway_days"] is not None else "-",
            ]
            for position in positions
        ],
        positions,
    )
