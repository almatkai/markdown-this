// Build-time only: lower the locked Highlight.js grammars to declarative data.
// Production evaluates no JS: callbacks below are explicit native operations.
const fs=require('node:fs'),vm=require('node:vm');
const hljs=require('./vendor/highlight.js/lib/index.js');
const callbacks=[];
function callback(fn){if(!fn)return null;const s=fn.toString();let id=callbacks.indexOf(s);if(id<0){id=callbacks.length;callbacks.push(s);}return id+1;}
const names=hljs.listLanguages();const languages=[];
for(const name of names){
 hljs.highlight('',{language:name});const lang=hljs.getLanguage(name);const ids=new Map(),modes=[];
 function visit(m){if(ids.has(m))return ids.get(m);const id=modes.length;ids.set(m,id);modes.push(null);
  const o={};
  for(const key of ['scope','beginScope','endScope','skip','excludeBegin','excludeEnd','returnBegin','returnEnd','endsParent','endsWithParent','subLanguage','relevance','keywords'])if(m[key]!==undefined && !(key==='keywords'&&!m[key]))o[key]=m[key];
  for(const key of ['beginRe','endRe','keywordPatternRe','illegalRe'])if(m[key])o[key]=m[key].source;
  o.before=callback(m.__beforeBegin);o.onBegin=callback(m['on:begin']);o.onEnd=callback(m['on:end']);
  o.rules=(m.matcher?.rules||[]).map(([re,opts])=>({regex:typeof re==='string'?re:re.source,type:opts.type,...(opts.rule?{mode:visit(opts.rule)}:{})}));
  if(m.starts)o.starts=visit(m.starts);
  modes[id]=o;return id;
 }
 visit(lang);languages.push({name,aliases:lang.aliases||[],caseInsensitive:!!lang.case_insensitive,unicode:!!lang.unicodeRegex,disableAutodetect:!!lang.disableAutodetect,supersetOf:lang.supersetOf||null,classNameAliases:lang.classNameAliases,modes});
}
const mathSource=fs.readFileSync(__dirname+'/vendor/highlight.js/lib/languages/mathematica.js','utf8');
const math=vm.runInNewContext(mathSource.split('];')[0]+']; SYSTEM_SYMBOLS;');
if(callbacks.length!==8)throw Error('Callback inventory changed');
fs.writeFileSync(__dirname+'/../data/highlight-11.11.1.json',JSON.stringify({languages,mathSymbols:math}));
fs.writeFileSync(__dirname+'/callback-inventory.json',JSON.stringify(callbacks,null,2)+'\n');
console.log(languages.length,'languages,',languages.reduce((n,x)=>n+x.modes.length,0),'modes,',callbacks.length,'callbacks');
