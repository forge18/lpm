# Git Hooks Setup

LPM uses **RustyHook** to enforce code quality before commits, similar to Husky in Node.js projects.

## Quick Setup

RustyHook is already configured! The hooks are automatically set up when you build the project:

```bash
cargo build
```

The configuration is in `.rusty-hook.toml`:

```toml
[hooks]
pre-commit = "cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings"
```

## What the Hook Does

The pre-commit hook automatically runs:
- ✅ `cargo fmt --check` - Formatting check
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` - Linting

The hook **prevents commits** if:
- Code is not properly formatted
- Clippy finds linting issues

Tests are run in CI, not in the pre-commit hook (for speed).

## How RustyHook Works

RustyHook is a Rust-native Git hook runner that:
- ✅ Automatically installs hooks when you build the project
- ✅ Uses a simple TOML configuration file (`.rusty-hook.toml`)
- ✅ Runs hooks defined in the config before commits
- ✅ Blocks commits if hooks fail

## Configuration

Edit `.rusty-hook.toml` to customize hooks:

```toml
[hooks]
pre-commit = "cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings"
pre-push = "cargo test"  # Optional: run tests before push
```

## Alternative Solutions

If you want different features, consider these alternatives:

### 1. Pre-commit Framework

Python-based but language-agnostic:

```bash
# Install pre-commit
pip install pre-commit

# Create .pre-commit-config.yaml
cat > .pre-commit-config.yaml << EOF
repos:
  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
      - id: clippy
      - id: test
EOF

# Install hooks
pre-commit install
```

**Pros:**
- Widely used, well-documented
- Supports many languages
- Easy to share configuration

**Cons:**
- Requires Python
- Slightly slower startup

### 2. Prek

Rust-based alternative to pre-commit:

```bash
# Install Prek
cargo install prek

# Create prek.toml
cat > prek.toml << EOF
[[hooks]]
id = "fmt"
command = "cargo"
args = ["fmt", "--check"]

[[hooks]]
id = "clippy"
command = "cargo"
args = ["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]
EOF

# Install
prek install
```

**Pros:**
- Fast (Rust-native)
- Single binary, no dependencies
- Drop-in replacement for pre-commit

**Cons:**
- Less mature than pre-commit
- Smaller ecosystem

## Current Setup (RustyHook)

The current setup uses **RustyHook**, which is:
- ✅ Rust-native and fast
- ✅ Simple TOML configuration
- ✅ Automatically installs hooks on build
- ✅ Easy to customize
- ✅ No manual hook management needed

## Bypassing Hooks

If you need to bypass the hook (not recommended):

```bash
git commit --no-verify -m "Emergency fix"
```

⚠️ **Warning:** Only use `--no-verify` in emergencies. CI will still catch issues.

## Troubleshooting

### Hook not running

RustyHook hooks are automatically installed when you build. If hooks aren't working:

1. **Rebuild the project:**
   ```bash
   cargo build
   ```

2. **Check if rusty-hook CLI is installed:**
   ```bash
   cargo install rusty-hook
   ```

3. **Verify the hook exists:**
   ```bash
   ls -la .git/hooks/pre-commit
   ```

### Hook too slow

The hook runs formatting and linting checks. If it's too slow:
- Consider using `cargo check` instead of `cargo clippy` for faster feedback
- Or modify `.rusty-hook.toml` to run only essential checks

### Sharing hooks with team

The configuration is in `.rusty-hook.toml` and tracked in git. Team members just need to:
1. Clone the repository
2. Run `cargo build` (hooks are automatically installed)

No manual setup required!

## Recommendations

RustyHook is the recommended solution for this project because:
1. ✅ Rust-native and fast
2. ✅ Simple TOML configuration
3. ✅ Automatic hook installation
4. ✅ No manual setup needed for team members
5. ✅ Works immediately after cloning and building

If the project grows and needs more sophisticated hook management, consider migrating to RustyHook or Prek.

