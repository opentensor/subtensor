"""
Inspect mainnet state for orphaned Axon/Prometheus/NeuronCertificate entries.

An entry is "orphaned" when the hotkey is not registered on that subnet (not in Uids),
yet has an entry in Axons, Prometheus, or NeuronCertificates for that netuid.

This can happen because serve_axon / serve_prometheus used
is_hotkey_registered_on_any_network instead of is_hotkey_registered_on_network.

Usage:
    scripts/.venv/bin/python scripts/inspect_orphan_axons.py
"""

from substrateinterface import SubstrateInterface

MAINNET_URL = "wss://entrypoint-finney.opentensor.ai:443"

def query_double_map_keys(substrate, pallet, storage, block_hash):
    """Return all (key1, key2) pairs from a StorageDoubleMap."""
    result = substrate.query_map(
        module=pallet,
        storage_function=storage,
        block_hash=block_hash,
    )
    keys = []
    for (key, _value) in result:
        # key is a list of two decoded keys for a double map
        keys.append((key[0].value, key[1].value))
    return keys

def main():
    print(f"Connecting to {MAINNET_URL} ...")
    substrate = SubstrateInterface(url=MAINNET_URL)

    block_hash = substrate.get_block_hash()
    print(f"At block hash: {block_hash}\n")

    # --- Build registered set: (netuid, hotkey) ---
    print("Fetching Uids (registered neurons) ...")
    uids_entries = substrate.query_map(
        module="SubtensorModule",
        storage_function="Uids",
        block_hash=block_hash,
    )
    registered = set()
    for (key, _value) in uids_entries:
        netuid = key[0].value
        hotkey = key[1].value
        registered.add((netuid, hotkey))
    print(f"  {len(registered)} registered (netuid, hotkey) pairs")

    # --- Check Axons ---
    print("\nFetching Axons ...")
    axon_keys = query_double_map_keys(substrate, "SubtensorModule", "Axons", block_hash)
    print(f"  {len(axon_keys)} total Axons entries")
    orphan_axons = [(n, h) for (n, h) in axon_keys if (n, h) not in registered]
    print(f"  {len(orphan_axons)} orphaned Axons entries (hotkey not registered on that netuid)")

    # --- Check Prometheus ---
    print("\nFetching Prometheus ...")
    prom_keys = query_double_map_keys(substrate, "SubtensorModule", "Prometheus", block_hash)
    print(f"  {len(prom_keys)} total Prometheus entries")
    orphan_prom = [(n, h) for (n, h) in prom_keys if (n, h) not in registered]
    print(f"  {len(orphan_prom)} orphaned Prometheus entries")

    # --- Check NeuronCertificates ---
    print("\nFetching NeuronCertificates ...")
    cert_keys = query_double_map_keys(substrate, "SubtensorModule", "NeuronCertificates", block_hash)
    print(f"  {len(cert_keys)} total NeuronCertificates entries")
    orphan_certs = [(n, h) for (n, h) in cert_keys if (n, h) not in registered]
    print(f"  {len(orphan_certs)} orphaned NeuronCertificates entries")

    total_orphans = len(orphan_axons) + len(orphan_prom) + len(orphan_certs)
    print(f"\n=== TOTAL orphaned entries: {total_orphans} ===")

    if orphan_axons:
        print("\nOrphaned Axon entries by netuid:")
        by_net = {}
        for (n, h) in orphan_axons:
            by_net.setdefault(n, []).append(h)
        for netuid, hotkeys in sorted(by_net.items()):
            print(f"  netuid={netuid}: {len(hotkeys)} orphans")

    if orphan_prom:
        print("\nOrphaned Prometheus entries by netuid:")
        by_net = {}
        for (n, h) in orphan_prom:
            by_net.setdefault(n, []).append(h)
        for netuid, hotkeys in sorted(by_net.items()):
            print(f"  netuid={netuid}: {len(hotkeys)} orphans")

    if orphan_certs:
        print("\nOrphaned NeuronCertificate entries by netuid:")
        by_net = {}
        for (n, h) in orphan_certs:
            by_net.setdefault(n, []).append(h)
        for netuid, hotkeys in sorted(by_net.items()):
            print(f"  netuid={netuid}: {len(hotkeys)} orphans")

    return total_orphans

if __name__ == "__main__":
    total = main()
    print(f"\nDecision threshold: >1000 total orphans => use sudo extrinsic; <=1000 => simple migration")
    print(f"Result: {total} orphans => {'SUDO EXTRINSIC' if total > 1000 else 'MIGRATION'}")
