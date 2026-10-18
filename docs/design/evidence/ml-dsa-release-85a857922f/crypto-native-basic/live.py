import asyncio,json,sys,hashlib,os,traceback
from pathlib import Path
sys.path.insert(0,'/tmp/ms-release-85a857922f/crypto-live')
from shared import bt,Keypair,snapshot,find_inner,ENDPOINT
OUT=Path(__file__).resolve().parent
REPORT={'head':'85a857922f70c218d499d2485b899fa9ef261a05','cases':[]}
def save():(OUT/'results.json').write_text(json.dumps(REPORT,indent=2,default=str)+'\n')
async def one(code):
    row={'code':code,'status':'RUNNING','transfers':[]};REPORT['cases'].append(row);save()
    try:
        wallet=bt.Wallet(f'http-{code}',path='/tmp/ms-release-85a857922f/crypto-live/private-wallets-rerun');key=wallet.hotkey
        seed=bytes.fromhex(json.loads((Path(wallet.path)/wallet.name/'hotkeys'/'default').read_text())['secretSeed'].removeprefix('0x'))
        assert key.hashed_generation==0;row['account']=key.ss58_address
        recipient=Keypair.create_from_seed(os.urandom(32),1).ss58_address
        async with bt.Client(ENDPOINT,fallback_endpoints=[],archive_endpoints=[]) as client:
            for attempt in (1,2):
                before=await snapshot(client,key)
                dest_before=(await client.query(bt.storage.System.Account,[recipient]))['data']['free']
                call=await client.compose(bt.calls.Balances.transfer_keep_alive(dest=recipient,value=1_000_000))
                proc=await asyncio.create_subprocess_exec(str(OUT/'native_transfer'),stdin=asyncio.subprocess.PIPE,stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.PIPE)
                request={'endpoint':ENDPOINT.replace('ws://','http://'),'seed':seed.hex(),'code':code,'call':bytes(call.data).hex()}
                stdout,stderr=await asyncio.wait_for(proc.communicate(json.dumps(request).encode()),180)
                assert proc.returncode==0,stderr.decode()
                result=json.loads(stdout)
                entry={'attempt':attempt,'native_result':result,'authority_before':before};row['transfers'].append(entry);save()
                assert result['success'] and result['block_hash'],result
                proof=await find_inner(client,result['extrinsic_hash'],before['height'])
                assert proof['success'];assert proof['block_hash']==result['block_hash'] and proof['extrinsic_index']==result['extrinsic_index']
                after=await snapshot(client,key)
                dest_after=(await client.query(bt.storage.System.Account,[recipient]))['data']['free']
                assert dest_after-dest_before==1_000_000
                assert after['account']['nonce']==before['account']['nonce']+1
                assert after['registry']['generation']==before['registry']['generation']+1
                assert (after['registry']['commitment']==before['registry']['commitment'])==(code==8)
                fee=result['fee_rao']; assert fee is not None
                assert before['account']['data']['free']-after['account']['data']['free']==1_000_000+fee
                assert bt.Wallet(wallet.name,path=wallet.path).hotkey.hashed_generation==0
                entry.update(status='PASS',canonical_extrinsic=proof,authority_after=after,recipient=recipient,recipient_delta=dest_after-dest_before,fee_rao=fee,disk_generation_zero=True);save()
            row['status']='PASS';save()
    except Exception as exc:row.update(status='FAIL',error=repr(exc),traceback=traceback.format_exc());save()
    print(json.dumps({'code':code,'status':row['status'],'error':row.get('error')}),flush=True)
async def main():
    await asyncio.gather(*(one(c) for c in (4,6,7,8)))
    REPORT['status']='PASS' if all(c['status']=='PASS' for c in REPORT['cases']) else 'FAIL';save()
    assert REPORT['status']=='PASS'
asyncio.run(main())
