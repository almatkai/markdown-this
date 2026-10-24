# Markdown This — Rust Native V2

Native replacement for `raycast.markdown-this`, alongside the untouched V1 implementation. Canonical ID: `com.tezbar.native.markdown-this`. One no-view command, `index` / **Convert to HTML**. There are no preferences, arguments, interactive actions, selection reads, stored data or legacy migrations.

The production engine is a standalone Rust library, MSRV 1.85. It reads no files, runs no JS/Bun, starts no subprocesses, and uses no network. `adapter.rs` is a concrete compiled-in adapter for Tezbar beta `fd300e276356587726566026f072563eb9c34cb1`; a manifest alone cannot install executable Rust. See [INTEGRATION.md](INTEGRATION.md) for the required host registration and rebuild.

The command reads `clipboard:read []`, converts the entire string, then calls the beta SDK's `copy`, which routes `clipboard:write [html]`. The source calls `Clipboard.copy(fullHtml)` **with a string**, so the copied format is plain text containing literal HTML. It does not produce an HTML/rich-clipboard representation. No platform helper is needed for this source behavior. A successful write requires an object with `ok: true`, no non-null `error`, and `success` absent or `true`. Transport and acknowledgement failures return an error; no automatic retry occurs. Per-invocation checkpoints prevent reusing interrupted state.

The returned result is the SDK's real `no-view` result. Its message adapts the source's success toast; beta has no true toast/HUD API. Error results use the host's normal command-error path. This package does not claim a visible toast, retained session, or new renderer capability.

The exact source HTML wrapper is retained, including `lang="zh-TW"`, CSS whitespace, Highlight.js 11.9.0 CDN stylesheet URLs, and the existing `prefers-color- scheme` typo. Conversion never fetches those URLs; an application displaying the resulting HTML may do so. Raw HTML and raw image alt attributes remain unsanitized because that is the original serialization behavior.

Marked 15.0.4 (`gfm: true`, `breaks: true`) and marked-highlight 2.2.1 are the oracle. The native parser adapts Markast's Rust parser, with Marked 15-specific serialization, tabs, task lists, references, HTML regexes and continuation behavior. Highlighting uses a Rust mode evaluator plus declarative grammar data lowered from the actual locked Highlight.js 11.11.1: 192 languages, 5,179 modes and eight explicitly ported callbacks. Declarative regex/keyword data is compiled in; there is no script interpreter. Unknown code languages fall back to the original plaintext highlighting behavior.

The reproducible unchanged-source oracle currently contains 2,711 cases (2,710 string inputs), covering all bundled languages and aliases, callback families, nested sublanguages, raw HTML, entities, Unicode, GFM, breaks, malformed HTML, list/task/reference quirks and 138 Marked 15 fixture inputs. Successful native HTML is compared byte for byte, without whitespace normalization or hand-written expected conversions. This is substantial regression evidence, not a proof of equivalence for every possible Markdown input.

Explicit resource limits return errors before copying: 256 KiB UTF-8 input; 128 nested parser/render calls; 100,000 parser/render calls; 64 `>` characters on one line; 4,096 lines beginning with four spaces; 256 highlighting modes; 16 nested sublanguages; and a shared 1,000,000-step highlighting budget per code block. No input or output is silently truncated. These are native limits, not V1 behavior.

The icon in `assets/command-icon.png` is the unchanged original PNG: 196,200 bytes, SHA-256 `7e0a1ddf92be463dd2ae7335fdbe00d33f944ba012c33d784d652f5c388727f3`. The manifest points to the actual host public URL `/extensions/icons/markdown-this.png`.

Credits: original Raycast author **aravindballa**, contributor **chihkang**, own repository maintainer **almatkai**. Original extension MIT copyright: 2024 chihkang_lin. Third-party parser: Yang Pan / Markast. See [NOTICES.md](NOTICES.md), [PROVENANCE.json](PROVENANCE.json), and [VALIDATION.md](VALIDATION.md).

Host registration/rebuild and real macOS/Windows UI plus clipboard text/format verification have **not run**. Both manifest verification entries remain `not-run`. Headless and cross-compilation checks do not change that status.
