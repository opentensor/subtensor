"""
Utilization Analysis Script

Reads on-chain state to compute dividend-efficiency-based utilization
per subnet, then applies hard cap (< 0.5 -> zero) and scaling (< 1.0 ->
multiply by utilization) to root alpha dividends.

Implements the same logic as compute_and_store_effective_root_prop() and
the utilization scaling in distribute_emission() from run_coinbase.rs.

Usage:
    python utilization_analysis.py              # Normal analysis
    python utilization_analysis.py --debug      # Debug mode for a single subnet
    python utilization_analysis.py --debug 104  # Debug mode for subnet 104
"""

import argparse
import sys

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
from substrateinterface import SubstrateInterface

plt.style.use("ggplot")

ROOT_NETUID = 0
HARD_CAP_THRESHOLD = 0.5
NANO = 1e9


def iter_storage_map(node, storage_name):
    return node.query_map("SubtensorModule", storage_name, [])


def nano_to_float(x) -> float:
    return x / NANO


def as_bits(x) -> int:
    if isinstance(x, int):
        return x
    if isinstance(x, dict) and "bits" in x:
        return int(x["bits"])
    if hasattr(x, "value"):
        return as_bits(x.value)
    return int(x)


def extract_key(k):
    """Extract (netuid, hotkey) from a storage map key."""
    if hasattr(k, "value"):
        k = k.value
    if not isinstance(k, (list, tuple)) or len(k) < 2:
        return None, None
    netuid_obj, hotkey_obj = k[0], k[1]
    netuid = int(netuid_obj.value) if hasattr(netuid_obj, "value") else int(netuid_obj)
    hotkey = str(hotkey_obj.value) if hasattr(hotkey_obj, "value") else str(hotkey_obj)
    return netuid, hotkey


def extract_value(v) -> float:
    raw = v.value if hasattr(v, "value") else v
    return nano_to_float(as_bits(raw))


def get_dividends_per_hotkey(
    node, storage_name: str, netuids: list[int]
) -> dict[int, dict[str, float]]:
    """Read per-hotkey dividends from a (netuid, hotkey) -> amount storage map."""
    wanted = set(netuids)
    result: dict[int, dict[str, float]] = {n: {} for n in netuids}
    for key, value in iter_storage_map(node, storage_name):
        netuid, hotkey = extract_key(
            key
            if isinstance(key, (list, tuple))
            else key.value
            if hasattr(key, "value")
            else key
        )
        if netuid is None or netuid not in wanted:
            continue
        amount = extract_value(value)
        if amount > 0:
            result[netuid][hotkey] = amount
    return result


def get_root_stakes(node) -> dict[str, float]:
    """Read root stake (TotalHotkeyAlpha on root netuid) for all hotkeys."""
    root_stakes: dict[str, float] = {}
    for key, value in iter_storage_map(node, "TotalHotkeyAlpha"):
        k = key if isinstance(key, (list, tuple)) else (
            key.value if hasattr(key, "value") else key
        )
        if not isinstance(k, (list, tuple)) or len(k) < 2:
            continue
        # TotalHotkeyAlpha key order: (hotkey, netuid)
        hotkey_obj, netuid_obj = k[0], k[1]
        netuid = int(netuid_obj.value) if hasattr(netuid_obj, "value") else int(netuid_obj)
        if netuid != ROOT_NETUID:
            continue
        hotkey = str(hotkey_obj.value) if hasattr(hotkey_obj, "value") else str(hotkey_obj)
        root_stakes[hotkey] = extract_value(value)
    return root_stakes


def get_subnet_hotkeys(node, netuids: list[int]) -> dict[int, set[str]]:
    """Read Keys storage to find hotkeys registered on each subnet."""
    wanted = set(netuids)
    result: dict[int, set[str]] = {n: set() for n in netuids}
    for key, value in iter_storage_map(node, "Keys"):
        k = key if isinstance(key, (list, tuple)) else (
            key.value if hasattr(key, "value") else key
        )
        if not isinstance(k, (list, tuple)) or len(k) < 1:
            continue
        netuid_obj = k[0]
        netuid = int(netuid_obj.value) if hasattr(netuid_obj, "value") else int(netuid_obj)
        if netuid not in wanted:
            continue
        hotkey = str(value.value) if hasattr(value, "value") else str(value)
        result[netuid].add(hotkey)
    return result


