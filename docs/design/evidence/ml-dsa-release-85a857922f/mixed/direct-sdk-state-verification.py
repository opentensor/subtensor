"""Independently verify 16 completed direct SDK cases using public files only."""

import asyncio
from collections import defaultdict
from datetime import datetime, timezone
import hashlib
from itertools import product
import json
from pathlib import Path
import re

import bittensor as bt
from bittensor.sp_core import ss58_decode


ROOT = Path('/tmp/ms-release-85a857922f')
RUN = ROOT / 'mixed'
FUNCTION = 'test_direct_sdk_multisig_receiving_transfer'
PREFIX = 'tests/e2e/test_mixed_multisig.py::' + FUNCTION
REPO = Path('/home/johnreed/Desktop/work/subtensor')
CASE_PATTERN = re.compile(r'\[(receiving|local_ss58)-(4|6)-(1|2)-(4|6)\]$')
TAO = 10**9


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fixture_matrix():
    started = json.loads((ROOT / 'initial-manifest.json').read_text())['started_at']
    expected = {
        f'{PREFIX}[{form}-{signer}-{threshold}-{recipient}]'
        for form, signer, threshold, recipient in product(('receiving', 'local_ss58'), (4, 6), (1, 2), (4, 6))
    }
    assert expected == {n for n in json.loads((ROOT / 'expected-live-cases.json').read_text())['nodeids'] if n.startswith(PREFIX + '[')}
    log = (RUN / 'pytest.log').read_text()
    completed = defaultdict(list)
    for worker, nodeid in re.findall(r'\[(gw\d+)\]\s+\[\s*\d+%\]\s+PASSED\s+(' + re.escape(PREFIX) + r'\[[^\]]+\])', log):
        completed[worker].append(nodeid)
    passed = [case for cases in completed.values() for case in cases]
    assert len(passed) == len(set(passed)), 'duplicate passed case IDs'
    if set(passed) != expected:
        raise RuntimeError('WAITING for direct SDK cases: ' + ', '.join(sorted(expected - set(passed))))
    paths = defaultdict(list)
    for base in Path('/tmp').glob('pytest-of-*'):
        for public_path in base.rglob('coldkeypub.txt'):
            if public_path.is_symlink() or public_path.parent.name != 'recipient' or public_path.stat().st_mtime < started:
                continue
            directory = public_path.parent.parent
            if not re.fullmatch(r'test_direct_sdk_multisig_recei\d+', directory.name):
                continue
            worker = directory.parent.name.removeprefix('popen-')
            paths[worker].append((public_path.stat().st_mtime_ns, directory))
    assert set(paths) == set(completed), (set(paths), set(completed))
    fixtures = []
    for worker, nodeids in sorted(completed.items()):
        ordered = sorted(paths[worker])
        assert len(ordered) == len(nodeids), (worker, len(ordered), len(nodeids))
        assert len({mtime for mtime, _ in ordered}) == len(ordered), 'ambiguous fixture creation order'
        # A pytest worker runs its cases sequentially. Its numbered temporary
        # directories and public-file creation times identify the same order.
        for nodeid, (mtime, directory) in zip(nodeids, ordered):
            match = CASE_PATTERN.search(nodeid)
            assert match
            form, signer, threshold, recipient = match.groups()
            public = {}
            source_hashes = {}
            for name in ('member-0', 'member-1', 'member-2', 'recipient'):
                path = directory / name / 'coldkeypub.txt'
                assert path.is_file() and not path.is_symlink() and path.stat().st_mtime >= started
                obj = json.loads(path.read_text())
                public[name] = {key: obj[key] for key in ('ss58Address', 'cryptoType', 'publicKey')}
                source_hashes[str(path)] = digest(path)
            assert tuple(p['cryptoType'] for p in public.values()) == (int(signer), 1, 6 if signer == '4' else 4, int(recipient)), nodeid
            fixtures.append({'nodeid': nodeid, 'fixture': str(directory), 'worker': worker,
                             'recipient_public_file_mtime_ns': mtime, 'public_file_sha256': source_hashes,
                             'address_form': form, 'threshold': int(threshold), 'public': public})
    assert len(fixtures) == 16
    assert len({p['ss58Address'] for case in fixtures for p in case['public'].values()}) == 64
    return sorted(fixtures, key=lambda case: case['nodeid'])


