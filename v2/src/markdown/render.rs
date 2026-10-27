use crate::markdown::{
    ast::{self, inline::Inline},
    block::parse_html_entity,
};
use crate::RenderOptions;

static HEADING_OPEN: [&str; 7] = ["", "<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
static HEADING_CLOSE: [&str; 7] = [
    "", "</h1>\n", "</h2>\n", "</h3>\n", "</h4>\n", "</h5>\n", "</h6>\n",
];

pub(crate) fn render_document(
    doc: &ast::Document,
    options: RenderOptions,
    input_len: usize,
) -> String {
    let mut out = String::with_capacity(input_len + input_len / 2);
    render_document_into(doc, options, &mut out);
    out
}

pub(crate) fn render_document_into(doc: &ast::Document, options: RenderOptions, out: &mut String) {
    match doc {
        ast::Document::Nodes(blocks) => {
            for block in blocks {
                render_block(block, out, options);
            }
        }
    }
}

fn render_block(block: &ast::Block, out: &mut String, options: RenderOptions) {
    let Some(_guard) = crate::work_guard() else {
        return;
    };
    match block {
        ast::Block::Paragraph { source, inlines } => {
            out.push_str("<p>");
            render_inlines(inlines, out, options, source);
            out.push_str("</p>\n");
        }
        ast::Block::Heading { level, inlines } => {
            let l = *level as usize;
            out.push_str(HEADING_OPEN[l]);
            render_inlines(inlines, out, options, "");
            out.push_str(HEADING_CLOSE[l]);
        }
        ast::Block::List {
            ordered,
            start,
            tight,
            items,
        } => {
            if *ordered {
                if *start == 1 {
                    out.push_str("<ol>\n");
                } else {
                    use std::fmt::Write as _;
                    writeln!(out, "<ol start=\"{start}\">").expect("write ordered list start");
                }
            } else {
                out.push_str("<ul>\n");
            }
            for (item_index, item) in items.iter().enumerate() {
                out.push_str("<li>");
                let children = &item.children;
                if *tight {
                    render_tight_list_item(children, item.task, out, options);
                    out.push_str("</li>\n");
                    continue;
                }

                let mut rendered_task_paragraph = false;
                if let Some(done) = item.task {
                    if let Some(ast::Block::Paragraph { source, inlines }) = children.first() {
                        out.push_str("<p>");
                        render_task_checkbox(done, out);
                        out.push_str(" \n");
                        render_inlines(inlines, out, options, source);
                        out.push_str("</p>\n");
                        rendered_task_paragraph = true;
                    } else if let Some(ast::Block::HtmlBlock(raw)) = children.first() {
                        out.push_str("<p>");
                        render_task_checkbox(done, out);
                        out.push_str(" </p>\n");
                        render_raw_html(raw, out, options);
                        if item_index + 1 < items.len() && !raw.ends_with('\n') {
                            out.push('\n');
                        }
                        rendered_task_paragraph = true;
                    } else {
                        out.push_str("<p>");
                        render_task_checkbox(done, out);
                        out.push_str(" </p>\n");
                    }
                }

                for (idx, child) in children.iter().enumerate() {
                    if rendered_task_paragraph && idx == 0 {
                        continue;
                    }
                    render_block(child, out, options);
                }
                out.push_str("</li>\n");
            }
            if *ordered {
                out.push_str("</ol>\n");
            } else {
                out.push_str("</ul>\n");
            }
        }
        ast::Block::BlockQuote { children } => {
            out.push_str("<blockquote>\n");
            for child in children {
                render_block(child, out, options);
            }
            out.push_str("</blockquote>\n");
        }
        ast::Block::CodeBlock { info, content } => {
            let language = info
                .as_deref()
                .and_then(extract_code_block_language)
                .map(unescape_code_block_language)
                .unwrap_or_default();
            out.push_str(&crate::highlight_code(content, &language));
        }
        ast::Block::ThematicBreak => {
            out.push_str("<hr>\n");
        }
        ast::Block::Table {
            aligns,
            header,
            rows,
        } => {
            out.push_str("<table>\n<thead>\n<tr>\n");
            for (idx, cell) in header.iter().enumerate() {
                out.push_str("<th");
                render_table_align_attr(aligns.get(idx).copied().flatten(), out);
                out.push('>');
                render_inlines(cell, out, options, "");
                out.push_str("</th>\n");
            }
            out.push_str("</tr>\n</thead>\n");
            if !rows.is_empty() {
                out.push_str("<tbody>");
                for row in rows {
                    out.push_str("<tr>\n");
                    for (idx, cell) in row.iter().enumerate() {
                        out.push_str("<td");
                        render_table_align_attr(aligns.get(idx).copied().flatten(), out);
                        out.push('>');
                        render_inlines(cell, out, options, "");
                        out.push_str("</td>\n");
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</tbody>");
            }
            out.push_str("</table>\n");
        }
        ast::Block::HtmlBlock(raw) => {
            render_raw_html(raw, out, options);
        }
    }
}

fn render_task_checkbox(done: bool, out: &mut String) {
    out.push_str("<input ");
    if done {
        out.push_str("checked=\"\" ");
    }
    out.push_str("disabled=\"\" type=\"checkbox\">");
}

fn render_tight_list_item(
    children: &[ast::Block],
    task: Option<bool>,
    out: &mut String,
    options: RenderOptions,
) {
    match task {
        Some(done) if !matches!(children.first(), Some(ast::Block::Paragraph { .. })) => {
            render_task_checkbox(done, out);
            out.push(' ');
        }
        _ => {}
    }
    for (idx, child) in children.iter().enumerate() {
        match child {
            ast::Block::Paragraph { source, inlines } => {
                if idx == 0 {
                    if let Some(done) = task {
                        render_task_checkbox(done, out);
                        if !inlines.is_empty() {
                            out.push(' ');
                        }
                    }
                }
                render_inlines(inlines, out, options, source);
            }
            _ => render_block(child, out, options),
        }
    }

    if matches!(children.last(), Some(ast::Block::HtmlBlock(_))) && out.ends_with('\n') {
        out.pop();
    }
}

fn render_inlines(inlines: &[Inline], out: &mut String, options: RenderOptions, source: &str) {
    let Some(_guard) = crate::work_guard() else {
        return;
    };
    if let [Inline::Text(text)] = inlines {
        escape_text_html_to(text, out);
        return;
    }
    if let [Inline::TextSpan(span)] = inlines {
        escape_text_html_to(span.as_str(source), out);
        return;
    }

    for inline in inlines {
        match inline {
            Inline::Text(text) => render_text(text, out, options),
            Inline::TextSpan(span) => render_text(span.as_str(source), out, options),
            Inline::RawHtml(html) => render_raw_html(html, out, options),
            Inline::RawHtmlSpan(span) => render_raw_html(span.as_str(source), out, options),
            Inline::SoftBreak => {
                while out.ends_with([' ', '\t']) {
                    out.pop();
                }
                if options.breaks {
                    out.push_str("<br>");
                } else {
                    out.push('\n');
                }
            }
            Inline::HardBreak => out.push_str("<br>"),
            Inline::Code(text) => {
                out.push_str("<code>");
                escape_code_html_to(text, out);
                out.push_str("</code>");
            }
            Inline::CodeSpan(span) => {
                out.push_str("<code>");
                escape_normalized_code_span_to(span.as_str(source), out);
                out.push_str("</code>");
            }
            Inline::Em(children) => {
                out.push_str("<em>");
                render_inlines(children, out, options, source);
                out.push_str("</em>");
            }
            Inline::Strong(children) => {
                out.push_str("<strong>");
                render_inlines(children, out, options, source);
                out.push_str("</strong>");
            }
            Inline::Del(children) => {
                out.push_str("<del>");
                render_inlines(children, out, options, source);
                out.push_str("</del>");
            }
            Inline::Link { label, href, title } => {
                out.push_str("<a href=\"");
                escape_href_attr_to(href, out);
                out.push('"');
                if let Some(title) = title {
                    out.push_str(" title=\"");
                    escape_html_attr_to(title, out);
                    out.push('"');
                }
                out.push('>');
                render_inlines(label, out, options, source);
                out.push_str("</a>");
            }
            Inline::Image { alt, src, title } => {
                out.push_str("<img src=\"");
                escape_href_attr_to(src, out);
                out.push_str("\" alt=\"");
                for node in alt {
                    match node {
                        Inline::Text(s) => out.push_str(s),
                        Inline::TextSpan(span) => out.push_str(span.as_str(source)),
                        _ => render_inline_text_content_escaped(
                            std::slice::from_ref(node),
                            out,
                            source,
                        ),
                    }
                }
                out.push('"');
                if let Some(title) = title {
                    out.push_str(" title=\"");
                    escape_html_attr_to(title, out);
                    out.push('"');
                }
                out.push('>');
            }
        }
    }
}

fn render_text(text: &str, out: &mut String, options: RenderOptions) {
    if options.breaks && text.contains('\n') {
        for (i, part) in text.split('\n').enumerate() {
            if i > 0 {
                while out.ends_with([' ', '\t']) {
                    out.pop();
                }
                out.push_str("<br>");
            }
            escape_text_html_to(part, out);
        }
    } else {
        escape_text_html_to(text, out);
    }
}

fn render_raw_html(raw: &str, out: &mut String, options: RenderOptions) {
    let _ = options;
    out.push_str(raw);
}

#[inline]
pub(crate) fn escape_html_to_pub(text: &str, out: &mut String) {
    escape_code_html_to(text, out);
}

#[inline]
fn escape_code_html_to(text: &str, out: &mut String) {
    escape_html_without_entities_to(text, out);
}

#[inline]
fn escape_text_html_to(text: &str, out: &mut String) {
    escape_html_preserving_entities_to(text, out);
}

fn extract_code_block_language(info: &str) -> Option<&str> {
    info.split_whitespace().next()
}

fn unescape_code_block_language(language: &str) -> String {
    let mut out = String::with_capacity(language.len());
    let mut chars = language.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                if next.is_ascii_punctuation() {
                    out.push(next);
                } else {
                    out.push('\\');
                    out.push(next);
                }
                continue;
            }
        }
        out.push(ch);
    }

    out
}

