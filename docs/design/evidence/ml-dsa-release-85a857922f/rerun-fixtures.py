import concurrent.futures,json,os,subprocess,time,xml.etree.ElementTree as E
from pathlib import Path
N=Path(__file__).parent;R=Path('/home/johnreed/Desktop/work/subtensor');P=R/'sdk/python/.venv/bin/python'
chains={c['name']:c for c in json.loads((N/'chains.json').read_text())}
suites={'extended':('test_mldsa_extended.py',3),'nested':('test_mldsa_nested_authority.py',0),'sponsorship':('test_mldsa_sponsorship_live.py',4),'release':('test_mldsa_release.py',0),'authority':('test_mldsa_localnet.py',3)}
results={}
def run(name,filename,workers):
 d=N/name;c=chains[name]
 while name not in json.loads((N/'live-results.json').read_text()):time.sleep(3)
 env={**os.environ,'PYTHONPATH':str(N/'native')+':'+str(R/'sdk/python'),'E2E_WALLET_ENDPOINT':f'ws://127.0.0.1:{c["rpc_port"]}','E2E_WALLET_ALICE_LOCK':str(d/'alice.lock'),'E2E_EVIDENCE_DIR':str(d/'attempt-2/receipts'),'E2E_EVM_ASSOCIATION_EXPECT_RATE_LIMIT':'1','E2E_WALLET_PEERS':','.join(f'ws://127.0.0.1:{c["rpc_port"]+j}' for j in range(3))}
 collection=subprocess.check_output([str(P),'-m','pytest','--collect-only','-q','tests/e2e/'+filename],env=env,cwd=R/'sdk/python',text=True)
 expected=[l for l in collection.splitlines() if l.startswith('tests/e2e/') and '::' in l]
 old=E.parse(d/'results.xml')
 passed={'tests/e2e/'+filename+'::'+case.attrib['name'] for case in old.iter('testcase') if all(case.find(tag) is None for tag in ['failure','error','skipped'])}
 targets=[t for t in expected if t not in passed]
 assert targets,(name,expected,passed)
 args=[str(P),'-m','pytest',*targets,'-vv','--tb=short','-x',f'--junitxml={d}/results-rerun1.xml']
 if workers:args+=['-n',str(workers),'--dist=load']
 with (d/'pytest-rerun1.log').open('w') as out:rc=subprocess.call(args,env=env,cwd=R/'sdk/python',stdout=out,stderr=subprocess.STDOUT)
 return dict(exit=rc,expected=expected,retained_prior_passes=sorted(set(expected)&passed),rerun=targets,reason='Update exact nonce/generation and actual-fee assertions for automatic Shield; retain unchanged previously passing cases.')
with concurrent.futures.ThreadPoolExecutor() as pool:
 fs={pool.submit(run,n,*v):n for n,v in suites.items()}
 for f in concurrent.futures.as_completed(fs):
  n=fs[f]
  try:results[n]=f.result()
  except Exception as e:results[n]={'error':repr(e)}
  (N/'fixture-rerun-results.json').write_text(json.dumps(results,indent=2));print(n,results[n],flush=True)
