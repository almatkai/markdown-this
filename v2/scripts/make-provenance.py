#!/usr/bin/env python3
"""Refresh hash inventory after reviewing changes to pinned or generated files."""
import pathlib,json,hashlib,subprocess
v=pathlib.Path(__file__).resolve().parents[1];root=v.parent
sha='4515628ec7698893f251782f9429c5c0d46f7e76'
def digest(b):return hashlib.sha256(b).hexdigest()
files={}
for folder in ['fixtures','oracle','data','assets','licenses']:
    for p in sorted((v/folder).rglob('*')):
        if p.is_file():files[str(p.relative_to(v))]={'bytes':p.stat().st_size,'sha256':digest(p.read_bytes())}
for name in ['Cargo.lock','harness/Cargo.lock.in','LICENSE','extension.json']:
    p=v/name;files[name]={'bytes':p.stat().st_size,'sha256':digest(p.read_bytes())}
baseline={}
for p in subprocess.check_output(['git','-C',str(root),'ls-tree','-r','--name-only',sha],text=True).splitlines():
    b=subprocess.check_output(['git','-C',str(root),'show',sha+':'+p]);baseline[p]={'bytes':len(b),'sha256':digest(b)}
lock=json.loads((v/'fixtures/own/package-lock.json').read_text())
record={'ownRepository':'https://github.com/almatkai/markdown-this','ownCommit':sha,'raycastRepository':'https://github.com/raycast/extensions','raycastCommit':'6734d85992290066fd31a7f5bee3cca3d3545a1a','raycastPath':'extensions/markdown-this','betaCommit':'fd300e276356587726566026f072563eb9c34cb1','markastRepository':'https://github.com/ericyangpan/markast','markastCommit':'60d85897e043f67761792f5fa3ec9243e2812198','markedFixtureCommit':'dba1327fb66cface17dd7e0e175ea2916eee74cf','npmPackages':{n:lock['packages']['node_modules/'+n]for n in ['marked','marked-highlight','highlight.js']},'v1Baseline':baseline,'files':files}
(v/'PROVENANCE.json').write_text(json.dumps(record,indent=2,ensure_ascii=False)+'\n')
print(len(files),'artifact hashes;',len(baseline),'V1 baseline file hashes')
