"""ML-DSA transport and receiving boundaries with real native wallet primitives."""

from hashlib import blake2b
from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest

from bittensor._generated import calls
from bittensor._transport.errors import SubstrateRequestException
from bittensor._transport.extrinsics import hashed_proof_length, prepare_extrinsic
from bittensor._transport.interface import SubstrateConnection
from bittensor.evm.addresses import (
    resolve_evm_funding_recipient,
    resolve_evm_recipient,
    ss58_to_h160_truncated,
)
from bittensor.hashed import (
    descriptor_bytes,
    descriptor_storage,
    descriptor_value,
    prepare_recipient_intent,
    protected_account_record,
    registration_descriptor_value,
    with_recipient_registration,
)
from bittensor.intents import Transfer
from bittensor.receiving import account_for_registered_write, parse_recipient, receiving_address
from bittensor.sp_core import CRYPTO_MLDSA65, Keypair, account_crypto_type
from tests.harness.fake_substrate import FakeSubstrate
from tests.harness.samples import dev_wallet

GENESIS = "0x" + "11" * 32


def key(crypto=CRYPTO_MLDSA65):
    return Keypair.create_from_seed(bytes([7]) * 32, crypto)


@pytest.fixture(params=[7, 8])
def wallet(request):
    return key(request.param)


def record(wallet, generation=7):
    return {
        "descriptor": descriptor_value(bytes(wallet.hashed_descriptor)),
        "generation": generation,
        "commitment": bytes(wallet.at_generation(generation).hashed_current_commitment),
    }


def chain():
    substrate = FakeSubstrate()
    substrate.seed_constant("HashedAccounts", "Enabled", True)
    substrate.seed_constant("HashedAccounts", "MldsaAccountProfiles", "0x0405")
    substrate.seed_constant("HashedAccounts", "RegistrationDeposit", 100)
    substrate.seed_constant("HashedAccounts", "V2RegistrationDeposit", 200)
    substrate.seed("HashedAccounts", "V2RegistrationEnabled", [], True)
    substrate.seed("HashedAccounts", "V2AuthorizationEnabled", [], True)
    return substrate


class Codec:
    """Observe exact transport inputs; native codec tests prove SCALE encoding."""

    def constant(self, pallet, name):
        assert pallet == "HashedAccounts"
        if name == "MldsaAccountProfiles":
            return "0x0405"
        if name == "Enabled":
            return True
        assert name == "V2RegistrationDeposit"
        return 200

    @staticmethod
    def call_data(call):
        return bytes(call)

    @staticmethod
    def signature_payload_parts(call, *, nonce, **_kwargs):
        return bytes(call), nonce.to_bytes(4, "little"), b"runtime-genesis-era"

    def encode_signed_extrinsic(
        self, call, *, public_key, signature, signature_version, nonce, **_kwargs
    ):
        assert signature_version in (4, 6, 7, 8)
        self.proof = bytes(signature)
        self.nonce = nonce
        self.account = bytes(public_key)
        self.call = bytes(call)
        body = bytes(call) + bytes(signature)
        return body, "0x" + blake2b(body, digest_size=32).hexdigest()


def connection(wallet):
    codec = Codec()
    conn = SubstrateConnection("ws://unused.invalid")
    conn._runtimes = SimpleNamespace(codec_at=AsyncMock(return_value=codec))
    conn.get_chain_head = AsyncMock(side_effect=AssertionError("must use finalized authority"))
    conn.get_chain_finalised_head = AsyncMock(return_value="0xfinalized")
    conn.genesis_hash = AsyncMock(return_value=GENESIS)
    conn._nonces = SimpleNamespace(next_for=AsyncMock(return_value=13))
    conn._normalize_era = AsyncMock(return_value=("00", GENESIS))
    state = record(wallet)

    async def query(pallet, item, params=None, *, block_hash):
        assert block_hash == "0xfinalized"
        if item == "V2AuthorizationEnabled":
            return True
        assert params == [wallet.ss58_address]
        if pallet == "HashedAccounts" and item == (
            "V2Records" if wallet.crypto_type in (7, 8) else "Accounts"
        ):
            return state
        assert (pallet, item) == ("System", "Account")
        return {"nonce": 13}

    conn.query = AsyncMock(side_effect=query)
    return conn, codec, state


