"""Task-owned localnet replay capacity; does not change flags, code, or validators.

Run only after rpc/run.py has finished. Normal 12-second authoring continues.
254 pool calls plus Timestamp/Shield inherents fill the 256-step snapshot;
256 and 257 pool calls must fail explicitly rather than return a partial state.
"""

import argparse
import asyncio
import hashlib
import json
import os
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

import bittensor as bt
import bittensor_core
from bittensor.evm.addresses import h160_to_ss58
from tests.harness.samples import dev_wallet

ROOT = Path(__file__).parent
LIMIT = "Pending execution deadline or work limit reached; retry later"
VALUE_RAO = 1_000_000


async def run(args, report):
    async with bt.Client(args.endpoint, fallback_endpoints=[], archive_endpoints=[]) as client:
        substrate = client._substrate
        raw = substrate.raw
        rpc = raw._session.request
        signer = dev_wallet("//Bob").coldkey
        ed = int(await client.constant(("Balances", "ExistentialDeposit")))
        report["genesis"] = await rpc("chain_getBlockHash", [0])
        report["runtime_sha256"] = hashlib.sha256(
            bytes.fromhex((await rpc("state_getStorage", ["0x3a636f6465"]))[2:])
        ).hexdigest()
        report["existential_deposit_rao"] = ed
        assert VALUE_RAO >= 2 * ed
        assert not await client.query(("AdminUtils", "HashedAccountsEnabled"))

        async def wait_for(check, reason, timeout=240):
            deadline = time.monotonic() + timeout
            while True:
                result = await check()
                if result:
                    return result
                assert time.monotonic() < deadline, reason
                await asyncio.sleep(0.2)

        async def empty_pool():
            return not await rpc("author_pendingExtrinsics", [])

        async def account(address, block_hash=None):
            value = await substrate.query("System", "Account", [address], block_hash=block_hash)
            return {"nonce": int(value["nonce"]), "free": int(value["data"]["free"])}

        for count in args.counts:
            await wait_for(empty_pool, "unrelated ready transactions remain")
            target = "0x" + os.urandom(20).hex()
            destination = h160_to_ss58(target)
            nonce = int(await rpc("system_accountNextIndex", [signer.ss58_address]))
            call = await client.compose(
                bt.calls.Balances.transfer_keep_alive(dest=destination, value=VALUE_RAO)
            )
            transactions = [
                await raw.create_signed_extrinsic(call, signer, nonce=nonce + i, era="00")
                for i in range(count)
            ]
            previous = await rpc("chain_getBlockHash", [])

            async def next_head():
                head = await rpc("chain_getBlockHash", [])
                return head if head != previous else None

            head = await wait_for(next_head, "next normal block did not arrive", timeout=60)
            case = {"count": count, "target": target, "snapshot_head": head, "nonce": nonce}
            report["cases"].append(case)
            started = time.monotonic()
            hashes = await asyncio.gather(
                *(rpc("author_submitExtrinsic", [tx.data_hex]) for tx in transactions)
            )
            case["submission_seconds"] = time.monotonic() - started
            case["hashes"] = hashes
            ready = await rpc("author_pendingExtrinsics", [])
            expected = {tx.data_hex.lower() for tx in transactions}
            assert expected.issubset({encoded.lower() for encoded in ready}), len(ready)
            assert len(ready) == count, len(ready)
            case["ready_count"] = len(ready)
            case["ready_bytes"] = sum((len(tx) - 2) // 2 for tx in ready)
            assert await rpc("chain_getBlockHash", []) == head, "authoring raced pool setup"
            assert int(await rpc("eth_getBalance", [target, "latest"]), 16) == 0
            before = await account(signer.ss58_address)
            assert before["nonce"] == nonce
            case["before"] = before
            case["queries"] = []
            for _ in range(2):
                started = time.monotonic()
                try:
                    reply = {"result": await rpc("eth_getBalance", [target, "pending"])}
                except Exception as error:
                    reply = {"error": str(error)}
                reply["seconds"] = time.monotonic() - started
                case["queries"].append(reply)
                args.output.write_text(json.dumps(report, indent=2) + "\n")
                assert reply["seconds"] < 15, reply
                if count <= 254:
                    assert int(reply["result"], 16) == (count * VALUE_RAO - ed) * 10**9, reply
                else:
                    assert LIMIT in reply.get("error", ""), reply
            assert await rpc("chain_getBlockHash", []) == head, "authoring raced rollback check"
            assert int(await rpc("eth_getBalance", [target, "latest"]), 16) == 0
            after = await account(signer.ss58_address)
            assert before == after, (before, after)
            case["after"] = after
            case["queries_left_latest_unchanged"] = True

            async def finalized_payment():
                finalized = await rpc("chain_getFinalizedHead", [])
                recipient = await account(destination, finalized)
                sender = await account(signer.ss58_address, finalized)
                if recipient["free"] != count * VALUE_RAO or sender["nonce"] != nonce + count:
                    return None
                return {"block": finalized, "recipient": recipient, "sender": sender}

            case["finalized"] = await wait_for(
                finalized_payment, "all accepted payments did not finalize exactly once"
            )
            case["passed"] = True
            args.output.write_text(json.dumps(report, indent=2) + "\n")
            print(json.dumps({k: case[k] for k in ("count", "ready_count", "queries", "passed")}), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", default="ws://127.0.0.1:27446")
    parser.add_argument("--counts", nargs="+", type=int, default=[80, 100, 254, 256, 257])
    parser.add_argument("--output", type=Path, default=ROOT / "pending-capacity-results.json")
    args = parser.parse_args()
    endpoint = urlparse(args.endpoint)
    assert endpoint.hostname in ("localhost", "127.0.0.1") and endpoint.port in (27444, 27445, 27446)
    assert all(count in (80, 100, 254, 256, 257) for count in args.counts)
    baseline = json.loads((ROOT / "results.json").read_text())
    assert len(baseline) == 5 and all(item["exit"] == 0 for item in baseline), baseline
    report = {
        "at": datetime.now(timezone.utc).isoformat(),
        "endpoint": args.endpoint,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "native_sha256": hashlib.sha256(Path(bittensor_core.__file__).read_bytes()).hexdigest(),
        "authoring": "normal 12-second blocks; no validator suspension or flags changed",
        "cases": [],
    }
    try:
        asyncio.run(run(args, report))
        report["passed"] = True
    except BaseException as error:
        report["error"] = repr(error)
        raise
    finally:
        args.output.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
