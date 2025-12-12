## Contributing to Hyp

Thank you for considering contributing to **Hyp**, a Rust code quality and analysis toolkit.
This project is designed to be **extensible** (custom checkers, custom CLIs) and **educational**
(problem examples that compile but are problematic), so contributions in both code and docs
are very welcome.

Before large changes, please scan:
- `README.md` for the overall vision and conceptual model
- `crates/hyp-checks-generic/README.md` for checker architecture and macros
- `crates/hyp/BUILD_YOUR_OWN_HYP_CLI.md` for custom CLI patterns

If you plan a substantial redesign, new category family, or major API change, please open
an issue first so we can align on direction.

---

## How to Get Started

1. **Fork and clone**
   - Fork the repository on your Git hosting service
   - Clone your fork locally and add the original repo as `upstream` if you like

2. **Set up Rust toolchain**
   - Install a recent stable Rust (via `rustup`)
   - Recommended components:
     - `rustfmt` (for formatting)
     - `clippy` (for lints, where applicable)

3. **Build and test**
   - From the repo root:

     ```bash
     make build # ensure compilation
     make test # run all unit tests and ensure problem examples are compilable and executable
     ```

   - Run the analyzer CLI against the bundled problem examples:

     ```bash
     cargo run --bin hyp -- -s crates/hyp-examples/src -v
     cargo run --bin hyp -- --list
     ```

---

## Project Layout

- `crates/hyp-examples/`
  Compilable but problematic Rust snippets, grouped by categories `E10`–`E18`.

- `crates/hyp-checks-generic/`
  Core analysis **library** with:
  - `define_checker!` and `register_checker!` macros
  - Checker trait and registry
  - Analyzer configuration and violation reporting

- `crates/hyp/`
  Reference **CLI** that wires the analyzer into a usable `hyp` binary.
  Also contains `BUILD_YOUR_OWN_HYP_CLI.md` showing how to build your own `cargo hyp-myproject`.

The top-level `README.md` explains the conceptual model, built-in categories, and how Hyp differs
from tools like Clippy, Kani, Miri, Prusti, and MIRAI.

---

## Contribution Types

### 1. Problem Examples

These live in `crates/hyp-examples/src/` and are grouped by category, e.g.:
- `e10_unsafe_code/`
- `e11_code_surface_complexity/`
- `e14_type_safety/`

**Goal**: Realistic Rust code that **compiles** but illustrates a specific problematic pattern.

- **Do:**
  - Keep examples as small as possible while still realistic.
  - Include comments/docstrings explaining the problem and mitigation.
  - Reference the relevant checker code (e.g. `E1002`, `E1401`) in the file name and docs.

- **Checklist when adding an example:**
  1. Add a new `.rs` file in the appropriate `eXX_...` folder.
  2. Make sure it compiles with `cargo test -p hyp-examples` (or full `cargo test`).
  3. Update any listing/registry in the hyp-examples crate if needed.

### 2. New or Improved Checkers

Checkers live in `crates/hyp-checks-generic/src/checkers/` under:
- `e10/`, `e11/`, `e12/`, …, `e18/`

Each checker is implemented with the `define_checker!` macro and usually follows the
patterns in:
- `e10/e1001_direct_panic.rs`
- `e11/e1106_long_function.rs`
- `e14/e1401_integer_overflow.rs`

**Basic steps to add a checker:**

1. **Pick a code and category**
   - Use the roadmap table in `crates/hyp-checks-generic/README.md` to find the next free code
     (or extend with a new checker not yet listed).

2. **Create the checker file**
   - Place it in the appropriate module, e.g.
     - `crates/hyp-checks-generic/src/checkers/e10/e1004_unsafe_without_comment.rs`
   - Use `define_checker!` to declare:
     - `code` (e.g. `"E1004"`)
     - `name`
     - `suggestions`
     - `target_items` (e.g. `[Function]`)
     - `config_entry_name` (TOML key, e.g. `"e1004_unsafe_without_comment"`)
     - `config` struct fields and defaults
     - `check_item` body that walks the `syn` AST and returns `Vec<Violation>`

3. **Register the checker**
   - Export checker + config in the group `mod.rs`, e.g. `checkers/e10/mod.rs`
   - Add it to the group registry with `register_checker!`, e.g. `checkers/e10/registry.rs`
   - Ensure `crates/hyp-checks-generic/src/registry.rs` includes the group (for new groups)