def compute_utilization(
    root_alpha_divs: dict[str, float],
    subnet_hotkeys: set[str],
    root_stakes: dict[str, float],
) -> float:
    """
    Compute dividend-efficiency-based utilization for a subnet.

    For each root-staked validator registered on the subnet:
        expected_share = root_stake_i / total_root_stake
        actual_share   = root_dividends_i / total_root_divs
        efficiency     = min(actual_share / expected_share, 1.0)
        utilization    = sum(root_stake_i * efficiency_i) / total_root_stake

    Only root stake of validators with UIDs on the subnet is counted.

    IMPORTANT: RootAlphaDividendsPerSubnet on chain contains post-delegation
    amounts (dividends flowed to parent hotkeys not registered on the subnet).
    The Rust utilization code uses the pre-delegation map which only contains
    registered hotkeys. We must filter to only registered hotkeys here too.
    """
    hotkey_root_stakes: list[tuple[str, float]] = []
    total_root_stake = 0.0
    for hotkey in subnet_hotkeys:
        rs = root_stakes.get(hotkey, 0.0)
        if rs > 0:
            hotkey_root_stakes.append((hotkey, rs))
            total_root_stake += rs

    if total_root_stake == 0:
        return 0.0

    # Only count root dividends for hotkeys registered on the subnet (pre-delegation).
    # Chain storage includes delegated amounts to parent hotkeys not on the subnet.
    total_root_divs = sum(root_alpha_divs.get(hk, 0.0) for hk in subnet_hotkeys)
    if total_root_divs == 0:
        return 0.0

    weighted_efficiency_sum = 0.0
    for hotkey, rs in hotkey_root_stakes:
        expected_share = rs / total_root_stake
        actual_div = root_alpha_divs.get(hotkey, 0.0)
        actual_share = actual_div / total_root_divs
        if expected_share > 0:
            efficiency = min(actual_share / expected_share, 1.0)
        else:
            efficiency = 0.0
        weighted_efficiency_sum += rs * efficiency

    return weighted_efficiency_sum / total_root_stake


def analyze_subnets(
    root_alpha_divs_per_subnet: dict[int, dict[str, float]],
    alpha_divs_per_subnet: dict[int, dict[str, float]],
    subnet_hotkeys: dict[int, set[str]],
    root_stakes: dict[str, float],
    netuids: list[int],
) -> tuple[dict[int, float], dict[int, float], dict[int, float], dict[int, float]]:
    """
    Compute utilization, raw/scaled root dividends, and effective root prop per subnet.

    Returns (utilizations, old_sums, new_sums, effective_root_props).
    """
    utilizations: dict[int, float] = {}
    old_sums: dict[int, float] = {}
    new_sums: dict[int, float] = {}
    effective_root_props: dict[int, float] = {}

    for netuid in netuids:
        root_divs = root_alpha_divs_per_subnet.get(netuid, {})
        alpha_divs = alpha_divs_per_subnet.get(netuid, {})
        old_total = sum(root_divs.values())
        alpha_total = sum(alpha_divs.values())
        old_sums[netuid] = old_total

        hotkeys = subnet_hotkeys.get(netuid, set())
        util = compute_utilization(root_divs, hotkeys, root_stakes)
        utilizations[netuid] = util

        # Apply hard cap: util < 0.5 → withhold all; util >= 0.5 → full dividends
        if old_total == 0:
            new_sums[netuid] = 0.0
        elif util < HARD_CAP_THRESHOLD:
            new_sums[netuid] = 0.0
        else:
            new_sums[netuid] = old_total

        # Compute effective root prop
        denom = alpha_total + old_total
        raw_root_prop = old_total / denom if denom > 0 else 0.0
        if old_total > 0 and util < HARD_CAP_THRESHOLD:
            effective_root_props[netuid] = 0.0
        else:
            effective_root_props[netuid] = raw_root_prop

    return utilizations, old_sums, new_sums, effective_root_props


