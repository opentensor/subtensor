"""Assemble the release record only after its required evidence is complete."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET

N = Path(__file__).parent
R = Path('/home/johnreed/Desktop/work/subtensor')
def read(path):
    return json.loads((N / path).read_text())
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

initial = read('initial-manifest.json')
sources = read('final-repair-sources.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=R, text=True).strip()
assert head == initial['head'] == sources['head']
assert all(sha(R / path) == digest for path, digest in initial['lockfiles'].items())
assert all(sha(R / path) == digest for path, digest in sources['files'].items())
changed = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD'], cwd=R, text=True).splitlines()
assert not any(p.startswith(('runtime/', 'pallets/', 'primitives/', 'node/', 'vendor/')) for p in changed)
subprocess.run(['git', 'diff', '--check'], cwd=R, check=True)
subprocess.run(['python3', str(N / 'collect-live-results.py')], stdout=subprocess.DEVNULL, check=True)
live = read('live-case-summary.json')
assert live['all_expected_passed'] and live['expected_cases'] == 119
assert (N / 'preflight.exit').read_text().strip() == '0'
assert (N / 'sdk-final.exit').read_text().strip() == '0'
assert (N / 'sdk-types-final.exit').read_text().strip() == '0'
assert (N / 'additional.exit').read_text().strip() == '0'
sdk = ET.parse(N / 'sdk-final.xml')
sdk_cases = list(sdk.iter('testcase'))
assert len(sdk_cases) == 3880
assert all(not any(c.find(tag) is not None for tag in ['failure', 'error', 'skipped']) for c in sdk_cases)
gate = (N / 'preflight.log').read_text()
rust = [tuple(map(int, m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', gate)]
rust_totals = dict(zip(['passed', 'failed', 'ignored'], [sum(row[i] for row in rust) for i in range(3)]))
assert rust_totals == {'passed': 3610, 'failed': 0, 'ignored': 47}
upgrade = read('mainnet-verified/ml-dsa-mainnet-upgrade-results.json')
assert upgrade['successful']
mixed_state = read('mixed/cli-state-verification.json')
assert mixed_state['all_verified']
direct_mixed_state = read('mixed/direct-sdk-state-verification.json')
assert direct_mixed_state['all_verified'] and direct_mixed_state['cases_verified'] == 16
assert direct_mixed_state['post_test_best_block_awaited']['finalized']
rpc_capacity = read('rpc/pending-capacity-results.json')
assert rpc_capacity['passed'] and [c['count'] for c in rpc_capacity['cases']] == [80, 100, 254, 256, 257]
assert all(c['passed'] and c['queries_left_latest_unchanged'] for c in rpc_capacity['cases'])
receipts = read('receipt-verification/final-summary.json')
assert receipts['all_recorded_receipts_verified']
chains = read('chains-verified-final.json')
assert chains['runtime_code_matches_frozen_artifact'] and chains['distinct_chains'] == 15
shutdown = read('shutdown.json')
assert shutdown['all_closed']
cleanup = read('cleanup.json')
assert cleanup['all_owned_processes_stopped'] and cleanup['all_validation_ports_closed']

result = {
    'completed_at': datetime.now(timezone.utc).isoformat(),
    'status': 'LOCAL_PR_VALIDATION_PASSED_WITH_LOCAL_SDK_REPAIRS_AND_DOCUMENTED_PREEXISTING_LIMITS',
    'head': head, 'base': initial['base'],
    'scope': 'Current-head runtime and weights plus the four local SDK production files in final-repair-sources.json. Earlier live suites retained their recorded source timing; final regressions and all offline SDK checks use the final repaired source.',
    'local_repairs': ['Aggregate distinct reserve and fee payers by AccountId and include Shield carrier affordability', 'Combine multisig approval reserve with immediate inner/carrier fees in all three approval APIs and nested planning'],
    'lockfiles_unchanged': True, 'runtime_and_weights_unchanged': True,
    'production_source_hashes': {p: digest for p, digest in sources['files'].items() if p.startswith('sdk/python/bittensor/')},
    'offline_sdk': {'passed': 3880, 'failed': 0, 'skipped': 0},
    'rust_workspace': rust_totals,
    'live_unique_pytest_cases': {'passed': 119, 'failed': 0, 'skipped': 0, 'evidence': 'live-case-summary.json'},
    'full_preflight': {'exit': 0, 'runtime_path_workaround': 'preflight-path-workaround.json'},
    'typecheck': {'exit': 0, 'nonfatal_diagnostics': 250},
    'dispatch_matrix': {'call_variants': 323, 'cases': 1292, 'unresolved': []},
    'cryptographic_crosschecks': read('crypto/validation-summary.json'),
    'canonical_receipts': receipts,
    'mainnet_upgrade': {'passed': True, 'source': upgrade['source'], 'snapshot_sha256': upgrade['snapshot_sha256'], 'runtime_wasm_sha256': upgrade['runtime_wasm_sha256'], 'post_upgrade_blocks': 10},
    'evidence': {'pending_rpc': 'rpc/pending-capacity-results.json', 'mixed_finalized_state': 'mixed/cli-state-verification.json', 'direct_mixed_finalized_state': 'mixed/direct-sdk-state-verification.json', 'codecs': 'codecs/javascript.json', 'native_transfers': 'crypto-native-basic/report.md', 'new_live_regressions': 'costs/new-regressions-summary.json'},
    'boundaries': ['Exact native Rust Shield uses an unsupported 64-block era and is rejected; also present at PR base', 'Registration before public disclosure/funding remains required', 'Recorded tests do not prove all inputs, side-channel resistance, production saturation, physical Vault compatibility or positive-yield root-claim payout'],
    'shutdown': cleanup, 'committed': False, 'pushed': False,
}
(N / 'validation-summary.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['status', 'head', 'offline_sdk', 'rust_workspace', 'live_unique_pytest_cases']}))
