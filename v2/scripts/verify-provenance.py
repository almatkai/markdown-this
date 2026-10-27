#!/usr/bin/env python3
import pathlib,json,hashlib,subprocess,sys
v=pathlib.Path(__file__).resolve().parents[1];record=json.loads((v/'PROVENANCE.json').read_text())
def digest(b):return hashlib.sha256(b).hexdigest()
for p,expected in record['files'].items():
    b=(v/p).read_bytes();assert len(b)==expected['bytes']and digest(b)==expected['sha256'],p
for name,pkg in record['npmPackages'].items():
    actual=json.loads((v/'oracle/vendor'/name/'package.json').read_text());assert actual['version']==pkg['version']
icon=(v/'assets/command-icon.png').read_bytes();assert icon==(v/'fixtures/original/assets/command-icon.png').read_bytes()
assert len(icon)==196200 and digest(icon)=='7e0a1ddf92be463dd2ae7335fdbe00d33f944ba012c33d784d652f5c388727f3'
for dep in json.loads((v/'licenses/dependencies.json').read_text()):
    assert dep['files']
    for f in dep['files']:assert digest((v/f['path']).read_bytes())==f['sha256']
assert (v/'LICENSE').read_bytes()==(v/'fixtures/own/LICENSE').read_bytes()
if '--git' in sys.argv:
    root=v.parent;actual_root=subprocess.check_output(['git','-C',str(root),'rev-parse','--show-toplevel'],text=True).strip();assert pathlib.Path(actual_root).resolve()==root
    visible=set(subprocess.check_output(['git','-C',str(root),'ls-files','-co','--exclude-standard'],text=True).splitlines())
    assert all('v2/'+p in visible for p in record['files']),[p for p in record['files']if 'v2/'+p not in visible]
    for p,expected in record['v1Baseline'].items():
        if p=='README.md':continue
        assert digest((root/p).read_bytes())==expected['sha256'],'V1 changed: '+p
    original=subprocess.check_output(['git','-C',str(root),'show',record['ownCommit']+':README.md'])
    assert (root/'README.md').read_bytes()==original+b'\n\nNative V2 Rust implementation: [v2/README.md](v2/README.md).\n'
    assert subprocess.check_output(['git','-C',str(root),'diff','--cached','--name-only'],text=True)==''
print('Exact icon,',len(record['files']),'provenance hashes and license text hashes verified'+('; V1 and Git visibility verified'if '--git'in sys.argv else ''))
