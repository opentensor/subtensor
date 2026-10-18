import asyncio,json,hashlib
from pathlib import Path
import bittensor as bt
RUN=Path(__file__).parent
async def inspect(chain):
    async with bt.Client(f"ws://127.0.0.1:{chain['rpc_port']}",fallback_endpoints=[],archive_endpoints=[]) as client:
        rpc=client._substrate.raw._session.request
        genesis=await rpc('chain_getBlockHash',[0])
        final=await rpc('chain_getFinalizedHead',[])
        head=await rpc('chain_getHeader',[final])
        code=await rpc('state_getStorage',['0x3a636f6465',final])
        period=await client.constant(bt.constants.Timestamp.MinimumPeriod)
        assert period==6000,(chain['name'],period)
        return {**chain,'genesis':genesis,'finalized_block':final,'finalized_height':int(head['number'],16),'minimum_period_ms':period,'runtime_code_sha256':hashlib.sha256(bytes.fromhex(code[2:])).hexdigest()}
async def main():
    chains=json.loads((RUN/'chains.json').read_text())
    result=await asyncio.gather(*(inspect(chain) for chain in chains))
    assert len(result)==15 and len({chain['genesis'] for chain in result})==15
    expected=hashlib.sha256((RUN/'runtime.wasm').read_bytes()).hexdigest()
    assert all(chain['runtime_code_sha256']==expected for chain in result)
    (RUN/'chains-verified-final.json').write_text(json.dumps({'distinct_chains':15,'normal_slot_seconds':12,'runtime_code_matches_frozen_artifact':True,'chains':result},indent=2))
    print('All15 unique-genesis chains verified with minimum period6000ms')
asyncio.run(main())
