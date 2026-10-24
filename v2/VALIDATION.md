# Validation record and reproduction

Source baseline: `4515628ec7698893f251782f9429c5c0d46f7e76`, fresh fetched `origin/main`, branch `feat/native-v2-rust`. Original Raycast pin: `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Beta fixture pin: `fd300e276356587726566026f072563eb9c34cb1`. No staging, commit, push, PR, host registration, host edits, profile/ledger changes or completed-clone removal were performed.

Executed checks:

- Stable and Rust 1.85 formatting and strict `--all-targets` Clippy for the library and concrete headless adapter harness. Only targeted naming/argument-count exemptions apply to retained parser interfaces; no blanket Clippy exemption.
- Engine: 20 unit tests and two integration tests. The unchanged-source oracle has 2,711 cases: 2,710 string inputs plus a null-source control; all string-input output/error comparisons pass. It includes all 192 Highlight.js languages, their aliases, eight callback families, GFM/breaks/entities/Unicode, raw and malformed HTML, source serialization quirks and 138 Marked 15 fixture inputs. No expected converted HTML was hand-written, normalized or filtered into an xfail list.
- Headless beta harness: 11 unit tests plus two integration tests, using exact beta common/SDK/native session source. Tests cover macOS and Windows route seams, exact argument shape, plain-text HTML format, malformed reads, transport errors, strict acknowledgement failures, sanitized errors, cancellation checkpoints, no blind retries, lifecycle operations, forged/stale IDs, no-view dispatch and manifest/compiled-command coverage. Tauri, registration and installation-record stubs are explicitly labeled. No real clipboard, selected text, browser, focus, window or paste operations run.
- Windows GNU `cargo check` and strict `cargo clippy --all-targets`, on stable and 1.85, for the engine and concrete adapter harness. These are cross-compilation checks, not Windows execution or desktop QA.
- Complete pinned Draft 2020-12 extension schema validation, with 28 invalid manifests rejected. Author/unknown root fields, missing required fields, invalid runtime/IDs/icons, duplicate replacements/platforms, invalid commands and verification states are included.
- Exact original icon size/hash; provenance hashes and Git visibility; immutable beta/own/original/parser fixture hashes; actual Rust license texts checked against checksum-verified locked `.crate` archives; npm package versions/content pinned by the own lock and original SHA-512 integrity checks.
- `cargo package`, package-content audit, self-contained extracted engine and materialized harness tests under stable and 1.85 with Cargo offline. The source oracle regenerates inside the extracted package without network or npm installation. Nested Cargo packages are shipped as `.in` templates and materialized with a pinned lock; required fixtures/libraries/licenses are included explicitly.
- Disposable V1 `npm ci --ignore-scripts`, `npm test` (one smoke test) and `npm run typecheck`, using unchanged original source and lock. All 12 non-README V1 tracked files remain byte-exact. Root README contains only the discovery-link addition.

XERJ used canonical root prefix `ax-44518416d32c` and isolated state. The first generation indexed all three V1 code files, with one non-code junk/skip reported. The final refresh could not connect to the server, so the updated V2 index is incomplete. No global service was restarted. Exact `rg`, source reads, hashes, Git and executable checks were used for the final audit.

From this directory, reproduce the core and harness checks:

```sh
cargo +stable fmt --check
cargo +1.85.0 fmt --check
cargo +stable test --locked --offline
cargo +1.85.0 test --locked --offline
cargo +stable clippy --all-targets --locked -- -D warnings
cargo +1.85.0 clippy --all-targets --locked -- -D warnings
python3 scripts/materialize-harness.py
CARGO_TARGET_DIR=target/beta cargo +stable test --manifest-path work/harness/Cargo.toml --locked --offline
CARGO_TARGET_DIR=target/beta cargo +1.85.0 test --manifest-path work/harness/Cargo.toml --locked --offline
CARGO_TARGET_DIR=target/beta cargo +stable clippy --manifest-path work/harness/Cargo.toml --all-targets --locked -- -D warnings
CARGO_TARGET_DIR=target/beta cargo +1.85.0 clippy --manifest-path work/harness/Cargo.toml --all-targets --locked -- -D warnings
```

Check the external adapter/template source formatting explicitly without traversing immutable fixtures:

```sh
rustfmt +stable --edition 2021 --check --config skip_children=true adapter.rs harness/lib.rs.in harness/session/lib.rs.in harness/adapter_tests.rs harness/tauri/lib.rs harness/tests/session.rs harness/tests/coverage.rs
rustfmt +1.85.0 --edition 2021 --check --config skip_children=true adapter.rs harness/lib.rs.in harness/session/lib.rs.in harness/adapter_tests.rs harness/tauri/lib.rs harness/tests/session.rs harness/tests/coverage.rs
```

For each toolchain, repeat `cargo check` and `cargo clippy --all-targets -- -D warnings` with `--target x86_64-pc-windows-gnu` on both `Cargo.toml` and `work/harness/Cargo.toml`. The target must already be installed; no Windows linker or OS execution is needed for these checks.

Oracle, schema, provenance and package checks:

```sh
python3 scripts/verify-oracle.py
python3 scripts/verify-provenance.py --git
uv venv --python 3.13 work/schema-env
uv pip install --python work/schema-env/bin/python jsonschema==4.25.1
work/schema-env/bin/python scripts/validate-schema.py
python3 scripts/validate-package.py
```

Cargo offline checks require the locked registry crates to have been fetched previously. Node is required **only** for oracle generation/verification; Python and jsonschema are validation tools. They are not dependencies of the product runtime. `scripts/make-provenance.py` and `scripts/collect-licenses.py` intentionally refresh inventories; use the `verify-*` scripts for read-only verification.

Real host registration/rebuild and real macOS/Windows UI + clipboard text/format verification remain **not-run**. The complete Tauri app, actual renderer display timing, original V1 Raycast build, real Windows execution, formal all-input parser equivalence, and performance/availability guarantees for every possible regexp/input are not established by these tests. The explicit resource limits and retained external stylesheet URLs are documented in README. Both platform verification states remain `not-run`.
