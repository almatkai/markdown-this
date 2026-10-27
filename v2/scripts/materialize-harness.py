#!/usr/bin/env python3
"""Materialize a self-contained, relocatable headless beta harness from templates.
Nested Cargo packages aren't included by cargo package; only templates are shipped.
"""
import pathlib, shutil
root=pathlib.Path(__file__).resolve().parents[1]
dest=root/'work/harness'
if dest.exists():shutil.rmtree(dest)
dest.mkdir(parents=True)
shutil.copytree(root,dest/'engine',ignore=shutil.ignore_patterns('target','work','.git','work-*.json'))
for p in (root/'harness').rglob('*'):
    if not p.is_file():continue
    rel=p.relative_to(root/'harness');out=dest/rel
    if out.suffix=='.in':out=out.with_suffix('')
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(p.read_bytes().replace(b'@@ENGINE@@',str(dest/'engine').encode()))
print(dest/'Cargo.toml')
