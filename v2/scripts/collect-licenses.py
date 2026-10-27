#!/usr/bin/env python3
"""Refresh exact license texts from the locked Cargo cache (development only)."""
import pathlib,json,hashlib,subprocess,re,tarfile
v=pathlib.Path(__file__).resolve().parents[1]
m=json.loads(subprocess.check_output(['cargo','metadata','--manifest-path',str(v/'work/harness/Cargo.toml'),'--locked','--offline','--format-version','1']))
lock=(v/'harness/Cargo.lock.in').read_text()
checksums={}
for block in lock.split('[[package]]')[1:]:
    fields=dict(re.findall(r'^(name|version|checksum) = "([^"]+)"',block,re.M))
    if 'checksum'in fields:checksums[(fields['name'],fields['version'])]=fields['checksum']
inventory=[]
for p in m['packages']:
    if not p['source']:continue
    d=pathlib.Path(p['manifest_path']).parent
    fs=[x for x in d.rglob('*')if x.is_file()and ('license'in x.name.lower()or x.name.upper().startswith('COPYING')or'LICENSES'in x.parts or p['name']=='r-efi'and x.name=='AUTHORS')]
    assert fs,(p['name'],'license text missing')
    checksum=checksums[(p['name'],p['version'])]
    crate=next(pathlib.Path.home().glob('.cargo/registry/cache/*/'+p['name']+'-'+p['version']+'.crate'))
    assert hashlib.sha256(crate.read_bytes()).hexdigest()==checksum,crate
    archive=tarfile.open(crate,'r:gz')
    rec={'name':p['name'],'version':p['version'],'spdx':p['license'],'crateChecksum':checksum,'files':[]}
    for f in fs:
        assert archive.extractfile(p['name']+'-'+p['version']+'/'+str(f.relative_to(d))).read()==f.read_bytes(),f
        dest=v/'licenses/rust'/(p['name']+'-'+p['version'])/f.relative_to(d);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(f.read_bytes())
        rec['files'].append({'path':str(dest.relative_to(v)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
    archive.close()
    inventory.append(rec)
(v/'licenses/dependencies.json').write_text(json.dumps(inventory,indent=2)+'\n')
for src,dest in [('fixtures/beta/LICENSE','licenses/Tezbar-Apache-2.0.txt'),('fixtures/markast/LICENSE','licenses/Markast-MIT.txt'),('oracle/vendor/marked/LICENSE.md','licenses/Marked-MIT.txt'),('oracle/vendor/marked-highlight/LICENSE','licenses/Marked-Highlight-MIT.txt'),('oracle/vendor/highlight.js/LICENSE','licenses/Highlight-BSD-3-Clause.txt')]:
    (v/dest).write_bytes((v/src).read_bytes())
(v/'LICENSE').write_bytes((v/'fixtures/own/LICENSE').read_bytes())
print('Exact license texts:',len(inventory),'locked Rust dependencies')
