from pathlib import Path
import hashlib,json,subprocess
root=Path('/home/johnreed/Desktop/work/subtensor'); out=Path(__file__).resolve().parent; release=root/'target/release'; fp=release/'.fingerprint'
start=max(fp.glob('bittensor-core-py-*/lib-bittensor_core.json'),key=lambda p:p.stat().st_mtime_ns)
index={}
for path in fp.glob('*/lib-*'):
    if path.suffix:continue
    try:index[int.from_bytes(bytes.fromhex(path.read_text()),'little')]=path.with_suffix('.json')
    except ValueError:pass
selected={}
def walk(path):
    for _,name,_,digest in json.loads(path.read_text())['deps']:
        dep=index.get(digest)
        if dep is not None and name not in selected:selected[name]=dep;walk(dep)
walk(start)
def library(name):
    p=selected[name];return release/'deps'/f'lib{p.stem.removeprefix("lib-")}-{p.parent.name.rsplit("-",1)[1]}.rlib'
libs={name:library(name) for name in ['bittensor_core','serde_json','hex']}
command=['/home/johnreed/.cargo/bin/rustc','--edition=2021','-O',str(out/'native_shield.rs'),'-o',str(out/'native_shield'),'-L',f'dependency={release/"deps"}']
for name,path in libs.items():command+=['--extern',f'{name}={path}']
subprocess.run(command,cwd=root,check=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
(out/'native-artifacts.json').write_text(json.dumps({'native_fingerprint':str(start),'libraries':{k:{'path':str(v),'sha256':sha(v)} for k,v in libs.items()},'source_sha256':sha(out/'native_shield.rs'),'binary_sha256':sha(out/'native_shield'),'command':command},indent=2)+'\n')
print('Native Rust Client harness built against matching SDK graph')
