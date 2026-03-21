# Contributing to Chetna

Thank you for your interest in contributing to Chetna! This document provides guidelines and instructions for contributing.

---

## Quick Start

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/yourusername/Chetna.git`
3. **Create a branch**: `git checkout -b feature/your-feature-name`
4. **Make changes** and test locally
5. **Submit a PR**: Push to your fork and open a pull request

---

## License & Copyright

### By Contributing, You Agree To:

1. **License Your Contributions**: Your contributions will be licensed under the same terms as Chetna (MIT License with trademark protections).

2. **Original Work**: Your contributions are your original work or you have the right to submit them.

3. **No Additional Restrictions**: You won't add additional licensing restrictions to your contributions.

4. **Trademark Assignment**: You won't claim trademark rights to any names, logos, or branding elements.

### Copyright Notices

**For new files you create:**
```rust
// Copyright (c) Chetna Contributors
// SPDX-License-Identifier: MIT
```

**For documentation:**
```markdown
<!-- Copyright (c) Chetna Contributors -->
<!-- SPDX-License-Identifier: MIT -->
```

You don't need to add your personal copyright to files - the "Chetna Contributors" notice is sufficient.

---

## What We're Looking For

### ✅ Welcome Contributions

- **Bug fixes**: Especially for reported issues
- **Performance improvements**: Faster queries, better memory usage
- **Documentation**: Tutorials, examples, clarifications
- **Tests**: Unit tests, integration tests
- **Feature improvements**: Enhancements to existing features
- **Bug reports**: Detailed issue reports with reproduction steps
- **Feature requests**: Well-thought-out suggestions

### ⚠️ Discuss First

- **Major new features**: Open an issue to discuss before implementing
- **Breaking changes**: Need community discussion
- **New dependencies**: Should be justified
- **API changes**: Affect downstream users

### ❌ Not Acceptable

- **Plagiarized code**: Must be your original work
- **Licensed code from other projects**: Unless compatible with MIT
- **Test spam**: Meaningless PRs for contribution count
- **License changes**: Don't modify LICENSE or TRADEMARK files without discussion

---

## Development Setup

### Prerequisites

- **Rust**: 1.70 or later ([install](https://rustup.rs/))
- **Ollama**: For embeddings ([install](https://ollama.ai/))
- **SQLite**: Usually pre-installed on macOS/Linux

### Setup Steps

```bash
# Clone your fork
git clone https://github.com/yourusername/Chetna.git
cd Chetna

# Build in development mode
cargo build

# Run tests
cargo test

# Run locally
cargo run

# Format code
cargo fmt

# Lint
cargo clippy
```

### Environment Setup

Create `ChetnaData/.env` for local development:

```bash
# ChetnaData/.env
CHETNA_PORT=1987
CHETNA_DB_PATH=./ChetnaData/chetna.db

# Embeddings
EMBEDDING_PROVIDER=ollama
EMBEDDING_MODEL=qwen3-embedding:4b
EMBEDDING_BASE_URL=http://localhost:11434
```

---

## Pull Request Guidelines

### PR Title Format

```
type: brief description

Examples:
fix: Memory leak in semantic search
feat: Add batch embedding support
docs: Update API reference examples
perf: Optimize recall scoring algorithm
```

### PR Description Template

```markdown
## What does this do?
[Brief description]

## Why is this needed?
[Problem statement]

## How was it tested?
[Test approach]

## Related Issues
[Link to issues]

## Checklist
- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] Documentation updated (if needed)
- [ ] No breaking changes (or documented if breaking)
```

### Code Style

**Rust Code:**
- Follow Rust idioms and best practices
- Use `cargo fmt` before committing
- Address `cargo clippy` warnings
- Write meaningful error messages
- Add tests for new functionality

**Documentation:**
- Clear, concise writing
- Include examples where helpful
- Update API docs if changing endpoints
- Keep README in sync with features

---

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test module
cargo test --lib db::brain

# With output
cargo test -- --nocapture

# Integration tests
cargo test --test integration
```

### Writing Tests

**Unit Test Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_semantic_search() {
        // Async test implementation
    }
}
```

### Test Coverage

While we don't enforce 100% coverage, please:
- Test critical paths
- Add tests for bug fixes
- Test edge cases
- Include integration tests for new features

---

## Documentation Contributions

### Where to Update Docs

| Change Type | File(s) to Update |
|-------------|-------------------|
| API changes | `docs/api.md` |
| New features | `README.md`, `docs/` |
| Bug fixes | Affected documentation |
| Examples | `docs/agent-integration.md` |
| Setup changes | `docs/QUICKSTART.md` |

### Documentation Style

- **Clear headings**: Use `##`, `###`, etc. hierarchically
- **Code examples**: Show real usage
- **Tables**: For comparisons and options
- **Warnings**: Use `> ⚠️` for important notes
- **Links**: Keep internal links relative, external links absolute

---

## Reporting Issues

### Bug Report Template

```markdown
**Describe the bug**
[Clear description]

**To Reproduce**
Steps to reproduce:
1. [First step]
2. [Second step]
3. [etc.]

**Expected behavior**
[What should happen]

**Environment:**
- OS: [e.g., macOS 14.0]
- Rust version: [e.g., 1.75.0]
- Chetna version: [e.g., 0.3.0]
- Ollama version: [if applicable]

**Logs**
[Relevant error messages]

**Additional context**
[Any other details]
```

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
[Description of the problem]

**Describe the solution you'd like**
[What you want to happen]

**Describe alternatives you've considered**
[Other solutions you've thought about]

**Use case**
[Who would benefit from this?]

**Additional context**
[Any other details]
```

---

## Community Guidelines

### Be Respectful

- Treat all contributors with respect
- Constructive criticism is welcome
- No personal attacks or harassment
- Help newcomers

### Communication

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **PR Comments**: Keep focused on the code
- **Response Time**: Maintainers typically respond within a few days

### Code of Conduct

- Be inclusive and welcoming
- Accept constructive feedback
- Focus on what's best for the community
- Show empathy towards other contributors

---

## Recognition

Contributors are recognized in:

1. **CONTRIBUTORS.md**: List of all contributors
2. **Release Notes**: Mentioned in release announcements
3. **README.md**: Notable contributors may be mentioned

---

## Questions?

If you have questions about contributing:

- **General questions**: Open a GitHub Discussion
- **Bug reports**: Open an Issue
- **Feature discussions**: Open an Issue or Discussion
- **License questions**: See [LICENSE](LICENSE) and [TRADEMARK.md](TRADEMARK.md)

---

## Thank You!

Every contribution helps make Chetna better. Whether it's a typo fix, a bug report, or a major feature - it's all appreciated! 🎉

**Chetna** - Give your AI agents permanent memory

---

*Last Updated: March 17, 2026*
