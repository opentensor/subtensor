import asyncio,json,subprocess,datetime,hashlib
from pathlib import Path
import websockets
D=Path(__file__).parent
async def main():
 endpoint='wss://archive.dev.opentensor.ai:443'
 async with websockets.connect(endpoint,max_size=30*1024*1024,open_timeout=30) as ws:
  counter=0
  async def rpc(method,params=[]):
   nonlocal counter
   counter+=1;await ws.send(json.dumps(dict(jsonrpc='2.0',id=counter,method=method,params=params)))
   r=json.loads(await ws.recv());assert 'error' not in r,r
   return r['result']
  genesis=await rpc('chain_getBlockHash',[0]);head=await rpc('chain_getFinalizedHead');header=await rpc('chain_getHeader',[head]);v=await rpc('state_getRuntimeVersion',[head])
  assert genesis=='0x2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03'
  data=dict(endpoint=endpoint,genesis_hash=genesis,finalized_block_hash=head,finalized_block_number=int(header['number'],16),state_root=header['stateRoot'],source_spec_name=v['specName'],source_spec_version=v['specVersion'],queried_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),acquisition={'mode':'fresh read-only archive snapshot','rpc_clients':4,'all_top_level_keys':True,'child_tries':True})
  (D/'source.json').write_text(json.dumps(data,indent=2));(D/'header.json').write_text(json.dumps(header,indent=2));print(json.dumps(data),flush=True)
 cmd=['/tmp/ms-release-85a857922f/bin/try-runtime','create-snapshot',str(D/'mainnet.snap'),'--uri']+[endpoint]*4+['--at',head]
 with (D/'snapshot.log').open('w') as out:
  p=subprocess.run(cmd,stdout=out,stderr=subprocess.STDOUT)
 (D/'snapshot.exit').write_text(str(p.returncode))
 if p.returncode:raise RuntimeError('snapshot failed')
 (D/'snapshot.sha256').write_text(hashlib.sha256((D/'mainnet.snap').read_bytes()).hexdigest()+'\n')
 print('snapshot complete',flush=True)
asyncio.run(main())