4. **Add tests**
   - Add unit tests in the same file:
     - Positive cases (violations are reported).
     - Negative cases (no violations).
   - Use `syn::parse_file` and call `checker.check_item(&item, "test.rs")`.

5. **Update docs**
   - Mark the checker as supported in the roadmap table in `crates/hyp-checks-generic/README.md`.
   - Optionally add or cross-link problem examples that showcase the pattern.

6. **Run tests**
   - `cargo test -p hyp-checks-generic`
   - `cargo test` at the workspace root.

### 3. CLI and Tooling

The reference CLI lives in `crates/hyp/`.

You can:
- Improve UX (flags, output formats, listing checkers).
- Add examples or documentation for building project-specific CLIs.

For a full, opinionated guide on custom CLIs (like `cargo hyp-myproject`), see:
- `crates/hyp/BUILD_YOUR_OWN_HYP_CLI.md`

If you add new CLI functionality, please:
- Add or update integration tests if applicable.
- Extend `crates/hyp/README.md` and/or the top-level `README.md`.

### 4. Documentation and Examples

High‑quality documentation is a core goal of Hyp.

Good contributions include:
- Clarifying existing docs or READMEs.
- Adding small, focused examples that make new features easy to understand.
- Improving cross‑links between:
  - `README.md`
  - `crates/hyp-checks-generic/README.md`
  - `crates/hyp/BUILD_YOUR_OWN_HYP_CLI.md`

When updating docs, keep headings and style consistent:
- Use `##` / `###` headings.
- Use bullet lists with short, focused sentences.
- Refer to files and modules with backticks, e.g. `crates/hyp-checks-generic/src/checkers/e10/`.

---

## Coding Style and Quality

- **Rust style**
  - Run `cargo fmt` before committing.
  - Try to keep functions short and focused; new checkers should be readable examples.

- **Testing**
  - Prefer adding tests alongside the code you change.
  - For checkers, aim for:
    - 1–3 positive tests (violations).
    - 1–3 negative tests (no violations).

- **Error handling**
  - Use `Result` and `AnalyzerError` (where applicable in the analyzer).
  - Prefer explicit error messages that help users understand misconfigurations.

---

## How to Submit a Change

1. **Open an issue (recommended for non-trivial changes)**
   - Describe what you want to add/change and why.
   - Link to any relevant problem examples or external references.

### Commit Message Convention

Use a short **prefix** in your commit messages to make history easy to scan:

- **feat:** New feature (e.g., new checker, new CLI capability)
- **fix:** Bug fix (including incorrect checker behavior)
- **docs:** Documentation changes only (README, guides, comments)
- **refactor:** Internal code changes that don’t alter behavior
- **test:** Add or update tests without behavior changes
- **chore:** Tooling, dependency bumps, CI, formatting, trivial plumbing

Examples:

- `feat: add E1305 non-exhaustive match checker`
- `fix: correct line number reporting in E1106LongFunction`
- `docs: clarify custom hyp-myproject CLI setup`
- `test: add regression test for E1015 unwrap detection`

2. **Create a branch and implement the change**
   - Follow the guidelines above for checkers, examples, or CLI changes.

3. **Run the full test suite**
   - `make test`

4. **Open a Pull Request**
   - Keep the PR focused on one logical change (one checker, one CLI feature, or one doc theme).
   - In the PR description:
     - Explain what you changed and why.
     - Reference related issues or roadmap items (e.g. specific `E1xxx` codes).

5. **Review and iterate**
   - Be prepared to discuss architectural trade‑offs (especially for new macros, registries,
     or configuration behavior).
   - Small follow‑up commits to address review feedback are encouraged.

---

## Questions and Ideas

If you are unsure where to start:
- Browse the roadmap in `crates/hyp-checks-generic/README.md` and pick an unimplemented checker.
- Look at existing checkers (`E1001`, `E1002`, `E1003`, `E1106`, `E1401`, `E1402`, `E1403`)
  to understand the standard patterns.
- Explore `crates/hyp-examples/` and think about additional problematic patterns that
  deserve detection.

Thank you for helping make Hyp a better tool for Rust teams and for AI‑assisted development. 🙌