async def main():
    fixtures = fixture_matrix()
    async with bt.Client('ws://127.0.0.1:27244', fallback_endpoints=[], archive_endpoints=[]) as client:
        raw = client._substrate.raw
        rpc = raw._session.request
        # Some SDK execution tests wait for inclusion. Pin the best block after
        # all target cases passed, then require that exact block to finalize.
        best_hash = await rpc('chain_getBlockHash', [])
        best_height = int((await rpc('chain_getHeader', [best_hash]))['number'], 16)
        while True:
            final_hash = await rpc('chain_getFinalizedHead', [])
            height = int((await rpc('chain_getHeader', [final_hash]))['number'], 16)
            if height >= best_height:
                assert await rpc('chain_getBlockHash', [best_height]) == best_hash, 'pinned post-test best block was reorganized'
                break
            await asyncio.sleep(1)
        assert await rpc('chain_getBlockHash', [height]) == final_hash
        genesis = await rpc('chain_getBlockHash', [0])
        assert genesis == json.loads((RUN / 'activation.json').read_text())['genesis']
        codec = await raw._runtimes.codec_at(final_hash)
        registration_deposit = int(await client.constant(bt.constants.HashedAccounts.RegistrationDeposit))
        assert registration_deposit == 200_000_000
        multisig_base = int(await client.constant(bt.constants.Multisig.DepositBase))
        multisig_factor = int(await client.constant(bt.constants.Multisig.DepositFactor))
        alice = await client._substrate.query('Sudo', 'Key', [], block_hash=final_hash)

        async def state(address):
            return {'address': address,
                    'account': await client._substrate.query('System', 'Account', [address], block_hash=final_hash),
                    'hashed': await client._substrate.query('HashedAccounts', 'Accounts', [address], block_hash=final_hash)}

        tokens = set()
        for case in fixtures:
            members = [case['public'][f'member-{i}']['ss58Address'] for i in range(3)]
            group = await client.multisig(members, case['threshold'])
            alternate = await client.multisig(members, 3 - case['threshold'])
            tokens.add(bytes(ss58_decode(group.address)).hex())
            case['group'] = await state(group.address)
            case['alternate_threshold_group'] = await state(alternate.address)
            case['states'] = {name: await state(pub['ss58Address']) for name, pub in case['public'].items()}
            case['pending_multisigs'] = await client._substrate.query_map('Multisig', 'Multisigs', [group.address], block_hash=final_hash)
            assert not case['pending_multisigs'], case['nodeid']
            assert case['group']['account']['data']['free'] == 9 * TAO, case['nodeid']
            assert case['group']['account']['data']['reserved'] == 0 and case['group']['account']['nonce'] == 0
            assert case['group']['hashed'] is None
            assert case['alternate_threshold_group']['account']['data']['free'] == 0
            receiver = case['states']['recipient']
            assert receiver['account']['data']['free'] == TAO and receiver['account']['data']['reserved'] == 0
            assert receiver['account']['nonce'] == 1 and receiver['hashed']['generation'] == 0
            for name, pub in case['public'].items():
                current = case['states'][name]
                assert current['account']['data']['reserved'] == 0
                tokens.add(pub['publicKey'].removeprefix('0x'))
                tokens.add(bytes(ss58_decode(pub['ss58Address'])).hex())
                if pub['cryptoType'] in (4, 6):
                    assert current['hashed'] is not None
                    assert current['hashed']['descriptor']['scheme'] == ('Sr25519' if pub['cryptoType'] == 4 else 'Ed25519')
                    tokens.add(current['hashed']['descriptor']['initial_commitment'].removeprefix('0x'))
                else:
                    assert current['hashed'] is None

        proof = []
        for start in range(1, height + 1, 12):
            numbers = list(range(start, min(height + 1, start + 12)))
            hashes = await raw._session.request_batch([('chain_getBlockHash', [n]) for n in numbers])
            blocks = await raw._session.request_batch([('chain_getBlock', [h]) for h in hashes])
            for number, block_hash, block in zip(numbers, hashes, blocks):
                for index, wire in enumerate(block['block']['extrinsics']):
                    if not any(token in wire for token in tokens):
                        continue
                    tx_hash = '0x' + hashlib.blake2b(bytes.fromhex(wire[2:]), digest_size=32).hexdigest()
                    outcome = await raw.resolve_extrinsic(tx_hash, block_hash, finalized=True)
                    decoded = codec.decode_extrinsic(wire)
                    events = [entry['event'] for entry in outcome.triggered_events]
                    proof.append({'block_number': number, 'block_hash': block_hash, 'extrinsic_index': index,
                                  'extrinsic_hash': tx_hash, 'encoded_extrinsic': wire,
                                  'call': decoded['call']['call_module'] + '.' + decoded['call']['call_function'],
                                  'success': outcome.is_success, 'error': outcome.error_message, 'events': events})

        def matching(module, event, **attributes):
            return [(row, item) for row in proof for item in row['events']
                    if item['module_id'] == module and item['event_id'] == event
                    and all(item['attributes'].get(key) == value for key, value in attributes.items())]

        for case in fixtures:
            group = case['group']['address']
            receiver = case['states']['recipient']['address']
            transferred = matching('Balances', 'Transfer', **{'from': group, 'to': receiver, 'amount': TAO})
            funded = matching('Balances', 'Transfer', **{'from': alice, 'to': group, 'amount': 10 * TAO})
            assert len(transferred) == len(funded) == 1, case['nodeid']
            assert transferred[0][0]['success'] and funded[0][0]['success']
            case_proof = {transferred[0][0]['extrinsic_hash'], funded[0][0]['extrinsic_hash']}
            registrations = []
            member_fees = {}
            for name, pub in case['public'].items():
                address = pub['ss58Address']
                current = case['states'][name]
                if pub['cryptoType'] in (4, 6):
                    registered = matching('HashedAccounts', 'Registered', account=address)
                    assert len(registered) == 1, (case['nodeid'], name, 'registration count', len(registered))
                    row, event = registered[0]
                    assert row['success'] and event['attributes']['sponsor'] == alice and event['attributes']['deposit'] == registration_deposit
                    assert matching('Balances', 'Reserved', who=alice, amount=registration_deposit)
                    assert any(e['module_id'] == 'Balances' and e['event_id'] == 'Reserved' and e['attributes']['who'] == alice and e['attributes']['amount'] == registration_deposit for e in row['events'])
                    registrations.append({'account': address, 'sponsor': alice, 'deposit_rao': registration_deposit, 'extrinsic_hash': row['extrinsic_hash']})
                    case_proof.add(row['extrinsic_hash'])
                if name == 'recipient':
                    continue
                funding = matching('Balances', 'Transfer', **{'from': alice, 'to': address, 'amount': 100 * TAO})
                assert len(funding) == 1 and funding[0][0]['success']
                case_proof.add(funding[0][0]['extrinsic_hash'])
                fees = matching('TransactionPayment', 'TransactionFeePaid', who=address)
                assert all(row['success'] and event['attributes']['tip'] == 0 for row, event in fees)
                total = sum(event['attributes']['actual_fee'] for _, event in fees)
                assert current['account']['data']['free'] == 100 * TAO - total, (case['nodeid'], name, total, current)
                assert current['account']['nonce'] == len(fees) + (1 if pub['cryptoType'] in (4, 6) else 0)
                if pub['cryptoType'] in (4, 6):
                    assert current['hashed']['generation'] == len(fees)
                if name == 'member-0' or (name == 'member-1' and case['threshold'] == 2):
                    assert fees
                else:
                    assert not fees
                case_proof.update(row['extrinsic_hash'] for row, _ in fees)
                member_fees[name] = {'paid_rao': total, 'included_fee_events': len(fees)}
            opener = case['states']['member-0']['address']
            reserved = matching('Balances', 'Reserved', who=opener)
            unreserved = matching('Balances', 'Unreserved', who=opener)
            expected = multisig_base + 2 * multisig_factor if case['threshold'] == 2 else 0
            assert sum(e['attributes']['amount'] for _, e in reserved) == expected
            assert sum(e['attributes']['amount'] for _, e in unreserved) == expected
            assert len(reserved) == len(unreserved) == (1 if case['threshold'] == 2 else 0)
            if case['threshold'] == 2:
                executed = matching('Multisig', 'MultisigExecuted', multisig=group)
                assert len(executed) == 1 and 'Ok' in executed[0][1]['attributes']['result']
                case_proof.add(executed[0][0]['extrinsic_hash'])
            case['registration_deposits'] = registrations
            case['approval_deposit_rao'] = expected
            case['member_fees'] = member_fees
            case['proof_extrinsic_hashes'] = sorted(case_proof)
            case['verified'] = True

        result = {'candidate': json.loads((ROOT / 'initial-manifest.json').read_text())['head'],
                  'captured_at_utc': datetime.now(timezone.utc).isoformat(), 'all_verified': True,
                  'cases_verified': len(fixtures), 'genesis': genesis,
                  'anchor': {'block_number': height, 'block_hash': final_hash},
                  'post_test_best_block_awaited': {'block_number': best_height, 'block_hash': best_hash, 'finalized': True},
                  'runtime': await rpc('state_getRuntimeVersion', [final_hash]),
                  'public_read_policy': 'Only fresh coldkeypub.txt files and public run logs/manifests; no private wallet files or signing.',
                  'source_sha256': {'helper': digest(Path(__file__)), 'test': digest(REPO / 'sdk/python/tests/e2e/test_mixed_multisig.py'), 'pytest_log': digest(RUN / 'pytest.log')},
                  'registration_deposit_rao': registration_deposit, 'sponsor': alice,
                  'cases': fixtures, 'canonical_event_proof': proof}
        output = RUN / 'direct-sdk-state-verification.json'
        output.write_text(json.dumps(result, indent=2) + '\n')
        (RUN / 'direct-sdk-state-verification.md').write_text(
            '---\ntitle: "Independent direct SDK multisig final-state verification"\n'
            'description: "Sixteen public fixtures checked against finalized balances, authority, deposits and canonical events."\n---\n\n'
            f'All 16 current-run cases passed independent checks at finalized block {height} (`{final_hash}`). '
            'Both receiving-address and local-SS58 forms, thresholds one and two, and hashed SR/ED signer and recipient combinations are represented exactly once.\n\n'
            'Each group holds 9 TAO after funding with 10; its recipient holds 1 TAO. There are no pending multisigs or remaining member approval reserves. '
            'Member free balances equal their 100 TAO funding minus canonical fee events; exact nonces and hashed generations match those included transactions. '
            'Each permanent 0.2 TAO registration reserve was paid by Alice, verified through matching Registered/Reserved events and current registry presence. '
            'Threshold-two approval reserve and refund events match the runtime deposit constants.\n\n'
            'Only public coldkeypub.txt files were read. Fixture ordering was matched to each worker\'s completed case IDs, then independently cross-checked against crypto codes and the unambiguously funded threshold-derived group. '
            'The verifier made only chain reads and did not change source, sign transactions, or operate chains. Full-suite completion is a separate result.\n')
        print(json.dumps({'cases_verified': len(fixtures), 'all_verified': True, 'finalized_block': height,
                          'canonical_extrinsics': len(proof), 'evidence_sha256': digest(output)}))


asyncio.run(asyncio.wait_for(main(), 300))
