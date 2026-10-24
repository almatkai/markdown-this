// Executes the unchanged checked-in TypeScript source; only imports are supplied
// through a lexical vm binding. Clipboard and Toast are strict in-memory stubs.
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const root = __dirname;
const { marked } = require('./vendor/marked/lib/marked.cjs');
const { markedHighlight } = require('./vendor/marked-highlight/lib/index.cjs');
const hljs = require('./vendor/highlight.js/lib/index.js');
const original = fs.readFileSync(path.join(root, '../fixtures/own/src/index.ts'),'utf8');
const importRE = /^import (.+?) from "(.+?)";$/gm;
let imported = [];
const executable = original.replace(importRE, (_, names, module) => {
    imported.push(module);
    if (module === '@raycast/api') return 'const {Clipboard, Toast, showToast} = bindings;';
    if (module === 'marked') return 'const {marked} = bindings;';
    if (module === 'marked-highlight') return 'const {markedHighlight} = bindings;';
    if (module === 'highlight.js') return 'const hljs = bindings.hljs;';
    if (module.startsWith('highlight.js/lib/languages/')) return `const ${names} = bindings.requireLanguage(${JSON.stringify(module.split('/').pop())});`;
    throw Error('Unexpected source import '+module);
}).replace('export default async function Command()', 'async function Command()');
if (imported.length !== 9) throw Error('Source import count drift');
const calls = [];let input;
const bindings = {
    marked,markedHighlight,hljs,
    requireLanguage: name => require('./vendor/highlight.js/lib/languages/'+name+'.js'),
    Clipboard: {readText: async()=>{calls.push(['read']); return input;},copy: async text=>{calls.push(['copy',text]);}},
    Toast: {Style:{Failure:'failure',Success:'success'}},
    showToast: async x=>{calls.push(['toast',x]);},
};
const context = vm.createContext({bindings,console:{error(){}}});
const command = vm.runInContext(executable+'\nCommand;',context,{filename:'unchanged-own-index.ts'});
module.exports = async function(text) {input=text;calls.length=0;await command();return JSON.parse(JSON.stringify(calls));};
if(require.main===module){
 const inputs=JSON.parse(fs.readFileSync(0,'utf8'));
 (async()=>{let out=[];for(const x of inputs)out.push({input:x,calls:await module.exports(x)});process.stdout.write(JSON.stringify(out));})();
}