fn escape_html_attr_to(text: &str, out: &mut String) {
    escape_html_preserving_entities_to(text, out);
}

fn escape_href_attr_to(text: &str, out: &mut String) {
    if !text.as_bytes().contains(&b'"') {
        out.push_str(text);
        return;
    }

    let bytes = text.as_bytes();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let replacement = match b {
            b'"' => "%22",
            _ => continue,
        };
        out.push_str(&text[start..i]);
        out.push_str(replacement);
        start = i + 1;
    }
    out.push_str(&text[start..]);
}

fn escape_html_preserving_entities_to(text: &str, out: &mut String) {
    if !text
        .as_bytes()
        .iter()
        .any(|&b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\''))
    {
        out.push_str(text);
        return;
    }

    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(entity_len) = parse_html_entity(&text[i..]) {
                i += entity_len;
                continue;
            }
        }

        let replacement = match bytes[i] {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&#39;",
            _ => {
                i += 1;
                continue;
            }
        };

        out.push_str(&text[start..i]);
        out.push_str(replacement);
        i += 1;
        start = i;
    }

    out.push_str(&text[start..]);
}

fn escape_html_without_entities_to(text: &str, out: &mut String) {
    if !text
        .as_bytes()
        .iter()
        .any(|&b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\''))
    {
        out.push_str(text);
        return;
    }

    let bytes = text.as_bytes();
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        let replacement = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&#39;",
            _ => continue,
        };
        out.push_str(&text[start..i]);
        out.push_str(replacement);
        start = i + 1;
    }
    out.push_str(&text[start..]);
}