def plot_results(
    old_sums: dict[int, float],
    new_sums: dict[int, float],
    utilizations: dict[int, float],
    netuids: list[int],
    output_path: str = "utilization_analysis.png",
):
    """Generate a two-panel plot: root dividends comparison and utilization bars."""
    active = [
        n for n in netuids if old_sums.get(n, 0) > 0 or new_sums.get(n, 0) > 0
    ]
    if not active:
        print("No active subnets to plot.")
        return

    old_vals = [old_sums.get(n, 0) for n in active]
    new_vals = [new_sums.get(n, 0) for n in active]
    utils = [utilizations.get(n, 0) for n in active]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 10))

    # Panel 1: root dividends comparison
    x = range(len(active))
    width = 0.35
    ax1.bar(x, old_vals, width, label="Raw Root Dividends")
    ax1.bar([i + width for i in x], new_vals, width, label="After Utilization Scaling")
    ax1.set_xlabel("Netuid")
    ax1.set_ylabel("Root Alpha Dividends (ALPHA)")
    ax1.set_title("Root Alpha Dividends: Before vs After Utilization Scaling + Hard Cap")
    ax1.set_xticks([i + width / 2 for i in x])
    ax1.set_xticklabels(active, rotation=90, fontsize=6)
    ax1.legend()

    # Panel 2: utilization per subnet
    colors = [
        "red" if u < HARD_CAP_THRESHOLD else "orange" if u < 1.0 else "green"
        for u in utils
    ]
    ax2.bar(range(len(active)), utils, color=colors)
    ax2.axhline(
        y=HARD_CAP_THRESHOLD,
        color="red",
        linestyle="--",
        label=f"Hard Cap ({HARD_CAP_THRESHOLD})",
    )
    ax2.axhline(y=1.0, color="green", linestyle="--", label="Full Utilization")
    ax2.set_xlabel("Netuid")
    ax2.set_ylabel("Utilization")
    ax2.set_title("Dividend-Efficiency Utilization per Subnet")
    ax2.set_xticks(range(len(active)))
    ax2.set_xticklabels(active, rotation=90, fontsize=6)
    ax2.legend()

    plt.tight_layout()
    plt.savefig(output_path, dpi=150)
    print(f"Plot saved to {output_path}")


# =========================================================================
# Chain data reader (shared between normal and debug modes)
# =========================================================================

def read_chain_data(node, netuids):
    """Read all chain state needed for analysis. Returns a dict of data."""
    print("Reading chain state...")

    print("  Reading root alpha dividends per hotkey...")
    root_alpha_divs = get_dividends_per_hotkey(
        node, "RootAlphaDividendsPerSubnet", netuids
    )

    print("  Reading alpha dividends per hotkey...")
    alpha_divs = get_dividends_per_hotkey(
        node, "AlphaDividendsPerSubnet", netuids
    )

    print("  Reading root stakes...")
    root_stakes = get_root_stakes(node)

    print("  Reading subnet hotkeys...")
    subnet_hotkeys = get_subnet_hotkeys(node, netuids)

    return {
        "root_alpha_divs": root_alpha_divs,
        "alpha_divs": alpha_divs,
        "root_stakes": root_stakes,
        "subnet_hotkeys": subnet_hotkeys,
    }


# =========================================================================
# Debug mode: per-hotkey breakdown for a single subnet
# =========================================================================

def run_debug(node, netuids, target_netuid):
    """Debug mode: show per-hotkey detail and overlap analysis for one subnet."""
    data = read_chain_data(node, netuids)
    root_alpha_divs = data["root_alpha_divs"]
    root_stakes = data["root_stakes"]
    subnet_hotkeys = data["subnet_hotkeys"]

    root_divs = root_alpha_divs.get(target_netuid, {})
    hotkeys = subnet_hotkeys.get(target_netuid, set())

    div_set = set(root_divs.keys())
    stake_set = set(root_stakes.keys())

    sep = "=" * 90
    print(f"\n{sep}")
    print(f"DEBUG: Subnet {target_netuid}")
    print(f"{sep}\n")

    print(f"Hotkeys with root dividends (post-delegation): {len(div_set)}")
    print(f"Hotkeys registered on subnet (Keys):           {len(hotkeys)}")
    print(f"Hotkeys with root stake (global):              {len(stake_set)}")

    print(f"\nRoot div hotkeys in root_stakes:  {len(div_set & stake_set)} / {len(div_set)}")
    print(f"Root div hotkeys in subnet Keys:  {len(div_set & hotkeys)} / {len(div_set)}")
    print(f"Subnet hotkeys with root stake:   {len(hotkeys & stake_set)} / {len(hotkeys)}")

    if div_set and not (div_set & stake_set):
        print("\n*** WARNING: No overlap between root_div hotkeys and root_stakes! ***")

    # Find root-staked validators on this subnet
    hotkey_rs: list[tuple[str, float]] = []
    total_root_stake = 0.0
    for hk in hotkeys:
        rs = root_stakes.get(hk, 0.0)
        if rs > 0:
            hotkey_rs.append((hk, rs))
            total_root_stake += rs

    # Only count registered-hotkey root divs (pre-delegation)
    registered_root_divs = sum(root_divs.get(hk, 0.0) for hk in hotkeys)
    total_root_divs_all = sum(root_divs.values())

    print(f"\nRoot-staked validators on subnet: {len(hotkey_rs)}")
    print(f"Total root stake on subnet:       {total_root_stake:.2f}")
    print(f"Root divs (registered only):      {registered_root_divs:.6f}")
    print(f"Root divs (all, post-delegation): {total_root_divs_all:.6f}")

    if total_root_stake > 0 and registered_root_divs > 0:
        print(f"\n{'Hotkey':>20} {'Root Stake':>12} {'Expected':>10} "
              f"{'Actual Div':>12} {'Actual Share':>12} {'Efficiency':>12}")
        print("-" * 82)

        weighted_eff_sum = 0.0
        for hk, rs in sorted(hotkey_rs, key=lambda x: -x[1]):
            expected = rs / total_root_stake
            actual_div = root_divs.get(hk, 0.0)
            actual = actual_div / registered_root_divs if registered_root_divs > 0 else 0.0
            eff = min(actual / expected, 1.0) if expected > 0 else 0.0
            weighted_eff_sum += rs * eff
            print(
                f"{hk[:20]:>20} {rs:>12.2f} {expected:>10.6f} "
                f"{actual_div:>12.6f} {actual:>12.6f} {eff:>12.6f}"
            )

        util = weighted_eff_sum / total_root_stake
        status = "HARD-CAP" if util < HARD_CAP_THRESHOLD else "ACTIVE"
        print(f"\nUtilization: {util:.6f} ({status})")
    else:
        print("\nUtilization: 0.000000 (no root stake or no root divs)")


