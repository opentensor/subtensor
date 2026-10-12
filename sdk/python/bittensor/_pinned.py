"""Reuse approved dynamic inputs while verifying an entire multisig call.

This context is only used during reconstruction followed by exact byte equality.
It is not permission to accept an imported call or to skip recipient guards.
"""

from __future__ import annotations

from contextlib import contextmanager
from contextvars import ContextVar

from ._generated.calls import Call

_inputs: ContextVar[_Inputs | None] = ContextVar("multisig_pinned_inputs", default=None)


class _Inputs:
    def __init__(self, substrate, data):
        self.substrate = substrate
        self.data = data
        self.calls = None
        self.indices = {}

    async def current(self, module, function):
        if self.calls is None:
            decoded_calls = {}

            def visit(value):
                if isinstance(value, dict):
                    if "call_module" in value and "call_function" in value:
                        key = (value["call_module"], value["call_function"])
                        args = {arg["name"]: arg["value"] for arg in value["call_args"]}
                        decoded_calls.setdefault(key, []).append(args)
                    for child in value.values():
                        visit(child)
                elif isinstance(value, list):
                    for child in value:
                        visit(child)

            visit(await self.substrate.decode_scale("Call", self.data))
            self.calls = decoded_calls
        key = (module, function)
        index = self.indices.get(key, 0)
        try:
            return self.calls[key][index]
        except (KeyError, IndexError) as error:
            raise ValueError(
                "imported multisig call does not match the staking operation"
            ) from error

    def advance(self, module, function):
        key = (module, function)
        self.indices[key] = self.indices.get(key, 0) + 1


@contextmanager
def pinned_inputs(substrate, data):
    token = _inputs.set(_Inputs(substrate, data))
    try:
        yield
    finally:
        _inputs.reset(token)


async def compose_staking_call(substrate, function, params, *, derived=None):
    """Freeze only inputs normally derived from mutable chain state.

    Explicit arguments are never overwritten. Every staking call uses this path,
    so repeated calls (including explicit limits) consume their own approved inputs.
    The executor subsequently compares the full reconstructed call, including all
    guards and origin wrappers, against the pinned bytes before permitting signing.
    """
    derived = derived or {}
    state = _inputs.get()
    approved = await state.current("SubtensorModule", function) if state is not None else None
    params = dict(params)
    for name, resolve in derived.items():
        if approved is None:
            params[name] = await resolve()
        else:
            if name not in approved:
                raise ValueError("imported multisig call is missing an approved staking input")
            params[name] = approved[name]
    call = await substrate.compose(Call("SubtensorModule", function, params))
    if state is not None:
        state.advance("SubtensorModule", function)
    return call


async def approved_staking_amount(function, parameter):
    """Read the originally resolved amount for an `all` request, if replaying."""
    state = _inputs.get()
    if state is None:
        return None
    approved = await state.current("SubtensorModule", function)
    if parameter not in approved:
        raise ValueError("imported multisig call is missing its approved staking amount")
    return int(approved[parameter])
