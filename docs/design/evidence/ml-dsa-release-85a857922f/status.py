from pathlib import Path
from datetime import datetime,timezone
import re,json
r=Path(__file__).parent
print(datetime.now(timezone.utc).isoformat())
for d in sorted(r.iterdir()):
 if d.is_dir():
  logs=list(d.glob('*pytest*.log'))
  for p in sorted(logs):
   s=p.read_text();passed=len(re.findall(r'\bPASSED\b',s));failed=len(re.findall(r'\bFAILED\b',s));last=s.splitlines()[-1:] or ['']
   print(f'{p.relative_to(r)} pass={passed} failed-markers={failed} last={last[0][:150]}')
for p in sorted(r.glob('*.exit')):print(p.name,p.read_text().strip())