async def test_finalized_number_never_falls_back_to_best_head():
    conn = SubstrateConnection("ws://unused.invalid")
    conn.get_chain_finalised_head = AsyncMock(return_value=None)
    conn.get_block_number = AsyncMock(side_effect=AssertionError("must not read best head"))
    with pytest.raises(SubstrateRequestException, match="finalized block"):
        await conn.get_finalized_block_number()
    conn.get_block_number.assert_not_called()


def test_native_aliases_keep_identity_and_variable_width_authorization(wallet):
    assert wallet.crypto_type in (7, 8)
    assert len(wallet.public_key) == 32
    assert len(wallet.hashed_descriptor) == 66
    assert len(wallet.hashed_public_key) == 1952
    assert len(wallet.hashed_current_commitment) == 64
    assert len(wallet.sign_hashed(b"full payload")) == hashed_proof_length(7) == 5367
    assert wallet.account_type == ("hashed" if wallet.crypto_type == 7 else "standard")
    assert wallet.signing_scheme == "mldsa65"
    assert account_crypto_type(7, "hashed") == 7
    assert account_crypto_type(7, "standard") == 8
    assert account_crypto_type(8, "hashed") == 7
    with pytest.raises(ValueError, match="capability"):
        wallet.sign(b"cannot silently reuse spending key")
    with pytest.raises(ValueError, match="generation synchronization"):
        prepare_extrinsic(
            None,
            b"call",
            public_key=wallet.public_key,
            address=wallet.ss58_address,
            crypto_type=7,
            era="00",
            nonce=0,
            genesis_hash=GENESIS,
            era_block_hash=GENESIS,
        )


def test_bth2_descriptor_roundtrip_preserves_v1_namespace(wallet):
    address = receiving_address(wallet)
    assert address.startswith("bth2_") and len(address) == 104
    recipient = parse_recipient(address)
    assert recipient.account == wallet.ss58_address
    assert recipient.descriptor == bytes(wallet.hashed_descriptor)
    assert descriptor_storage(recipient.descriptor) == "V2Records"
    assert descriptor_bytes(descriptor_value(recipient.descriptor)) == recipient.descriptor
    assert receiving_address(wallet.public_only()) == address
    for invalid in (address + "=", address.replace("bth2_", "bth1_"), " " + address):
        with pytest.raises(ValueError):
            parse_recipient(invalid)
    with pytest.raises(ValueError):
        descriptor_value(bytes([2, 4]) + bytes(64))


async def test_transport_uses_finalized_v2_record_full_proof_and_stable_identity(wallet):
    conn, codec, _state = connection(wallet)
    call = b"large ordinary call" * 40
    await conn.create_signed_extrinsic(call, wallet)
    proof = codec.proof
    assert len(proof) == 5367
    assert proof[:32] == bytes(wallet.public_key)
    assert proof[32:34] == bytes(wallet.hashed_descriptor[:2])
    assert int.from_bytes(proof[34:42], "little") == 7
    assert proof[42:1994] == bytes(wallet.at_generation(7).hashed_public_key)
    assert proof[1994:2058] == bytes(wallet.hashed_commitment(8))
    assert codec.call == call and codec.nonce == 13
    assert wallet.hashed_generation == 0
    assert all(call.kwargs["block_hash"] == "0xfinalized" for call in conn.query.await_args_list)


