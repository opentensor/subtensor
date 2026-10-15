#!/usr/bin/env python3
"""Select the benchmarks a pull request must re-measure.

A benchmark is selected when its own definition (the `#[benchmark]` function)
or its `SubstrateWeight` function in `weights.rs` differs from the merge base,
so new benchmarks, edited benchmarks and hand-written weights are measured on
the reference machine. Once the weights bot has committed measurements, only
benchmarks that changed again after its latest commit are selected, so each
definition is measured once rather than on every push.

Code that a benchmark merely calls is not tracked; the scheduled full-suite
run catches drift from logic changes.

Usage: benchmark_targets.py --base REV [--head REV]
Prints `pallet:benchmark,benchmark` lines (one per pallet with targets).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from benchmark_action import find_matching_brace, find_section  # noqa: E402

# Commits made by the Validate-Benchmarks workflow (run-benchmarks.yml).
BOT_AUTHOR = "subtensor-benchmark-bot@users.noreply.github.com"
BOT_SUBJECT = "chore: update benchmark weights"
BENCHMARK_FN = re.compile(
    r"#\[benchmark(?:\([^)]*\))?\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?fn\s+(\w+)\s*\("
)
SUBSTRATE_WEIGHT_IMPL = (
    "SubstrateWeight impl",
    r"impl\s*<[^\{]*>\s*WeightInfo\s+for\s+SubstrateWeight\s*<[^\{]*>\s*\{",
    True,
)


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, check=True, text=True, capture_output=True
    ).stdout


def read_at(rev: str, path: str) -> str:
    try:
        return git("show", f"{rev}:{path}")
    except subprocess.CalledProcessError:
        return ""


def benchmarked_pallets() -> Dict[str, str]:
    """Pallet name -> pallet directory, for pallets registered for benchmarking."""
    out = subprocess.run(
        [str(ROOT / "scripts" / "discover_pallets.sh")],
        check=True,
        text=True,
        capture_output=True,
    ).stdout
    pallets = {}
    for line in out.splitlines():
        name, weights = line.split()
        pallets[name] = str(Path(weights).parent.parent)
    return pallets


def benchmark_definitions(rev: str, pallet_dir: str) -> Dict[str, str]:
    """Benchmark name -> full `#[benchmark]` function text at `rev`."""
    definitions: Dict[str, str] = {}
    for path in git("ls-tree", "-r", "--name-only", rev, f"{pallet_dir}/src").split():
        if not path.endswith(".rs"):
            continue
        text = read_at(rev, path)
        for match in BENCHMARK_FN.finditer(text):
            body_start = text.find("{", match.end())
            body_end = find_matching_brace(text, body_start)
            definitions[match.group(1)] = text[match.start() : body_end + 1]
    return definitions


def weight_functions(rev: str, pallet_dir: str) -> Dict[str, str]:
    text = read_at(rev, f"{pallet_dir}/src/weights.rs")
    section = find_section(text, *SUBSTRATE_WEIGHT_IMPL) if text else None
    return {item.name: item.text for item in section.items} if section else {}


def changed_benchmarks(old: str, new: str, pallet_dir: str) -> Set[str]:
    new_definitions = benchmark_definitions(new, pallet_dir)
    old_definitions = benchmark_definitions(old, pallet_dir)
    new_weights = weight_functions(new, pallet_dir)
    old_weights = weight_functions(old, pallet_dir)
    return {
        name
        for name, definition in new_definitions.items()
        if old_definitions.get(name) != definition
        or old_weights.get(name) != new_weights.get(name)
    }


def latest_bot_commit(base: str, head: str) -> Optional[str]:
    commits = git(
        "log",
        "--format=%H",
        f"--author={BOT_AUTHOR}",
        "--fixed-strings",
        f"--grep={BOT_SUBJECT}",
        f"{base}..{head}",
    ).split()
    return commits[0] if commits else None


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--base", required=True, help="branch or commit the PR targets")
    parser.add_argument("--head", default="HEAD")
    args = parser.parse_args()

    head = git("rev-parse", args.head).strip()
    merge_base = git("merge-base", args.base, head).strip()
    measured_at = latest_bot_commit(merge_base, head)

    targets: Dict[str, List[str]] = {}
    for pallet, pallet_dir in sorted(benchmarked_pallets().items()):
        selected = changed_benchmarks(merge_base, head, pallet_dir)
        if measured_at:
            selected &= changed_benchmarks(measured_at, head, pallet_dir)
        if selected:
            targets[pallet] = sorted(selected)

    if measured_at:
        print(f"Weights last measured at {measured_at[:12]}", file=sys.stderr)
    for pallet, names in targets.items():
        print(f"{pallet}:{','.join(names)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
