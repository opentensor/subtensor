"""Block-pinned positions, lending inventories and runtime loan quotes."""

from __future__ import annotations

from typing import Any, Optional, cast

from .._generated.runtime_apis import Method
from .._generated.storage import Item
from ..balance import Balance
from ..intents._money import alpha_amount, tao_amount
from ..intents.lending import POSITIONS, check_side
from ..result import BittensorError, ChainError, chain_error_from_dispatch
from .base import read

VAULTS = Item("Lending", "Vaults", "Vault")
OPEN_BY_NETUID = Item("Lending", "OpenByNetuid")
DISSOLUTIONS = Item("Lending", "Dissolutions", "Dissolution")
QUOTE_OPEN = Method("LendingRuntimeApi", "quote_open")
QUOTE_OPEN_FOR = Method("LendingRuntimeApi", "quote_open_for")
QUOTE_OPEN_EXACT_FOR = Method("LendingRuntimeApi", "quote_open_exact_for")
QUOTE_CLOSE = Method("LendingRuntimeApi", "quote_close")


def _ok(raw: Any) -> dict:
    if not isinstance(raw, dict):
        raise BittensorError("lending quote unavailable; use a node with the lending runtime")
    if "Err" in raw:
        error = raw["Err"]
        if isinstance(error, dict) and "Module" in error and not isinstance(error["Module"], dict):
            raise BittensorError("invalid lending quote error response")
        raise chain_error_from_dispatch(error)
    result = raw.get("Ok", raw)
    if not isinstance(result, dict):
        raise BittensorError("invalid lending quote response")
    return result


async def _interest_clock(view, netuid: int) -> tuple[int, int]:
    year = int(await view.constant(("Lending", "BlocksPerYear")))
    if year <= 0:
        raise BittensorError("invalid lending interest year")
    dissolution = await view.query(DISSOLUTIONS, [netuid])
    now = min(view.block, int(dissolution["frozen_at"])) if dissolution else view.block
    return now, year


def _position_record(
    view, owner: str, netuid: int, raw: Any, clock: tuple[int, int]
) -> Optional[dict]:
    if not isinstance(raw, dict):
        return None
    variant = raw["side"]
    side = check_side(str(next(iter(variant)) if isinstance(variant, dict) else variant))
    collateral_unit = 0 if side == "Short" else netuid
    principal_unit = netuid if side == "Short" else 0
    annual = int(raw["annual_interest"])
    now, blocks_per_year = clock
    accrued_blocks = max(now - int(raw["last_accrued"]), 0)
    interest_due = (
        annual * accrued_blocks + int(raw.get("interest_remainder") or 0)
    ) // blocks_per_year
    collateral = int(raw["collateral"])
    return {
        "coldkey": owner,
        "netuid": netuid,
        "side": side,
        "hotkey": str(raw["hotkey"]),
        "principal": view.balance(int(raw["principal"]), principal_unit),
        "collateral": view.balance(collateral, collateral_unit),
        "proceeds": Balance.from_rao(int(raw["proceeds"])),
        "annual_interest": view.balance(annual, collateral_unit),
        "interest_due": view.balance(interest_due, collateral_unit),
        "last_accrued": int(raw["last_accrued"]),
        "due": int(raw["due"]),
        "runway_days": max(collateral - interest_due, 0) * 365 / annual if annual else None,
    }


@read(
    "lending_position",
    {"coldkey_ss58": "string", "netuid": "integer"},
    category="Prices & swaps",
    param_docs={"coldkey_ss58": "Position owner.", "netuid": "Subnet of the position."},
)
async def lending_position(view, coldkey_ss58: str, netuid: int) -> Optional[dict]:
    """A coldkey's fixed-principal loan on one subnet, or None.

    Principal increases when more is borrowed and never falls when interest is
    collected. This record reports the combined position. Amounts retain their
    currency: short collateral is TAO and short debt is alpha; long collateral
    is alpha and long debt is TAO. The retained ``proceeds`` compatibility field
    is zero for newly opened shorts; it is not a borrowed-alpha sale balance.
    Interest and
    remaining runway are estimates at the selected block, not a close quote.
    """
    view = await view.at()
    raw = await view.query(POSITIONS, [coldkey_ss58, netuid])
    if not isinstance(raw, dict):
        return None
    return _position_record(view, coldkey_ss58, netuid, raw, await _interest_clock(view, netuid))


@read(
    "lending_positions",
    {"netuid": "integer"},
    category="Prices & swaps",
    param_docs={"netuid": "Subnet whose open loans to list."},
)
async def lending_positions(view, netuid: int) -> list[dict]:
    """All open loans on a subnet, pinned to one block and sorted by owner."""
    view = await view.at()
    owners = await view.query_map(OPEN_BY_NETUID, [netuid])
    if not owners:
        return []
    clock = await _interest_clock(view, netuid)
    records = []
    for key, _ in owners:
        owner = str(key[0] if isinstance(key, (list, tuple)) else key)
        raw = await view.query(POSITIONS, [owner, netuid])
        record = _position_record(view, owner, netuid, raw, clock)
        if record is not None:
            records.append(record)
    return sorted(records, key=lambda record: record["coldkey"])


