//! Native evaluation of the pinned Highlight.js 11.11.1 declarative grammars.
//! No script interpreter, subprocess, network or clipboard access.
use regress::{Flags, Regex};
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

thread_local! { static ERROR: RefCell<Option<String>> = const { RefCell::new(None) }; }
pub(crate) fn take_error() -> Result<(), String> {
    ERROR.with(|e| e.borrow_mut().take().map_or(Ok(()), Err))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawData {
    languages: Vec<RawLanguage>,
    math_symbols: HashSet<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLanguage {
    name: String,
    aliases: Vec<String>,
    case_insensitive: bool,
    unicode: bool,
    disable_autodetect: bool,
    superset_of: Option<String>,
    class_name_aliases: HashMap<String, String>,
    modes: Vec<RawMode>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct RawMode {
    scope: Option<String>,
    begin_scope: Value,
    end_scope: Value,
    skip: bool,
    exclude_begin: bool,
    exclude_end: bool,
    return_begin: bool,
    return_end: bool,
    ends_parent: bool,
    ends_with_parent: bool,
    sub_language: Value,
    relevance: f64,
    keywords: HashMap<String, (String, Option<f64>)>,
    end_re: Option<String>,
    keyword_pattern_re: String,
    before: Option<u8>,
    on_begin: Option<u8>,
    on_end: Option<u8>,
    rules: Vec<RawRule>,
    starts: Option<usize>,
}
#[derive(Deserialize)]
struct RawRule {
    regex: String,
    #[serde(rename = "type")]
    kind: String,
    mode: Option<usize>,
}
struct Rule {
    re: Regex,
    kind: String,
    mode: Option<usize>,
}
struct Mode {
    raw: RawMode,
    rules: Vec<Rule>,
    end: Option<Regex>,
    keyword: Regex,
}
struct Language {
    raw: RawLanguage,
    modes: Vec<Mode>,
}
struct Data {
    languages: Vec<Language>,
    names: HashMap<String, usize>,
    math: HashSet<String>,
}
static DATA: OnceLock<Result<Data, String>> = OnceLock::new();
fn data() -> Result<&'static Data, String> {
    DATA.get_or_init(|| {
        let raw: RawData = serde_json::from_str(include_str!("../data/highlight-11.11.1.json"))
            .map_err(|e| format!("Invalid bundled highlighting grammar: {e}"))?;
        let mut languages = Vec::new();
        let mut names = HashMap::new();
        for mut lang in raw.languages {
            let flags = Flags {
                multiline: true,
                icase: lang.case_insensitive,
                unicode: lang.unicode,
                ..Flags::default()
            };
            let compile = |s: &str| {
                Regex::with_flags(s, flags)
                    .map_err(|e| format!("Bundled {} grammar: {e}", lang.name))
            };
            let mut modes = Vec::new();
            for mode in std::mem::take(&mut lang.modes) {
                let mut rules = Vec::new();
                for rule in &mode.rules {
                    rules.push(Rule {
                        re: compile(&rule.regex)?,
                        kind: rule.kind.clone(),
                        mode: rule.mode,
                    });
                }
                let end = mode.end_re.as_deref().map(compile).transpose()?;
                let keyword = compile(&mode.keyword_pattern_re)?;
                modes.push(Mode {
                    raw: mode,
                    rules,
                    end,
                    keyword,
                });
            }
            let id = languages.len();
            names.insert(lang.name.clone(), id);
            for alias in &lang.aliases {
                names.insert(alias.to_lowercase(), id);
            }
            languages.push(Language { raw: lang, modes });
        }
        Ok(Data {
            languages,
            names,
            math: raw.math_symbols,
        })
    })
    .as_ref()
    .map_err(Clone::clone)
}
#[derive(Clone, Default)]
struct Continuation {
    stack: Vec<usize>,
    data: HashMap<usize, String>,
}
struct Highlight {
    html: String,
    relevance: f64,
    continuation: Continuation,
    language: Option<usize>,
}
#[derive(Clone)]
struct Match {
    start: usize,
    end: usize,
    groups: Vec<Option<String>>,
    rule: usize,
    kind: String,
    mode: Option<usize>,
}
fn find(re: &Regex, input: &str, at: usize) -> Option<(usize, usize, Vec<Option<String>>)> {
    re.find_from(input, at).next().map(|m| {
        let mut groups = vec![Some(input[m.range()].to_string())];
        groups.extend(
            m.captures
                .iter()
                .map(|r| r.as_ref().map(|r| input[r.clone()].to_string())),
        );
        (m.start(), m.end(), groups)
    })
}
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        out.push_str(match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#x27;",
            _ => {
                out.push(c);
                continue;
            }
        });
    }
    out
}
fn class(scope: &str) -> String {
    if let Some(s) = scope.strip_prefix("language:") {
        return format!("language-{s}");
    }
    let mut parts = scope.split('.');
    let mut s = format!("hljs-{}", parts.next().unwrap_or_default());
    for (i, p) in parts.enumerate() {
        s.push(' ');
        s.push_str(p);
        s.push_str(&"_".repeat(i + 1));
    }
    s
}
fn open(out: &mut String, scope: &str) {
    out.push_str("<span class=\"");
    out.push_str(&class(scope));
    out.push_str("\">");
}
fn emit(out: &mut String, text: &str, scope: &str) {
    if text.is_empty() {
        return;
    }
    open(out, scope);
    out.push_str(&escape(text));
    out.push_str("</span>");
}
fn next_char(s: &str, at: usize) -> usize {
    s[at..].chars().next().map_or(at, |c| at + c.len_utf8())
}
struct Machine<'a> {
    all: &'a Data,
    lang: usize,
    input: &'a str,
    stack: Vec<usize>,
    state: HashMap<usize, String>,
    buffer: String,
    out: String,
    relevance: f64,
    hits: HashMap<String, usize>,
    continuations: HashMap<usize, Continuation>,
    depth: usize,
    budget: &'a mut usize,
}
impl Machine<'_> {
    fn language(&self) -> &Language {
        &self.all.languages[self.lang]
    }
    fn top(&self) -> usize {
        *self.stack.last().expect("root mode exists")
    }
    fn alias(&self, s: &str) -> String {
        self.language()
            .raw
            .class_name_aliases
            .get(s)
            .cloned()
            .unwrap_or_else(|| s.into())
    }
    fn keyword_buffer(&mut self, text: &str, mode: usize) {
        let m = &self.language().modes[mode];
        if m.raw.keywords.is_empty() {
            self.out.push_str(&escape(text));
            return;
        }
        let mut at = 0;
        let mut matches = Vec::new();
        for item in m.keyword.find_iter(text) {
            if item.start() < at {
                continue;
            }
            matches.push((item.start(), item.end()));
            at = item.end();
            if matches.len() > 100_000 {
                break;
            }
        }
        at = 0;
        for (start, end) in matches {
            self.out.push_str(&escape(&text[at..start]));
            let word = &text[start..end];
            let key = if self.language().raw.case_insensitive {
                word.to_lowercase()
            } else {
                word.to_string()
            };
            if let Some((kind, score)) = self.language().modes[mode].raw.keywords.get(&key).cloned()
            {
                let hit = self.hits.entry(key).or_default();
                *hit += 1;
                if *hit <= 7 {
                    self.relevance += score.unwrap_or(f64::NAN);
                }
                if kind.starts_with('_') {
                    self.out.push_str(&escape(word));
                } else {
                    let scope = self.alias(&kind);
                    emit(&mut self.out, word, &scope);
                }
            } else {
                self.out.push_str(&escape(word));
            }
            at = end;
        }
        self.out.push_str(&escape(&text[at..]));
    }
    fn flush(&mut self) -> Result<(), String> {
        let text = std::mem::take(&mut self.buffer);
        let top = self.top();
        let sub = self.language().modes[top].raw.sub_language.clone();
        if sub.is_null() {
            self.keyword_buffer(&text, top);
            return Ok(());
        }
        if text.is_empty() {
            return Ok(());
        }
        let result = if let Some(name) = sub.as_str() {
            let Some(&lang) = self.all.names.get(&name.to_lowercase()) else {
                self.out.push_str(&escape(&text));
                return Ok(());
            };
            let continuation = self.continuations.remove(&lang);
            run(
                self.all,
                lang,
                &text,
                true,
                continuation,
                self.depth + 1,
                self.budget,
            )?
        } else {
            let subset = sub
                .as_array()
                .map(|xs| xs.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                .unwrap_or_default();
            auto(self.all, &text, &subset, self.depth + 1, self.budget)?
        };
        if self.language().modes[top].raw.relevance > 0.0 {
            self.relevance += result.relevance;
        }
        if let Some(lang) = result.language {
            let scope = format!("language:{}", self.all.languages[lang].raw.name);
            open(&mut self.out, &scope);
            self.out.push_str(&result.html);
            self.out.push_str("</span>");
            self.continuations.insert(lang, result.continuation);
        } else {
            self.out.push_str(&result.html);
        }
        Ok(())
    }
    fn multi(&mut self, scope: &Value, m: &Match, mode: usize) {
        let Some(obj) = scope.as_object() else {
            return;
        };
        for i in 1..m.groups.len() {
            let key = i.to_string();
            if obj
                .get("_emit")
                .and_then(|x| x.get(&key))
                .and_then(Value::as_bool)
                != Some(true)
            {
                continue;
            }
            let Some(text) = &m.groups[i] else {
                continue;
            };
            if let Some(name) = obj.get(&key).and_then(Value::as_str) {
                let name = self.alias(name);
                emit(&mut self.out, text, &name);
            } else {
                self.keyword_buffer(text, mode);
            }
        }
    }
    fn start(&mut self, mode: usize, m: &Match) {
        if let Some(scope) = self.language().modes[mode].raw.scope.clone() {
            let scope = self.alias(&scope);
            open(&mut self.out, &scope);
        }
        let scope = self.language().modes[mode].raw.begin_scope.clone();
        if let Some(wrap) = scope.get("_wrap").and_then(Value::as_str) {
            let wrap = self.alias(wrap);
            let buf = std::mem::take(&mut self.buffer);
            emit(&mut self.out, &buf, &wrap);
        } else if scope.get("_multi").and_then(Value::as_bool) == Some(true) {
            self.multi(&scope, m, self.top());
            self.buffer.clear();
        }
        self.stack.push(mode);
    }
    fn callback(&mut self, id: Option<u8>, mode: usize, m: &Match) -> bool {
        let group = |i: usize| m.groups.get(i).and_then(|s| s.as_deref()).unwrap_or("");
        let preceding = self.input[..m.start].chars().next_back();
        match id {
            None => true,
            Some(1) => preceding != Some('.'),
            Some(2) => {
                self.state.insert(mode, group(1).into());
                true
            }
            Some(3) => self.state.get(&mode).map(String::as_str) == Some(group(1)),
            Some(4) => m.start == 0,
            Some(5) => m.start == 0 || preceding.is_some_and(|c| c.is_ascii_digit() || c == '_'),
            Some(6) => {
                let after = &self.input[m.end..];
                let next = after.chars().next();
                if matches!(next, Some('<' | ',')) {
                    return false;
                }
                if next == Some('>')
                    && !after.contains(&format!(
                        "</{}",
                        group(0).strip_prefix('<').unwrap_or(group(0))
                    ))
                {
                    return false;
                }
                let trimmed = after.trim_start_matches(char::is_whitespace);
                if trimmed.starts_with('=') {
                    return false;
                }
                !(trimmed.len() < after.len()
                    && trimmed
                        .strip_prefix("extends")
                        .is_some_and(|s| s.starts_with(char::is_whitespace)))
            }
            Some(7) => self.all.math.contains(group(0)),
            Some(8) => {
                let s = if group(1).is_empty() {
                    group(2)
                } else {
                    group(1)
                };
                self.state.insert(mode, s.into());
                true
            }
            Some(_) => false,
        }
    }
    fn end_mode(&mut self, mut idx: usize, m: &Match) -> Option<usize> {
        loop {
            let mode = self.stack[idx];
            let matched = self.language().modes[mode].end.as_ref().is_some_and(|re| {
                re.find(&self.input[m.start..])
                    .is_some_and(|r| r.start() == 0)
            });
            if matched && self.callback(self.language().modes[mode].raw.on_end, mode, m) {
                while self.language().modes[self.stack[idx]].raw.ends_parent && idx > 0 {
                    idx -= 1;
                }
                return Some(idx);
            }
            if idx > 0 && self.language().modes[mode].raw.ends_with_parent {
                idx -= 1;
            } else {
                return None;
            }
        }
    }
    fn find_match(&self, at: usize, from: usize) -> Option<Match> {
        let rules = &self.language().modes[self.top()].rules;
        let mut result: Option<Match> = None;
        for (idx, rule) in rules.iter().enumerate().skip(from) {
            if let Some((start, end, groups)) = find(&rule.re, self.input, at) {
                if result.as_ref().is_none_or(|m| start < m.start) {
                    result = Some(Match {
                        start,
                        end,
                        groups,
                        rule: idx,
                        kind: rule.kind.clone(),
                        mode: rule.mode,
                    });
                }
            }
        }
        result
    }
    fn execute(mut self, ignore_illegals: bool) -> Result<Highlight, String> {
        for &mode in self.stack.iter().skip(1) {
            if let Some(scope) = &self.language().modes[mode].raw.scope {
                let scope = self.alias(scope);
                open(&mut self.out, &scope);
            }
        }
        let mut at = 0;
        let mut resume = 0;
        let mut last_begin = None;
        let mut zero_steps = 0;
        loop {
            if *self.budget == 0 {
                return Err("Syntax highlighting exceeds the native work limit".into());
            }
            *self.budget -= 1;
            let mut candidate = self.find_match(at, resume);
            if resume > 0 && candidate.as_ref().is_none_or(|m| m.start != at) {
                candidate = if at < self.input.len() {
                    self.find_match(next_char(self.input, at), 0)
                } else {
                    None
                };
            }
            resume = 0;
            let Some(m) = candidate else {
                self.buffer.push_str(&self.input[at..]);
                break;
            };
            self.buffer.push_str(&self.input[at..m.start]);
            let lex = &self.input[m.start..m.end];
            if last_begin == Some(m.start) && m.kind == "end" && lex.is_empty() {
                let n = next_char(self.input, m.start);
                self.buffer.push_str(&self.input[m.start..n]);
                if n == m.start {
                    break;
                }
                at = n;
                continue;
            }
            last_begin = if m.kind == "begin" {
                Some(m.start)
            } else {
                None
            };
            let mut consumed = m.end - m.start;
            if m.kind == "begin" {
                let mode = m.mode.ok_or("Invalid bundled begin rule")?;
                let raw = &self.language().modes[mode].raw;
                let before = raw.before;
                let begin = raw.on_begin;
                if !self.callback(before, mode, &m) || !self.callback(begin, mode, &m) {
                    let count = self.language().modes[self.top()]
                        .rules
                        .iter()
                        .filter(|r| r.kind == "begin")
                        .count();
                    if m.rule + 1 < count {
                        resume = m.rule + 1;
                        consumed = 0;
                    } else {
                        let n = next_char(self.input, m.start);
                        self.buffer.push_str(&self.input[m.start..n]);
                        consumed = n - m.start;
                    }
                } else {
                    let raw = &self.language().modes[mode].raw;
                    let (skip, exclude, ret) = (raw.skip, raw.exclude_begin, raw.return_begin);
                    if skip {
                        self.buffer.push_str(lex);
                    } else {
                        if exclude {
                            self.buffer.push_str(lex);
                        }
                        self.flush()?;
                        if !ret && !exclude {
                            self.buffer.push_str(lex);
                        }
                    }
                    self.start(mode, &m);
                    if ret {
                        consumed = 0;
                    }
                    if self.stack.len() > 256 {
                        return Err("Syntax highlighting exceeds the native nesting limit".into());
                    }
                }
            } else if m.kind == "end" {
                if let Some(end) = self.end_mode(self.stack.len() - 1, &m) {
                    let origin = self.top();
                    let raw = &self.language().modes[origin].raw;
                    let (skip, exclude, ret, scope) = (
                        raw.skip,
                        raw.exclude_end,
                        raw.return_end,
                        raw.end_scope.clone(),
                    );
                    if let Some(wrap) = scope.get("_wrap").and_then(Value::as_str) {
                        self.flush()?;
                        emit(&mut self.out, lex, wrap);
                    } else if scope.get("_multi").and_then(Value::as_bool) == Some(true) {
                        self.flush()?;
                        self.multi(&scope, &m, origin);
                    } else if skip {
                        self.buffer.push_str(lex);
                    } else {
                        if !ret && !exclude {
                            self.buffer.push_str(lex);
                        }
                        self.flush()?;
                        if exclude {
                            self.buffer.push_str(lex);
                        }
                    }
                    let starts = self.language().modes[self.stack[end]].raw.starts;
                    while self.stack.len() > end {
                        let mode = self.stack.pop().ok_or("Invalid highlight stack")?;
                        let raw = &self.language().modes[mode].raw;
                        let scoped = raw.scope.is_some();
                        let relevance = if !raw.skip && raw.sub_language.is_null() {
                            raw.relevance
                        } else {
                            0.0
                        };
                        if scoped {
                            self.out.push_str("</span>");
                        }
                        self.relevance += relevance;
                    }
                    if self.stack.is_empty() {
                        return Err("Invalid highlighting root termination".into());
                    }
                    if let Some(starts) = starts {
                        self.start(starts, &m);
                    }
                    if ret {
                        consumed = 0;
                    }
                } else {
                    self.buffer.push_str(lex);
                }
            } else if m.kind == "illegal" && !ignore_illegals {
                return Ok(Highlight {
                    html: escape(self.input),
                    relevance: 0.0,
                    continuation: Continuation::default(),
                    language: Some(self.lang),
                });
            } else if m.kind == "illegal" && lex.is_empty() {
                self.buffer.push('\n');
                consumed = usize::from(m.start < self.input.len());
            } else {
                self.buffer.push_str(lex);
            }
            let new_at = m.start + consumed;
            if new_at == at {
                zero_steps += 1;
                if zero_steps > 1024 {
                    return Err("Syntax highlighting exceeds the native progress limit".into());
                }
            } else {
                zero_steps = 0;
            }
            at = new_at;
            if at > self.input.len() || !self.input.is_char_boundary(at) {
                return Err("Syntax highlighting reached a non-Unicode boundary".into());
            }
        }
        self.flush()?;
        for &mode in self.stack.iter().skip(1).rev() {
            if self.language().modes[mode].raw.scope.is_some() {
                self.out.push_str("</span>");
            }
        }
        Ok(Highlight {
            html: self.out,
            relevance: self.relevance,
            continuation: Continuation {
                stack: self.stack,
                data: self.state,
            },
            language: Some(self.lang),
        })
    }
}
fn run(
    all: &Data,
    lang: usize,
    input: &str,
    ignore_illegals: bool,
    continuation: Option<Continuation>,
    depth: usize,
    budget: &mut usize,
) -> Result<Highlight, String> {
    if depth > 16 {
        return Err("Syntax highlighting exceeds the native sublanguage limit".into());
    }
    let c = continuation.unwrap_or_default();
    Machine {
        all,
        lang,
        input,
        stack: if c.stack.is_empty() { vec![0] } else { c.stack },
        state: c.data,
        buffer: String::new(),
        out: String::new(),
        relevance: 0.0,
        hits: HashMap::new(),
        continuations: HashMap::new(),
        depth,
        budget,
    }
    .execute(ignore_illegals)
}
fn auto(
    all: &Data,
    input: &str,
    subset: &[&str],
    depth: usize,
    budget: &mut usize,
) -> Result<Highlight, String> {
    let mut best = Highlight {
        html: escape(input),
        relevance: 0.0,
        continuation: Continuation::default(),
        language: None,
    };
    for (id, language) in all.languages.iter().enumerate() {
        if language.raw.disable_autodetect
            || (!subset.is_empty() && !subset.iter().any(|s| all.names.get(*s) == Some(&id)))
        {
            continue;
        }
        let result = run(all, id, input, false, None, depth, budget)?;
        let base_tie = best.language.is_some_and(|prev| {
            all.languages[prev].raw.superset_of.as_deref() == Some(&language.raw.name)
        });
        if result.relevance > best.relevance || (result.relevance == best.relevance && base_tie) {
            best = result;
        }
    }
    Ok(best)
}
pub(crate) fn code_block(code: &str, language: &str) -> String {
    let result = (|| {
        let all = data()?;
        let lang = all
            .names
            .get(&language.to_lowercase())
            .copied()
            .or_else(|| all.names.get("plaintext").copied())
            .ok_or("Missing plaintext grammar")?;
        let code = code.strip_suffix('\n').unwrap_or(code);
        let mut budget = 1_000_000;
        let result = run(all, lang, code, true, None, 0, &mut budget)?;
        let mut out = String::from("<pre><code");
        if !language.is_empty() {
            out.push_str(" class=\"hljs language-");
            let mut class_value = String::new();
            for (i, part) in language.split('&').enumerate() {
                if i > 0 {
                    let valid = part.find(';').is_some_and(|n| {
                        let entity = &part[..n];
                        entity.strip_prefix('#').is_some_and(|x| {
                            x.strip_prefix(['x', 'X']).map_or_else(
                                || {
                                    !x.is_empty()
                                        && x.len() <= 7
                                        && x.bytes().all(|b| b.is_ascii_digit())
                                },
                                |x| {
                                    !x.is_empty()
                                        && x.len() <= 6
                                        && x.bytes().all(|b| b.is_ascii_hexdigit())
                                },
                            )
                        }) || !entity.is_empty()
                            && entity
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    });
                    class_value.push_str(if valid { "&" } else { "&amp;" });
                }
                class_value.push_str(&escape(part).replace("&#x27;", "&#39;"));
            }
            out.push_str(&class_value);
            out.push('"');
        }
        out.push('>');
        out.push_str(result.html.strip_suffix('\n').unwrap_or(&result.html));
        out.push_str("\n</code></pre>");
        Ok::<_, String>(out)
    })();
    match result {
        Ok(html) => html,
        Err(error) => {
            ERROR.with(|e| {
                if e.borrow().is_none() {
                    *e.borrow_mut() = Some(error);
                }
            });
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_pinned_grammars_compile() {
        let d = data().unwrap();
        assert_eq!(d.languages.len(), 192);
        assert_eq!(
            d.languages.iter().map(|l| l.modes.len()).sum::<usize>(),
            5179
        );
    }
}
