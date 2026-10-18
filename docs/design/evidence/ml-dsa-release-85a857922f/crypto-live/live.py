"""Disposable-chain disk-restore HTTP auth and exact native Rust Client Shield proof.
Private wallet material stays under private-wallets and is excluded from evidence.
"""
import asyncio, hashlib, json, os, time, traceback
from pathlib import Path
import bittensor as bt
from bittensor import http_auth
from bittensor.mldsa import install_message_capability,message_capability
from bittensor.receiving import receiving_address
from bittensor.sp_core import Keypair
from tests.e2e.test_wallet_types import alice_submission
from tests.harness.samples import dev_wallet

OUT=Path(__file__).resolve().parent
ENDPOINT='ws://127.0.0.1:26444'
REPORT={'head':'85a857922f70c218d499d2485b899fa9ef261a05','endpoint':ENDPOINT,'cases':[],'errors':[]}
def save(): (OUT/'results.json').write_text(json.dumps(REPORT,indent=2,default=str)+'\n')
def reject(f,kind):
    try:f()
    except kind:return
    raise AssertionError(f'expected {kind.__name__}')

async def snapshot(client,key):
    number=await client.finalized_block(); bh=await client._substrate.block_hash(number)
    account=await client._substrate.query('System','Account',[key.ss58_address],block_hash=bh)
    registry=await client._substrate.query('HashedAccounts','MlDsaAccounts' if key.crypto_type in (7,8) else 'Accounts',[key.ss58_address],block_hash=bh)
    return {'height':number,'hash':bh,'account':account,'registry':registry}

async def find_inner(client,hash_value,start):
    raw=client._substrate.raw; rpc=raw._session.request; end=time.monotonic()+150; seen=start-1
    while time.monotonic()<end:
        final=await client.finalized_block()
        for number in range(max(start,seen+1),final+1):
            bh=await rpc('chain_getBlockHash',[number]); block=(await rpc('chain_getBlock',[bh]))['block']
            for index,encoded in enumerate(block['extrinsics']):
                binary=bytes.fromhex(encoded.removeprefix('0x'))
                if '0x'+hashlib.blake2b(binary,digest_size=32).hexdigest()!=hash_value:continue
                events=[e for e in await raw.get_events(bh) if e.get('extrinsic_idx')==index]
                statuses=[e['event']['event_id'] for e in events if e['event']['module_id']=='System' and e['event']['event_id'] in ('ExtrinsicSuccess','ExtrinsicFailed')]
                assert len(statuses)==1,(number,index,statuses)
                assert await rpc('chain_getBlockHash',[number])==bh
                return {'block_number':number,'block_hash':bh,'extrinsic_index':index,'extrinsic_hash':hash_value,'extrinsic_hex':encoded,'success':statuses[0]=='ExtrinsicSuccess','events':events,'finalized_at':final}
            seen=number
        await asyncio.sleep(2)
    raise AssertionError(f'inner extrinsic not finalized: {hash_value}')

async def run_native(client,seed,code,call):
    composed=await client.compose(call)
    proc=await asyncio.create_subprocess_exec(str(OUT/'native_shield'),stdin=asyncio.subprocess.PIPE,stdout=asyncio.subprocess.PIPE,stderr=asyncio.subprocess.PIPE)
    request={'endpoint':ENDPOINT.replace('ws://','http://'),'seed':seed.hex(),'code':code,'call':bytes(composed.data).hex()}
    stdout,stderr=await asyncio.wait_for(proc.communicate(json.dumps(request).encode()),180)
    assert proc.returncode==0,stderr.decode()
    return json.loads(stdout)

