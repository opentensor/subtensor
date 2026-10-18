"""Collect unique final live case outcomes, retaining all earlier attempts."""
import json
from pathlib import Path
import xml.etree.ElementTree as E
R=Path(__file__).parent
expected=json.loads((R/'expected-live-cases.json').read_text())['nodeids']
rows=[]
for path in sorted(R.glob('*/*.xml'),key=lambda p:p.stat().st_mtime):
    try: cases=E.parse(path).iter('testcase')
    except E.ParseError: continue
    for case in cases:
        classname=case.get('classname','')
        if not classname.startswith('tests.e2e.'): continue
        nodeid=classname.replace('.','/')+'.py::'+case.attrib['name']
        status=next((name for name in ['failure','error','skipped'] if case.find(name) is not None),'passed')
        rows.append({'nodeid':nodeid,'status':status,'seconds':float(case.get('time','0')),'evidence':str(path.relative_to(R)),'written_at':path.stat().st_mtime})
for path in R.glob('costs/*accepted-nodeids.json'):
    obj=json.loads(path.read_text());assert obj['exit_code']==0
    for nodeid in obj['accepted_nodeids']:
        rows.append({'nodeid':nodeid,'status':'passed','evidence':str(path.relative_to(R)),'written_at':path.stat().st_mtime,'source_kind':'verbose pytest log plus exit0; no XML'})
rows.sort(key=lambda x:x['written_at'])
latest={row['nodeid']:row for row in rows}
accepted=[latest[n] for n in expected if n in latest and latest[n]['status']=='passed']
missing=[n for n in expected if n not in latest]
failed=[latest[n] for n in expected if n in latest and latest[n]['status']!='passed']
result={'expected_cases':len(expected),'passed_unique_cases':len(accepted),'all_expected_passed':len(accepted)==len(expected),'missing':missing,'failed_or_skipped':failed,'accepted':accepted,'all_attempts':rows,'not_in_current_expected_matrix':sorted(set(latest)-set(expected))}
(R/'live-case-summary.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['expected_cases','passed_unique_cases','all_expected_passed','missing','failed_or_skipped']}))
