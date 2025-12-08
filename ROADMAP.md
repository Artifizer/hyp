# ROADMAP

## Core Functionality

- [x] Initial hyp-examples version with a CLI tool
- [x] Initial hyp-checks-generic version with a CLI tool
- [x] TOML-based configuration (Hyp.toml)
- [x] CLI refactoring with subcommands (check, list, print-config, guideline, verify-examples)
- [x] Category-level configuration (e.g., `e11.enabled = false`)
- [ ] Polish the checks accuracy as they are mostly too naive
- [ ] To achieve 100% code coverage for Hyp (now 84%)
- [ ] Print code fragment when reporting an error
- [ ] Implement remaining Phase 3 checks in [crates/hyp-checks-generic/README.md](crates/hyp-checks-generic/README.md)
- [ ] Fix all issues in `hyp verify-examples`
- [ ] Support for inline suppression comments (e.g., `// hyp:ignore e1001`) if allowed in `Hyp.toml`

## IDE Integration

- [ ] VS Code extension
  - [ ] Real-time diagnostics
  - [ ] Hover documentation for violations
  - [ ] Configuration UI
- [ ] IntelliJ IDEA / RustRover plugin
- [ ] Language Server Protocol (LSP) implementation for universal IDE support
- [ ] Add auto-fix suggestions for common issues (e.g., replace `unwrap()` with `?`)

## LLM & AI Integration

- [ ] GitHub Copilot integration (custom instructions based on Hyp guidelines)
- [ ] Pre-commit hook generator for CI/CD pipelines
- [ ] LLM-friendly output format for AI code review tools

## Custom Checker Ecosystem

- [ ] Example E2xx checkers (project-specific Rust patterns)
- [ ] Example E3xx checkers (repository layout rules)
- [ ] Example E4xx checkers (business logic and security rules)
- [ ] Checker marketplace/registry (community-contributed checkers)
- [ ] Checker testing framework improvements
- [ ] Documentation generator for custom checkers

## Configuration & Usability

- [ ] Configuration wizard (`hyp init`) to generate Hyp.toml interactively
- [ ] Configuration profiles (strict, relaxed, beginner-friendly, production)
- [ ] Migration tool from Clippy configuration to Hyp configuration

## Reporting & Output

- [ ] HTML report generation with interactive filtering
- [ ] SARIF format support for GitHub Code Scanning integration
- [ ] Trend analysis (track violations over time)
- [ ] Diff mode (only report violations in changed lines)
- [ ] Integration with code review tools (GitHub PR comments, GitLab MR comments)

## Performance & Scalability

- [ ] Parallel file processing for large codebases
- [ ] Incremental analysis (only re-analyze changed files)
- [ ] Caching mechanism for faster repeated runs

## Documentation & Education

- [ ] Polish all the *.md files content
- [ ] Video tutorials for common use cases
- [ ] Blog posts on cognitive complexity and LLM-friendly code
- [ ] Case studies from real-world usage
- [ ] Detailed feature parity & comparison guide: when to use Hyp vs Clippy vs Miri

## Community & Distribution

- [ ] Publish to crates.io
- [ ] Set up GitHub Actions for CI/CD
- [ ] Create Discord/Slack community for users
- [ ] Governance model for accepting community checkers

## Advanced Features

- [ ] Integration with cargo-deny for dependency analysis
- [ ] Custom metrics and scoring (code quality score per file/crate)
- [ ] Machine learning model to predict problematic patterns
- [ ] Cross-crate analysis for library API misuse detection
- [ ] Support for analyzing generated code (macros, build scripts)
- [ ] Workspace-aware analysis (understand relationships between crates)
