# Contributing to LPM

Thank you for your interest in contributing to LPM! This guide will help you get started.

## Getting Started

### Prerequisites

- Rust (latest stable) - [rustup.rs](https://rustup.rs/)
- Lua 5.1, 5.3, or 5.4 installed
- Git

### Development Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/yourusername/lpm.git
   cd lpm
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Run LPM locally:**
   ```bash
   cargo run -- install
   ```

## Development Workflow

### Making Changes

1. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes

3. Run tests:
   ```bash
   cargo test
   ```

4. Run linter:
   ```bash
   cargo clippy
   ```

5. Format code:
   ```bash
   cargo fmt
   ```

6. Commit your changes:
   ```bash
   git commit -m "Add feature: description"
   ```

7. Push and create a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Write tests for new features
- Update documentation as needed

## Project Structure

```
lpm/
├── crates/
│   ├── lpm-core/      # Core library (shared with plugins)
│   │   ├── src/
│   │   │   ├── core/  # Core utilities (errors, paths, version)
│   │   │   ├── package/# Package manifest
│   │   │   └── path_setup/# Lua path setup and runner
│   │   └── Cargo.toml
│   ├── lpm-watch/     # Watch mode plugin
│   │   └── src/
│   └── lpm-bundle/    # Bundle plugin (experimental)
│       └── src/
├── src/
│   ├── cli/           # CLI command implementations
│   │   └── plugin.rs  # Plugin discovery and execution
│   ├── core/          # Core functionality (errors, paths, version)
│   ├── package/       # Package management
│   ├── luarocks/      # LuaRocks integration
│   ├── resolver/      # Dependency resolution
│   ├── build/         # Rust extension building
│   ├── security/      # Security auditing
│   └── ...
├── tests/             # Integration tests
├── benches/           # Performance benchmarks
└── docs/              # Documentation
```

### Plugin Development

LPM supports plugins as separate executables. See [Plugin Development Guide](Plugin-Development) for details.

## Testing

### Unit Tests

Unit tests are in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        // Test code
    }
}
```

Run unit tests:
```bash
cargo test
```

### Integration Tests

Integration tests are in `tests/`:

```bash
cargo test --test integration_tests
```

### Security Tests

```bash
cargo test --test security
```

### Benchmarks

```bash
cargo bench
```

## Documentation

- User documentation: `docs/user/` (synced to wiki)
- Contributor documentation: `docs/contributing/` (synced to wiki)
- Code documentation: Inline with `///` comments

When adding features, update relevant documentation.

## Pull Request Process

1. **Before submitting:**
   - [ ] Code compiles without warnings
   - [ ] All tests pass
   - [ ] Code is formatted (`cargo fmt`)
   - [ ] No clippy warnings (`cargo clippy`)
   - [ ] Documentation updated
   - [ ] Commit messages are clear

2. **Create PR:**
   - Clear title and description
   - Reference related issues
   - Describe changes and motivation

3. **Review process:**
   - Address feedback
   - Keep PR focused (one feature/fix per PR)
   - Update PR if requested

## Areas for Contribution

- Bug fixes
- New features (check issues for ideas)
- Documentation improvements
- Performance optimizations
- Test coverage
- Cross-platform compatibility

## Questions?

- Open an issue for discussion
- Check existing issues and PRs
- Review [Architecture](Architecture) documentation

Thank you for contributing!

