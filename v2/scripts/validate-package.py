#!/usr/bin/env python3
"""Build package; test extracted self-contained engine/harness with Cargo offline."""
import pathlib,subprocess,tarfile,shutil,os,json,sys
v=pathlib.Path(__file__).resolve().parents[1]
subprocess.run(['cargo','package','--manifest-path',str(v/'Cargo.toml'),'--locked','--offline','--allow-dirty'],check=True)
archive=v/'target/package/tezbar-markdown-this-0.1.0.crate';dest=v/'work/extracted-package'
if dest.exists():shutil.rmtree(dest)
dest.mkdir(parents=True)
with tarfile.open(archive,'r:gz')as t:
    for m in t.getmembers():assert (dest/m.name).resolve().is_relative_to(dest.resolve()) if hasattr(pathlib.Path,'is_relative_to')else not m.name.startswith('/')and'..'not in pathlib.PurePosixPath(m.name).parts
    t.extractall(dest)
root=dest/'tezbar-markdown-this-0.1.0'
assert not(root/'work').exists(), 'disposable work files leaked into package'
assert not(root/'target').exists(), 'build files leaked into package'
try:
    git_root=pathlib.Path(subprocess.check_output(['git','-C',str(v),'rev-parse','--show-toplevel'],text=True,stderr=subprocess.DEVNULL).strip()).resolve()
except subprocess.CalledProcessError:git_root=None
if git_root==v.parent:
    visible=set(subprocess.check_output(['git','-C',str(git_root),'ls-files','-co','--exclude-standard'],text=True).splitlines())
    for p in root.rglob('*'):
        if p.is_file() and p.name not in ['.cargo_vcs_info.json','Cargo.toml.orig']:
            assert 'v2/'+str(p.relative_to(root)) in visible,'Git-invisible archive file: '+str(p)

for p in json.loads((v/'PROVENANCE.json').read_text())['files']:assert (root/p).read_bytes()==(v/p).read_bytes(),p
for p in ['harness/Cargo.toml.in','harness/session/Cargo.toml.in','harness/tauri/Cargo.toml.in','harness/Cargo.lock.in']:assert(root/p).is_file()
assert not(root/'harness/Cargo.toml').exists()
env=os.environ.copy();env['CARGO_NET_OFFLINE']='true';env['CARGO_TARGET_DIR']=str(v/'target/extracted')
subprocess.run([sys.executable,str(root/'scripts/verify-provenance.py')],check=True)
subprocess.run([sys.executable,str(root/'scripts/verify-oracle.py')],check=True)
subprocess.run([sys.executable,str(root/'scripts/materialize-harness.py')],check=True)
for toolchain in ['stable','1.85.0']:
    for manifest in [root/'Cargo.toml',root/'work/harness/Cargo.toml']:
        subprocess.run(['cargo','+'+toolchain,'test','--manifest-path',str(manifest),'--locked','--offline'],env=env,check=True)
print('Package and extracted offline stable + MSRV engine/actual-beta harness tests pass')