@pytest.mark.parametrize("failure", ["descriptor", "commitment", "pending", "generation", "paused"])
async def test_transport_rejects_wrong_or_stale_authority_before_signing(failure, wallet):
    conn, codec, state = connection(wallet)
    if failure == "descriptor":
        state["descriptor"]["algorithm"] = 5 if wallet.crypto_type == 7 else 4
    elif failure == "commitment":
        state["commitment"] = bytes([1]) * 64
    elif failure == "pending":
        conn._nonces.next_for.return_value = 14
    elif failure == "generation":
        state["generation"] = 2**64 - 1
    else:
        original = conn.query.side_effect

        async def paused(pallet, item, params=None, *, block_hash):
            return (
                False
                if item == "V2AuthorizationEnabled"
                else await original(pallet, item, params, block_hash=block_hash)
            )

        conn.query.side_effect = paused
    with pytest.raises(SubstrateRequestException):
        await conn.create_signed_extrinsic(b"call", wallet)
    assert not hasattr(codec, "proof")


async def test_fee_quote_uses_complete_dummy_proof_without_private_wallet(wallet):
    conn, codec, _ = connection(wallet)
    await conn.create_signed_extrinsic(b"quote", wallet.public_only(), signature=bytes(64))
    assert len(codec.proof) == 5367
    assert codec.proof[:32] == bytes(wallet.public_key)
    assert codec.proof[32:34] == bytes(wallet.hashed_descriptor[:2])
    assert codec.proof[42:1994] == bytes(1952)
    assert codec.proof[2058:] == bytes(3309)
    conn._nonces.next_for.assert_not_awaited()


@pytest.mark.parametrize("exists,enabled", [(False, True), (True, True), (True, False)])
async def test_receiving_guard_routes_to_v2_and_prices_v2_deposit(exists, enabled, wallet):
    substrate = chain()
    substrate.seed("HashedAccounts", "V2RegistrationEnabled", [], enabled)
    if exists:
        substrate.seed("HashedAccounts", "V2Records", [wallet.ss58_address], record(wallet))
    intent, recipients = await prepare_recipient_intent(
        substrate, Transfer(dest_ss58=receiving_address(wallet), amount_tao=1)
    )
    payment = await substrate.compose(
        calls.Balances.transfer_keep_alive(dest=wallet.ss58_address, value=1)
    )
    guarded, extras = await with_recipient_registration(
        substrate, dev_wallet(), intent, payment, recipients=recipients, as_calls=True
    )
    assert guarded[0].module == "HashedAccounts"
    assert guarded[0].function == ("check_registered" if exists else "register")
    assert guarded[0].params["descriptor"] == registration_descriptor_value(
        bytes(wallet.hashed_descriptor)
    )
    assert extras["hashed_registration_max_deposit_rao"] == (0 if exists else 200)
    assert guarded[-1].params["dest"] == wallet.ss58_address


async def test_disabled_v2_registration_cannot_fund_unregistered_identity(wallet):
    substrate = chain()
    substrate.seed("HashedAccounts", "V2RegistrationEnabled", [], False)
    intent, recipients = await prepare_recipient_intent(
        substrate, Transfer(dest_ss58=receiving_address(wallet), amount_tao=1)
    )
    payment = await substrate.compose(
        calls.Balances.transfer_keep_alive(dest=wallet.ss58_address, value=1)
    )
    with pytest.raises(ValueError, match="not enabled"):
        await with_recipient_registration(
            substrate, dev_wallet(), intent, payment, recipients=recipients
        )


async def test_registered_write_and_evm_alias_resolve_v2_even_when_setup_is_paused(wallet):
    substrate = chain()
    substrate.seed("HashedAccounts", "V2RegistrationEnabled", [], False)
    substrate.seed("HashedAccounts", "V2Records", [wallet.ss58_address], record(wallet))
    address = receiving_address(wallet)
    assert await account_for_registered_write(substrate, address) == wallet.ss58_address
    alias = ss58_to_h160_truncated(wallet.ss58_address)
    substrate.seed("HashedAccounts", "EvmAliases", [alias], wallet.ss58_address)
    recipient = await resolve_evm_recipient(substrate, alias)
    assert recipient.address == address
    assert recipient.descriptor == bytes(wallet.hashed_descriptor)


