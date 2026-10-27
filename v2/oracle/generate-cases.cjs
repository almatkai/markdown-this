const fs=require('node:fs');const oracle=require('./source-oracle.cjs');
let cases=[null, '', ' ', '\ufeff\u00a0\n', '\u0085', '\u200b', '# Hello', 'hello\nworld', 'hello\r\nworld','\tcode', '    x\n\n    y\n', '<div>raw &copy;</div>', '<div>raw</div>\n', '<script>alert(1)</script>\n\ntext', '<!--a-->\n\ntext', '<textarea>**hi**\n&x;</textarea>', 'a  \nb', 'a\\\nb', '[ref][x]\n\n[x]: /url "Title"', '[x](javascript:alert(1))', '<https://x.test/a?b=1&c=2>', 'www.example.com, x@y.test', '~~gone~~', '|a|b|\n|:-|--:|\n|c|d|', '- [x] done\n- [ ] todo', '> # Header\n> a\n> b', '- a\n  - b\n\n- c', '3. a\n4. b', '<svg><path /></svg>','<img src="x" onerror="x">','<span>hi\nthere</span>','日本語 😀 **қазақ**','&copy; &#123; &#x1f600; &unknown; &amp; & text', '# x #\n\n---\n\ntext', '` &amp; < " \' `'];
const inline=['plain','*em*','**strong**','***both***','~~del~~','`code`','[link](https://x.test/?a=1&b=2 "title")','![alt *em*](image.png "title")','<b>raw</b>','&copy; &amp; < > " \'','https://example.com/a_(b).','<a@b.com>','\\*literal\\*','日本語 😀','_a_b_c_','a\nb'];
for(const a of inline){for(const b of ['',' tail','\nnext','\n\nnext']){cases.push(a+b);cases.push('# '+a+b);cases.push('- '+a+b);cases.push('> '+a+b);}}
const code=['const x = "hello"; // comment\nconsole.log(x);','{"x": true, "n": 42, "s":"<&\\\""}','<html><script>const x=1;</script><p class="a">Hello &amp;</p></html>','function f<T>(x: T): T { return x; }','if x == 2:\n    print("hi")','# hi\n**strong**\n[link](url)','fn main() { println!("hello"); }','a < b & "x" \'y\'','\n','', '😀 &copy;\n\nend\n'];
const langs=['','plaintext','unknown-x','js','javascript','ts','typescript','json','xml','html','markdown','python','rust','bash','css','sql','c','cpp','java','ruby','php','mathematica','gcode','yaml','diff','ini','go','swift'];
for(const lang of langs)for(const text of code)cases.push('```'+lang+'\n'+text+'\n```');
for(const fence of ['~~~','```'])for(const text of code.slice(0,4))cases.push(fence+'unknown extra info\n'+text+'\n'+fence+'\nparagraph');
// Sweep every locked language plus aliases and all eight callback families.
const hljs = require('./vendor/highlight.js/lib/index.js');
for(const lang of hljs.listLanguages()) {
 for(const text of code.slice(0,8))cases.push('```'+lang+'\n'+text+'\n```');
 for(const alias of (hljs.getLanguage(lang).aliases||[]))cases.push('```'+alias+'\n'+code[0]+'\n```');
}
const specialCode=[['js','const f = <T extends string>(x:T) => x;\nconst v = <Tag>Hello</Tag>;\nconst re=/[a-z]+/g;'],['js','#!/usr/bin/node\n// a\n`hello ${world}`; object.if;'],['c','R"tag(a \" b)tag"'],['bash',"cat <<'EOF'\nhello\nEOF\necho end"],['php','<<<FOO\nhello\nFOO;'],['gcode','G1 X12 Y3\nabcG4 _G4 0G1'],['mathematica','Sin[x] + Unknown[x] + Pi'],['html','<style>p { color: red; }</style><script>let x=1;</script>'],['markdown','```js\nconst x = 1;\n```']];
for(const [lang,text]of specialCode)cases.push('~~~~'+lang+'\n'+text+'\n~~~~');
// Independent HTML, Unicode, URL, whitespace and Markdown edge-case matrix.
for(const tag of ['div','custom-tag','span','script','style','pre','textarea','details','table','svg','iframe','x:y']) {
 for(const tail of ['', '\n','\n\nnext','\n  \nnext']) {
  for(const content of ['raw &copy; &#x1f600; &bogus;','**md**\nnew line','<b>nested</b>','日本語 😀'])cases.push('<'+tag+'>'+content+'</'+tag+'>'+tail);
 }
}
for(const alt of ['a *b*','a **b**','`<a>`','a\\]b','[nested]','a "b" \'c\'','&copy; &bogus;','日本語 😀','a\nb'])cases.push('!['+alt+'](https://x.test/?a=1&b=2 "title &copy;")');
for(const info of ['a&b','a&amp;b','&#x61;','a"b',"a'b",'JS','JSON','xml extra','\\*','a<'])cases.push('```'+info+'\nx<&"\n```');
for(const input of ['> foo\n[5]: foo\n> bar','> foo\nbar\n> baz','> foo\nbar\nbaz\n> end','- [x] foo\n\n- bar','- [ ] <span>x</span>\n\n- end','- [ ] <pre>x</pre>','- [x] \n\n- [ ] end','- [ ]\ttext','-\tfoo\n\tbar','-    foo\n     bar','> - a\nb\n> c','*Hello [not*link] end*!','*Hello [not*a*link] end*!'])cases.push(input);
for(const lang of ['js','ts'])for(const codeText of ['const a = <Tag>hello</Wrong>;','const a = <Tag>hello</TagOther>;','const f = <T, A>(x: T) => x;','const f = <T = string>(x: T) => x;','<Array<Array<number>>>','obj.if; object.while;','const re = /a(.*?)b/g;','const s = `x ${foo({a: "b"})}`;'])cases.push('```'+lang+'\n'+codeText+'\n```');
const path=require('node:path');
function walk(dir){for(const ent of fs.readdirSync(dir,{withFileTypes:true})){const p=path.join(dir,ent.name);if(ent.isDirectory())walk(p);else if(ent.name.endsWith('.md'))cases.push(fs.readFileSync(p,'utf8'));}}
if(fs.existsSync(__dirname+'/../fixtures/marked15/test/specs'))walk(__dirname+'/../fixtures/marked15/test/specs');
(async()=>{let out=[];for(let i=0;i<cases.length;i++)out.push({id:i,input:cases[i],calls:await oracle(cases[i])});fs.writeFileSync(__dirname+'/cases.json',JSON.stringify(out,null,2)+'\n');console.log(out.length,'unchanged source oracle cases');})();
