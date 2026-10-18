#!/usr/bin/env python3
"""Select public release evidence; dry-run unless --copy is explicitly supplied.

Run from any directory. Default paths identify the 85a857922f validation run.
The dry-run writes only package-plan.json beside this script. --copy atomically
writes selected files and PACKAGE-MANIFEST.json, refusing unowned collisions.
Files left over from an earlier package are retained and declared, never deleted.
No wallet files, databases, chain specifications, executables or snapshot bytes
are selected. The three crypto subtrees are governed by their exact manifests.

Secret checks are defense in depth, not proof that arbitrary text is secret-free:
BIP39 word runs, private-key/token markers, sensitive JSON fields, literal secret
assignments, and exact known private-wallet secrets are checked before any copy.
CLI wallet-creation output is rejected, including output embedded in receipts.
The external English word list is a scanner dependency, not package evidence.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import html
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys
import tempfile


HEAD = "85a857922f70c218d499d2485b899fa9ef261a05"
DEFAULT_SOURCE = Path("/tmp/ms-release-85a857922f")
DEFAULT_REPO = Path("/home/johnreed/Desktop/work/subtensor")
DEST_REL = Path("docs/design/evidence/ml-dsa-release-85a857922f")
PACKAGE_MANIFEST = "PACKAGE-MANIFEST.json"
PLAN = "package-plan.json"
COMPRESS_AT = 64 * 1024
BIP39_ENGLISH_SHA256 = "2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda"
COMPRESS_EXTENSIONS = {".log", ".xml", ".json"}
CRYPTO_DIRS = ("crypto", "crypto-live", "crypto-native-basic")
SUITES = {
    "authority", "costs", "extended", "lifecycle", "mixed", "nested",
    "operations", "parity", "proxy-boundaries", "release", "rpc", "shield",
    "sponsorship", "validator-proxy", "validators",
}
TEXT_EXTENSIONS = {".json", ".log", ".xml", ".md", ".py", ".sh", ".cjs",
                   ".rs", ".patch", ".exit", ".sha256", ".pids"}
ROOT_EXACT = set("""
activate.py additional.sh additional.exit artifacts.sha256 build-native.log build-node.log
build.exit build.sh chains-verified.json chains.json default-cooldown.log
default-evm-meter.log dispatch-matrix-summary.json dispatch-matrix.log
fee-quotes.json final-preflight.sh fixture-rerun-results.json fixture-reruns.log
harness-manifest.json initial-manifest.json launchers.json live-results.json
live-run.log mainnet-build.log preflight.exit preflight.log preflight.sh
preflight-initial.exit preflight-initial.log preflight-path-failure.exit
preflight-path-failure.log preflight-path-workaround.json
production-tests.log repair-manifest.json rerun-fixtures.py
run-live.py sdk-legacy.exit sdk-legacy.log sdk-legacy.xml sdk-multisig-draft.log
sdk-multisig-draft.xml sdk-types.exit sdk-types.log sdk.exit sdk.log sdk.sh
sdk.xml status.py verify-chains.py verify-weight-selection.py wasm-binding.log
wasm-build.log wasm-core.log wasm-smoke.log weight-selection-verification.json
package-evidence.py collect-live-results.py expected-live-cases.json
final-repair-sources.json validation-summary.json finalize-summary.py verify-cleanup.py verify-chains-final.py chains-verified-final.json partial-shutdown.json remaining-node-ownership.json ignored-tests.json pr-metadata-verification.json
""".split())
ROOT_FUTURE = re.compile(
    r"(?:sdk-final|sdk-types-final|sdk-legacy-final|final-summary|final-status|final-source|final-preflight|"
    r"shutdown|cleanup|multisig-costs|mainnet-verified)(?:[-_][a-zA-Z0-9_-]+)?"
    r"\.(?:json|log|xml|exit|md|sha256|py|sh)$"
)
SUITE_EXACT = set("""
genesis.py localnet-operational.patch localnet.log localnet.sh nodes.pids
cli-accepted-nodeids.json distinct-fee-payer-repair.md prepare-root-cli.py
root-cli-accepted-nodeids.json root-cli-parameters.json root-cli-parameters.log
new-regressions-summary.json
verify-cli-state.py cli-state-verification.json cli-state-verification.log
direct-sdk-state-verification-manifest.json direct-sdk-state-verification.py direct-sdk-state-verification.json direct-sdk-state-verification.md direct-sdk-state-verification.log
""".split())
SUITE_RUN = re.compile(
    r"(?:activation|pytest|results|cli-pytest|root-cli-pytest|multisig-cost-pytest|"
    r"multisig-costs-pytest|multisig-cost-junit|delegate-final-pytest|delegate-final-junit|summary|final-summary)"
    r"(?:[-_][a-zA-Z0-9_-]+)?\.(?:json|log|xml|exit|md)$"
)
RPC_FILES = re.compile(
    r"(?:pending|ethereum|live|fixed|admission|additional|capacity|run|runner|"
    r"results|report|summary)(?:[-_][a-zA-Z0-9_-]+)?"
    r"\.(?:py|json|md|log|xml|exit|sha256)$"
)
CODEC_FILES = re.compile(
    r"(?:capture|check|exit|javascript|native-current|native-legacy|run|results|summary)"
    r"(?:[-_][a-zA-Z0-9_-]+)?(?:\.(?:json|log|cjs|py|sh|md|exit))?$"
)
RECEIPT_FILES = re.compile(
    r"(?:interim|final|summary|results|verify)(?:[-_][a-zA-Z0-9_-]+)?"
    r"\.(?:json|log|md|py|exit)$"
)
MAINNET_EXACT = {"acquire.log", "acquire.py", "header.json", "snapshot.exit",
                 "snapshot.log", "snapshot.sha256", "source.json"}
MAINNET_VERIFIED = re.compile(
    r"(?:ml-dsa-mainnet|mainnet|runtime|snapshot|source|header|results|summary|"
    r"verify|run|final)(?:[-_][a-zA-Z0-9_-]+)?"
    r"\.(?:json|log|md|py|sh|exit|sha256)$"
)
SENSITIVE_KEYS = {
    "secretseed", "secretphrase", "mnemonic", "privatekey", "secretkey",
    "password", "passphrase", "masterseed", "seed", "entropy", "apikey",
    "accesstoken", "refreshtoken",
}
SECRET_MARKERS = [
    ("private-key PEM", re.compile(r"-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----")),
    ("GitHub token", re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")),
    ("AWS access key", re.compile(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b")),
    ("credential-bearing URL", re.compile(r"https?://[^\s/:@]+:[^\s/@]+@")),
    ("bearer credential", re.compile(r"(?i)\bBearer\s+[A-Za-z0-9_.~+/-]{24,}={0,2}")),
    ("wallet-creation mnemonic heading", re.compile(r"(?i)(?:your new|secret recovery|recovery|backup)\s+(?:mnemonic|seed phrase)\s*[:=]")),
]
SECRET_LITERAL = re.compile(
    r"(?i)[\"']?(secret[_-]?seed|secret[_-]?phrase|mnemonic|private[_-]?key|"
    r"secret[_-]?key|master[_-]?seed|password|passphrase|api[_-]?key|access[_-]?token)"
    r"[\"']?\s*[:=]\s*[\"']([^\"'\r\n]{8,})[\"']"
)
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
WORD = re.compile(r"\b[a-z]+\b")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: object) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def valid_relative(name: str) -> PurePosixPath:
    p = PurePosixPath(name)
    if not name or p.is_absolute() or any(x in {"", ".", ".."} for x in p.parts) or str(p) != name:
        raise ValueError(f"unsafe relative path: {name!r}")
    return p


def plain_file(root: Path, name: str) -> Path:
    rel = valid_relative(name)
    p = root
    for part in rel.parts:
        p /= part
        if p.is_symlink():
            raise ValueError(f"symlink is not portable evidence: {name}")
    if not p.is_file() or not stat.S_ISREG(p.stat().st_mode):
        raise ValueError(f"not a regular evidence file: {name}")
    return p


def add_file(selected: dict[str, str], root: Path, p: Path, reason: str) -> None:
    name = p.relative_to(root).as_posix()
    plain_file(root, name)
    # Enforce exclusions even if a future manifest or whitelist is mistaken.
    if any(re.search(r"(?i)(?:^|[-_])(private|wallets?|keystore|database|specs?|node_modules)(?:$|[-_])", x)
           for x in PurePosixPath(name).parts[:-1]):
        raise ValueError(f"forbidden evidence subtree selected: {name}")
    if p.suffix not in TEXT_EXTENSIONS and p.name != "exit" and name != "bin/try-runtime":
        raise ValueError(f"non-text evidence type selected: {name}")
    if p.name in {"mainnet.snap", "node-subtensor", "runtime.wasm"}:
        raise ValueError(f"binary/snapshot selected: {name}")
    selected[name] = reason


def select(root: Path) -> tuple[dict[str, str], dict[str, str]]:
    selected: dict[str, str] = {}
    crypto_expected: dict[str, str] = {}
    for p in sorted(root.iterdir()):
        if p.is_file() and (p.name in ROOT_EXACT or ROOT_FUTURE.fullmatch(p.name)):
            add_file(selected, root, p, "top-level run evidence")
    wrapper_path = root / "bin/try-runtime"
    if wrapper_path.exists():
        wrapper = plain_file(root, "bin/try-runtime").read_bytes()
        if not wrapper.startswith(b"#!") or len(wrapper) > COMPRESS_AT:
            raise ValueError("bin/try-runtime must be the reviewed small text path wrapper, not the executable binary")
        add_file(selected, root, wrapper_path, "try-runtime lowercase-path text workaround")
    summary_path = root / "live-case-summary.json"
    if summary_path.is_file():
        summary = json.loads(plain_file(root, "live-case-summary.json").read_bytes())
        expected = json.loads(plain_file(root, "expected-live-cases.json").read_bytes())
        if (summary.get("all_expected_passed") is True
                and summary.get("passed_unique_cases") == summary.get("expected_cases") == expected.get("count")
                and not summary.get("missing") and not summary.get("failed_or_skipped")
                and {row["nodeid"] for row in summary.get("accepted", [])} == set(expected["nodeids"])):
            add_file(selected, root, summary_path, "completed expected live-case matrix")
    for name in CRYPTO_DIRS:
        mp = plain_file(root, f"{name}/manifest.json")
        manifest = json.loads(mp.read_bytes())
        if not isinstance(manifest, dict) or not manifest:
            raise ValueError(f"invalid crypto manifest: {name}")
        add_file(selected, root, mp, "exact crypto manifest")
        for rel, expected in sorted(manifest.items()):
            valid_relative(rel)
            if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
                raise ValueError(f"invalid crypto hash: {name}/{rel}")
            full = f"{name}/{rel}"
            add_file(selected, root, plain_file(root, full), "exact crypto manifest entry")
            crypto_expected[full] = expected
    for name in sorted(SUITES):
        d = root / name
        if not d.is_dir() or d.is_symlink():
            continue
        containers = [d]
        containers += [p for p in sorted(d.iterdir())
                       if p.is_dir() and not p.is_symlink() and re.fullmatch(r"attempt-[0-9]+", p.name)]
        for container in containers:
            for p in sorted(container.iterdir()):
                if p.is_file() and (p.name in SUITE_EXACT or SUITE_RUN.fullmatch(p.name)
                                    or (name == "rpc" and RPC_FILES.fullmatch(p.name))):
                    add_file(selected, root, p, "suite run evidence")
                elif p.is_dir() and not p.is_symlink() and re.fullmatch(r"[a-zA-Z0-9_-]*receipts", p.name):
                    for receipt in sorted(p.iterdir()):
                        if receipt.suffix == ".json" and receipt.is_file():
                            add_file(selected, root, receipt, "public finalized receipt")
    for folder, pattern, exact in [
        ("codecs", CODEC_FILES, set()),
        ("receipt-verification", RECEIPT_FILES, set()),
        ("mainnet", None, MAINNET_EXACT),
        ("mainnet-verified", MAINNET_VERIFIED, set()),
    ]:
        d = root / folder
        if d.is_dir() and not d.is_symlink():
            for p in sorted(d.iterdir()):
                if p.is_file() and (p.name in exact or (pattern and pattern.fullmatch(p.name))):
                    add_file(selected, root, p, f"{folder} public evidence")
    return selected, crypto_expected


def normalized_key(key: str) -> str:
    return re.sub(r"[^a-z0-9]", "", key.lower())


def private_secret_corpus(root: Path) -> set[str]:
    """Read known private fixture seeds only for exact leak checks; never export."""
    secrets: set[str] = set()

    def visit(obj: object) -> None:
        if isinstance(obj, dict):
            for key, value in obj.items():
                if normalized_key(str(key)) in SENSITIVE_KEYS and isinstance(value, str) and len(value) >= 8:
                    secrets.add(value)
                    if value.startswith("0x") and len(value) >= 32:
                        secrets.add(value[2:])
                visit(value)
        elif isinstance(obj, list):
            for item in obj:
                visit(item)

    private_root = root / "crypto-live"
    for d in sorted(private_root.glob("private-wallets*")):
        if d.is_dir() and not d.is_symlink():
            for p in d.rglob("*"):
                if p.is_file() and not p.is_symlink():
                    try:
                        visit(json.loads(p.read_bytes()))
                    except (json.JSONDecodeError, UnicodeDecodeError):
                        pass
    return secrets


def check_json(obj: object, location: str = "$", issues: list[str] | None = None) -> list[str]:
    if issues is None:
        issues = []
    if isinstance(obj, dict):
        # Reject captured CLI creation output even if a phrase detector misses it.
        command = " ".join(str(obj.get(key, "")) for key in ("command", "cmd", "args", "argv"))
        if re.search(r"(?i)wallet.{0,30}create", command) and any(key in obj for key in ("stdout", "output")):
            issues.append(f"{location}: wallet-creation stdout is excluded")
        for key, value in obj.items():
            loc = location + "." + str(key)
            if normalized_key(str(key)) in SENSITIVE_KEYS:
                if isinstance(value, str) and value.strip():
                    issues.append(f"{loc}: nonempty sensitive-key string (value withheld)")
                elif isinstance(value, (list, dict)) and value:
                    issues.append(f"{loc}: sensitive-key collection (value withheld)")
            check_json(value, loc, issues)
    elif isinstance(obj, list):
        for i, item in enumerate(obj):
            check_json(item, f"{location}[{i}]", issues)
    return issues


def check_string(text: str, wordlist: set[str], secrets: set[str]) -> list[str]:
    issues: list[str] = []
    if "\x00" in text:
        issues.append("NUL byte in text evidence")
    clean = html.unescape(ANSI.sub("", text))
    for label, pattern in SECRET_MARKERS:
        for match in pattern.finditer(clean):
            issues.append(f"line {clean.count(chr(10), 0, match.start()) + 1}: {label} (value withheld)")
    # Source helpers refer to secret fields; only literal assignments are flagged.
    for match in SECRET_LITERAL.finditer(clean):
        value = match.group(2)
        if len(value.split()) >= 12 or re.fullmatch(r"(?:0x)?[0-9a-fA-F]{32,}", value):
            issues.append(f"line {clean.count(chr(10), 0, match.start()) + 1}: literal secret assignment (value withheld)")
    for secret in secrets:
        if secret in clean:
            issues.append("exact private fixture secret detected (value and source withheld)")
            break
    # Twelve consecutive English BIP39 words separated by whitespace is enough
    # to refuse publication, without depending on checksum/phrase length choices.
    run = 0
    previous_end = 0
    for word in WORD.finditer(clean):
        separator = clean[previous_end:word.start()]
        if word.group() in wordlist and (not run or separator.isspace()):
            run += 1
        else:
            run = 1 if word.group() in wordlist else 0
        if run >= 12:
            issues.append(f"line {clean.count(chr(10), 0, word.start()) + 1}: BIP39 word run (phrase withheld)")
            break
        previous_end = word.end()
    return issues


def json_strings(obj: object, location: str = "$"):
    if isinstance(obj, str):
        yield location, obj
    elif isinstance(obj, dict):
        for key, value in obj.items():
            yield from json_strings(value, location + "." + str(key))
    elif isinstance(obj, list):
        for i, item in enumerate(obj):
            yield from json_strings(item, f"{location}[{i}]")


def check_secret_text(name: str, data: bytes, wordlist: set[str], secrets: set[str]) -> list[str]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return ["not UTF-8 text"]
    issues = check_string(text, wordlist, secrets)
    if name.endswith(".json"):
        try:
            obj = json.loads(text)
            issues.extend(check_json(obj))
            # Decode JSON escaping before checking captured output and strings:
            # wrapped mnemonic words may otherwise be separated by literal \\n.
            for loc, value in json_strings(obj):
                issues.extend(f"{loc}: {issue}" for issue in check_string(value, wordlist, secrets))
        except json.JSONDecodeError as exc:
            issues.append(f"invalid/incomplete JSON at line {exc.lineno}, column {exc.colno}")
    return issues


def gzip_bytes(data: bytes) -> bytes:
    out = io.BytesIO()
    with gzip.GzipFile(fileobj=out, filename="", mode="wb", compresslevel=9, mtime=0) as stream:
        stream.write(data)
    return out.getvalue()


def atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temp = tempfile.mkstemp(prefix=".evidence-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.chmod(temp, 0o644)
        os.replace(temp, path)
    finally:
        if os.path.exists(temp):
            os.unlink(temp)


def validate_destination(dest: Path, outputs: dict[str, bytes]) -> dict[str, object]:
    # Reject symlink ancestors, including an existing destination itself.
    for part in [dest, *dest.parents]:
        if part.is_symlink():
            raise ValueError(f"destination has symlink ancestor: {part}")
    manifest_path = dest / PACKAGE_MANIFEST
    prior: dict[str, object] = {}
    owned: dict[str, str] = {}
    if manifest_path.exists():
        prior = json.loads(plain_file(dest, PACKAGE_MANIFEST).read_bytes())
        if prior.get("package_schema") != 1 or prior.get("candidate_head") != HEAD:
            raise ValueError("existing package manifest is not owned by this validation run")
        for entry in prior.get("files", []):
            name = str(entry["packaged_path"])
            valid_relative(name)
            owned[name] = str(entry["packaged_sha256"])
        for name, digest in prior.get("retained_previous_outputs", {}).items():
            valid_relative(name)
            owned[name] = digest
    for name in sorted(outputs):
        valid_relative(name)
        p = dest / name
        for ancestor in [p, *p.parents]:
            if ancestor == dest.parent:
                break
            if ancestor.is_symlink():
                raise ValueError(f"destination symlink collision: {name}")
        if p.exists():
            if name not in owned:
                raise ValueError(f"refusing to overwrite unowned destination file: {name}")
            if sha(plain_file(dest, name).read_bytes()) != owned[name]:
                raise ValueError(f"refusing to overwrite modified destination evidence: {name}")
    retained: dict[str, str] = {}
    for name, digest in sorted(owned.items()):
        if name not in outputs and (dest / name).exists():
            actual = sha(plain_file(dest, name).read_bytes())
            if actual != digest:
                raise ValueError(f"previous evidence was modified: {name}")
            retained[name] = digest
    return retained


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--repo", type=Path, default=DEFAULT_REPO)
    parser.add_argument("--destination", type=Path, help="default: REPO/" + str(DEST_REL))
    parser.add_argument("--bip39-wordlist", type=Path)
    parser.add_argument("--copy", action="store_true", help="explicitly copy the validated package; otherwise dry-run only")
    args = parser.parse_args()
    root = args.source.absolute()
    repo = args.repo.absolute()
    dest = (args.destination or repo / DEST_REL).absolute()
    if root.is_symlink() or not root.is_dir():
        raise ValueError("source must be a real directory")
    wordlist_path = args.bip39_wordlist or repo / "sdk/python/.venv/lib/python3.12/site-packages/eth_account/hdaccount/wordlist/english.txt"
    wordlist_data = wordlist_path.read_bytes()
    words = set(wordlist_data.decode().split())
    if len(words) != 2048 or sha(wordlist_data) != BIP39_ENGLISH_SHA256:
        raise ValueError("BIP39 scanner requires the pinned 2048-word English list")
    selected, crypto_expected = select(root)
    secrets = private_secret_corpus(root)
    issues: dict[str, list[str]] = {}
    outputs: dict[str, bytes] = {}
    entries: list[dict[str, object]] = []
    for name, reason in sorted(selected.items()):
        data = plain_file(root, name).read_bytes()
        digest = sha(data)
        if name in crypto_expected and digest != crypto_expected[name]:
            issues[name] = ["content no longer matches the exact crypto manifest"]
        found = check_secret_text(name, data, words, secrets)
        if found:
            issues.setdefault(name, []).extend(found)
        compress = len(data) >= COMPRESS_AT and Path(name).suffix in COMPRESS_EXTENSIONS
        packaged_path = name + ".gz" if compress else name
        packaged = gzip_bytes(data) if compress else data
        if packaged_path in outputs:
            raise ValueError(f"packaged name collision: {packaged_path}")
        outputs[packaged_path] = packaged
        entries.append({
            "source_path": name, "source_sha256": digest, "source_bytes": len(data),
            "packaged_path": packaged_path, "packaged_sha256": sha(packaged),
            "packaged_bytes": len(packaged), "compression": "gzip-mtime0" if compress else "none",
            "selection": reason,
        })
    plan = {
        "package_schema": 1, "candidate_head": HEAD,
        "source_root": str(root), "destination": str(dest),
        "compression_threshold_bytes": COMPRESS_AT,
        "scanner": {
            "bip39_wordlist_sha256": sha(wordlist_data),
            "known_private_secret_corpus_checked": bool(secrets),
            "checks": ["UTF-8 text", "JSON parses completely", "sensitive JSON fields",
                       "private-key/token/credential markers", "literal secret seed assignments",
                       "12 consecutive BIP39 words", "known private fixture secret exact match",
                       "CLI wallet-creation stdout", "exact crypto manifest hashes"],
            "limitations": "Heuristic and exact-match checks supplement a strict public-file whitelist; they are not a universal secret-detection guarantee.",
        },
        "selection_policy": {
            "suites": sorted(SUITES), "crypto_manifests": [f"{d}/manifest.json" for d in CRYPTO_DIRS],
            "future_files": "Explicit final/sdk-final/validation-summary/shutdown/cleanup/multisig-costs/mainnet-verified names; approved suite attempts and receipt directories.",
            "excluded": ["all wallet/private/keystore directories", "node databases and specs", "compiled binaries and libraries", "mainnet.snap", "unknown files", "package-plan.json", "report-draft.md", "live-case-summary.json until the expected case matrix is complete"],
        },
        "deferred_summary": "live-case-summary.json" if (root / "live-case-summary.json").exists() and "live-case-summary.json" not in selected else None,
        "files": entries, "issues": issues,
        "totals": {"files": len(entries), "source_bytes": sum(int(e["source_bytes"]) for e in entries),
                   "packaged_bytes": sum(int(e["packaged_bytes"]) for e in entries)},
    }
    atomic_write(root / PLAN, canonical_json(plan))
    if issues:
        print(json.dumps({"status": "blocked", "plan": str(root / PLAN), "issues": issues}, indent=2))
        return 1
    retained = validate_destination(dest, outputs)
    if args.copy:
        for name, data in sorted(outputs.items()):
            target = dest / name
            if not target.exists() or target.read_bytes() != data:
                atomic_write(target, data)
        package_manifest = dict(plan)
        package_manifest["retained_previous_outputs"] = retained
        # The manifest intentionally does not hash itself; its entries hash all
        # payloads and the source packaging script. It contains no wall clock.
        atomic_write(dest / PACKAGE_MANIFEST, canonical_json(package_manifest))
    print(json.dumps({"status": "copied" if args.copy else "dry-run-passed", "plan": str(root / PLAN),
                      "destination": str(dest), "totals": plan["totals"],
                      "retained_previous_outputs": len(retained)}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f"Evidence packaging refused: {exc}", file=sys.stderr)
        raise SystemExit(1) from None
