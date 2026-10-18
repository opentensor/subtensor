import asyncio,json,time,traceback
from pathlib import Path
from shared import bt,http_auth,Keypair,install_message_capability,message_capability,snapshot,reject,run_native,OUT,ENDPOINT
REPORT={'head':'85a857922f70c218d499d2485b899fa9ef261a05','cases':[]}
def save():(OUT/'ms-resume-results.json').write_text(json.dumps(REPORT,indent=2,default=str)+'\n')
async def one(code):
    row={'code':code,'status':'RUNNING','receipts':[]};REPORT['cases'].append(row);save()
    try:
        while next(c for c in json.loads((OUT/'results.json').read_text())['cases'] if c['code']==code)['status']=='RUNNING':await asyncio.sleep(2)
        wallet=bt.Wallet(f'http-{code}',path=str(OUT/'private-wallets-rerun'));key=wallet.hotkey
        assert key.hashed_generation==0;row['account']=key.ss58_address
        async with bt.Client(ENDPOINT,fallback_endpoints=[],archive_endpoints=[]) as client:
            prepared=await http_auth.prepare_signer(wallet,client=client)
            assert prepared.crypto_type==code and prepared.ss58_address==key.ss58_address
            grant=install_message_capability(key,epoch=1,expires_at=await client.block()+1000,audience=b'fresh-disk-http')
            granted=await client.submit_call(grant,wallet,signer='hotkey',wait_for_finalization=True,shielded=False)
            row['receipts'].append(granted.to_dict());save();assert granted.success,granted.to_dict()
            restored=bt.Wallet(f'http-{code}',path=str(OUT/'private-wallets-rerun'));assert restored.hotkey.hashed_generation==0
            cap=await message_capability(client._substrate,key.ss58_address,b'fresh-disk-http')
            before=await snapshot(client,key)
            nonce=time.time_ns(); receiver=Keypair.create_from_uri('//Bob').ss58_address
            wrong=Keypair.create_from_uri('//Charlie').ss58_address;assert wrong!=receiver
            request=http_auth.sign_mldsa(restored,capability=cap,method='POST',path='/audit?x=1',body=b'fresh disk restoration',receiver_ss58=receiver,nonce_ns=nonce)
            args={'capability':cap,'method':'POST','path':'/audit?x=1','self_hotkey_ss58':receiver,'now_ns':nonce,'nonce_store':http_auth.InMemoryNonceStore()}
            verified=http_auth.verify_mldsa(request.headers,request.body,**args)
            assert verified.caller.hotkey_ss58==key.ss58_address and verified.body==b'fresh disk restoration'
            reject(lambda:http_auth.verify_mldsa(request.headers,request.body,**args),http_auth.ReplayedRequest)
            reject(lambda:http_auth.verify_mldsa(request.headers,request.body[:-1]+b'X',**{**args,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.BadSignature)
            reject(lambda:http_auth.verify_mldsa(request.headers,request.body,**{**args,'self_hotkey_ss58':wrong,'nonce_store':http_auth.InMemoryNonceStore()}),http_auth.WrongReceiver)
            after=await snapshot(client,key)
            assert before['registry']==after['registry'] and before['account']==after['account']
            row['http']={'status':'PASS','disk_generation_remains_zero':True,'finalized_capability':True,'replay_tamper_receiver_rejected':True,'offchain_state_unchanged':True,'authority_before':before,'authority_after':after};save()
            revoked=await client.submit_call(bt.calls.HashedAccounts.revoke_capability(role='Message'),restored,signer='hotkey',wait_for_finalization=True,shielded=False)
            row['receipts'].append(revoked.to_dict());save();assert revoked.success,revoked.to_dict()
            try:await message_capability(client._substrate,key.ss58_address,b'fresh-disk-http')
            except ValueError:row['http']['fresh_resolution_rejects_revoked_capability']=True
            else:raise AssertionError('revoked capability resolved')
            before=await snapshot(client,key)
            seed=bytes.fromhex(json.loads((OUT/'private-wallets-rerun'/f'http-{code}'/'hotkeys'/'default').read_text())['secretSeed'].removeprefix('0x'))
            result=await run_native(client,seed,code,bt.calls.Balances.transfer_keep_alive(dest=receiver,value=1_000_000))
            after=await snapshot(client,key)
            row['native']={'native_return':result,'authority_before':before,'authority_after':after,'status':'PREEXISTING_MORTALITY_REJECTION'};save()
            assert not result['success'] and 'Transaction is outdated' in result['message']
            assert before['registry']==after['registry'] and before['account']==after['account']
            row['status']='HTTP_PASS_NATIVE_PREEXISTING_MORTALITY_REJECTION';save()
    except Exception as exc:
        row.update(status='FAIL',error=repr(exc),traceback=traceback.format_exc());save()
    print(json.dumps({'code':code,'status':row['status'],'error':row.get('error')}),flush=True)
async def main():
    await asyncio.gather(*(one(c) for c in (7,8)));REPORT['status']='PASS' if all(c['status']=='HTTP_PASS_NATIVE_PREEXISTING_MORTALITY_REJECTION' for c in REPORT['cases']) else 'FAIL';save()
asyncio.run(main())