async def one(code):
    row={'code':code,'status':'RUNNING','receipts':[]}; REPORT['cases'].append(row);save()
    try:
        seed=os.urandom(32)
        path=OUT/'private-wallets-rerun'; wallet=bt.Wallet(f'http-{code}',path=str(path))
        wallet.regenerate_hotkey(seed=seed,crypto_type=code,use_password=False,suppress=True)
        restored=bt.Wallet(f'http-{code}',path=str(path)); key=restored.hotkey
        assert key.hashed_generation==0
        row['account']=key.ss58_address; row['descriptor']=bytes(key.hashed_descriptor).hex()
        async with bt.Client(ENDPOINT,fallback_endpoints=[],archive_endpoints=[]) as client:
            assert await client.query(('AdminUtils','HashedAccountsEnabled')) is True
            assert await client.constant(('Timestamp','MinimumPeriod'))==6000
            async with alice_submission():
                funded=await client.execute(bt.intents.Transfer(receiving_address(key),50),dev_wallet(),wait_for_finalization=True,max_registration_deposit=bt.Balance.from_tao(2),shielded=True)
            row['receipts'].append(funded.to_dict()); save(); assert funded.success,funded.to_dict()
            initial=await snapshot(client,key); assert initial['registry']['generation']==0
            if code in (4,6):
                reject(lambda:http_auth.sign(restored,method='POST',path='/audit'),ValueError)
                try:await http_auth.prepare_signer(restored,client=client)
                except ValueError as exc:assert 'generation zero' in str(exc)
                else:raise AssertionError('unretired generation unexpectedly prepared')
                row['unretired_http_rejected']=True
            rotated=await client.submit_call(bt.calls.System.remark(remark='0x63727970746f'),restored,signer='hotkey',wait_for_finalization=True,shielded=False)
            row['receipts'].append(rotated.to_dict());save(); assert rotated.success,rotated.to_dict()
            restored=bt.Wallet(f'http-{code}',path=str(path));assert restored.hotkey.hashed_generation==0
            prepared=await http_auth.prepare_signer(restored,client=client)
            nonce=time.time_ns();receiver=Keypair.create_from_uri('//Bob').ss58_address
            assert receiver != Keypair.create_from_uri('//Charlie').ss58_address
            kwargs={'method':'POST','path':'/audit?x=1','body':b'fresh disk restoration','receiver_ss58':receiver,'nonce_ns':nonce}
            if code in (4,6):
                assert prepared.hashed_generation==(await snapshot(client,key))['registry']['generation']>=1
                before=await snapshot(client,key)
                headers=http_auth.sign(prepared,**kwargs)
                verify_kwargs={'method':'POST','path':'/audit?x=1','self_hotkey_ss58':receiver,'expected_crypto_type':code,'now_ns':nonce,'nonce_store':http_auth.InMemoryNonceStore()}
                caller=http_auth.verify(headers,kwargs['body'],**verify_kwargs);assert caller.hotkey_ss58==key.ss58_address
                reject(lambda:http_auth.verify(headers,kwargs['body'],**verify_kwargs),http_auth.ReplayedRequest)
                reject(lambda:http_auth.verify(headers,b'changed',**{**verify_kwargs,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.BadSignature)
                reject(lambda:http_auth.verify(headers,kwargs['body'],**{**verify_kwargs,'self_hotkey_ss58':Keypair.create_from_uri('//Charlie').ss58_address,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.WrongReceiver)
                reject(lambda:http_auth.sign(bt.Wallet(f'http-{code}',path=str(path)),method='POST',path='/audit'),ValueError)
            else:
                assert prepared.hashed_generation==0
                grant=install_message_capability(restored.hotkey,epoch=1,expires_at=await client.block()+1000,audience=b'fresh-disk-http')
                granted=await client.submit_call(grant,restored,signer='hotkey',wait_for_finalization=True,shielded=False)
                row['receipts'].append(granted.to_dict());save();assert granted.success,granted.to_dict()
                restored=bt.Wallet(f'http-{code}',path=str(path));assert restored.hotkey.hashed_generation==0
                cap=await message_capability(client._substrate,key.ss58_address,b'fresh-disk-http')
                before=await snapshot(client,key)
                req=http_auth.sign_mldsa(restored,capability=cap,**kwargs)
                verify_kwargs={'capability':cap,'method':'POST','path':'/audit?x=1','self_hotkey_ss58':receiver,'now_ns':nonce,'nonce_store':http_auth.InMemoryNonceStore()}
                verified=http_auth.verify_mldsa(req.headers,req.body,**verify_kwargs);assert verified.caller.hotkey_ss58==key.ss58_address and verified.body==kwargs['body']
                reject(lambda:http_auth.verify_mldsa(req.headers,req.body,**verify_kwargs),http_auth.ReplayedRequest)
                reject(lambda:http_auth.verify_mldsa(req.headers,req.body[:-1]+b'X',**{**verify_kwargs,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.BadSignature)
                reject(lambda:http_auth.verify_mldsa(req.headers,req.body,**{**verify_kwargs,'self_hotkey_ss58':Keypair.create_from_uri('//Charlie').ss58_address,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.WrongReceiver)
            after=await snapshot(client,key)
            assert before['registry']==after['registry'] and before['account']==after['account']
            row['http']={'status':'PASS','disk_generation_remains_zero':True,'finalized_preparation_or_capability':True,'replay_tamper_receiver_rejected':True,'offchain_state_unchanged':True,'authority_before':before,'authority_after':after}
            if code in (7,8):
                revoked=await client.submit_call(bt.calls.HashedAccounts.revoke_capability(role='Message'),restored,signer='hotkey',wait_for_finalization=True,shielded=False)
                row['receipts'].append(revoked.to_dict());save();assert revoked.success,revoked.to_dict()
                try:await message_capability(client._substrate,key.ss58_address,b'fresh-disk-http')
                except ValueError:row['http']['fresh_resolution_rejects_revoked_capability']=True
                else:raise AssertionError('revoked capability resolved')
            row['native']=[]
            recipient=Keypair.create_from_seed(os.urandom(32),1).ss58_address
            for expected,value in [(True,1_000_000),(False,10**18)]:
                before=await snapshot(client,key)
                dest_before=(await client.query(bt.storage.System.Account,[recipient]))['data']['free']
                result=await run_native(client,seed,code,bt.calls.Balances.transfer_keep_alive(dest=recipient,value=value))
                entry={'expected_inner_success':expected,'native_return':result,'authority_before':before};row['native'].append(entry);save()
                assert result['success'] and result['block_hash'] and result['data']['inner_extrinsic_hash'],result
                proof=await find_inner(client,result['data']['inner_extrinsic_hash'],before['height'])
                entry['canonical_inner']=proof;save();assert proof['success']==expected,proof
                after=await snapshot(client,key)
                dest_after=(await client.query(bt.storage.System.Account,[recipient]))['data']['free']
                assert dest_after-dest_before==(value if expected else 0)
                assert after['account']['nonce']==before['account']['nonce']+2
                assert after['registry']['generation']==before['registry']['generation']+2
                assert (after['registry']['commitment']==before['registry']['commitment'])==(code==8)
                entry.update(authority_after=after,recipient=recipient,recipient_delta=dest_after-dest_before,status='PASS',carrier_status_only=(result['success']!=proof['success']))
                save()
            row['status']='PASS_WITH_DOCUMENTED_NATIVE_CARRIER_ONLY_RESULT';save()
    except Exception as exc:
        row['status']='FAIL';row['error']=repr(exc);row['traceback']=traceback.format_exc();REPORT['errors'].append({'code':code,'error':repr(exc)});save()
        print(json.dumps({'code':code,'status':'FAIL','error':repr(exc)}),flush=True)
        return
    print(json.dumps({'code':code,'status':row['status']}),flush=True)

async def main():
    save()
    await asyncio.gather(*(one(code) for code in (4,6,7,8)))
    REPORT['status']='PASS_WITH_DOCUMENTED_NATIVE_CARRIER_ONLY_RESULT' if not REPORT['errors'] else 'FAIL'
    save()
    assert not REPORT['errors'],REPORT['errors']
asyncio.run(main())