async def test_derived_alias_uses_finalized_policy_without_inventing_a_key_descriptor(wallet):
    substrate = chain()
    substrate.seed(
        "HashedAccounts",
        "DerivedAuthorities",
        [wallet.ss58_address],
        {"descriptor": {"Multisig": {}}},
    )
    alias = ss58_to_h160_truncated(wallet.ss58_address)
    substrate.seed("HashedAccounts", "EvmAliases", [alias], wallet.ss58_address)
    kind, _ = await protected_account_record(substrate, wallet.ss58_address)
    assert kind == "derived"
    recipient = await resolve_evm_funding_recipient(substrate, alias)
    assert recipient.account == wallet.ss58_address
    assert recipient.descriptor is None


async def test_protected_proxy_and_derivative_routes_preserve_exact_inner_call(wallet):
    from bittensor.hashed import compose_derivative
    from bittensor.intents import CreatePureProxy

    substrate = chain()
    substrate.seed("HashedAccounts", "V2Records", [wallet.ss58_address], record(wallet))
    pure = await CreatePureProxy().build(substrate, wallet)
    assert pure.function == "create_protected_pure"
    inner = b"pinned exact economic call bytes"
    derivative = await compose_derivative(substrate, wallet.ss58_address, 17, inner)
    assert derivative.function == "as_protected_derivative"
    assert derivative.params == {"index": 17, "call": inner}
    legacy = dev_wallet()
    pure = await CreatePureProxy().build(substrate, legacy)
    assert pure.function == "create_pure"
    derivative = await compose_derivative(substrate, legacy.coldkey.ss58_address, 17, inner)
    assert derivative.function == "as_derivative"
    assert derivative.params["call"] is inner


async def test_multisig_protection_requires_a_pq_member_in_every_possible_quorum(wallet):
    from bittensor.hashed import multisig_protection_call

    substrate = chain()
    substrate.seed("HashedAccounts", "V2Records", [wallet.ss58_address], record(wallet))
    classic = dev_wallet().coldkey.ss58_address
    members = [classic, receiving_address(wallet)]
    registration = await multisig_protection_call(
        substrate, members, 2, deposit_payer=wallet.ss58_address
    )
    assert registration.function == "register_derived_sponsored"
    descriptor = registration.params["descriptor"]
    assert descriptor["Multisig"]["threshold"] == 2
    assert set(descriptor["Multisig"]["signatories"]) == {classic, wallet.ss58_address}
    account = substrate.multisig_account([classic, wallet.ss58_address], 2).ss58_address
    substrate.seed("HashedAccounts", "DerivedAuthorities", [account], {"descriptor": descriptor})
    substrate.seed("HashedAccounts", "V2RegistrationEnabled", [], False)
    check = await multisig_protection_call(substrate, members, 2, deposit_payer=wallet.ss58_address)
    assert check.function == "check_derived"
    assert check.params["descriptor"] == registration.params["descriptor"]
    with pytest.raises(ValueError, match="complete multisig quorum"):
        await multisig_protection_call(substrate, members, 1, deposit_payer=wallet.ss58_address)


async def test_pq_multisig_requires_quorum_registration_before_funding(wallet):
    from bittensor.client import Client

    substrate, owner = chain(), wallet
    other = Keypair.create_from_seed(bytes([8]) * 32, CRYPTO_MLDSA65)
    for member in (owner, other):
        substrate.seed("HashedAccounts", "V2Records", [member.ss58_address], record(member))
    group = await Client(substrate=substrate).multisig([owner.ss58_address, other.ss58_address], 1)
    with pytest.raises(ValueError, match="through its quorum"):
        await group.funding_call(1_000_000)
    registration = await group.protection_call(deposit_payer=owner.ss58_address)
    assert registration.function == "register_derived_sponsored"
    assert registration.params["deposit_payer"] == owner.ss58_address
    substrate.seed(
        "HashedAccounts",
        "DerivedAuthorities",
        [group.address],
        {"descriptor": registration.params["descriptor"]},
    )
    payment = await group.funding_call(1_000_000)
    guard, transfer = payment.params["calls"]
    assert guard.function == "check_derived"
    assert transfer.params == {"dest": group.address, "value": 1_000_000}


