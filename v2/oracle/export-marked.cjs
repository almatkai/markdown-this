const fs=require('node:fs');const {marked}=require('./vendor/marked/lib/marked.cjs');
fs.writeFileSync(__dirname+'/../data/marked15-inline-tag.regex',marked.Lexer.rules.inline.normal.tag.source);
fs.writeFileSync(__dirname+'/../data/marked15-block-html.regex',marked.Lexer.rules.block.normal.html.source);
