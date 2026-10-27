#!/usr/bin/env python3
import copy,json,pathlib
from jsonschema import Draft202012Validator
v=pathlib.Path(__file__).resolve().parents[1]
schema=json.loads((v/'fixtures/beta/schemas/extension-v2.schema.json').read_text());Draft202012Validator.check_schema(schema);validator=Draft202012Validator(schema)
manifest=json.loads((v/'extension.json').read_text());validator.validate(manifest)
negatives=[]
def bad(path,value):
    x=copy.deepcopy(manifest);ref=x
    for p in path[:-1]:ref=ref[p]
    ref[path[-1]]=value;negatives.append(x)
for path,value in [(['schemaVersion'],1),(['runtime'],'javascript'),(['id'],'raycast.markdown-this'),(['slug'],'../escape'),(['icon'],'/tmp/icon.png'),(['author'],'forbidden'),(['unexpected'],True),(['replaces'],[]),(['replaces'],['raycast.markdown-this']*2),(['preferences'],{}),(['commands'],[]),(['commands',0,'runtime'],'bun'),(['commands',0,'platforms'],[]),(['commands',0,'platforms'],['Solaris']),(['platforms'],['macOS']*2),(['verification'],[]),(['verification',0,'state'],'verified'),(['verification',0,'platform'],'Solaris')]:bad(path,value)
for required in schema['required']:
    x=copy.deepcopy(manifest);del x[required];negatives.append(x)
for x in negatives:assert list(validator.iter_errors(x)),'negative unexpectedly accepted'
assert manifest['id']=='com.tezbar.native.markdown-this'and manifest['replaces']==['raycast.markdown-this']
assert manifest['preferences']==[]and [c['name']for c in manifest['commands']]==['index']
assert [(p['platform'],p['state'])for p in manifest['verification']]==[('macOS','not-run'),('Windows','not-run')]
assert 'author'not in manifest
print('Full pinned schema: manifest valid;',len(negatives),'negative cases rejected')