@pytest.mark.parametrize("crypto", [4, 6, 7, 8])
async def test_shield_pair_uses_one_finalized_generation_snapshot_and_rejects_race(crypto):
    from dataclasses import replace

    wallet = Keypair.create_from_seed(bytes([7]) * 32, crypto)
    conn, codec, _state = connection(wallet)
    snapshot = await conn.prepare_hashed_snapshot(wallet)
    assert (
        snapshot.block_hash == "0xfinalized" and snapshot.generation == 7 and snapshot.nonce == 13
    )
    await conn.create_signed_extrinsic(
        b"inner", wallet, nonce=14, hashed_generation_offset=1, hashed_signing_snapshot=snapshot
    )
    offset = 34 if crypto in (7, 8) else 0
    assert int.from_bytes(codec.proof[offset : offset + 8], "little") == 8
    conn.get_chain_finalised_head.return_value = "0xnew-head-must-not-change-authority"
    await conn.create_signed_extrinsic(b"outer", wallet, nonce=13, hashed_signing_snapshot=snapshot)
    assert int.from_bytes(codec.proof[offset : offset + 8], "little") == 7
    with pytest.raises(SubstrateRequestException, match="snapshot does not match"):
        await conn.create_signed_extrinsic(
            b"outer", wallet, nonce=13, hashed_signing_snapshot=replace(snapshot, generation=8)
        )
    conn._nonces.next_for.return_value = 14
    with pytest.raises(SubstrateRequestException, match="pending transaction"):
        await conn.create_signed_extrinsic(
            b"outer", wallet, nonce=13, hashed_signing_snapshot=snapshot
        )


@pytest.mark.parametrize("crypto", [4, 6, 7, 8])
async def test_executor_pairs_same_snapshot_and_refuses_intervening_acceptance(monkeypatch, crypto):
    from bittensor.executor import Executor

    substrate = FakeSubstrate()
    substrate.mev_key = bytes([1]) * 32
    signer = Keypair.create_from_seed(bytes([7]) * 32, crypto)
    monkeypatch.setattr(
        "bittensor.executor._core.encrypt_mlkem768", lambda *args, **kwargs: b"encrypted"
    )
    original_sign = substrate.sign_extrinsic
    seen = []

    async def intercept(*args, **kwargs):
        seen.append(kwargs["hashed_signing_snapshot"])
        return await original_sign(*args, **kwargs)

    substrate.sign_extrinsic = intercept
    result = await Executor(substrate)._submit_encrypted_call(
        b"call", signer, {}, period=8, wait_for_inclusion=False, wait_for_finalization=False
    )
    assert result.success
    carrier = substrate.submissions[0][0]
    assert (carrier.module, carrier.function) == ("MevShield", "submit_encrypted")
    assert substrate.submissions[0][2]["hashed_signing_snapshot"] is seen[0]

    async def raced(*args, **kwargs):
        result = await original_sign(*args, **kwargs)
        substrate._nonces[signer.ss58_address] += 1
        return result

    substrate.sign_extrinsic = raced
    with pytest.raises(ValueError, match="stale nonce"):
        await Executor(substrate)._submit_encrypted_call(
            b"second", signer, {}, period=8, wait_for_inclusion=False, wait_for_finalization=False
        )
    assert len(substrate.submissions) == 1


