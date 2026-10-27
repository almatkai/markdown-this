//! Rust-only offline conversion. Host clipboard effects live in adapter.rs.
#![forbid(unsafe_code)]
mod highlight;
#[allow(dead_code, clippy::enum_variant_names, clippy::too_many_arguments)]
// Retained parser helpers; original fixture is byte-exact.
mod markdown;

#[derive(Clone, Copy, Default)]
pub(crate) struct RenderOptions {
    pub gfm: bool,
    pub breaks: bool,
    pub pedantic: bool,
}

thread_local! { static LIMITS: std::cell::RefCell<(usize,usize,bool)> = const { std::cell::RefCell::new((0,0,false)) }; }
pub(crate) struct WorkGuard;
impl Drop for WorkGuard {
    fn drop(&mut self) {
        LIMITS.with(|s| s.borrow_mut().0 -= 1);
    }
}
pub(crate) fn work_guard() -> Option<WorkGuard> {
    LIMITS.with(|s| {
        let mut s = s.borrow_mut();
        if s.0 >= 128 || s.1 >= 100_000 {
            s.2 = true;
            None
        } else {
            s.0 += 1;
            s.1 += 1;
            Some(WorkGuard)
        }
    })
}

/// Bound work explicitly; never truncate input or output.
pub const MAX_INPUT_BYTES: usize = 256 * 1024;

/// ECMAScript String.trim whitespace, which differs from Rust's Unicode trim.
pub fn has_content(input: &str) -> bool {
    input.chars().any(|c| {
        !matches!(c,
        '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
        '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
        '\u{205f}' | '\u{3000}' | '\u{feff}')
    })
}

pub fn convert(input: &str) -> Result<String, String> {
    LIMITS.with(|s| *s.borrow_mut() = (0, 0, false));
    let _ = highlight::take_error();
    if !has_content(input) {
        return Err("The clipboard has no content".into());
    }
    if input.len() > MAX_INPUT_BYTES {
        return Err("Markdown exceeds the 256 KiB native conversion limit".into());
    }
    // A bounded thread stack also prevents malformed deeply nested Markdown from
    // exhausting the host thread's stack. Nesting is checked before parsing.
    if input
        .lines()
        .any(|line| line.chars().filter(|c| *c == '>').count() > 64)
        || input
            .lines()
            .filter(|line| line.starts_with("    "))
            .count()
            > 4096
    {
        return Err("Markdown exceeds the native nesting/work limit".into());
    }
    let body = markdown::render_markdown_to_html(
        input,
        RenderOptions {
            gfm: true,
            breaks: true,
            pedantic: false,
        },
    );
    if LIMITS.with(|s| s.borrow().2) {
        return Err("Markdown exceeds the native nesting/work limit".into());
    }
    highlight::take_error()?;
    Ok(format!(
        "{}{}{}",
        include_str!("../data/document-prefix.html"),
        body,
        include_str!("../data/document-suffix.html")
    ))
}

pub(crate) fn html_tag_len(input: &str) -> Option<usize> {
    static RE: std::sync::OnceLock<regress::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regress::Regex::new(include_str!("../data/marked15-inline-tag.regex"))
            .expect("pinned inline HTML regex")
    })
    .find(input)
    .filter(|m| m.start() == 0)
    .map(|m| m.end())
}
pub(crate) fn block_html_len(input: &str) -> Option<usize> {
    static RE: std::sync::OnceLock<regress::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regress::Regex::with_flags(
            include_str!("../data/marked15-block-html.regex"),
            regress::Flags {
                icase: true,
                ..Default::default()
            },
        )
        .expect("pinned block HTML regex")
    })
    .find(input)
    .filter(|m| m.start() == 0)
    .map(|m| m.end())
}

pub(crate) fn highlight_code(code: &str, language: &str) -> String {
    highlight::code_block(code, language)
}