@read(
    "lending_reserves",
    {"netuid": "integer"},
    category="Prices & swaps",
    param_docs={"netuid": "Subnet whose separate lending vault to inspect."},
)
async def lending_reserves(view, netuid: int) -> dict:
    """Available, borrowed, lost and pending balances of the lending vault.

    The 10% cap uses available inventory plus outstanding principal in each
    asset. AMM reserves, borrower collateral, legacy escrow proceeds and pending
    conversions are excluded. This reports inventory, not a promise that
    loans will be repaid.
    """
    raw = await view.query(VAULTS, [netuid])
    raw = raw if isinstance(raw, dict) else {}
    result = {"netuid": netuid}
    for asset, unit in (("tao", 0), ("alpha", netuid)):
        for kind in ("available", "outstanding", "lost", "pending"):
            result[f"{kind}_{asset}"] = view.balance(int(raw.get(f"{kind}_{asset}") or 0), unit)
        available = int(raw.get(f"available_{asset}") or 0)
        outstanding = int(raw.get(f"outstanding_{asset}") or 0)
        headroom = max((available + outstanding) // 10 - outstanding, 0)
        result[f"borrow_headroom_{asset}"] = view.balance(headroom, unit)
    return result


@read(
    "lending_open_quote",
    {
        "netuid": "integer",
        "side": "string",
        "collateral": "string",
        "coldkey_ss58": "string",
        "hotkey_ss58": "string",
        "amount": "string",
    },
    category="Prices & swaps",
    param_docs={
        "netuid": "Subnet to borrow against.",
        "side": "short or long.",
        "collateral": "Additional collateral amount: TAO for a short, alpha for a long.",
        "coldkey_ss58": "Owner to quote an opening or increase for; supply its hotkey too.",
        "hotkey_ss58": "Alpha delivery, collateral or repayment hotkey; supply the owner too.",
        "amount": "Exact additional principal: alpha for a short, TAO for a long; optional.",
    },
)
async def lending_open_quote(
    view,
    netuid: int,
    side: str,
    collateral: str,
    coldkey_ss58: Optional[str] = None,
    hotkey_ss58: Optional[str] = None,
    amount: Optional[str] = None,
) -> dict:
    """Quote additional fixed debt, opening value and collateral coupon.

    Supply the owner and hotkey together to quote a new position or increase
    an existing position with the same side and hotkey. The runtime accrues old
    interest and checks the combined position. Alpha loans require conservative
    funded-claim coverage; TAO loans require conservative funded collateral
    backing. Both protect each other same-side loan after accrued interest.
    Returned principal, annual_interest and opening_value
    are additions, not totals. Short opening_value is the loan's TAO value at
    the lending EMA, rounded up, and fixes its annual TAO coupon. A simulated
    buy limits alpha principal but opening makes no AMM swap and pays no sale
    proceeds. Omitting both addresses quotes a fresh loan only.
    A refusal is an error; it never becomes a zero-protection quote. The result
    is indicative until the transaction executes. Supply ``amount`` with both
    addresses to quote exactly that additional principal instead of the
    maximum supported by the collateral.
    """
    view = await view.at()
    side = check_side(side)
    collateral_amount = cast(
        Balance, tao_amount(collateral) if side == "Short" else alpha_amount(collateral, netuid)
    )
    if (coldkey_ss58 is None) != (hotkey_ss58 is None):
        raise BittensorError("an owner-aware lending quote requires both coldkey and hotkey")
    method = QUOTE_OPEN if coldkey_ss58 is None else QUOTE_OPEN_FOR
    params = (
        [netuid, side, collateral_amount.rao]
        if coldkey_ss58 is None
        else [coldkey_ss58, netuid, side, collateral_amount.rao, hotkey_ss58]
    )
    if amount is not None:
        if coldkey_ss58 is None or hotkey_ss58 is None:
            raise BittensorError("an exact lending quote requires both coldkey and hotkey")
        principal = cast(
            Balance, alpha_amount(amount, netuid) if side == "Short" else tao_amount(amount)
        )
        if not 0 < principal.rao <= (1 << 64) - 1:
            raise BittensorError("borrow amount must be positive and fit the chain's token units")
        method = QUOTE_OPEN_EXACT_FOR
        params = [
            coldkey_ss58,
            netuid,
            side,
            collateral_amount.rao,
            hotkey_ss58,
            principal.rao,
        ]
    raw = _ok(await view.runtime(method, params))
    if amount is not None and int(raw["principal"]) != principal.rao:
        raise BittensorError("exact lending quote returned a different borrowed amount")
    collateral_unit = 0 if side == "Short" else netuid
    return {
        "principal": view.balance(int(raw["principal"]), netuid if side == "Short" else 0),
        "annual_interest": view.balance(int(raw["annual_interest"]), collateral_unit),
        "opening_value": Balance.from_rao(int(raw["opening_value"])),
    }


@read(
    "lending_required_collateral",
    {
        "netuid": "integer",
        "side": "string",
        "amount": "string",
        "coldkey_ss58": "string",
        "hotkey_ss58": "string",
    },
    category="Prices & swaps",
    param_docs={
        "netuid": "Subnet to borrow against.",
        "side": "short or long.",
        "amount": "Exact additional principal: alpha for a short, TAO for a long.",
        "coldkey_ss58": "Owner of the new or increased loan.",
        "hotkey_ss58": "Delivery or collateral hotkey; must match an existing loan.",
    },
)
async def lending_required_collateral(
    view,
    netuid: int,
    side: str,
    amount: str,
    coldkey_ss58: str,
    hotkey_ss58: str,
) -> dict:
    """Minimum additional collateral for exactly the requested borrowed amount.

    The search uses exact-principal runtime quotes at one block. It includes
    existing principal, accrued interest, market depth and funded-redemption
    guards. Only the owner's ``InsufficientCollateral`` response means to
    search higher; unavailable references, inventory, borrowing caps and
    other positions' uncovered debt remain hard refusals. The requested
    opening value must meet the runtime's minimum, currently 0.1 TAO.

    Returns collateral and its opening quote. This read does not promise
    that the wallet owns the collateral or can pay transaction fees. Wallet
    affordability is checked before review and signing. The exact-principal
    transaction rechecks admission without borrowing extra if state changes.
    """
    view = await view.at()
    side = check_side(side)
    principal = cast(
        Balance, alpha_amount(amount, netuid) if side == "Short" else tao_amount(amount)
    )
    if not 0 < principal.rao <= (1 << 64) - 1:
        raise BittensorError("borrow amount must be positive and fit the chain's token units")
    unit = 0 if side == "Short" else netuid
    maximum = (1 << 64) - 1
    existing = await lending_position(view, coldkey_ss58, netuid)
    if existing is not None:
        remaining = existing["collateral"].rao - existing["interest_due"].rao
        if remaining <= 0:
            raise BittensorError("the existing position's collateral is exhausted")
        maximum -= remaining
    if maximum <= 0:
        raise BittensorError("the position cannot accept additional collateral")

    quotes: dict[int, Optional[dict]] = {}

    async def quote(collateral: int) -> Optional[dict]:
        if collateral not in quotes:
            try:
                quotes[collateral] = await lending_open_quote(
                    view,
                    netuid,
                    side,
                    view.balance(collateral, unit),
                    coldkey_ss58,
                    hotkey_ss58,
                    principal,
                )
            except ChainError as error:
                if error.name != "InsufficientCollateral":
                    raise
                quotes[collateral] = None
        return quotes[collateral]

    low, high = 0, min(1_000_000_000, maximum)
    for _ in range(64):
        if await quote(high) is not None:
            break
        if high == maximum:
            raise BittensorError("the required collateral exceeds the chain's position limit")
        low, high = high, min(high * 2, maximum)
    else:
        raise BittensorError("could not bound the required collateral")

    for _ in range(64):
        if high - low <= 1:
            break
        middle = (low + high) // 2
        if await quote(middle) is None:
            low = middle
        else:
            high = middle
    result = await quote(high)
    if result is None:
        raise BittensorError("the requested loan has no admissible collateral quote")
    return {"collateral": view.balance(high, unit), **result}


@read(
    "lending_close_quote",
    {"coldkey_ss58": "string", "netuid": "integer", "repay_from_wallet": "boolean"},
    category="Prices & swaps",
    param_docs={
        "coldkey_ss58": "Position owner.",
        "netuid": "Subnet of the position.",
        "repay_from_wallet": (
            "Repay short alpha from its saved hotkey (default); false buys with collateral."
        ),
    },
)
async def lending_close_quote(
    view, coldkey_ss58: str, netuid: int, repay_from_wallet: bool = True
) -> dict:
    """Runtime quote of the combined debt repayment and refund, after accrued interest.

    Payment is TAO except for a wallet-repaid short, which returns its fixed
    alpha principal. Refund is TAO for a short and alpha for a long. A short
    AMM buyback must fit within the curve's remaining buy range and remaining
    TAO collateral after interest. Borrowed alpha need not have been sold.
    """
    view = await view.at()
    position = await lending_position(view, coldkey_ss58, netuid)
    if position is None:
        raise BittensorError(f"no lending position on netuid {netuid}")
    raw = _ok(await view.runtime(QUOTE_CLOSE, [coldkey_ss58, netuid, repay_from_wallet]))
    short = position["side"] == "Short"
    return {
        "payment": view.balance(int(raw["payment"]), netuid if short and repay_from_wallet else 0),
        "refund": view.balance(int(raw["refund"]), 0 if short else netuid),
    }
