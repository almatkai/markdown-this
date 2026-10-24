#!/usr/bin/env python3
"""Development diagnostic; assertions in tests/oracle.rs are the release gate."""
import json, subprocess, pathlib, sys
v=pathlib.Path(__file__).resolve().parents[1]
cs=json.loads((v/'oracle/cases.json').read_text());cs=[c for c in cs if isinstance(c['input'],str)]
p=subprocess.run([str(v/'target/debug/examples/convert')],input=json.dumps([c['input']for c in cs]),text=True,capture_output=True)
if p.returncode:sys.exit(p.stderr)
out=json.loads(p.stdout);bad=[];pre=(v/'data/document-prefix.html').read_text();suf=(v/'data/document-suffix.html').read_text()
for c,r in zip(cs,out):
 copies=[x[1]for x in c['calls']if x[0]=='copy']
 if copies and r.get('Ok')!=copies[0] or not copies and 'Err'not in r:bad.append((c,r))
(v/'work').mkdir(exist_ok=True);(v/'work/mismatches.json').write_text(json.dumps(bad,ensure_ascii=False,indent=2))
print('mismatches',len(bad),'of',len(cs))
for c,r in bad[:int(sys.argv[1]) if len(sys.argv)>1 else 15]:
 e=next((x[1]for x in c['calls']if x[0]=='copy'),'');a=r.get('Ok')
 print(c['id'],repr(c['input'])[:150]);print('EXPECTED',repr(e[len(pre):-len(suf)]));print('ACTUAL',repr(a[len(pre):-len(suf)])if a is not None else r)