async def test_fixed_profile_rejects_old_runtime_before_private_key_use():
    from unittest.mock import Mock

    fixed = key(8)
    conn, codec, _ = connection(fixed)
    original = codec.constant
    codec.constant = lambda pallet, name: (
        None if name == "MldsaAccountProfiles" else original(pallet, name)
    )
    private = Mock(wraps=fixed)
    private.crypto_type = fixed.crypto_type
    private.hashed_descriptor = fixed.hashed_descriptor
    with pytest.raises(SubstrateRequestException, match="fixed-key"):
        await conn.create_signed_extrinsic(b"call", private)
    private.at_generation.assert_not_called()
    assert not hasattr(codec, "proof")


@pytest.mark.parametrize("profiles", ["0x0405", b"\x04\x05", bytearray([4, 5]), [4, 5], (4, 5)])
def test_fixed_profile_accepts_metadata_byte_representations(profiles):
    from bittensor.mldsa import supports_fixed_profile

    assert supports_fixed_profile(profiles)


@pytest.mark.parametrize(
    "profiles", [None, 5, True, "0405", "0xzz", "0x04", [], [4], [-1, 5], {"5": True}]
)
def test_fixed_profile_rejects_absent_or_malformed_advertisements(profiles):
    from bittensor.mldsa import supports_fixed_profile

    assert not supports_fixed_profile(profiles)


async def test_master_switch_blocks_ms_signing_and_receiving_despite_subgates(wallet):
    conn, codec, _ = connection(wallet)
    old_constant = codec.constant
    codec.constant = lambda pallet, name: None if name == "Enabled" else old_constant(pallet, name)
    original = conn.query.side_effect

    async def query(pallet, item, params=None, *, block_hash):
        if (pallet, item) == ("AdminUtils", "HashedAccountsEnabled"):
            assert block_hash == "0xfinalized"
            return False
        return await original(pallet, item, params, block_hash=block_hash)

    conn.query.side_effect = query
    with pytest.raises(SubstrateRequestException, match="not enabled"):
        await conn.create_signed_extrinsic(b"call", wallet)
    assert not hasattr(codec, "proof")
    substrate = chain()
    substrate.seed_constant("HashedAccounts", "Enabled", None)
    substrate.seed("AdminUtils", "HashedAccountsEnabled", [], False)
    substrate.seed("HashedAccounts", "V2Records", [wallet.ss58_address], record(wallet))
    with pytest.raises(ValueError, match="not supported"):
        await prepare_recipient_intent(
            substrate, Transfer(dest_ss58=receiving_address(wallet), amount_tao=1)
        )


@pytest.mark.parametrize("crypto", [4, 6, 7, 8])
async def test_shield_quote_counts_complete_registered_envelope_without_signing(crypto):
    from bittensor._transport.extrinsics import SHIELD_ENCRYPTION_OVERHEAD

    wallet = key(crypto)
    conn, codec, _ = connection(wallet)
    if crypto in (4, 6):
        conn.get_chain_head = AsyncMock(return_value="0xfinalized")
    call = b"private shield call"
    public = Keypair.from_hashed_descriptor(bytes(wallet.hashed_descriptor))
    conn.compose_call = AsyncMock(return_value=b"carrier")
    conn.get_payment_info = AsyncMock(return_value={"partial_fee": 17})
    await conn.get_shielded_payment_info(call, public, period=8)
    quoted_proof = codec.proof
    # The fixture codec emits call bytes plus the entire proof. Compare to the
    # real native signature/proof, not a hand-picked maximum transaction size.
    assert len(quoted_proof) == hashed_proof_length(crypto)
    assert bytes(wallet.at_generation(7).hashed_public_key) not in quoted_proof
    actual = await conn.create_signed_extrinsic(
        call, wallet, nonce=14, era={"period": 8}, hashed_generation_offset=1
    )
    ciphertext = conn.compose_call.call_args.args[2]["ciphertext"]
    assert len(ciphertext) == len(actual.data) + SHIELD_ENCRYPTION_OVERHEAD
    assert conn.get_payment_info.call_args.kwargs == {"nonce": 13, "era": {"period": 8}}