# =========================================================================
# Normal mode: full analysis across all subnets
# =========================================================================

def run_analysis(node, netuids):
    """Normal mode: analyze all subnets, print table, generate plot."""
    data = read_chain_data(node, netuids)

    print("Computing utilization and scaling...")
    utilizations, old_sums, new_sums, effective_root_props = analyze_subnets(
        data["root_alpha_divs"],
        data["alpha_divs"],
        data["subnet_hotkeys"],
        data["root_stakes"],
        netuids,
    )

    # Categorize subnets
    hard_capped = [
        n for n in netuids
        if utilizations.get(n, 0) < HARD_CAP_THRESHOLD and old_sums.get(n, 0) > 0
    ]
    active = [
        n for n in netuids
        if utilizations.get(n, 0) >= HARD_CAP_THRESHOLD and old_sums.get(n, 0) > 0
    ]

    sep = "=" * 90
    print(f"\n{sep}")
    print("UTILIZATION ANALYSIS")
    print(f"{sep}\n")

    print(
        f"Hard-capped subnets (util < {HARD_CAP_THRESHOLD}, all root divs recycled): "
        f"{hard_capped}"
    )
    print(f"Active subnets (util >= {HARD_CAP_THRESHOLD}, full dividends): {active}")

    header = (
        f"{'Netuid':>8} {'Utilization':>12} {'Raw Root Divs':>15} "
        f"{'Effective Root Divs':>20} {'ERP':>12} {'Status':>12}"
    )
    print(f"\n{header}")
    print("-" * 90)
    for netuid in netuids:
        util = utilizations.get(netuid, 0)
        old = old_sums.get(netuid, 0)
        new = new_sums.get(netuid, 0)
        erp = effective_root_props.get(netuid, 0)
        if old > 0 or new > 0:
            status = "HARD-CAP" if util < HARD_CAP_THRESHOLD and old > 0 else "ACTIVE"
            print(
                f"{netuid:>8} {util:>12.6f} {old:>15.2f} "
                f"{new:>20.2f} {erp:>12.8f} {status:>12}"
            )

    total_old = sum(old_sums.values())
    total_new = sum(new_sums.values())
    recycled = total_old - total_new
    print(f"\nTotal raw root dividends:    {total_old:.2f}")
    print(f"Total after scaling:         {total_new:.2f}")
    if total_old > 0:
        pct = recycled / total_old * 100
        print(f"Total recycled:              {recycled:.2f} ({pct:.1f}%)")

    plot_results(old_sums, new_sums, utilizations, netuids)


def main():
    parser = argparse.ArgumentParser(
        description="Analyze dividend-efficiency utilization per subnet"
    )
    parser.add_argument(
        "--debug",
        nargs="?",
        const=1,
        type=int,
        metavar="NETUID",
        help="Debug mode: show per-hotkey breakdown for a single subnet (default: 1)",
    )
    args = parser.parse_args()

    node = SubstrateInterface(url="wss://entrypoint-finney.opentensor.ai:443")
    netuids = list(range(1, 129))

    if args.debug is not None:
        run_debug(node, netuids, args.debug)
    else:
        run_analysis(node, netuids)


if __name__ == "__main__":
    main()
