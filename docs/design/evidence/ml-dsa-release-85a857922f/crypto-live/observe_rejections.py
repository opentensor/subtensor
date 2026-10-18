import asyncio,json
from shared import bt,OUT,ENDPOINT
async def main():
    first=json.loads((OUT/'results.json').read_text())
    resumed=json.loads((OUT/'ms-resume-results.json').read_text())
    out={'status':'PASS','cases':[]}
    async with bt.Client(ENDPOINT,fallback_endpoints=[],archive_endpoints=[]) as client:
        final=await client.finalized_block();bh=await client._substrate.block_hash(final)
        out['height']=final;out['hash']=bh
        for code in (4,6,7,8):
            source=next(x for x in (first if code in (4,6) else resumed)['cases'] if x['code']==code)
            native=source['native'][0] if code in (4,6) else source['native']
            before=native['authority_before']
            account=await client._substrate.query('System','Account',[source['account']],block_hash=bh)
            registry=await client._substrate.query('HashedAccounts','Accounts' if code in (4,6) else 'MlDsaAccounts',[source['account']],block_hash=bh)
            assert before['account']==account and before['registry']==registry
            out['cases'].append({'code':code,'account':source['account'],'pool_rejection':native['native_return']['message'],'registry_unchanged':True,'system_account_unchanged':True,'account_after':account,'registry_after':registry})
    (OUT/'native-rejection-observations.json').write_text(json.dumps(out,indent=2)+'\n')
    print('PASS: all four rejected native submissions left finalized account and authorization state unchanged.')
asyncio.run(main())