fn render_table_align_attr(align: Option<ast::TableAlignment>, out: &mut String) {
    match align {
        Some(ast::TableAlignment::Left) => out.push_str(" align=\"left\""),
        Some(ast::TableAlignment::Center) => out.push_str(" align=\"center\""),
        Some(ast::TableAlignment::Right) => out.push_str(" align=\"right\""),
        None => {}
    }
}

fn render_inline_text_content_escaped(inlines: &[Inline], out: &mut String, source: &str) {
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Code(text) | Inline::RawHtml(text) => {
                escape_html_attr_to(text, out)
            }
            Inline::TextSpan(span) | Inline::RawHtmlSpan(span) => {
                escape_html_attr_to(span.as_str(source), out)
            }
            Inline::CodeSpan(span) => escape_normalized_code_span_to(span.as_str(source), out),
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            Inline::Em(children) | Inline::Strong(children) | Inline::Del(children) => {
                render_inline_text_content_escaped(children, out, source);
            }
            Inline::Link { label, .. } => render_inline_text_content_escaped(label, out, source),
            Inline::Image { alt, .. } => render_inline_text_content_escaped(alt, out, source),
        }
    }
}

fn escape_normalized_code_span_to(raw: &str, out: &mut String) {
    if raw.is_empty() {
        return;
    }

    if !raw.contains('\n') {
        let trimmed = if raw.len() > 1 && raw.starts_with(' ') && raw.ends_with(' ') {
            &raw[1..raw.len() - 1]
        } else {
            raw
        };
        escape_code_html_to(trimmed, out);
        return;
    }

    let bytes = raw.as_bytes();
    let mut start = 0usize;
    let mut end = bytes.len();

    if end > 1 {
        let first = if bytes[0] == b'\n' { b' ' } else { bytes[0] };
        let last = if bytes[end - 1] == b'\n' {
            b' '
        } else {
            bytes[end - 1]
        };
        if first == b' ' && last == b' ' {
            start = 1;
            end -= 1;
        }
    }

    for &b in &bytes[start..end] {
        let b = if b == b'\n' { b' ' } else { b };
        match b {
            b'&' => out.push_str("&amp;"),
            b'<' => out.push_str("&lt;"),
            b'>' => out.push_str("&gt;"),
            b'"' => out.push_str("&quot;"),
            _ => out.push(b as char),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_image_title_without_duplication() {
        let node = ast::Block::Paragraph {
            source: String::new(),
            inlines: vec![ast::inline::Inline::Image {
                alt: vec![ast::inline::Inline::Text("logo".to_string())],
                src: "logo.png".to_string(),
                title: Some("Markec logo".to_string()),
            }],
        };

        let mut out = String::new();
        render_block(&node, &mut out, RenderOptions::default());
        assert_eq!(
            out,
            "<p><img src=\"logo.png\" alt=\"logo\" title=\"Markec logo\"></p>\n"
        );
    }

    #[test]
    fn renders_ordered_list_start_when_non_one() {
        let node = ast::Block::List {
            ordered: true,
            start: 2,
            tight: true,
            items: vec![ast::ListItem {
                children: vec![ast::Block::Paragraph {
                    source: String::new(),
                    inlines: vec![ast::inline::Inline::Text("two".to_string())],
                }],
                task: None,
            }],
        };

        let mut out = String::new();
        render_block(&node, &mut out, RenderOptions::default());
        assert_eq!(out, "<ol start=\"2\">\n<li>two</li>\n</ol>\n");
    }

    #[test]
    fn renders_tight_list_without_separator_before_following_blocks() {
        let node = ast::Block::List {
            ordered: false,
            start: 1,
            tight: true,
            items: vec![ast::ListItem {
                children: vec![
                    ast::Block::Paragraph {
                        source: String::new(),
                        inlines: vec![ast::inline::Inline::Text("list".to_string())],
                    },
                    ast::Block::Heading {
                        level: 1,
                        inlines: vec![ast::inline::Inline::Text("header".to_string())],
                    },
                ],
                task: None,
            }],
        };

        let mut out = String::new();
        render_block(&node, &mut out, RenderOptions::default());
        assert_eq!(out, "<ul>\n<li>list<h1>header</h1>\n</li>\n</ul>\n");
    }
}
