# Compile into the unchanged beta host

Integration is a parent/reviewer task. No host, profile, installed-package database, ledger or platform file was edited by this port.

1. Vendor this entire `v2/` directory at `src-tauri/native-extensions/markdown-this/`. Keep the stable location rather than a temporary clone path. Preserve `Cargo.lock`, data, licenses and assets.
2. Add the library to the host `src-tauri/Cargo.toml`:

   ```toml
   tezbar-markdown-this = { path = "native-extensions/markdown-this" }
   ```

3. Declare it alongside the existing SDK in `src-tauri/src/native_core/extensions/mod.rs`:

   ```rust
   #[path = "../../../native-extensions/markdown-this/adapter.rs"]
   mod v2_markdown_this;
   ```

   The adapter's `super::v2_sdk` refers to the existing unchanged beta SDK. It uses the host's existing Tokio, serde_json and CoreContext types.

4. Extend all three dispatch points in `v2_runtime.rs`: recognize `markdown-this` in `slug`; return `super::v2_markdown_this::commands()` from `commands`; dispatch `handle(ctx, command, operation, payload, state).await` to `super::v2_markdown_this::handle`. Existing canonical and `raycast.` normalization should recognize both IDs. Do not add a JS-worker fallback for a canonical Native V2 ID.
5. Add `extension.json` as an entry in `v2_catalog.json`, preserving canonical ID, `replaces`, the sole `index` command, empty preferences, both platforms and both `not-run` verification entries. Do not add a root `author` field; credits are in description and notices. The host derives coverage from compiled dispatch, not from this manifest's runtime string.
6. Copy the exact `assets/command-icon.png` bytes to the actual host path `src/renderer/public/extensions/icons/markdown-this.png`. Hash the destination and compare with the hash in README. Keep V1 installed and unchanged for reversible fallback.
7. Add coverage assertions alongside the host's catalog tests: `slug` recognizes the canonical and legacy IDs; `commands("markdown-this") == &["index"]`; `supports` accepts exactly `index`; the catalog has that same command and intended macOS/Windows declarations; unknown commands are unavailable; verification is still `not-run`.
8. Rebuild the actual desktop host and validate its canonical install and command dispatch. `native-v2.json`, a copied manifest, or the source Cargo package alone cannot install executable Rust.

The source has no UI or actions, so no List/Detail/Form renderer adaptation is needed. Installed launch payloads may contain `argumentValues` and `preferences`; this source consumes neither. Refresh/search/load-more return the SDK's unchanged value and mint no capabilities. Invoke-action is rejected. Dispose only discards local invocation state. A no-view launch receives no persistent native session ID; the actual pinned `native.rs` dispatch is tested with stubs.

The only OS boundary is the existing clipboard router. Read must be a string. SDK copy must acknowledge an object with `ok: true`, no non-null `error`, and absent/true `success`. Errors are sanitized and never reported as successful conversion. A copy attempt is checkpointed before awaiting its response; state cannot replay after transport uncertainty or cancellation. There are no focus, selection, paste, shell-open, browser, storage or native-helper calls.

`harness/` contains template manifests, an inert Tauri type stub, and explicit registry/runtime-registration stubs. `common.rs`, `v2_sdk.rs`, `native.rs`, and all other beta fixture files are byte-exact. The session fixture links the original native session implementation with test dispatch/installation records; it is not a desktop build or a substitute for real registration and clipboard QA.

Before marking either OS verified, test a real rebuilt host with Unicode Markdown, empty/whitespace clipboard, representative code languages, GFM, HTML edge cases, clipboard read/write failures and the actual **plain-text HTML** clipboard format. Confirm no rich HTML is claimed, no native toast API is fabricated, and V1 still works. Record that QA separately before changing verification state.
