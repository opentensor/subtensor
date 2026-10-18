import asyncio
import json
from pathlib import Path

import bittensor as bt
from tests.e2e.test_wallet_types import alice_submission
from tests.harness.samples import dev_wallet


async def main():
    async with bt.Client(
        "ws://127.0.0.1:26444", fallback_endpoints=[], archive_endpoints=[]
    ) as client:
        assert await client.query(bt.storage.Sudo.Key) == dev_wallet().coldkey.ss58_address
        assert await client.constant(("Timestamp", "MinimumPeriod")) == 6000
        names = [
            "NetworkRegistrationAllowed", "SubtokenEnabled", "MaxRegistrationsPerBlock",
            "TargetRegistrationsPerInterval", "MinBurn", "MaxBurn", "Burn",
        ]
        before = {
            name: await client.query(("SubtensorModule", name), [0]) for name in names
        }
        builders = [
            bt.calls.AdminUtils.sudo_set_network_registration_allowed(
                netuid=0, registration_allowed=True
            ),
            bt.calls.AdminUtils.sudo_set_subtoken_enabled(netuid=0, subtoken_enabled=True),
            bt.calls.AdminUtils.sudo_set_max_registrations_per_block(
                netuid=0, max_registrations_per_block=10
            ),
            bt.calls.AdminUtils.sudo_set_target_registrations_per_interval(
                netuid=0, target_registrations_per_interval=100
            ),
            bt.calls.AdminUtils.sudo_set_min_burn(netuid=0, min_burn=100_000_000),
            bt.calls.AdminUtils.sudo_set_max_burn(netuid=0, max_burn=200_000_000),
        ]
        call = bt.calls.Utility.batch_all(calls=[await client.compose(b) for b in builders])
        async with alice_submission():
            receipt = await client.submit_call(
                bt.calls.Sudo.sudo(call=await client.compose(call)), dev_wallet(),
                wait_for_finalization=True,
            )
        assert receipt.success, receipt.to_dict()
        after = {name: await client.query(("SubtensorModule", name), [0]) for name in names}
        path = Path("/tmp/ms-release-85a857922f/costs/root-cli-parameters.json")
        path.write_text(json.dumps({"before": before, "after": after, "result": receipt.to_dict()}, indent=2))
        print(path)


asyncio.run(main())
